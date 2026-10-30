/* Drive Module v2.0 — 98 File Actions */
"use strict";

// ── File Actions ─────────────────────────────────────────────
async function deleteItem(path) {
    const fileName = path.split("/").pop();
    if (!confirm('Delete "' + fileName + '"?')) return;
    try {
        await apiRequest("/delete", {
            method: "POST",
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, scope: currentScope }),
        });
        showNotification("Item deleted", "success");
        selectedFiles.delete(path);
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Delete failed: " + err.message, "error");
    }
}

async function downloadSelected() {
    if (selectedFiles.size === 0) return;
    var paths = Array.from(selectedFiles).filter(function(p) {
        var el = document.querySelector('[data-path="' + CSS.escape(p) + '"]');
        return !el || el.dataset.type !== "folder";
    });
    if (paths.length === 0) { showNotification("Select files to download (folders are excluded)", "warning"); return; }
    showNotification("Downloading " + paths.length + " file(s)...", "info");
    for (var i = 0; i < paths.length; i++) {
        await downloadFile(paths[i]);
    }
}

async function deleteSelected() {
    if (selectedFiles.size === 0) return;
    const count = selectedFiles.size;
    if (!confirm("Delete " + count + " item(s)?")) return;
    var deleted = 0;
    for (const path of selectedFiles) {
        try {
            await apiRequest("/delete", {
                method: "POST",
                body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, scope: currentScope }),
            });
            deleted++;
        } catch (err) {
            console.error("Failed to delete " + path + ":", err);
        }
    }
    showNotification("Deleted " + deleted + " of " + count + " item(s)", deleted === count ? "success" : "warning");
    clearSelection();
    loadFiles(currentPath, currentBucket);
}

async function renameItem(path) {
    const oldName = path.split("/").pop();
    const newName = prompt("Enter new name:", oldName);
    if (!newName || newName === oldName || !newName.trim()) return;
    const parentPath = path.substring(0, path.lastIndexOf("/"));
    const newPath = parentPath ? parentPath + "/" + newName.trim() : newName.trim();
    try {
        await apiRequest("/move", {
            method: "POST",
            body: JSON.stringify({
                source_bucket: getEffectiveBucket(),
                source_path: path,
                dest_bucket: getEffectiveBucket(),
                dest_path: newPath,
                scope: currentScope,
            }),
        });
        showNotification('Renamed to "' + newName + '"', "success");
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Rename failed: " + err.message, "error");
    }
}

function copyToClipboard(path) {
    clipboardFiles = [path];
    clipboardOperation = "copy";
    showNotification("Copied to clipboard", "info");
}

function cutToClipboard(path) {
    clipboardFiles = [path];
    clipboardOperation = "cut";
    showNotification("Cut to clipboard", "info");
}

function copySelected() {
    clipboardFiles = Array.from(selectedFiles);
    clipboardOperation = "copy";
    showNotification(clipboardFiles.length + " item(s) copied", "info");
}

function cutSelected() {
    clipboardFiles = Array.from(selectedFiles);
    clipboardOperation = "cut";
    showNotification(clipboardFiles.length + " item(s) cut", "info");
}

async function pasteFiles() {
    if (clipboardFiles.length === 0) return;
    const operation = clipboardOperation;
    var processed = 0;
    for (const sourcePath of clipboardFiles) {
        const fileName = sourcePath.split("/").pop();
        const destPath = currentPath ? currentPath + "/" + fileName : fileName;
        try {
            const endpoint = operation === "copy" ? "/copy" : "/move";
            await apiRequest(endpoint, {
                method: "POST",
                body: JSON.stringify({
                    source_bucket: getEffectiveBucket(),
                    source_path: sourcePath,
                    dest_bucket: getEffectiveBucket(),
                    dest_path: destPath,
                    scope: currentScope,
                }),
            });
            processed++;
        } catch (err) {
            console.error("Failed to " + operation + " " + sourcePath + ":", err);
        }
    }
    if (operation === "cut") { clipboardFiles = []; clipboardOperation = null; }
    showNotification((operation === "copy" ? "Copied" : "Moved") + " " + processed + " item(s)", "success");
    loadFiles(currentPath, currentBucket);
}

// ── Trash operations ──────────────────────────────────────────────────
async function restoreTrash(id) {
    try {
        await apiRequest("/trash/restore", {
            method: "POST",
            body: JSON.stringify({ id: id }),
        });
        showNotification("Item restored", "success");
        loadTrashFiles();
    } catch (err) {
        showNotification("Restore failed: " + humanizeError(err.message), "error");
    }
}

async function emptyTrash() {
    if (!confirm("Permanently delete all items in trash? This cannot be undone.")) return;
    try {
        await apiRequest("/trash/empty", { method: "POST", body: JSON.stringify({ scope: currentScope }) });
        showNotification("Trash emptied", "success");
        loadTrashFiles();
    } catch (err) {
        showNotification("Empty trash failed: " + humanizeError(err.message), "error");
    }
}

window.restoreTrash = restoreTrash;
window.emptyTrash = emptyTrash;

// ── Version history (local fallback for desktop-local) ────────────────
function showVersionHistory(path) {
    var fileName = path.split("/").pop() || "file";
    var storageKey = "gb_file_versions_" + (getEffectiveBucket() || "") + "_" + path;
    var versions = [];
    try { versions = JSON.parse(localStorage.getItem(storageKey) || "[]"); } catch (e) {}

    var modal = document.getElementById("version-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "version-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;backdrop-filter:blur(8px);";
    var container = document.createElement("div");
    container.style.cssText = "background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:560px;max-height:80vh;display:flex;flex-direction:column;box-shadow:0 25px 50px -12px rgba(0,0,0,0.5);overflow:hidden;";
    var header = '<div style="display:flex;justify-content:space-between;align-items:center;padding:16px 24px;border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;font-size:1.125rem;">Version History: ' + escapeHtml(fileName) + '</h3><button onclick="document.getElementById(\'version-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>';
    var body = '<div style="padding:24px;overflow-y:auto;flex:1;">';
    if (versions.length === 0) {
        body += '<p style="color:#94a3b8;text-align:center;">No previous versions recorded.<br>Versions are saved automatically when you edit files.</p>';
    } else {
        versions.forEach(function(v, i) {
            var date = new Date(v.timestamp).toLocaleString();
            body += '<div style="display:flex;justify-content:space-between;align-items:center;padding:10px 12px;border:1px solid #334155;border-radius:8px;margin-bottom:8px;">'
                + '<div><div style="color:#f8fafc;font-size:14px;">' + escapeHtml(v.label || "Version " + (versions.length - i)) + '</div><div style="color:#64748b;font-size:12px;">' + escapeHtml(date) + ' &middot; ' + formatFileSize(v.size || 0) + '</div></div>'
                + '<button onclick="DriveModule.restoreVersion(\'' + escapeJs(path) + '\',' + i + ')" style="background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:6px 12px;cursor:pointer;font-size:12px;">Restore</button>'
                + '</div>';
        });
    }
    body += '</div>';
    container.innerHTML = header + body;
    modal.appendChild(container);
    modal.addEventListener("click", function(e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);
}

async function restoreVersion(path, index) {
    var storageKey = "gb_file_versions_" + (getEffectiveBucket() || "") + "_" + path;
    var versions = [];
    try { versions = JSON.parse(localStorage.getItem(storageKey) || "[]"); } catch (e) {}
    if (!versions[index] || !versions[index].content) { showNotification("Version data not available", "error"); return; }
    try {
        await apiRequest("/write", {
            method: "POST",
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, content: versions[index].content, scope: currentScope }),
        });
        showNotification("Version restored", "success");
        var modal = document.getElementById("version-modal");
        if (modal) modal.remove();
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Restore failed: " + humanizeError(err.message), "error");
    }
}

function saveFileVersion(path, content) {
    var storageKey = "gb_file_versions_" + (getEffectiveBucket() || "") + "_" + path;
    var versions = [];
    try { versions = JSON.parse(localStorage.getItem(storageKey) || "[]"); } catch (e) {}
    versions.unshift({ timestamp: Date.now(), label: "Auto-save", size: content.length, content: content });
    if (versions.length > 20) versions = versions.slice(0, 20);
    try { localStorage.setItem(storageKey, JSON.stringify(versions)); } catch (e) {}
}

// ── Extended file operations (enterprise drive) ─────────────────────────

function duplicateItem(path) {
    const fileName = path.split("/").pop() || "file";
    const dot = fileName.lastIndexOf(".");
    let base, ext;
    if (dot > 0) { base = fileName.slice(0, dot); ext = fileName.slice(dot); }
    else { base = fileName; ext = ""; }
    const parent = currentPath;
    const newName = base + " (copy)" + ext;
    const newPath = parent ? parent + "/" + newName : newName;
    apiRequest("/copy", {
        method: "POST",
        body: JSON.stringify({
            source_bucket: getEffectiveBucket(),
            source_path: path,
            dest_bucket: getEffectiveBucket(),
            dest_path: newPath,
            scope: currentScope,
        }),
    }).then(function() {
        showNotification('Duplicated as "' + newName + '"', "success");
        loadFiles(currentPath, currentBucket);
    }).catch(function(err) {
        showNotification("Duplicate failed: " + err.message, "error");
    });
}

async function copyLink(path) {
    try {
        showNotification("Creating secure link...", "info");
        var link = await apiRequest("/share-link", {
            method: "POST",
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, scope: currentScope }),
        });
        var fullUrl = window.location.origin + link.url;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(fullUrl);
        } else {
            var ta = document.createElement("textarea");
            ta.value = fullUrl;
            document.body.appendChild(ta);
            ta.select();
            document.execCommand("copy");
            document.body.removeChild(ta);
        }
        showNotification("Link copied to clipboard", "success");
    } catch (err) {
        showNotification("Copy link failed: " + err.message, "error");
    }
}

async function copyPathToClipboard(path) {
    var text = path;
    try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(text);
        } else {
            var ta = document.createElement("textarea");
            ta.value = text;
            document.body.appendChild(ta);
            ta.select();
            document.execCommand("copy");
            document.body.removeChild(ta);
        }
        showNotification("Path copied", "success");
    } catch (err) {
        showNotification("Copy path failed: " + err.message, "error");
    }
}

async function pasteInto(folderPath) {
    if (clipboardFiles.length === 0) return;
    const operation = clipboardOperation;
    var processed = 0;
    for (const sourcePath of clipboardFiles) {
        const fileName = sourcePath.split("/").pop();
        const destPath = folderPath + "/" + fileName;
        try {
            const endpoint = operation === "copy" ? "/copy" : "/move";
            await apiRequest(endpoint, {
                method: "POST",
                body: JSON.stringify({
                    source_bucket: getEffectiveBucket(),
                    source_path: sourcePath,
                    dest_bucket: getEffectiveBucket(),
                    dest_path: destPath,
                    scope: currentScope,
                }),
            });
            processed++;
        } catch (err) {
            console.error("Failed to " + operation + " " + sourcePath + ":", err);
        }
    }
    if (operation === "cut") { clipboardFiles = []; clipboardOperation = null; }
    showNotification((operation === "copy" ? "Copied" : "Moved") + " " + processed + " item(s) into " + folderPath, "success");
    loadFiles(currentPath, currentBucket);
}

function askAIOnFile(path) {
    if (window.DriveAI && typeof window.DriveAI.openWithFile === "function") {
        window.DriveAI.openWithFile(path);
    } else if (window.DriveModule && typeof window.DriveModule.showAIPanel === "function") {
        window.DriveModule.showAIPanel();
        if (window.DriveModule.setAIFile) window.DriveModule.setAIFile(path);
    }
}

function openComments(path) {
    if (window.DriveComments && typeof window.DriveComments.open === "function") {
        window.DriveComments.open(path);
    }
}

function uploadClipboardFiles(fileList) {
    const files = Array.from(fileList).filter(function(f) { return f && f.size > 0; });
    if (files.length === 0) return;
    showNotification("Uploading " + files.length + " pasted file(s)...", "info");
    files.forEach(function(file) {
        const reader = new FileReader();
        reader.onload = function() {
            apiRequest("/write", {
                method: "POST",
                body: JSON.stringify({
                    bucket: getEffectiveBucket(),
                    path: currentPath ? currentPath + "/" + file.name : file.name,
                    content: reader.result,
                    scope: currentScope,
                }),
            }).then(function() {
                showNotification("Uploaded " + file.name, "success");
                loadFiles(currentPath, currentBucket);
            }).catch(function(err) {
                showNotification("Upload failed for " + file.name + ": " + err.message, "error");
            });
        };
        reader.readAsDataURL(file);
    });
}

var _cachedRecentFiles = [];
function getRecentFileNames() { return _cachedRecentFiles; }
function cacheRecentFiles(files) {
    if (!Array.isArray(files)) return;
    _cachedRecentFiles = files.map(function (f) {
        return { name: f.name || f.key || "", path: f.path || f.key || "", size: f.size || 0 };
    }).slice(0, 30);
    window.__driveRecentFiles = _cachedRecentFiles;
}
