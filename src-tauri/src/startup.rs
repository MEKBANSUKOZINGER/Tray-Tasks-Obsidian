//! Windows login registration uses a stable name and an explicitly quoted executable.
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
pub const APP_NAME: &str = "Tray Tasks";
#[cfg(windows)]
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

#[cfg(windows)]
fn command(executable: &Path) -> Result<String, String> {
    let path = crate::paths::display_path(executable);
    if !executable.is_absolute() || path.contains(['"', '\0', '\r', '\n']) {
        return Err("Invalid startup executable path.".into());
    }
    Ok(format!("\"{path}\" --background"))
}
#[cfg(windows)]
pub fn apply(executable: &Path, enabled: bool, explicit: bool) -> Result<(), String> {
    apply_named(APP_NAME, executable, enabled, explicit)
}
#[cfg(windows)]
fn apply_named(name: &str, executable: &Path, enabled: bool, explicit: bool) -> Result<(), String> {
    use windows_registry::{Type, CURRENT_USER};
    let command = command(executable)?;
    let run = CURRENT_USER.create(RUN).map_err(|e| e.to_string())?;
    if !enabled {
        if let Err(error) = run.remove_value(name) {
            if std::io::Error::from(error).kind() != std::io::ErrorKind::NotFound {
                return Err("Could not remove the Windows login entry.".into());
            }
        }
        return Ok(());
    }
    // Refresh the path even after reinstalling or moving the executable.
    run.set_string(name, command).map_err(|e| e.to_string())?;
    let approved = CURRENT_USER.create(APPROVED).map_err(|e| e.to_string())?;
    if explicit {
        approved
            .set_bytes(name, Type::Bytes, &[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            .map_err(|e| e.to_string())?;
    } else if approved
        .get_value(name)
        .is_ok_and(|v| matches!(v.first(), Some(3 | 7)))
    {
        return Err("Windows has disabled Tray Tasks startup. Enable it in Windows Startup apps or save the autostart setting again.".into());
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows_registry::{Type, CURRENT_USER};
    struct Entry(String);
    impl Drop for Entry {
        fn drop(&mut self) {
            for key in [RUN, APPROVED] {
                if let Ok(key) = CURRENT_USER.create(key) {
                    let _ = key.remove_value(&self.0);
                }
            }
        }
    }
    #[test]
    fn quotes_spaces_unicode_and_extended_executable_paths() {
        assert_eq!(
            command(Path::new(
                r"\\?\C:\Program Files\트레이 작업\tray-tasks.exe"
            ))
            .unwrap(),
            "\"C:\\Program Files\\트레이 작업\\tray-tasks.exe\" --background"
        );
        assert!(command(Path::new("tray-tasks.exe")).is_err());
        assert!(command(Path::new("C:\\bad\"path.exe")).is_err());
    }
    #[test]
    fn repairs_missing_and_stale_entries_without_overriding_windows_disable() {
        let entry = Entry(format!("Tray Tasks test {}", uuid::Uuid::new_v4()));
        let exe = Path::new(r"C:\Program Files\Tray Tasks\tray-tasks.exe");
        apply_named(&entry.0, exe, true, false).unwrap();
        let run = CURRENT_USER.create(RUN).unwrap();
        assert_eq!(run.get_string(&entry.0).unwrap(), command(exe).unwrap());
        run.set_string(&entry.0, "old-path").unwrap();
        let approved = CURRENT_USER.create(APPROVED).unwrap();
        let disabled = [3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0];
        approved
            .set_bytes(&entry.0, Type::Bytes, &disabled)
            .unwrap();
        assert!(apply_named(&entry.0, exe, true, false).is_err());
        assert_eq!(run.get_string(&entry.0).unwrap(), command(exe).unwrap());
        assert_eq!(&*approved.get_value(&entry.0).unwrap(), &disabled);
        apply_named(&entry.0, exe, true, true).unwrap();
        assert_eq!(approved.get_value(&entry.0).unwrap()[0], 2);
        apply_named(&entry.0, exe, false, true).unwrap();
        assert!(run.get_string(&entry.0).is_err());
    }
    #[test]
    fn invalid_path_keeps_the_existing_registration() {
        let entry = Entry(format!("Tray Tasks test {}", uuid::Uuid::new_v4()));
        let run = CURRENT_USER.create(RUN).unwrap();
        run.set_string(&entry.0, "existing-entry").unwrap();
        assert!(apply_named(&entry.0, Path::new("relative.exe"), true, true).is_err());
        assert_eq!(run.get_string(&entry.0).unwrap(), "existing-entry");
    }
}
