/* Drive Module v2.0 — 05 Files: load, upload, search */
"use strict";

async function loadFiles(path, bucket) {
    if (path !== undefined) currentPath = path;
    if (bucket !== undefined) currentBucket = bucket;

    await discoverBuckets();

    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading files...</p></div>';
    updateBreadcrumb();

    try {
        const effectiveBucket = getEffectiveBucket();
        const params = new URLSearchParams();
        if (effectiveBucket) params.set("bucket", effectiveBucket);
        if (currentPath) params.set("path", currentPath);
        params.set("scope", currentScope);

        var files = await apiRequest("/list?" + params.toString());

        if (window.DriveOfflineSync) DriveOfflineSync.cacheFileListing(effectiveBucket, currentPath, files);
        renderFiles(files);
    } catch (err) {
        var msg = err.message || '';
        // Show a more user-friendly message for missing buckets
        if (msg.indexOf('NoSuchBucket') !== -1 || msg.indexOf('bucket does not exist') !== -1) {
            content.innerHTML = '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg><h3 style="color:#f8fafc;">Welcome to your Drive</h3><p style="color:#94a3b8;">Drag files here or click Upload to get started.</p></div>';
        } else {
            if (window.DriveOfflineSync) {
                DriveOfflineSync.getCachedListing(getEffectiveBucket(), currentPath).then(function (cached) {
                    if (cached && cached.files) {
                        renderFiles(cached.files);
                        showNotification("Showing cached files (offline)", "info");
                    } else {
                        content.innerHTML = '<div class="empty-state"><h3>Could not load files</h3><p>' + escapeHtml(humanizeError(msg)) + '</p><button class="btn-primary" onclick="DriveModule.loadFiles()">Try again</button></div>';
                    }
                });
                return;
            }
            content.innerHTML = '<div class="empty-state"><h3>Could not load files</h3><p>' + escapeHtml(humanizeError(msg)) + '</p><button class="btn-primary" onclick="DriveModule.loadFiles()">Try again</button></div>';
        }
    }
}

// Fetching the quota walks every bucket in the instance, so it runs once when
// the app loads and afterwards only when the user presses the refresh button.
// Folders, uploads, deletes and tab switches deliberately do NOT trigger it.
async function loadStorageInfo() {
    const btn = document.getElementById("storage-refresh");
    const usedEl = document.getElementById("storage-used");
    if (btn) {
        btn.disabled = true;
        btn.classList.add("spinning");
    }
    if (usedEl) usedEl.textContent = "Refreshing storage...";
    try {
        const quota = await apiRequest("/quota");
        const fillEl = document.getElementById("storage-fill");
        const detailEl = document.getElementById("storage-detail");
        if (usedEl) usedEl.textContent = formatFileSize(quota.used_bytes) + " of " + formatFileSize(quota.total_bytes);
        if (fillEl) fillEl.style.width = (quota.percentage_used || 0) + "%";
        if (detailEl) detailEl.textContent = formatFileSize(quota.available_bytes) + " available";
    } catch (err) {
        console.error("Failed to load storage info:", err);
        if (usedEl) usedEl.textContent = "Storage usage unavailable";
    } finally {
        if (btn) {
            btn.disabled = false;
            btn.classList.remove("spinning");
        }
    }
}

async function uploadFiles(files) {
    var total = files.length;
    var uploaded = 0;
    var failed = 0;
    var progressEl = showUploadProgress(total);
    for (var fi = 0; fi < files.length; fi++) {
        var file = files[fi];
        updateUploadProgress(progressEl, fi, total, file.name);
        try {
            const content = await readFileAsBase64(file);
            var filePath = currentPath ? currentPath + "/" + file.name : file.name;
            await apiRequest("/write", {
                method: "POST",
                body: JSON.stringify({
                    bucket: getEffectiveBucket(),
                    path: filePath,
                    content: content,
                    scope: currentScope,
                }),
            });
            uploaded++;
        } catch (err) {
            console.error("Upload error:", err);
            failed++;
        }
    }
    hideUploadProgress(progressEl);
    if (failed === 0) showNotification("Uploaded " + uploaded + " file(s)", "success");
    else showNotification("Uploaded " + uploaded + ", " + failed + " failed", "warning");
    loadFiles(currentPath, currentBucket);
}

function showUploadProgress(total) {
    var el = document.getElementById("gb-upload-progress");
    if (el) el.remove();
    el = document.createElement("div");
    el.id = "gb-upload-progress";
    el.style.cssText = "position:fixed;bottom:24px;right:24px;background:#1e293b;border:1px solid #334155;border-radius:10px;padding:14px 18px;min-width:280px;z-index:9999;box-shadow:0 8px 24px rgba(0,0,0,0.4);";
    el.innerHTML = '<div style="color:#f8fafc;font-size:13px;margin-bottom:8px;" id="gb-upload-label">Preparing upload...</div>'
        + '<div style="background:#0f172a;border-radius:4px;height:6px;overflow:hidden;"><div id="gb-upload-bar" style="height:100%;background:#3b82f6;border-radius:4px;width:0%;transition:width 0.3s;"></div></div>'
        + '<div style="color:#64748b;font-size:11px;margin-top:6px;" id="gb-upload-detail">0 / ' + total + '</div>';
    document.body.appendChild(el);
    return el;
}

function updateUploadProgress(el, current, total, fileName) {
    if (!el) return;
    var pct = Math.round(((current) / total) * 100);
    var bar = document.getElementById("gb-upload-bar");
    var label = document.getElementById("gb-upload-label");
    var detail = document.getElementById("gb-upload-detail");
    if (bar) bar.style.width = pct + "%";
    if (label) label.textContent = "Uploading " + (fileName.length > 30 ? fileName.substring(0, 27) + "..." : fileName);
    if (detail) detail.textContent = current + " / " + total;
}

function hideUploadProgress(el) {
    if (!el) return;
    var bar = document.getElementById("gb-upload-bar");
    if (bar) bar.style.width = "100%";
    var label = document.getElementById("gb-upload-label");
    if (label) label.textContent = "Upload complete";
    setTimeout(function () { if (el.parentNode) el.remove(); }, 2000);
}

async function createFolder() {
    var name = prompt("Enter folder name:");
    if (!name || !name.trim()) return;
    try {
        await apiRequest("/createFolder", {
            method: "POST",
            body: JSON.stringify({
                bucket: getEffectiveBucket(),
                path: currentPath,
                name: name.trim(),
                scope: currentScope,
            }),
        });
        showNotification('Folder "' + name + '" created', "success");
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Failed to create folder: " + err.message, "error");
    }
}

async function loadRecentFiles() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        if (currentBucket) params.set("bucket", getEffectiveBucket());
        const files = await apiRequest("/recent?" + params.toString());
        renderFiles(files);
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>No recent files</h3></div>';
    }
}

async function loadStarredFiles() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        const items = await apiRequest("/favorite?" + params.toString());
        if (!items || items.length === 0) {
            content.innerHTML = '<div class="empty-state"><h3>No starred files</h3><p>Click the star icon on any file to add it here.</p></div>';
            return;
        }
        var html = '<div class="file-list">';
        for (const item of items) {
            var name = item.path.split('/').pop() || item.path;
            html += '<div class="drive-file-item" data-path="' + escapeHtml(item.path) + '" data-bucket="' + escapeHtml(item.bucket) + '"><div class="file-col file-name-col">' + getFileIcon(name) + '<span>' + escapeHtml(name) + '</span></div><div class="file-col file-modified-col">' + escapeHtml(item.bucket) + '</div><div class="file-col file-size-col"></div><div class="file-col file-actions-col"><button class="btn-icon-sm star-btn active" onclick="window.toggleStar(\'' + escapeJs(item.path) + '\', \'' + escapeJs(item.bucket) + '\', false)" title="Unstar">&#9733;</button></div></div>';
        }
        html += '</div>';
        content.innerHTML = html;
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>No starred files</h3></div>';
    }
}

async function loadSharedFiles() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        const items = await apiRequest("/shared?" + params.toString());
        if (!items || items.length === 0) {
            content.innerHTML = '<div class="empty-state"><h3>No shared files</h3><p>Files shared with you will appear here.</p></div>';
            return;
        }
        var html = '<div class="file-list">';
        for (const item of items) {
            var name = item.path.split('/').pop() || item.path;
            html += '<div class="drive-file-item" data-path="' + escapeHtml(item.path) + '" data-bucket="' + escapeHtml(item.bucket) + '"><div class="file-col file-name-col">' + getFileIcon(name) + '<span>' + escapeHtml(name) + '</span></div><div class="file-col file-modified-col">' + escapeHtml(item.owner_id) + '</div><div class="file-col file-size-col">' + escapeHtml(item.permissions) + '</div><div class="file-col file-actions-col"></div></div>';
        }
        html += '</div>';
        content.innerHTML = html;
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>No shared files</h3></div>';
    }
}

async function loadTrashFiles() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        const items = await apiRequest("/trash?" + params.toString());
        if (!items || items.length === 0) {
            content.innerHTML = '<div class="empty-state"><h3>Trash is empty</h3></div>';
            return;
        }
        var html = '<div style="margin-bottom:12px"><button class="btn-danger" onclick="window.emptyTrash()">Empty Trash</button></div><div class="file-list">';
        for (const item of items) {
            var name = item.original_path ? item.original_path.split('/').pop() || item.path : item.path;
            html += '<div class="drive-file-item" data-trash-id="' + escapeHtml(item.id) + '"><div class="file-col file-name-col">' + getFileIcon(name) + '<span>' + escapeHtml(name) + '</span></div><div class="file-col file-modified-col">Deleted ' + escapeHtml(item.deleted_at) + '</div><div class="file-col file-size-col">' + formatFileSize(item.size) + '</div><div class="file-col file-actions-col"><button class="btn-primary" onclick="window.restoreTrash(\'' + escapeJs(item.id) + '\')">&#8617; Restore</button></div></div>';
        }
        html += '</div>';
        content.innerHTML = html;
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Trash is empty</h3></div>';
    }
}

async function searchFiles(query, allBuckets) {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Searching' + (allBuckets ? ' everywhere' : '') + '...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("query", query);
        if (!allBuckets && currentBucket) params.set("bucket", getEffectiveBucket());
        params.set("scope", currentScope);
        const files = await apiRequest("/search?" + params.toString());
        if ((!files || files.length === 0) && !allBuckets && currentBucket) {
            return searchFiles(query, true);
        }
        renderFiles(files);
        if (files && files.length > 0) {
            showNotification("Found " + files.length + " result(s)", "success");
        } else {
            content.innerHTML = '<div class="empty-state"><h3>No results</h3><p>No files matched "' + escapeHtml(query) + '"</p></div>';
        }
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Search failed</h3><p>' + escapeHtml(humanizeError(err.message)) + '</p></div>';
    }
}

// ── Tab Path Builders ────────────────────────────────────────────
function buildPathBranchDrive() {
    if (currentGborgBranch) {
        // If bucket is .gborg, files are nested inside the branch's .gbai
        // workspace under the BOT's own directory. That directory equals the
        // branch name only when the bot is its branch's namesake; a twin such
        // as `cristo-test`, or `oppbot` in branch `opportunity-oppbot`, keeps
        // its own `{bot}.gbdrive` inside the branch workspace.
        if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
            var botName = window.__INITIAL_BOT_NAME__ || currentGborgBranch;
            return currentGborgBranch + ".gbai/" + botName + ".gbdrive";
        }
        return currentGborgBranch + ".gbdrive";
    }
    // Standalone layout: the bucket IS the bot's {bot}.gbai and its Drive
    // sits at {bot}.gbdrive. Without this the Drive tab fell back to
    // scope=user with an empty path and rendered "This folder is empty"
    // for every standalone bot (beiner in production).
    if (currentBucket && currentBucket.indexOf('.gbai') > 0) {
        return currentBucket.replace('.gbai', '.gbdrive');
    }
    return "";
}

function buildPathShared() {
    if (!currentGborgBranch) return "";
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai/shared.gbdrive";
    }
    return "shared.gbdrive";
}

function buildPathPublic() {
    if (!currentGborgBranch) return "";
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai/public.gbdrive";
    }
    return "public.gbdrive";
}

function buildPathMyFiles() {
    if (!currentGborgBranch) return "";
    var login = (userInfo ? (userInfo.username || "unknown") : "unknown").toLowerCase();
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai/users.gbdrive/" + login;
    }
    return "users.gbdrive/" + login;
}

function buildPathRoot() {
    if (!currentGborgBranch) return "";
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai";
    }
    return "";
}

// ── Tab Loaders ───────────────────────────────────────────────────
async function loadBranchDriveTab() {
    if (!currentBucket) await discoverBuckets();
    var path = buildPathBranchDrive();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else {
        currentPath = "";
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

async function loadSharedTab() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading shared items...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        const items = await apiRequest("/shared?" + params.toString());
        sharedCache = Array.isArray(items) ? items : [];
        renderSharedList(sharedCache);
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Failed to load shared items</h3></div>';
    }
}

async function loadPublicTab() {
    var path = buildPathPublic();
    if (!currentBucket) await discoverBuckets();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else if (currentBucket) {
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

async function loadMyFilesTab() {
    var path = buildPathMyFiles();
    if (!currentBucket) await discoverBuckets();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else if (currentBucket) {
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

async function loadRootTab() {
    var path = buildPathRoot();
    if (!currentBucket) await discoverBuckets();
    if (path && currentGborgBucket) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket);
    } else {
        showNotification("Root tab requires an org (.gborg) bucket", "warning");
    }
}

// ── Desktop tab (#1154): the user's Desktop folder on Drive, where
// per-user shortcuts (.gbdesktop.json) and dropped files live.
async function loadDesktopTab() {
    var path = buildPathDesktop();
    if (!currentBucket) await discoverBuckets();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else if (currentBucket) {
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

function buildPathDesktop() {
    var base = buildPathMyFiles();
    if (!base) return "";
    return base + "/Desktop";
}
