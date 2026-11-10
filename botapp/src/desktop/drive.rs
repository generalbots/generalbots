use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{Emitter, Window};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

/// Orders directory entries the way the Drive app lists them: folders first,
/// then case-insensitive by name.
fn sort_files(files: &mut [FileItem]) {
    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
}

/// The destination of an upload: the source file name inside the destination
/// directory.
///
/// Refuses to continue when both resolve to the same file: creating the
/// destination would truncate the source before a single byte is copied.
fn upload_destination(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let name = src
        .file_name()
        .ok_or_else(|| "Invalid source file".to_string())?;
    let dest = dest_dir.join(name);
    if dest == src {
        return Err("The source and destination are the same file".to_string());
    }
    Ok(dest)
}

/// Upload progress as a percentage. An empty file (or an unknown size) reports
/// 100 so the UI never shows a stuck progress bar, and the result is clamped so
/// a file that grew mid-copy cannot report 120%.
fn upload_percent(done: u64, total: u64) -> u64 {
    if total == 0 {
        return 100;
    }
    done.saturating_mul(100)
        .checked_div(total)
        .unwrap_or(100)
        .min(100)
}

/// List files in a directory.
///
/// # Errors
/// Returns an error if the path does not exist or cannot be read.
#[tauri::command]
pub fn list_files(path: &str) -> Result<Vec<FileItem>, String> {
    let base_path = Path::new(path);
    let mut files = Vec::new();

    if !base_path.exists() {
        return Err("Path does not exist".into());
    }

    for entry in fs::read_dir(base_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let metadata = entry.metadata().ok();

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let size = metadata.as_ref().map(std::fs::Metadata::len);
        let is_dir = metadata.is_some_and(|m| m.is_dir());

        files.push(FileItem {
            name,
            path: path.to_str().unwrap_or("").to_string(),
            is_dir,
            size,
        });
    }

    sort_files(&mut files);

    Ok(files)
}

/// Upload a file to the specified destination.
///
/// # Errors
/// Returns an error if the source file is invalid or the copy operation fails.
#[tauri::command]
pub fn upload_file(window: Window, src_path: &str, dest_path: &str) -> Result<(), String> {
    let src = PathBuf::from(src_path);
    let dest_dir = PathBuf::from(dest_path);
    let dest = upload_destination(&src, &dest_dir)?;

    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    }

    let mut source_file = File::open(&src).map_err(|e| e.to_string())?;
    let mut dest_file = File::create(&dest).map_err(|e| e.to_string())?;
    let file_size = source_file.metadata().map_err(|e| e.to_string())?.len();

    let mut buffer = [0; 8192];
    let mut total_read: u64 = 0;

    loop {
        let bytes_read = source_file.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        dest_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| e.to_string())?;

        total_read += bytes_read as u64;

        let progress = upload_percent(total_read, file_size);

        window
            .emit("upload_progress", progress)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Create a new folder at the specified path.
///
/// # Errors
/// Returns an error if the folder already exists or cannot be created.
#[tauri::command]
pub fn create_folder(path: &str, name: &str) -> Result<(), String> {
    let full_path = Path::new(path).join(name);

    if full_path.exists() {
        return Err("Folder already exists".into());
    }

    fs::create_dir(&full_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete a file or folder at the specified path.
///
/// # Errors
/// Returns an error if the path does not exist or the item cannot be deleted.
#[tauri::command]
pub fn delete_path(path: &str) -> Result<(), String> {
    let target = Path::new(path);

    if !target.exists() {
        return Err("Path does not exist".into());
    }

    if target.is_dir() {
        fs::remove_dir_all(target).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(target).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get the user's home directory path.
///
/// # Errors
/// Returns an error if the home directory cannot be determined.
#[tauri::command]
pub fn get_home_dir() -> Result<String, String> {
    dirs::home_dir()
        .and_then(|p| p.to_str().map(String::from))
        .ok_or_else(|| "Could not determine home directory".into())
}

#[cfg(test)]
mod tests {
    use super::{sort_files, upload_destination, upload_percent, FileItem};
    use std::path::{Path, PathBuf};

    fn item(name: &str, is_dir: bool) -> FileItem {
        FileItem {
            name: name.to_string(),
            path: format!("/drive/{name}"),
            is_dir,
            size: None,
        }
    }

    #[test]
    fn folders_come_first_then_names_ignore_case() {
        let mut files = vec![
            item("zeta.txt", false),
            item("Beta.txt", false),
            item("alpha", true),
            item("Gamma.txt", false),
            item("delta", true),
        ];
        sort_files(&mut files);
        let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "delta", "Beta.txt", "Gamma.txt", "zeta.txt"]);
    }

    #[test]
    fn the_upload_keeps_the_source_name_inside_the_target_folder() {
        let dest = upload_destination(
            Path::new("/home/u/report.pdf"),
            Path::new("/drive/inbox"),
        )
        .expect("a valid destination");
        assert_eq!(dest, PathBuf::from("/drive/inbox/report.pdf"));
    }

    #[test]
    fn an_upload_onto_itself_is_refused() {
        // Creating this destination would truncate the source before copying.
        let refused = upload_destination(Path::new("/drive/a.txt"), Path::new("/drive"));
        assert!(refused.is_err());
        // A source with no file name cannot be placed anywhere.
        assert!(upload_destination(Path::new("/"), Path::new("/drive")).is_err());
    }

    #[test]
    fn progress_is_a_percentage_and_never_overflows() {
        assert_eq!(upload_percent(0, 0), 100);
        assert_eq!(upload_percent(0, 10), 0);
        assert_eq!(upload_percent(5, 10), 50);
        assert_eq!(upload_percent(10, 10), 100);
        // A file that grew mid-copy is clamped instead of reporting 120%.
        assert_eq!(upload_percent(2_000, 1_000), 100);
        // Saturating multiplication: a huge byte count cannot panic in debug.
        assert!(upload_percent(u64::MAX, u64::MAX) <= 100);
    }
}
