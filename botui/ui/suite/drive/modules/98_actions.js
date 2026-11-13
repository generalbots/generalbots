/* Drive Module v2.0 — 98 File Actions: delete, export, rename */
"use strict";

/* Drive Module v2.0 — 98 File Actions */
"use strict";

// ── File Actions ─────────────────────────────────────────────
function driveConfirmAction(msg, title) {
    return new Promise(function (resolve) {
        if (window.GBModal && typeof window.GBModal.confirm === "function") {
            try {
                window.GBModal.confirm(msg, title || "Confirm", function () { resolve(true); }, function () { resolve(false); });
                return;
            } catch (e) { /* fall through */ }
        }
        resolve(false);
    });
}

async function deleteItem(path, bucket) {
    const fileName = path.split("/").pop();
    var ok = await driveConfirmAction('Move "' + fileName + '" to Trash? It can be restored for 30 days.', "Confirm");
    if (!ok) return;
    try {
        // #1640 — user deletes go to Trash, not a hard delete; permanent
        // deletion only happens through Empty Trash.
        await apiRequest("/trash/move", {
            method: "POST",
            body: JSON.stringify({ bucket: bucket || driveBucketForPath(path), path: path, scope: currentScope }),
        });
        showNotification("Moved to Trash", "success");
        selectedFiles.delete(path);
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Delete failed: " + err.message, "error");
    }
}

// #1751 — selection rows can come from another bucket (search, starred,
// shared). Export accepts one bucket per request, so group by row bucket and
// build one archive per bucket instead of sending the browse bucket for all.
function groupPathsByBucket(paths) {
    var groups = {};
    (paths || []).forEach(function (path) {
        var bucket = driveBucketForPath(path) || getEffectiveBucket() || "";
        if (!groups[bucket]) groups[bucket] = [];
        groups[bucket].push(path);
    });
    return groups;
}

async function downloadSelected() {
    if (selectedFiles.size === 0) return;
    var paths = [];
    var folders = [];
    Array.from(selectedFiles).forEach(function (p) {
        var el = document.querySelector('[data-path="' + CSS.escape(p) + '"]');
        if (el && el.dataset.type === "folder") folders.push(p);
        else paths.push(p);
    });
    if (paths.length === 0 && folders.length === 0) {
        showNotification("Select files or a folder to download", "warning");
        return;
    }
    // #1639 — one archive instead of N separate downloads. A folder selection
    // exports recursively; mixed selections produce one archive per folder
    // plus one for the loose files.
    // #1751 — a mixed-bucket selection downloads one archive per bucket.
    var folderGroups = groupPathsByBucket(folders);
    var fileGroups = groupPathsByBucket(paths);
    var buckets = Object.keys(folderGroups);
    Object.keys(fileGroups).forEach(function (bucket) {
        if (buckets.indexOf(bucket) === -1) buckets.push(bucket);
    });
    var failures = [];
    for (var bi = 0; bi < buckets.length; bi++) {
        var bucket = buckets[bi];
        try {
            var bucketFolders = folderGroups[bucket] || [];
            for (var i = 0; i < bucketFolders.length; i++) {
                await exportPaths([], bucketFolders[i], bucket);
            }
            var bucketFiles = fileGroups[bucket] || [];
            if (bucketFiles.length) await exportPaths(bucketFiles, null, bucket);
        } catch (err) {
            failures.push((bucket || "default bucket") + ": " + err.message);
        }
    }
    if (failures.length) {
        showNotification("Export failed for " + failures.join("; "), "warning");
    }
}

async function exportPaths(paths, folder, bucket) {
    showNotification("Building archive" + (folder ? " for " + folder : "") + "...", "info");
    var token = "";
    try {
        token =
            (window.getGBAccessToken && window.getGBAccessToken()) ||
            localStorage.getItem("gb-access-token") ||
            sessionStorage.getItem("gb-access-token") ||
            "";
    } catch (e) {}
    var res = await fetch(API_BASE + "/export", {
        method: "POST",
        headers: Object.assign(
            { "Content-Type": "application/json" },
            token ? { Authorization: "Bearer " + token } : {},
        ),
        body: JSON.stringify({
            bucket: bucket || getEffectiveBucket(),
            scope: currentScope,
            paths: paths || [],
            folder: folder || null,
        }),
    });
    if (!res.ok) {
        var detail = await res.json().catch(function () { return {}; });
        throw new Error(detail.error || ("HTTP " + res.status));
    }
    var blob = await res.blob();
    var archiveName = folder ? folder.split("/").pop() + ".zip" : "general-bots-export.zip";
    if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
        var saved = await window.GBShared.saveBlob(blob, archiveName);
        if (!saved) showNotification("Archive downloaded", "success");
    } else {
        var url = URL.createObjectURL(blob);
        var link = document.createElement("a");
        link.href = url;
        link.download = archiveName;
        document.body.appendChild(link);
        link.click();
        link.remove();
        setTimeout(function () { URL.revokeObjectURL(url); }, 5000);
        showNotification("Archive downloaded", "success");
    }
}

async function deleteSelected() {
    if (selectedFiles.size === 0) return;
    const count = selectedFiles.size;
    var ok = await driveConfirmAction("Move " + count + " item(s) to Trash? They can be restored for 30 days.", "Confirm");
    if (!ok) return;
    var deleted = 0;
    for (const path of selectedFiles) {
        try {
            // #1640 — same Trash route as the single-item delete.
            await apiRequest("/trash/move", {
                method: "POST",
                body: JSON.stringify({ bucket: driveBucketForPath(path), path: path, scope: currentScope }),
            });
            deleted++;
        } catch (err) {
            console.error("Failed to trash " + path + ":", err);
        }
    }
    showNotification("Moved " + deleted + " of " + count + " item(s) to Trash", deleted === count ? "success" : "warning");
    clearSelection();
    loadFiles(currentPath, currentBucket);
}

async function renameItem(path, bucket) {
    const oldName = path.split("/").pop();
    var newName = null;
    if (window.GBModal && typeof window.GBModal.prompt === "function") {
        newName = await new Promise(function (resolve) {
            try {
                window.GBModal.prompt("Enter new name:", oldName, "Rename", function (val) { resolve(val); });
            } catch (e) { resolve(null); }
        });
    }
    if (!newName || newName === oldName || !String(newName).trim()) return;
    const parentPath = path.substring(0, path.lastIndexOf("/"));
    const newPath = parentPath ? parentPath + "/" + String(newName).trim() : String(newName).trim();
    try {
        var sourceBucket = bucket || driveBucketForPath(path);
        await apiRequest("/move", {
            method: "POST",
            body: JSON.stringify({
                source_bucket: sourceBucket,
                source_path: path,
                dest_bucket: currentBucket || sourceBucket,
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
