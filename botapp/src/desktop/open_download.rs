//! "Open" for a file of the conversation: the bytes the page already fetched
//! are written to the person's Downloads folder (a new name, never over an
//! existing file, no dialog) and opened with the default program.
//!
//! The page cannot choose the folder: the command takes a bare file name and
//! refuses separators, reserved device names and program types that would run
//! code when opened.

use std::io::Write;
use std::path::{Path, PathBuf};

use super::i18n;

/// Upper bound for one file (same as the Save dialog's).
const MAX_OPEN_BYTES: usize = 512 * 1024 * 1024;
/// Longest accepted file name, in characters.
const MAX_NAME_CHARS: usize = 200;
/// Attempts at a free name (`name (1).ext` …).
const MAX_SUFFIX: u32 = 999;

/// Types Windows runs (or hands to a script host) instead of showing them.
const RUNNABLE: &[&str] = &[
    "exe", "com", "bat", "cmd", "ps1", "psm1", "vbs", "vbe", "js", "jse", "wsf", "wsh", "msi", "msp", "scr",
    "hta", "cpl", "lnk", "pif", "reg", "jar", "appref-ms", "application", "url", "scf", "inf", "dll", "sh",
];

const RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "lpt1",
    "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Why a name was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRefusal {
    Invalid,
    Runnable,
}

/// Accepts a bare file name the person can recognise; returns it trimmed.
pub fn checked_name(raw: &str) -> Result<String, NameRefusal> {
    let name = raw.trim();
    let invalid = name.is_empty()
        || name.chars().count() > MAX_NAME_CHARS
        || name.starts_with('.')
        || name.ends_with('.')
        || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
    if invalid {
        return Err(NameRefusal::Invalid);
    }
    let lower = name.to_lowercase();
    let stem = lower.split('.').next().unwrap_or("");
    if RESERVED.contains(&stem.trim()) {
        return Err(NameRefusal::Invalid);
    }
    let extension = lower.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    if RUNNABLE.contains(&extension) {
        return Err(NameRefusal::Runnable);
    }
    Ok(name.to_string())
}

/// `name`, then `stem (1).ext`, `stem (2).ext` …
fn candidate(name: &str, attempt: u32) -> String {
    if attempt == 0 {
        return name.to_string();
    }
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem} ({attempt}).{ext}"),
        _ => format!("{name} ({attempt})"),
    }
}

/// Writes `bytes` under a free name in `folder`; never replaces a file.
pub fn write_unique(folder: &Path, name: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    for attempt in 0..=MAX_SUFFIX {
        let path = folder.join(candidate(name, attempt));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(bytes)?;
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "no free file name"))
}

/// Saves the file in Downloads and opens it. Returns the saved path.
///
/// # Errors
/// A message when the name is refused, the file is too large, Downloads is
/// unknown, or writing or opening fails.
#[tauri::command]
pub async fn open_download(file_name: String, bytes: Vec<u8>) -> Result<String, String> {
    let name = checked_name(&file_name).map_err(|refusal| match refusal {
        NameRefusal::Invalid => i18n::t("download-invalid-name"),
        NameRefusal::Runnable => i18n::t("download-blocked-type"),
    })?;
    if bytes.len() > MAX_OPEN_BYTES {
        return Err(i18n::t("download-too-large"));
    }
    let folder = dirs::download_dir().ok_or_else(|| i18n::t("download-no-folder"))?;
    let target = folder.clone();
    let path = tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&target)?;
        write_unique(&target, &name, &bytes)
    })
    .await
    .map_err(|e| i18n::t_args("download-write-failed", &[("path", &folder.display().to_string()), ("error", &e.to_string())]))?
    .map_err(|e| i18n::t_args("download-write-failed", &[("path", &folder.display().to_string()), ("error", &e.to_string())]))?;
    let shown = path.to_string_lossy().to_string();
    opener::open(&path)
        .map_err(|e| i18n::t_args("download-open-failed", &[("error", &e.to_string())]))?;
    Ok(shown)
}

#[cfg(test)]
mod tests {
    use super::{candidate, checked_name, write_unique, NameRefusal};

    #[test]
    fn a_plain_name_with_accents_is_accepted() {
        assert_eq!(checked_name(" memorando_vacinação.docx "), Ok("memorando_vacinação.docx".to_string()));
        assert_eq!(checked_name("relatório (final).pdf"), Ok("relatório (final).pdf".to_string()));
    }

    #[test]
    fn paths_and_odd_names_are_refused() {
        for bad in ["", "..", ".hidden", "a/b.txt", "..\\x.txt", "C:x.txt", "a\u{0}.txt", "nome.", "CON.txt", "lpt1", "a|b"] {
            assert_eq!(checked_name(bad), Err(NameRefusal::Invalid), "{bad:?}");
        }
        assert_eq!(checked_name(&"a".repeat(201)), Err(NameRefusal::Invalid));
    }

    #[test]
    fn programs_are_never_opened() {
        for bad in ["setup.exe", "run.BAT", "x.ps1", "atalho.lnk", "page.hta", "doc.pdf.js"] {
            assert_eq!(checked_name(bad), Err(NameRefusal::Runnable), "{bad:?}");
        }
    }

    #[test]
    fn an_existing_file_is_never_replaced() {
        assert_eq!(candidate("plan.xlsx", 2), "plan (2).xlsx");
        assert_eq!(candidate("LEIAME", 1), "LEIAME (1)");
        let folder = std::env::temp_dir().join(format!("gb-open-download-{}", std::process::id()));
        std::fs::create_dir_all(&folder).ok();
        let first = write_unique(&folder, "memo.txt", b"one");
        let second = write_unique(&folder, "memo.txt", b"two");
        assert!(matches!((&first, &second), (Ok(a), Ok(b)) if a != b));
        if let (Ok(a), Ok(b)) = (first, second) {
            assert_eq!(std::fs::read(&a).ok(), Some(b"one".to_vec()));
            assert_eq!(b.file_name().and_then(|n| n.to_str()), Some("memo (1).txt"));
        }
        std::fs::remove_dir_all(&folder).ok();
    }
}
