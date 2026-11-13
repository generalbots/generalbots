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
        if (!(await gbCopyText(fullUrl))) throw new Error("clipboard unavailable");
        showNotification("Link copied to clipboard", "success");
    } catch (err) {
        showNotification("Copy link failed: " + err.message, "error");
    }
}

async function copyPathToClipboard(path) {
    try {
        if (!(await gbCopyText(path))) throw new Error("clipboard unavailable");
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

// #1866 — Shared folder / drop box foundation.
// A drop box lets a user publish a folder so others can upload files into it
// without a full account. The P2P relay (botpublic crate) that will mint the
// URL and enforce the upload-only policy is tracked in #1866. This stub shows
// the user where the feature will appear so the surface is not a surprise when
// it ships.
function shareFolder(path, bucket) {
    var folderName = path.split("/").pop() || "folder";
    var modal = document.getElementById("share-folder-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "share-folder-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;backdrop-filter:blur(8px);";
    var container = document.createElement("div");
    container.style.cssText = "background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:480px;padding:24px;box-shadow:0 25px 50px -12px rgba(0,0,0,0.5);";
    container.innerHTML =
        '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px;">' +
        '<h3 style="margin:0;color:#f8fafc;font-size:1.25rem;">Share folder &ldquo;' + folderName.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;") + '&rdquo;</h3>' +
        '<button onclick="document.getElementById(\'share-folder-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>' +
        '<div style="border:1px solid #334155;border-radius:8px;padding:16px;background:#0f172a;margin-bottom:16px;">' +
        '<p style="margin:0 0 8px;color:#94a3b8;font-size:13px;line-height:1.5;">' +
        '<strong style="color:#f8fafc;">Drop box link</strong> &mdash; a URL others can visit to upload files into this folder without signing in. ' +
        'Your files stay private; they can only add, not read.</p>' +
        '<p style="margin:0;color:#64748b;font-size:12px;">Sharing coming soon &mdash; link will appear here once the botpublic relay ships (#1866).</p>' +
        '</div>' +
        '<div style="display:flex;justify-content:flex-end;">' +
        '<button onclick="document.getElementById(\'share-folder-modal\').remove()" style="background:#334155;border:none;border-radius:6px;color:#f8fafc;padding:8px 16px;cursor:pointer;font-size:13px;">Close</button>' +
        '</div>';
    modal.appendChild(container);
    modal.addEventListener("click", function(e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);
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
