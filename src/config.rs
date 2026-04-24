//! Tiny persistent config. We keep it deliberately simple — a single text
//! file next to the CEF profile — so there's no TOML/JSON dep pulled in just
//! for one value.

use std::path::{Path, PathBuf};

fn profile_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("io.alexieff.rustyteams"))
}

fn external_browser_file() -> Option<PathBuf> {
    profile_dir().map(|p| p.join("external_browser.txt"))
}

/// Returns the user-configured external browser, if any. When unset (or the
/// file is missing/empty), callers should fall back to the OS default.
pub fn external_browser() -> Option<PathBuf> {
    let path = external_browser_file()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

pub fn set_external_browser(browser: Option<&Path>) -> std::io::Result<()> {
    let path = external_browser_file()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no LOCALAPPDATA"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match browser {
        Some(b) => std::fs::write(path, b.to_string_lossy().as_ref()),
        None => match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        },
    }
}
