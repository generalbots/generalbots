/* Drive Module — 05 Upload: recursive folder/file-tree upload */
"use strict";

// A drop can carry directories, which DataTransfer.files flattens to nothing.
// Read every entry synchronously — DataTransferItem handles are only valid
// while the drop handler runs.
function webkitEntriesFromTransfer(transfer) {
    var entries = [];
    if (!transfer || !transfer.items) return entries;
    for (var i = 0; i < transfer.items.length; i++) {
        var item = transfer.items[i];
        if (!item || item.kind !== "file") continue;
        var entry = item.webkitGetAsEntry ? item.webkitGetAsEntry() : null;
        if (entry) entries.push(entry);
    }
    return entries;
}

// A directory reader returns entries in batches (usually 100 at a time) and
// an empty batch means the directory is exhausted — keep reading until then.
function readDirectoryBatch(dirEntry) {
    return new Promise(function (resolve, reject) {
        var reader = dirEntry.createReader();
        var all = [];
        function readNext() {
            reader.readEntries(function (batch) {
                if (!batch || batch.length === 0) { resolve(all); return; }
                all = all.concat(Array.prototype.slice.call(batch));
                readNext();
            }, reject);
        }
        readNext();
    });
}

// Flatten an entry tree into folders-first upload plan. Paths stay relative to
// the drop/pick root, so writing under currentPath preserves the folder shape.
async function collectUploadPlan(entries, base, plan) {
    for (var i = 0; i < entries.length; i++) {
        var entry = entries[i];
        if (!entry || !entry.name) continue;
        var rel = base ? base + "/" + entry.name : entry.name;
        if (entry.isDirectory) {
            plan.dirs.push(rel);
            try {
                var children = await readDirectoryBatch(entry);
                await collectUploadPlan(children, rel, plan);
            } catch (err) {
                // Keep walking siblings: one unreadable directory must not
                // abort the rest of the drop, but it is reported at the end.
                if (!plan.errors) plan.errors = [];
                plan.errors.push(rel);
                console.error("Folder read failed for " + rel + ":", err);
            }
        } else if (entry.isFile) {
            plan.files.push({ entry: entry, relPath: rel });
        }
    }
}

// Fallback for browsers without input.webkitEntries: webkitRelativePath still
// carries the picked folder tree, so derive the directory list from it.
function planFromFileList(fileList) {
    var plan = { dirs: [], files: [] };
    var seenDirs = {};
    Array.prototype.forEach.call(fileList, function (file) {
        var rel = (file.webkitRelativePath || file.name || "").replace(/^\/+/, "");
        var parts = rel.split("/");
        for (var i = 1; i < parts.length; i++) {
            var dir = parts.slice(0, i).join("/");
            if (!seenDirs[dir]) {
                seenDirs[dir] = true;
                plan.dirs.push(dir);
            }
        }
        plan.files.push({ file: file, relPath: rel });
    });
    plan.dirs.sort();
    return plan;
}

function entryToFile(planItem) {
    if (planItem.file) return Promise.resolve(planItem.file);
    return new Promise(function (resolve, reject) {
        planItem.entry.file(resolve, reject);
    });
}

// Create every directory before writing files: an empty directory is a
// zero-byte marker object server-side, so it survives in the listing.
async function createPlanDirectories(plan, basePath, bucket) {
    var failures = [];
    for (var i = 0; i < plan.dirs.length; i++) {
        var relDir = plan.dirs[i];
        var slash = relDir.lastIndexOf("/");
        var name = slash === -1 ? relDir : relDir.slice(slash + 1);
        var parent = slash === -1
            ? basePath
            : (basePath ? basePath + "/" + relDir.slice(0, slash) : relDir.slice(0, slash));
        try {
            await apiRequest("/createFolder", {
                method: "POST",
                body: JSON.stringify({
                    bucket: bucket,
                    path: parent,
                    name: name,
                    scope: currentScope,
                }),
            });
        } catch (err) {
            failures.push(relDir);
            console.error("Folder create failed for " + relDir + ":", err);
        }
    }
    return failures;
}

async function uploadPlan(plan) {
    var basePath = currentPath;
    var bucket = getEffectiveBucket();
    var total = plan.files.length;
    var uploaded = 0;
    var failed = [];
    var progressEl = showUploadProgress(total);

    var label = document.getElementById("gb-upload-label");
    if (label && plan.dirs.length) label.textContent = "Creating folders...";
    var dirFailures = (plan.errors || []).slice();
    dirFailures = dirFailures.concat(await createPlanDirectories(plan, basePath, bucket));

    for (var i = 0; i < total; i++) {
        var item = plan.files[i];
        updateUploadProgress(progressEl, i, total, item.relPath);
        try {
            var file = await entryToFile(item);
            var content = await readFileAsBase64(file);
            await apiRequest("/write", {
                method: "POST",
                body: JSON.stringify({
                    bucket: bucket,
                    path: basePath ? basePath + "/" + item.relPath : item.relPath,
                    content: content,
                    scope: currentScope,
                }),
            });
            uploaded++;
        } catch (err) {
            failed.push(item.relPath);
            console.error("Upload failed for " + item.relPath + ":", err);
        }
    }

    var failedCount = failed.length + dirFailures.length;
    if (failedCount === 0) {
        hideUploadProgress(progressEl);
        if (total > 0) {
            showNotification("Uploaded " + uploaded + " file(s)" + (plan.dirs.length ? " in " + plan.dirs.length + " folder(s)" : ""), "success");
        } else {
            showNotification("Created " + plan.dirs.length + " folder(s)", "success");
        }
    } else {
        // #1751 — per-file failures must be visible, never swallowed. Keep the
        // progress panel showing the count and surface a warning/error toast.
        var names = failed.concat(dirFailures);
        var preview = names.slice(0, 3).join(", ") + (names.length > 3 ? " +" + (names.length - 3) + " more" : "");
        var failLabel = document.getElementById("gb-upload-label");
        var failDetail = document.getElementById("gb-upload-detail");
        if (failLabel) failLabel.textContent = failedCount + " item(s) failed";
        if (failDetail) failDetail.textContent = uploaded + " of " + total + " file(s) uploaded";
        setTimeout(function () {
            if (progressEl && progressEl.parentNode) progressEl.remove();
        }, 15000);
        var summary = total > 0
            ? "Uploaded " + uploaded + " of " + total + " file(s)"
            : "Folder creation incomplete";
        showNotification(summary + "; " + failedCount + " failed: " + preview,
            uploaded === 0 && failed.length > 0 ? "error" : "warning");
    }
    loadFiles(currentPath, currentBucket);
}

async function uploadEntries(entries) {
    if (!entries || entries.length === 0) return;
    var plan = { dirs: [], files: [] };
    try {
        await collectUploadPlan(entries, "", plan);
    } catch (err) {
        showNotification("Could not read the dropped folder: " + err.message, "error");
        return;
    }
    if (plan.dirs.length === 0 && plan.files.length === 0) {
        showNotification("The dropped folder is empty", "warning");
        return;
    }
    await uploadPlan(plan);
}

function hasDirectoryEntry(entries) {
    for (var i = 0; i < entries.length; i++) {
        if (entries[i] && entries[i].isDirectory) return true;
    }
    return false;
}

function handleDroppedTransfer(transfer) {
    var entries = webkitEntriesFromTransfer(transfer);
    // Only take the tree path when a directory is involved; a plain file drop
    // keeps the existing flat upload (and its retry panel).
    if (hasDirectoryEntry(entries)) {
        uploadEntries(entries);
        return;
    }
    var files = transfer && transfer.files;
    if (files && files.length > 0) {
        uploadFiles(Array.from(files));
        return;
    }
    if (entries.length > 0) {
        uploadEntries(entries);
        return;
    }
    // File items without webkitGetAsEntry (unsupported browser): say so
    // instead of silently doing nothing on a folder drop.
    var hasFileItem = false;
    if (transfer && transfer.items) {
        for (var i = 0; i < transfer.items.length; i++) {
            if (transfer.items[i] && transfer.items[i].kind === "file") hasFileItem = true;
        }
    }
    if (hasFileItem) {
        showNotification('This browser cannot read dropped folders. Use "Upload folder" instead.', "warning");
    }
}

function handleFolderInputChange(e) {
    var input = e.target;
    var entries = input.webkitEntries ? Array.prototype.slice.call(input.webkitEntries) : [];
    var files = input.files;
    input.value = "";
    // Browser without directory entries: rebuild the tree from
    // webkitRelativePath instead of writing every file at the root.
    if (entries.length > 0 && hasDirectoryEntry(entries)) {
        uploadEntries(entries);
    } else if (files && files.length > 0) {
        uploadPlan(planFromFileList(files));
    } else if (entries.length > 0) {
        uploadEntries(entries);
    }
}

function triggerFolderUpload() {
    var input = document.getElementById("folder-input");
    if (input) input.click();
}

window.uploadFolder = triggerFolderUpload;
