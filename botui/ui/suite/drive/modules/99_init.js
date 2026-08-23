/* Drive Module v2.0 — 99 Init */
"use strict";

async function init() {
    await fetchUserInfo();

    // Show Root tab only for admins
    var rootTab = document.getElementById("tab-root");
    if (rootTab) rootTab.style.display = isAdmin ? "flex" : "none";

    bindTopTabs();
    bindNavigation();
    bindViewToggle();
    bindDragAndDrop();
    bindContextMenu();
    bindKeyboardShortcuts();
    bindUploadButton();
    bindNewFolderButton();
    bindSearchInput();
    bindRefreshBotsBtn();
    bindBotSearchInput();
    bindNewBotBtn();

    var savedTab = sessionStorage.getItem("drive-tab") || TAB_BRANCHDRIVE;
    switch (savedTab) {
        case TAB_BRANCHDRIVE:
            await loadBranchDriveTab();
            break;
        case TAB_SHARED:
            await loadSharedTab();
            break;
        case TAB_PUBLIC:
            await loadPublicTab();
            break;
        case TAB_MYFILES:
            await loadMyFilesTab();
            break;
        case TAB_BOTS:
            await loadBotsTab();
            break;
        case TAB_ROOT:
            if (isAdmin) await loadRootTab();
            else await loadBranchDriveTab();
            break;
        default:
            await loadBranchDriveTab();
            break;
    }
    loadStorageInfo();
}

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
        loadStorageInfo();
    } catch (err) {
        showNotification("Delete failed: " + err.message, "error");
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
    loadStorageInfo();
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

// Paste the in-app clipboard directly into a folder (right-click on folder).
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

// Open (or focus) the AI assistant panel with this file as context.
function askAIOnFile(path) {
    if (window.DriveAI && typeof window.DriveAI.openWithFile === "function") {
        window.DriveAI.openWithFile(path);
    } else if (window.DriveModule && typeof window.DriveModule.showAIPanel === "function") {
        window.DriveModule.showAIPanel();
        if (window.DriveModule.setAIFile) window.DriveModule.setAIFile(path);
    }
}

// Open the threaded comments + presence sidebar for a file.
function openComments(path) {
    if (window.DriveComments && typeof window.DriveComments.open === "function") {
        window.DriveComments.open(path);
    }
}

// Upload files pasted from the OS clipboard (images/screenshots).
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

window.DriveModule = {
    selectedFiles: selectedFiles,
    init: init,
    loadFiles: loadFiles,
    loadBotConfigs: loadBotConfigs,
    loadStorageInfo: loadStorageInfo,
    discoverBuckets: discoverBuckets,
    retryWithBackoff: retryWithBackoff,
    toggleSelection: toggleSelection,
    selectAll: selectAll,
    clearSelection: clearSelection,
    downloadFile: downloadFile,
    previewFile: previewFile,
    shareFile: shareFile,
    copyShareLink: copyShareLink,
    revokeShareLink: revokeShareLink,
    copyLink: copyLink,
    duplicateItem: duplicateItem,
    copyPathToClipboard: copyPathToClipboard,
    pasteInto: pasteInto,
    askAIOnFile: askAIOnFile,
    openComments: openComments,
    openFile: openFile,
    deleteItem: deleteItem,
    deleteSelected: deleteSelected,
    renameItem: renameItem,
    createFolder: createFolder,
    copyToClipboard: copyToClipboard,
    cutToClipboard: cutToClipboard,
    copySelected: copySelected,
    cutSelected: cutSelected,
    pasteFiles: pasteFiles,
    showContextMenuFor: showContextMenuFor,
    navigateUp: navigateUp,
    openSharedFile: openSharedFile,
    loadBranchDriveTab: loadBranchDriveTab,
    loadSharedTab: loadSharedTab,
    loadPublicTab: loadPublicTab,
    loadMyFilesTab: loadMyFilesTab,
    loadBotsTab: loadBotsTab,
    loadRootTab: loadRootTab,
};

// Deep-link from a desktop shortcut: jump straight to the file's folder
// once the module is ready (window.__gbAppParams__.path, set by the shell).
var __deepPath = (window.__gbAppParams__ || {}).path;
if (__deepPath) {
    var __origInit = init;
    init = async function () {
        await __origInit();
        try { await loadFiles(__deepPath); } catch (e) { /* default listing */ }
    };
}

if (document.readyState === "loading") {
    (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
} else {
    init();
}
