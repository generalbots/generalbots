use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Window};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
}

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
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        files.push(FileItem {
            name,
            path: path.to_str().unwrap_or("").to_string(),
            is_dir: path.is_dir(),
        });
    }

    // Sort directories first, then files
    files.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(files)
}

#[tauri::command]
pub async fn upload_file(window: Window, src_path: String, dest_path: String) -> Result<(), String> {
    use std::fs::File;
    use std::io::{Read, Write};

    let src = PathBuf::from(&src_path);
    let dest_dir = PathBuf::from(&dest_path);
    let dest = dest_dir.join(src.file_name().ok_or("Invalid source file")?);

    // Create destination directory if it doesn't exist
    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    }

    let mut source_file = File::open(&src).map_err(|e| e.to_string())?;
    let mut dest_file = File::create(&dest).map_err(|e| e.to_string())?;

    let file_size = source_file.metadata().map_err(|e| e.to_string())?.len();
    let mut buffer = [0; 8192];
    let mut total_read = 0;

    loop {
        let bytes_read = source_file.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }

        dest_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| e.to_string())?;
        total_read += bytes_read as u64;

        let progress = (total_read as f64 / file_size as f64) * 100.0;
        window
            .emit("upload_progress", progress)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<(), String> {
    let full_path = Path::new(&path).join(&name);
    if full_path.exists() {
        return Err("Folder already exists".into());
    }
    fs::create_dir(full_path).map_err(|e| e.to_string())?;
    Ok(())
}
