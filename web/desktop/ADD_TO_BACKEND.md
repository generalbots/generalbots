# Add These 2 Commands to drive.rs

Your `drive.rs` already has `list_files`, `upload_file`, and `create_folder`.

Just add these 2 commands for the text editor to work:

## 1. Read File Command

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

## 2. Write File Command

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

## 3. Delete File Command (Optional but recommended)

```rust
#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    use std::fs;
    
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
```

## Register in main.rs

Add to your invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    ui::drive::read_file,
    ui::drive::write_file,
    ui::drive::delete_file,  // optional
])
```

## That's it!

The frontend Drive module is already configured to use these commands via:
- `window.__TAURI__.invoke("read_file", { path })`
- `window.__TAURI__.invoke("write_file", { path, content })`
- `window.__TAURI__.invoke("delete_file", { path })`

The UI will automatically detect if Tauri is available and use the backend, or fall back to demo mode.