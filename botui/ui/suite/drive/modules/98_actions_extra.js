/* Drive Module v2.0 — 98 File Actions (extended operations) */
"use strict";

// ── Extended file operations (enterprise drive) ─────────────────────────

function duplicateItem(path, bucket) {
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
            source_bucket: bucket || driveBucketForPath(path),
            source_path: path,
            dest_bucket: currentBucket || bucket || driveBucketForPath(path),
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
                    bucket: currentBucket || getEffectiveBucket(),
                    path: currentPath ? currentPath + "/" + file.name : file.name,
                    content: String(reader.result).split(",")[1] || reader.result,
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
