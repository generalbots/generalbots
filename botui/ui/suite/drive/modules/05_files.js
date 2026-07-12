/* Drive Module v2.0 — 05 Files: discover, load, upload, search */
"use strict";

async function discoverBuckets() {
    try {
        var botName = window.__INITIAL_BOT_NAME__ || window.location.pathname.split('/').filter(Boolean)[0] || '';
        var url = botName ? '/buckets?bot=' + encodeURIComponent(botName) : '/buckets';
        const buckets = await apiRequest(url);
        availableBuckets = buckets || [];
        retryCount = 0;

        // Prefer the user's own org bucket (from suite session) over first alphabetically
        var myBucketName = (userInfo && userInfo.bucket) || "";
        var myBucket = null;
        if (myBucketName) {
            myBucket = availableBuckets.find(function(b) { return b.name === myBucketName; });
        }
        var gborg = myBucket || availableBuckets.find(function(b) { return b.is_gborg; });
        var gbai = availableBuckets.find(function(b) { return b.is_gbai; });

        if (gborg) {
            currentGborgBucket = gborg.name;
            var shortName = gborg.name.replace(".gborg", "");
            currentGborgBranch = shortName;
            currentBucket = gborg.name;
        } else if (gbai) {
            currentGborgBucket = null;
            currentGborgBranch = null;
            currentBucket = gbai.name;
        } else if (availableBuckets.length > 0) {
            currentGborgBucket = null;
            currentGborgBranch = null;
            currentBucket = availableBuckets[0].name;
        }

        updateBucketSelector();

        if (!currentBucket) {
            const content = document.getElementById("drive-content") || document.getElementById("file-grid");
            if (content) {
                content.innerHTML = '<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg><h3>No drive storage found</h3><p>Please contact your administrator to set up storage.</p></div>';
            }
        }

    } catch (err) {
        console.error("Failed to discover buckets:", err);
        const content = document.getElementById("drive-content") || document.getElementById("file-grid");
        if (content) {
            var canRetry = retryCount < MAX_RETRIES;
            var retryMsg = canRetry
                ? '<button class="btn-primary" onclick="DriveModule.retryWithBackoff()">Retry</button>'
                : '<p class="text-muted">Max retries reached. Please refresh the page.</p>';
            content.innerHTML = '<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg><h3>Drive connection error</h3><p>' + escapeHtml(err.message) + '</p>' + retryMsg + '</div>';
        }
    }
}

async function loadFiles(path, bucket) {
    if (path !== undefined) currentPath = path;
    if (bucket !== undefined) currentBucket = bucket;

    const effectiveBucket = getEffectiveBucket();
    if (!effectiveBucket) {
        await discoverBuckets();
        return;
    }

    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading files...</p></div>';
    updateBreadcrumb();

    try {
        const params = new URLSearchParams();
        if (effectiveBucket) params.set("bucket", effectiveBucket);
        if (currentPath) params.set("path", currentPath);
        params.set("scope", currentScope);

        var files = await apiRequest("/list?" + params.toString());

        renderFiles(files);
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Failed to load files</h3><p>' + escapeHtml(err.message) + '</p><button class="btn-primary" onclick="DriveModule.loadFiles()">Retry</button></div>';
    }
}

async function loadStorageInfo() {
    try {
        const quota = await apiRequest("/quota");
        const usedEl = document.getElementById("storage-used");
        const fillEl = document.getElementById("storage-fill");
        const detailEl = document.getElementById("storage-detail");
        if (usedEl) usedEl.textContent = formatFileSize(quota.used_bytes) + " of " + formatFileSize(quota.total_bytes);
        if (fillEl) fillEl.style.width = (quota.percentage_used || 0) + "%";
        if (detailEl) detailEl.textContent = formatFileSize(quota.available_bytes) + " available";
    } catch (err) {
        console.error("Failed to load storage info:", err);
    }
}

async function uploadFiles(files) {
    showNotification("Uploading " + files.length + " file(s)...", "info");
    var uploaded = 0;
    var failed = 0;
    for (const file of files) {
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
    if (failed === 0) showNotification("Uploaded " + uploaded + " file(s)", "success");
    else showNotification("Uploaded " + uploaded + ", " + failed + " failed", "warning");
    loadFiles(currentPath, currentBucket);
    loadStorageInfo();
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

async function searchFiles(query) {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Searching...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("query", query);
        if (currentBucket) params.set("bucket", getEffectiveBucket());
        params.set("scope", currentScope);
        const files = await apiRequest("/search?" + params.toString());
        renderFiles(files);
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Search failed</h3></div>';
    }
}

// ── Tab Path Builders ────────────────────────────────────────────
function buildPathBranchDrive() {
    if (!currentGborgBranch) return "";
    return currentGborgBranch + ".gbai/" + currentGborgBranch + ".gbdrive";
}

function buildPathShared() {
    if (!currentGborgBranch) return "";
    return currentGborgBranch + ".gbai/shared.gbdrive";
}

function buildPathPublic() {
    if (!currentGborgBranch) return "";
    return currentGborgBranch + ".gbai/public.gbdrive";
}

function buildPathMyFiles() {
    if (!currentGborgBranch) return "";
    var login = (userInfo.username || "unknown").toLowerCase();
    return currentGborgBranch + ".gbai/users.gbdrive/" + login;
}

function buildPathRoot() {
    if (!currentGborgBranch) return "";
    return currentGborgBranch + ".gbai";
}

// ── Tab Loaders ───────────────────────────────────────────────────
async function loadBranchDriveTab() {
    var path = buildPathBranchDrive();
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

async function loadBotsTab() {
    if (!currentBucket) await discoverBuckets();
    await loadBotConfigs();
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
