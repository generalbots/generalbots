/* Drive Module v2.0 — 98 File Actions (clipboard + trash) */
"use strict";

// Split from 98_actions.js for #1731. Loaded right after the core file;
// functions stay global, exactly as before.

var clipboardFileBuckets = {};

function copyToClipboard(path, bucket) {
    clipboardFiles = [path];
    clipboardFileBuckets = {};
    clipboardFileBuckets[path] = bucket || driveBucketForPath(path);
    clipboardOperation = "copy";
    showNotification("Copied to clipboard", "info");
}

function cutToClipboard(path, bucket) {
    clipboardFiles = [path];
    clipboardFileBuckets = {};
    clipboardFileBuckets[path] = bucket || driveBucketForPath(path);
    clipboardOperation = "cut";
    showNotification("Cut to clipboard", "info");
}

function copySelected() {
    clipboardFiles = Array.from(selectedFiles);
    clipboardFileBuckets = {};
    clipboardFiles.forEach(function (path) {
        clipboardFileBuckets[path] = driveBucketForPath(path);
    });
    clipboardOperation = "copy";
    showNotification(clipboardFiles.length + " item(s) copied", "info");
}

function cutSelected() {
    clipboardFiles = Array.from(selectedFiles);
    clipboardFileBuckets = {};
    clipboardFiles.forEach(function (path) {
        clipboardFileBuckets[path] = driveBucketForPath(path);
    });
    clipboardOperation = "cut";
    showNotification(clipboardFiles.length + " item(s) cut", "info");
}

async function pasteFiles() {
    if (clipboardFiles.length === 0) return;
    const operation = clipboardOperation;
    var processed = 0;
    var failures = [];
    for (const sourcePath of clipboardFiles) {
        const fileName = sourcePath.split("/").pop();
        const destPath = currentPath ? currentPath + "/" + fileName : fileName;
        try {
            const endpoint = operation === "copy" ? "/copy" : "/move";
            const sourceBucket = clipboardFileBuckets[sourcePath] || driveBucketForPath(sourcePath);
            await apiRequest(endpoint, {
                method: "POST",
                body: JSON.stringify({
                    source_bucket: sourceBucket,
                    source_path: sourcePath,
                    dest_bucket: currentBucket || sourceBucket,
                    dest_path: destPath,
                    scope: currentScope,
                }),
            });
            processed++;
        } catch (err) {
            failures.push(sourcePath);
            console.error("Failed to " + operation + " " + sourcePath + ":", err);
        }
    }
    if (operation === "cut") { clipboardFiles = []; clipboardFileBuckets = {}; clipboardOperation = null; }
    if (failures.length) {
        showNotification(
            (operation === "copy" ? "Copied" : "Moved") + " " + processed + " of " +
                (processed + failures.length) + " item(s); failed: " + failures.join(", "),
            "warning"
        );
    } else {
        showNotification((operation === "copy" ? "Copied" : "Moved") + " " + processed + " item(s)", "success");
    }
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
