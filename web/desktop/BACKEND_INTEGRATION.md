# Backend Integration Guide - General Bots Drive

## Overview

This document explains how to integrate the Drive module with the Rust/Tauri backend for file operations and editing.

---

## Required Backend Commands

Add these commands to your Rust backend (`src/ui/drive.rs`):

### 1. Read File Content

```rust
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    use std::fs;
    
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return Err("File does not exist".into());
    }
    
    if !file_path.is_file() {
        return Err("Path is not a file".into());
    }
    
    // Read file content as UTF-8 string
    fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}
```

### 2. Write File Content

```rust
#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    use std::fs;
    use std::io::Write;
    
    let file_path = Path::new(&path);
    
    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }
    }
    
    // Write content to file
    let mut file = fs::File::create(file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}
```

### 3. Delete File/Folder

```rust
#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    use std::fs;
    
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return Err("Path does not exist".into());
    }
    
    if file_path.is_dir() {
        // Remove directory and all contents
        fs::remove_dir_all(file_path)
            .map_err(|e| format!("Failed to delete directory: {}", e))?;
    } else {
        // Remove single file
        fs::remove_file(file_path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }
    
    Ok(())
}
```

### 4. Download File (Optional)

```rust
#[tauri::command]
pub async fn download_file(window: Window, path: String) -> Result<(), String> {
    use tauri::api::dialog::FileDialogBuilder;
    
    let file_path = Path::new(&path);
    
    if !file_path.exists() || !file_path.is_file() {
        return Err("File does not exist".into());
    }
    
    // Open file picker dialog
    let save_path = FileDialogBuilder::new()
        .set_file_name(
            file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("download")
        )
        .save_file();
    
    if let Some(dest_path) = save_path {
        std::fs::copy(&path, &dest_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
    }
    
    Ok(())
}
```

---

## Updated drive.rs (Complete)

Here's the complete `drive.rs` file with all commands:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Window};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
}

/// List files and directories in a path
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
    
    // Sort: directories first, then by name
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

/// Read file content as UTF-8 string
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return Err("File does not exist".into());
    }
    
    if !file_path.is_file() {
        return Err("Path is not a file".into());
    }
    
    fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Write content to file
#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    let file_path = Path::new(&path);
    
    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }
    }
    
    // Write content to file
    let mut file = fs::File::create(file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

/// Delete file or directory
#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return Err("Path does not exist".into());
    }
    
    if file_path.is_dir() {
        fs::remove_dir_all(file_path)
            .map_err(|e| format!("Failed to delete directory: {}", e))?;
    } else {
        fs::remove_file(file_path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }
    
    Ok(())
}

/// Upload file with progress tracking
#[tauri::command]
pub async fn upload_file(
    window: Window,
    src_path: String,
    dest_path: String,
) -> Result<(), String> {
    use std::fs::File;
    use std::io::Read;
    
    let src = PathBuf::from(&src_path);
    let dest_dir = PathBuf::from(&dest_path);
    let dest = dest_dir.join(src.file_name().ok_or("Invalid source file")?);
    
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

/// Create new folder
#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<(), String> {
    let full_path = Path::new(&path).join(&name);
    
    if full_path.exists() {
        return Err("Folder already exists".into());
    }
    
    fs::create_dir(full_path).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Download file (copy to user-selected location)
#[tauri::command]
pub async fn download_file(path: String) -> Result<(), String> {
    // For web version, this will trigger browser download
    // For Tauri, implement file picker dialog
    println!("Download requested for: {}", path);
    Ok(())
}
```

---

## Register Commands in main.rs

Add these commands to your Tauri builder:

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Existing commands...
            ui::drive::list_files,
            ui::drive::read_file,
            ui::drive::write_file,
            ui::drive::delete_file,
            ui::drive::upload_file,
            ui::drive::create_folder,
            ui::drive::download_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Frontend API Usage

The Drive JavaScript already includes these API calls:

### Load Files
```javascript
const files = await window.__TAURI__.invoke("list_files", { path: "/path" });
```

### Read File
```javascript
const content = await window.__TAURI__.invoke("read_file", { path: "/file.txt" });
```

### Write File
```javascript
await window.__TAURI__.invoke("write_file", { 
  path: "/file.txt", 
  content: "Hello World" 
});
```

### Delete File
```javascript
await window.__TAURI__.invoke("delete_file", { path: "/file.txt" });
```

### Create Folder
```javascript
await window.__TAURI__.invoke("create_folder", { 
  path: "/parent", 
  name: "newfolder" 
});
```

### Upload File
```javascript
await window.__TAURI__.invoke("upload_file", {
  srcPath: "/source/file.txt",
  destPath: "/destination/"
});
```

---

## Security Considerations

### 1. Path Validation

Add path validation to prevent directory traversal:

```rust
fn validate_path(path: &str, base_dir: &Path) -> Result<PathBuf, String> {
    let full_path = base_dir.join(path);
    let canonical = full_path
        .canonicalize()
        .map_err(|_| "Invalid path".to_string())?;
    
    if !canonical.starts_with(base_dir) {
        return Err("Access denied: path outside allowed directory".into());
    }
    
    Ok(canonical)
}
```

### 2. File Size Limits

Limit file sizes for read/write operations:

```rust
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);
    let metadata = fs::metadata(file_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err("File too large to edit (max 10MB)".into());
    }
    
    // ... rest of function
}
```

### 3. Allowed Extensions

Restrict editable file types:

```rust
const EDITABLE_EXTENSIONS: &[&str] = &[
    "txt", "md", "json", "js", "ts", "html", "css", 
    "xml", "csv", "log", "yml", "yaml", "ini", "conf"
];

fn is_editable(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| EDITABLE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}
```

---

## Error Handling

### Backend Error Types

```rust
#[derive(Debug, Serialize)]
pub enum DriveError {
    NotFound,
    PermissionDenied,
    InvalidPath,
    FileTooLarge,
    NotEditable,
    IoError(String),
}

impl From<std::io::Error> for DriveError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => DriveError::NotFound,
            std::io::ErrorKind::PermissionDenied => DriveError::PermissionDenied,
            _ => DriveError::IoError(err.to_string()),
        }
    }
}
```

### Frontend Error Handling

Already implemented in `drive.js`:

```javascript
try {
  const content = await window.__TAURI__.invoke("read_file", { path });
  this.editorContent = content;
} catch (err) {
  console.error("Error reading file:", err);
  alert(`Error opening file: ${err}`);
  this.showEditor = false;
}
```

---

## Testing

### 1. Test File Operations

```bash
# Create test directory
mkdir -p test_drive/subfolder

# Create test files
echo "Hello World" > test_drive/test.txt
echo "# Markdown" > test_drive/README.md
```

### 2. Test from Frontend

Open browser console and test:

```javascript
// List files
await window.__TAURI__.invoke("list_files", { path: "./test_drive" })

// Read file
await window.__TAURI__.invoke("read_file", { path: "./test_drive/test.txt" })

// Write file
await window.__TAURI__.invoke("write_file", { 
  path: "./test_drive/new.txt", 
  content: "Test content" 
})

// Create folder
await window.__TAURI__.invoke("create_folder", { 
  path: "./test_drive", 
  name: "newfolder" 
})

// Delete file
await window.__TAURI__.invoke("delete_file", { path: "./test_drive/new.txt" })
```

---

## Demo Mode Fallback

The frontend automatically falls back to demo mode when backend is unavailable:

```javascript
get isBackendAvailable() {
  return typeof window.__TAURI__ !== "undefined";
}

async loadFiles(path = "/") {
  if (this.isBackendAvailable) {
    // Call Tauri backend
    const files = await window.__TAURI__.invoke("list_files", { path });
    this.fileTree = this.convertToTree(files, path);
  } else {
    // Fallback to mock data for web version
    this.fileTree = this.getMockData();
  }
}
```

This allows testing the UI without the backend running.

---

## Deployment

### Development
```bash
# Run Tauri dev
cargo tauri dev
```

### Production
```bash
# Build Tauri app
cargo tauri build
```

### Web-only (without backend)
Simply serve the `web/desktop` directory - it will work in demo mode.

---

## Next Steps

1. **Implement the Rust commands** in `src/ui/drive.rs`
2. **Register commands** in `main.rs`
3. **Test file operations** from the UI
4. **Add security validation** for production
5. **Configure allowed directories** in Tauri config

---

## Additional Features (Optional)

### File Metadata
```rust
#[derive(Serialize)]
pub struct FileMetadata {
    size: u64,
    modified: SystemTime,
    created: SystemTime,
    permissions: String,
}

#[tauri::command]
pub fn get_file_metadata(path: String) -> Result<FileMetadata, String> {
    // Implementation...
}
```

### File Search
```rust
#[tauri::command]
pub fn search_files(path: String, query: String) -> Result<Vec<FileItem>, String> {
    // Implementation...
}
```

### File Preview
```rust
#[tauri::command]
pub fn preview_file(path: String) -> Result<Vec<u8>, String> {
    // Return file content as bytes for preview
}
```

---

**Status**: Ready for backend implementation  
**Frontend**: ✅ Complete  
**Backend**: ⏳ Needs implementation  
**Testing**: Ready to test once backend is implemented