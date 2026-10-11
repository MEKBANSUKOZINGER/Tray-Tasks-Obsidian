//! Dock a small window alongside the primary Windows notification area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
impl Rect {
    pub fn width(self) -> i32 {
        self.right - self.left
    }
    pub fn height(self) -> i32 {
        self.bottom - self.top
    }
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}
pub fn dock(
    bar: Rect,
    notification: Option<Rect>,
    size: [u32; 2],
    gap: i32,
    fallback_x: i32,
) -> [i32; 2] {
    let w = size[0] as i32;
    let h = size[1] as i32;
    if bar.width() >= bar.height() {
        let end = notification
            .filter(|r| r.width() > 0 && r.left > bar.left && r.left < bar.right)
            .map(|r| r.left - gap);
        let x = end
            .map(|end| end - w)
            .unwrap_or(fallback_x)
            .clamp(bar.left, (bar.right - w).max(bar.left));
        [x, bar.top + (bar.height() - h) / 2]
    } else {
        // Vertical taskbars have too little width; keep the widget beside them.
        let y = notification
            .map(|r| r.top - gap - h)
            .unwrap_or(bar.bottom - h - gap)
            .clamp(bar.top, (bar.bottom - h).max(bar.top));
        [
            if bar.left <= 0 {
                bar.right + gap
            } else {
                bar.left - w - gap
            },
            y,
        ]
    }
}
#[cfg(windows)]
mod native {
    use super::Rect;
    use windows_sys::Win32::{
        Foundation::{HWND, POINT, RECT},
        UI::WindowsAndMessaging::{
            FindWindowExW, FindWindowW, GetCursorPos, GetWindowRect, SetWindowPos, HWND_TOPMOST,
            SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        },
    };
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    unsafe fn rect(hwnd: HWND) -> Option<Rect> {
        let mut r = RECT::default();
        if hwnd.is_null()
            || GetWindowRect(hwnd, &mut r) == 0
            || r.right <= r.left
            || r.bottom <= r.top
        {
            return None;
        }
        Some(Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        })
    }
    pub fn taskbar() -> Option<(Rect, Option<Rect>)> {
        // Shell window classes are an optional hint, never a required dependency.
        unsafe {
            let bar = FindWindowW(wide("Shell_TrayWnd").as_ptr(), std::ptr::null());
            let bounds = rect(bar)?;
            let notify = FindWindowExW(
                bar,
                std::ptr::null_mut(),
                wide("TrayNotifyWnd").as_ptr(),
                std::ptr::null(),
            );
            Some((bounds, rect(notify)))
        }
    }
    // Post across input queues so shutdown can join the event thread without blocking the UI.
    pub fn keep_above(hwnd: isize) {
        unsafe {
            SetWindowPos(
                hwnd as HWND,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
        }
    }
    use std::{cell::Cell, sync::mpsc, thread};
    use windows_sys::Win32::{
        Foundation::{GetLastError, SetLastError},
        System::Threading::GetCurrentThreadId,
        UI::{
            Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
            WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, GetWindow, GetWindowLongPtrW, IsWindow,
                IsWindowVisible, PeekMessageW, PostThreadMessageW, SetWindowLongPtrW,
                TranslateMessage, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_REORDER,
                EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, GWLP_HWNDPARENT, GW_HWNDPREV, GW_OWNER,
                MSG, PM_NOREMOVE, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_QUIT,
            },
        },
    };
    thread_local! {static GUARDED_WINDOW:Cell<isize>=const {Cell::new(0)};}
    fn shell_window() -> HWND {
        unsafe { FindWindowW(wide("Shell_TrayWnd").as_ptr(), std::ptr::null()) }
    }
    pub fn bind_owner(hwnd: isize) -> Result<(), String> {
        unsafe {
            let widget = hwnd as HWND;
            let shell = shell_window();
            if shell.is_null() || IsWindow(widget) == 0 {
                return Ok(());
            }
            if GetWindow(widget, GW_OWNER) != shell {
                SetLastError(0);
                let previous = SetWindowLongPtrW(widget, GWLP_HWNDPARENT, shell as isize);
                if previous == 0 && GetLastError() != 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                #[cfg(debug_assertions)]
                eprintln!(
                    "Widget owner pinned to taskbar; hwnd={hwnd}, owner={}",
                    shell as isize
                );
            }
            Ok(())
        }
    }
    fn needs_raise(widget: HWND) -> bool {
        unsafe {
            let mut above = GetWindow(widget, GW_HWNDPREV);
            for _ in 0..4096 {
                if above.is_null() {
                    return false;
                }
                // A newly raised topmost window could overlap the widget. Avoid needless redraws.
                if IsWindowVisible(above) != 0 {
                    if let (Some(a), Some(b)) = (rect(widget), rect(above)) {
                        if a.left < b.right
                            && a.right > b.left
                            && a.top < b.bottom
                            && a.bottom > b.top
                        {
                            return true;
                        }
                    }
                }
                above = GetWindow(above, GW_HWNDPREV);
            }
            false
        }
    }
    unsafe extern "system" fn on_window_event(
        _: HWINEVENTHOOK,
        event: u32,
        changed: HWND,
        object: i32,
        _: i32,
        _: u32,
        _: u32,
    ) {
        if event != EVENT_SYSTEM_FOREGROUND && object != 0 {
            return;
        }
        GUARDED_WINDOW.with(|target| {
            let hwnd = target.get();
            let widget = hwnd as HWND;
            if hwnd == 0 || changed == widget || IsWindowVisible(widget) == 0 {
                return;
            }
            if needs_raise(widget) {
                keep_above(hwnd);
            }
        });
    }
    pub struct TopmostGuard {
        thread_id: u32,
        thread: Option<thread::JoinHandle<()>>,
        widget: isize,
        previous_owner: isize,
    }
    impl TopmostGuard {
        pub fn install(widget: isize) -> Result<Self, String> {
            let previous_owner = unsafe { GetWindowLongPtrW(widget as HWND, GWLP_HWNDPARENT) };
            bind_owner(widget)?;
            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            let thread = thread::spawn(move || {
                GUARDED_WINDOW.with(|target| target.set(widget));
                unsafe {
                    let mut message = MSG::default();
                    PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
                    let mut hooks = Vec::new();
                    for event in [
                        EVENT_SYSTEM_FOREGROUND,
                        EVENT_OBJECT_REORDER,
                        EVENT_OBJECT_SHOW,
                        EVENT_OBJECT_LOCATIONCHANGE,
                    ] {
                        let hook = SetWinEventHook(
                            event,
                            event,
                            std::ptr::null_mut(),
                            Some(on_window_event),
                            0,
                            0,
                            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                        );
                        if hook.is_null() {
                            for hook in hooks {
                                UnhookWinEvent(hook);
                            }
                            let _ = ready_tx.send(Err(std::io::Error::last_os_error().to_string()));
                            return;
                        }
                        hooks.push(hook);
                    }
                    if ready_tx.send(Ok(GetCurrentThreadId())).is_err() {
                        for hook in hooks {
                            UnhookWinEvent(hook);
                        }
                        return;
                    }
                    while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                    for hook in hooks {
                        UnhookWinEvent(hook);
                    }
                    GUARDED_WINDOW.with(|target| target.set(0));
                }
            });
            match ready_rx.recv().map_err(|e| e.to_string())? {
                Ok(thread_id) => Ok(Self {
                    thread_id,
                    thread: Some(thread),
                    widget,
                    previous_owner,
                }),
                Err(error) => {
                    let _ = thread.join();
                    unsafe {
                        SetWindowLongPtrW(widget as HWND, GWLP_HWNDPARENT, previous_owner);
                    }
                    Err(error)
                }
            }
        }
    }
    impl Drop for TopmostGuard {
        fn drop(&mut self) {
            unsafe {
                PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
            }
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
            unsafe {
                if IsWindow(self.widget as HWND) != 0 {
                    let owner = if IsWindow(self.previous_owner as HWND) != 0 {
                        self.previous_owner
                    } else {
                        0
                    };
                    SetWindowLongPtrW(self.widget as HWND, GWLP_HWNDPARENT, owner);
                }
            }
        }
    }

    pub fn cursor() -> Option<[i32; 2]> {
        let mut p = POINT::default();
        if unsafe { GetCursorPos(&mut p) } == 0 {
            None
        } else {
            Some([p.x, p.y])
        }
    }
}
#[cfg(windows)]
pub use native::{bind_owner, cursor, keep_above, taskbar, TopmostGuard};

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn owned_widget_stays_above_owner_without_a_timer() {
        use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};
        struct Window(HWND);
        impl Drop for Window {
            fn drop(&mut self) {
                unsafe {
                    DestroyWindow(self.0);
                }
            }
        }
        unsafe extern "system" fn collect(hwnd: HWND, state: isize) -> i32 {
            (*(state as *mut Vec<HWND>)).push(hwnd);
            1
        }
        let class: Vec<u16> = "STATIC".encode_utf16().chain(Some(0)).collect();
        unsafe {
            let create = || {
                let hwnd = CreateWindowExW(
                    WS_EX_TOOLWINDOW,
                    class.as_ptr(),
                    std::ptr::null(),
                    WS_POPUP | WS_VISIBLE,
                    -32000,
                    -32000,
                    1,
                    1,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                );
                assert!(!hwnd.is_null(), "{}", std::io::Error::last_os_error());
                Window(hwnd)
            };
            let owner = create();
            let widget = create();
            SetWindowLongPtrW(widget.0, GWLP_HWNDPARENT, owner.0 as isize);
            assert_eq!(GetWindow(widget.0, GW_OWNER), owner.0);
            assert_ne!(
                SetWindowPos(
                    widget.0,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                ),
                0,
                "{}",
                std::io::Error::last_os_error()
            );
            for _ in 0..100 {
                assert_ne!(
                    SetWindowPos(
                        owner.0,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                    ),
                    0,
                    "{}",
                    std::io::Error::last_os_error()
                );
                // GetWindow traversal can revisit windows when unrelated apps reorder them.
                let mut order: Vec<HWND> = Vec::new();
                assert_ne!(EnumWindows(Some(collect), &mut order as *mut _ as isize), 0);
                let widget_index = order
                    .iter()
                    .position(|&w| w == widget.0)
                    .expect("Missing widget");
                let owner_index = order
                    .iter()
                    .position(|&w| w == owner.0)
                    .expect("Missing owner");
                assert!(widget_index < owner_index, "Owner overtook its widget");
            }
            SetWindowLongPtrW(widget.0, GWLP_HWNDPARENT, 0);
        }
    }

    #[test]
    fn notification_area_dock_stays_left_and_vertically_centered() {
        let bar = Rect {
            left: 0,
            top: 1392,
            right: 2560,
            bottom: 1440,
        };
        let notification = Rect {
            left: 2238,
            top: 1392,
            right: 2560,
            bottom: 1440,
        };
        assert_eq!(
            dock(bar, Some(notification), [280, 40], 8, 1950),
            [1950, 1396]
        );
    }
    #[test]
    fn fallback_ignores_old_y_and_clamps_x_to_current_screen() {
        let bar = Rect {
            left: -1920,
            top: 1040,
            right: 0,
            bottom: 1080,
        };
        assert_eq!(dock(bar, None, [280, 40], 8, 1950), [-280, 1040]);
        assert_eq!(dock(bar, None, [280, 40], 8, -500), [-500, 1040]);
        assert_eq!(
            dock(
                bar,
                Some(Rect {
                    left: -1920,
                    top: 1040,
                    right: 0,
                    bottom: 1080
                }),
                [280, 40],
                8,
                -500
            ),
            [-500, 1040]
        );
    }
    #[test]
    fn top_and_vertical_taskbars_use_their_own_bounds() {
        let top = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 48,
        };
        assert_eq!(dock(top, None, [280, 40], 8, 1000), [1000, 4]);
        let side = Rect {
            left: 0,
            top: 0,
            right: 48,
            bottom: 1080,
        };
        assert_eq!(dock(side, None, [280, 40], 8, 1000), [56, 1032]);
    }
}
