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
pub fn write_atomic(path: &Path, data: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid file path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.to_string())?;
    Ok(())
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
    temporary.persist(path).map_err(|e| e.to_string())?;
    Ok(())
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
