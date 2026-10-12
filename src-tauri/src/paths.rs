//! Keep canonical paths for filesystem checks; use ordinary paths for other apps and the UI.
use std::path::Path;

pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        if let Some(Component::Prefix(prefix)) = path.components().next() {
            match prefix.kind() {
                Prefix::VerbatimDisk(_) => return text[4..].to_string(),
                Prefix::VerbatimUNC(_, _) => return format!(r"\\{}", &text[8..]),
                _ => {}
            }
        }
    }
    text.into_owned()
}

pub fn obsidian_uri(path: &Path) -> Result<String, String> {
    if !path.is_absolute() {
        return Err("The daily note must have an absolute path.".into());
    }
    let path = display_path(path);
    #[cfg(windows)]
    if path.starts_with(r"\\?\") || path.starts_with(r"\\.\") {
        return Err("Obsidian needs a drive-letter or network-share path.".into());
    }
    let encoded = path
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
    Ok(format!("obsidian://open?path={encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn extended_drive_path_becomes_an_ordinary_path_and_uri() {
        let path = Path::new(r"\\?\C:\Vaults\Second Brain\2026-10-12.md");
        assert_eq!(display_path(path), r"C:\Vaults\Second Brain\2026-10-12.md");
        assert_eq!(
            obsidian_uri(path).unwrap(),
            "obsidian://open?path=C%3A%5CVaults%5CSecond%20Brain%5C2026-10-12.md"
        );
        assert_eq!(
            obsidian_uri(path).unwrap(),
            obsidian_uri(Path::new(r"C:\Vaults\Second Brain\2026-10-12.md")).unwrap()
        );
    }

    #[cfg(windows)]
    #[test]
    fn extended_unc_path_keeps_the_network_share() {
        let path = Path::new(r"\\?\UNC\server\share\Daily\2026-10-12.md");
        assert_eq!(display_path(path), r"\\server\share\Daily\2026-10-12.md");
        assert_eq!(
            obsidian_uri(path).unwrap(),
            "obsidian://open?path=%5C%5Cserver%5Cshare%5CDaily%5C2026-10-12.md"
        );
    }

    #[cfg(windows)]
    #[test]
    fn unicode_and_query_delimiters_are_encoded_once() {
        assert_eq!(
            obsidian_uri(Path::new(r"\\?\C:\Vault\한글 #&%+.md")).unwrap(),
            "obsidian://open?path=C%3A%5CVault%5C%ED%95%9C%EA%B8%80%20%23%26%25%2B.md"
        );
    }

    #[test]
    fn relative_note_paths_are_rejected() {
        assert!(obsidian_uri(Path::new("Daily/2026-10-12.md")).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn device_paths_are_not_mistaken_for_ordinary_drive_paths() {
        let device = Path::new(r"\\?\Volume{not-a-drive}\Daily\2026-10-12.md");
        assert_eq!(display_path(device), device.to_string_lossy());
        assert!(obsidian_uri(device).is_err());
    }
}
