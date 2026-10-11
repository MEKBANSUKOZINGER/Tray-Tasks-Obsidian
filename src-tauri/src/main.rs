#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod engine;
mod markdown;
mod model;
mod storage;

use engine::Engine;
use model::{Settings, Snapshot};
use notify::Watcher;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State,
};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt as _;
use tauri_plugin_positioner::{Position, WindowExt};

struct AppState {
    engine: Mutex<Engine>,
    dirty: Arc<AtomicBool>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Action {
    Configure { settings: Settings },
    Add { title: String },
    Rename { id: String, title: String },
    Complete { id: String, done: bool },
    Delete { id: String },
    Reorder { ids: Vec<String> },
    Postpone { id: String, date: String },
    Reminders { selections: BTreeMap<String, i64> },
    ClearReminder { id: String },
    Acknowledge { ids: Vec<String> },
    Rollover { accept: bool },
    Recover,
    AbandonRecovery,
}
#[tauri::command]
fn get_snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    Ok(state
        .engine
        .lock()
        .map_err(|_| "App state is unavailable")?
        .snapshot())
}
#[tauri::command]
fn detect_daily_folder(path: String) -> Result<String, String> {
    Engine::detect_folder(&path)
}
#[tauri::command]
fn action(
    app: tauri::AppHandle,
    state: State<AppState>,
    action: Action,
) -> Result<Snapshot, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "App state is unavailable")?;
    match action {
        Action::Configure { settings } => {
            if !settings.icon_path.is_empty() {
                tauri::image::Image::from_path(&settings.icon_path).map_err(|e| e.to_string())?;
            }
            let previous = engine.data.clone();
            let displayed = engine.displayed_ids.clone();
            engine.configure(settings)?;
            let native_result = (|| -> Result<(), String> {
                if engine.data.settings.autostart {
                    app.autolaunch().enable().map_err(|e| e.to_string())?;
                } else {
                    app.autolaunch().disable().map_err(|e| e.to_string())?;
                }
                apply_icon(&app, &engine.data.settings)
            })();
            if let Err(e) = native_result {
                engine.data = previous;
                engine.displayed_ids = displayed;
                let rollback = engine.persist();
                if engine.data.settings.autostart {
                    let _ = app.autolaunch().enable();
                } else {
                    let _ = app.autolaunch().disable();
                }
                let _ = apply_icon(&app, &engine.data.settings);
                return Err(format!(
                    "{e}; preferences restored{}",
                    rollback
                        .err()
                        .map(|x| format!("; restore failed: {x}"))
                        .unwrap_or_default()
                ));
            }
        }
        Action::Add { title } => engine.add(&title)?,
        Action::Rename { id, title } => engine.rename(&id, &title)?,
        Action::Complete { id, done } => engine.complete(&id, done)?,
        Action::Delete { id } => engine.delete(&id)?,
        Action::Reorder { ids } => engine.reorder(ids)?,
        Action::Postpone { id, date } => engine.postpone(&id, &date)?,
        Action::Reminders { selections } => engine.set_reminders(selections)?,
        Action::ClearReminder { id } => engine.clear_reminder(&id)?,
        Action::Acknowledge { ids } => engine.acknowledge(ids)?,
        Action::Rollover { accept } => engine.rollover(accept)?,
        Action::Recover => engine.recover_move()?,
        Action::AbandonRecovery => engine.abandon_recovery()?,
    }
    let snapshot = engine.snapshot();
    drop(engine);
    let _ = app.emit("snapshot", &snapshot);
    if snapshot.alerts.is_empty() && snapshot.rollover.is_none() {
        if let Some(w) = app.get_webview_window("reminder") {
            let _ = w.hide();
        }
    }
    state.dirty.store(true, Ordering::Relaxed);
    Ok(snapshot)
}
#[tauri::command]
fn open_daily(app: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "App state is unavailable")?;
    let path = engine.ensure_note(engine.today)?;
    let encoded = path
        .to_string_lossy()
        .as_bytes()
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(b) {
                (*b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect::<String>();
    app.opener()
        .open_url(format!("obsidian://open?path={encoded}"), None::<&str>)
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn open_app_data(app: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "App state is unavailable")?;
    app.opener()
        .open_path(engine.data_dir.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}
fn apply_icon(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("tray") {
        let icon = if settings.icon_path.is_empty() {
            app.default_window_icon().cloned()
        } else {
            Some(tauri::image::Image::from_path(&settings.icon_path).map_err(|e| e.to_string())?)
        };
        tray.set_icon(icon).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn position(window: &tauri::WebviewWindow) {
    if window.move_window(Position::TrayCenter).is_err() {
        let _ = window.move_window(Position::BottomRight);
    }
    if let (Ok(Some(m)), Ok(p), Ok(size)) = (
        window.current_monitor(),
        window.outer_position(),
        window.outer_size(),
    ) {
        let area = m.work_area();
        let x = p.x.clamp(
            area.position.x,
            (area.position.x + area.size.width as i32 - size.width as i32).max(area.position.x),
        );
        let y = p.y.clamp(
            area.position.y,
            (area.position.y + area.size.height as i32 - size.height as i32).max(area.position.y),
        );
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }
}
#[tauri::command]
fn resize_main(window: tauri::WebviewWindow, height: f64) -> Result<(), String> {
    if window.label() != "main" || !height.is_finite() {
        return Err("Invalid resize request".into());
    }
    let max = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .map(|m| f64::from(m.work_area().size.height) / m.scale_factor() - 16.0)
        .unwrap_or(740.0);
    window
        .set_size(tauri::LogicalSize::new(
            390.0,
            height.clamp(320.0, max.max(320.0)),
        ))
        .map_err(|e| e.to_string())?;
    position(&window);
    Ok(())
}
fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        position(&window);
        let _ = window.show();
        let _ = window.set_focus();
    }
}
#[cfg(windows)]
fn play_sound() {
    #[link(name = "winmm")]
    extern "system" {
        fn PlaySoundW(psz_sound: *const u16, module: *mut std::ffi::c_void, flags: u32) -> i32;
    }
    let sound: Vec<u16> = "SystemNotification".encode_utf16().chain(Some(0)).collect();
    unsafe {
        PlaySoundW(
            sound.as_ptr(),
            std::ptr::null_mut(),
            0x00010000 | 0x00000001 | 0x00000002,
        );
    }
}
#[cfg(not(windows))]
fn play_sound() {}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if !args.iter().any(|a| a == "--background") {
                show_main(app);
            }
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--background"])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            detect_daily_folder,
            action,
            open_daily,
            open_app_data,
            resize_main
        ])
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            #[cfg(debug_assertions)]
            let data_dir = std::env::var_os("TRAY_TASKS_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or(data_dir);
            std::fs::create_dir_all(&data_dir)?;
            let engine = Engine::new(data_dir).map_err(std::io::Error::other)?;
            let first_run = engine.data.settings.vault_path.is_empty();
            let settings = engine.data.settings.clone();
            let dirty = Arc::new(AtomicBool::new(true));
            app.manage(AppState {
                engine: Mutex::new(engine),
                dirty: dirty.clone(),
            });
            let open =
                MenuItem::with_id(app, "open", "Open tasks / 할 일 열기", true, None::<&str>)?;
            let daily = MenuItem::with_id(
                app,
                "daily",
                "Open daily note / 데일리 노트",
                true,
                None::<&str>,
            )?;
            let config = MenuItem::with_id(app, "settings", "Settings / 설정", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit / 완전히 종료", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &daily, &config, &quit])?;
            TrayIconBuilder::with_id("tray")
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .ok_or("App icon is missing")?,
                )
                .tooltip("Tray Tasks · Obsidian daily notes")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "settings" => {
                        let _ = app.emit_to("main", "show-settings", ());
                        show_main(app);
                    }
                    "daily" => {
                        let state = app.state::<AppState>();
                        if let Err(e) = open_daily(app.clone(), state) {
                            let _ = app.emit("app-error", e);
                            show_main(app);
                        }
                    }
                    "quit" => {
                        let state = app.state::<AppState>();
                        if let Ok(e) = state.engine.lock() {
                            let _ = e.persist();
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                show_main(tray.app_handle());
                            }
                        }
                    }
                })
                .build(app)?;
            if let Err(error) = apply_icon(app.handle(), &settings) {
                let state = app.state::<AppState>();
                if let Ok(mut engine) = state.engine.lock() {
                    engine.error = Some(format!(
                        "Custom tray icon could not be loaded; using the default icon. {error}"
                    ));
                };
            }
            if first_run {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.center();
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut watched = String::new();
                let mut watcher: Option<notify::RecommendedWatcher> = None;
                let mut last_snapshot = String::new();
                let mut last_error = String::new();
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let state = app_handle.state::<AppState>();
                    let Ok(mut engine) = state.engine.lock() else {
                        break;
                    };
                    let now = chrono::Local::now();
                    let new_alert = match engine.tick_at(now.timestamp(), now.date_naive()) {
                        Ok(value) => value,
                        Err(e) => {
                            if e != last_error {
                                engine.error = Some(e.clone());
                                last_error = e;
                            }
                            false
                        }
                    };
                    let folder = engine.data.settings.vault_path.clone();
                    if folder != watched {
                        watcher = None;
                        watched = folder.clone();
                        if !folder.is_empty() {
                            let marker = dirty.clone();
                            if let Ok(mut w) = notify::recommended_watcher(
                                move |_: notify::Result<notify::Event>| {
                                    marker.store(true, Ordering::Relaxed);
                                },
                            ) {
                                if w.watch(
                                    std::path::Path::new(&folder),
                                    notify::RecursiveMode::Recursive,
                                )
                                .is_ok()
                                {
                                    watcher = Some(w);
                                }
                            }
                        }
                    }
                    let _ = watcher.as_ref();
                    let snapshot = engine.snapshot();
                    drop(engine);
                    let serialized = serde_json::to_string(&snapshot).unwrap_or_default();
                    if serialized != last_snapshot || dirty.swap(false, Ordering::Relaxed) {
                        let _ = app_handle.emit("snapshot", &snapshot);
                        last_snapshot = serialized;
                    }
                    if let Some(w) = app_handle.get_webview_window("reminder") {
                        if !snapshot.alerts.is_empty() || snapshot.rollover.is_some() {
                            let height = (180
                                + snapshot.alerts.len() * 84
                                + if snapshot.rollover.is_some() { 220 } else { 0 })
                            .min(650) as f64;
                            let max = w
                                .current_monitor()
                                .ok()
                                .flatten()
                                .map(|m| {
                                    f64::from(m.work_area().size.height) / m.scale_factor() - 16.0
                                })
                                .unwrap_or(650.0);
                            let _ = w.set_size(tauri::LogicalSize::new(370.0, height.min(max)));
                            position(&w);
                            let _ = w.show();
                            if new_alert && snapshot.settings.sound {
                                play_sound();
                            }
                        } else {
                            let _ = w.hide();
                        }
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("Tray Tasks could not start");
}
