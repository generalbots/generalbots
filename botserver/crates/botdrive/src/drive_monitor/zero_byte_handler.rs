use std::path::Path;

pub fn is_file_empty_or_missing(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }
    match std::fs::metadata(path) {
        Ok(meta) => meta.len() == 0,
        Err(_) => true,
    }
}

pub fn scan_for_empty_files(work_dir: &Path) -> Vec<String> {
    let mut needs_sync = Vec::new();
    if !work_dir.exists() {
        return needs_sync;
    }
    if let Ok(entries) = std::fs::read_dir(work_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let bot_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let gbdialog = path.join(format!("{}.gbdialog", bot_name));
                if gbdialog.exists() {
                    if let Ok(files) = std::fs::read_dir(&gbdialog) {
                        for file in files.flatten() {
                            let fpath = file.path();
                            if fpath.extension().map_or(false, |e| e == "bas" || e == "ast") {
                                if is_file_empty_or_missing(&fpath) {
                                    needs_sync.push(bot_name.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    needs_sync
}
