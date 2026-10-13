use crate::model::Persisted;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};
use tempfile::NamedTempFile;

pub fn relative_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let rel = Path::new(relative);
    if rel.is_absolute()
        || rel.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("Choose a folder inside the vault.".into());
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("Vault cannot be opened: {e}"))?;
    let path = root.join(rel);
    let mut ancestor = path.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or("Invalid vault path")?;
    }
    if !ancestor
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err("The selected folder points outside the vault.".into());
    }
    Ok(path)
}
pub fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))
}

fn persist_temporary(
    mut temporary: NamedTempFile,
    path: &Path,
    expected: Option<&str>,
) -> Result<(), String> {
    // Retry short Windows replacement conflicts without truncating the original.
    let delays = [20, 40, 80, 160, 320, 320];
    for attempt in 0..=delays.len() {
        if let Some(expected) = expected {
            if read(path)? != expected {
                return Err("The note changed while saving; refresh and try again. Nothing was overwritten.".into());
            }
        }
        match temporary.persist(path) {
            Ok(_) => return Ok(()),
            Err(failure) => {
                let retryable = cfg!(windows)
                    && matches!(failure.error.raw_os_error(), Some(5 | 32 | 33))
                    && !path
                        .metadata()
                        .is_ok_and(|m| m.is_dir() || m.permissions().readonly());
                if !retryable || attempt == delays.len() {
                    return Err(format!(
                        "Cannot replace {}: {}. Close apps locking this file, check write permissions, then retry. The existing file was not truncated.",
                        crate::paths::display_path(path), failure.error
                    ));
                }
                temporary = failure.file;
                std::thread::sleep(std::time::Duration::from_millis(delays[attempt]));
            }
        }
    }
    unreachable!()
}
pub fn write_atomic(path: &Path, data: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid file path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    persist_temporary(temporary, path, None)
}
pub fn checked_write(
    path: &Path,
    expected: &str,
    updated: &str,
    data_dir: &Path,
) -> Result<(), String> {
    if expected == updated {
        return Ok(());
    }
    if read(path)? != expected {
        return Err(
            "The note changed while editing; refresh and try again. Nothing was overwritten."
                .into(),
        );
    }
    let backup_dir = data_dir.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let key = format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes()));
    let stamp = chrono::Utc::now().timestamp_micros();
    write_atomic(
        &backup_dir.join(format!("{stamp}-{}.md", &key[..12])),
        expected,
    )?;
    // Recheck after preparing the backup and temporary file. External editors do not share a lock with this app.
    let parent = path.parent().ok_or("Invalid path")?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .write_all(updated.as_bytes())
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    if read(path)? != expected {
        return Err(
            "The note changed while saving; refresh and try again. Nothing was overwritten.".into(),
        );
    }
    persist_temporary(temporary, path, Some(expected))
}
pub fn load(data_dir: &Path) -> Result<Persisted, String> {
    let path = data_dir.join("state.json");
    if !path.exists() {
        return Ok(Persisted::default());
    }
    serde_json::from_str(&read(&path)?)
        .map_err(|e| format!("Saved app state is invalid; restore state.json from backup: {e}"))
}
pub fn save(data_dir: &Path, data: &Persisted) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    let path = data_dir.join("state.json");
    if path.exists() {
        write_atomic(&data_dir.join("state.previous.json"), &read(&path)?)?;
    }
    write_atomic(&path, &json)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn lock_replacement(path: &Path) -> std::fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(3)
            .open(path)
            .unwrap()
    }
    #[cfg(windows)]
    #[test]
    fn temporary_windows_lock_retries_without_losing_the_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "before").unwrap();
        let lock = lock_replacement(&path);
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(120));
            drop(lock);
        });
        checked_write(&path, "before", "after", dir.path()).unwrap();
        release.join().unwrap();
        assert_eq!(read(&path).unwrap(), "after");
        let backups: Vec<_> = fs::read_dir(dir.path().join("backups")).unwrap().collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            read(&backups[0].as_ref().unwrap().path()).unwrap(),
            "before"
        );
    }
    #[cfg(windows)]
    #[test]
    fn lasting_windows_lock_reports_the_file_and_preserves_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "before").unwrap();
        let _lock = lock_replacement(&path);
        let error = checked_write(&path, "before", "after", dir.path()).unwrap_err();
        assert!(
            error.contains("note.md") && error.contains("retry"),
            "{error}"
        );
        assert_eq!(read(&path).unwrap(), "before");
    }
    #[cfg(windows)]
    #[test]
    fn external_edit_during_a_retry_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "before").unwrap();
        let mut lock = lock_replacement(&path);
        let change = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            lock.set_len(0).unwrap();
            lock.write_all(b"external").unwrap();
            lock.sync_all().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
        });
        let error = checked_write(&path, "before", "after", dir.path()).unwrap_err();
        change.join().unwrap();
        assert!(error.contains("changed"), "{error}");
        assert_eq!(read(&path).unwrap(), "external");
    }
    #[test]
    fn rejects_parent_and_absolute_paths() {
        let temp = tempfile::tempdir().unwrap();
        assert!(relative_path(temp.path(), "../escape").is_err());
        assert!(relative_path(temp.path(), r"C:\escape").is_err());
        assert!(relative_path(temp.path(), "Daily")
            .unwrap()
            .starts_with(temp.path().canonicalize().unwrap()));
    }
    #[test]
    fn concurrent_change_is_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("note.md");
        fs::write(&path, "external edit").unwrap();
        assert!(checked_write(&path, "old content", "new content", temp.path()).is_err());
        assert_eq!(read(&path).unwrap(), "external edit");
    }
    #[test]
    fn backup_preserves_original_before_atomic_replacement() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("note.md");
        fs::write(&path, "before").unwrap();
        checked_write(&path, "before", "after", temp.path()).unwrap();
        assert_eq!(read(&path).unwrap(), "after");
        let backup = fs::read_dir(temp.path().join("backups"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(read(&backup).unwrap(), "before");
    }
}
