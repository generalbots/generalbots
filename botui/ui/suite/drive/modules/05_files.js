/* Drive Module v2.0 — 05 Files: load, upload, recent, trash and search */
"use strict";

/* Drive Module v2.0 — 05 Files: load, upload, search */
"use strict";

async function loadFiles(path, bucket) {
    if (path !== undefined) currentPath = path;
    if (bucket !== undefined) currentBucket = bucket;

    // #1801 — a `local:<root>` bucket stays on the local backend for every
    // nested folder; MinIO discovery is skipped entirely there.
    const localRoot = parseLocalBucket(currentBucket);
    if (localRoot) {
        currentBackend = "local";
        currentLocalRoot = localRoot;
    } else {
        currentBackend = "minio";
        await discoverBuckets();
    }

    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading files...</p></div>';
    updateBreadcrumb();

    try {
        const params = new URLSearchParams();
        if (currentBackend === "local") {
            params.set("backend", "local");
            params.set("root", currentLocalRoot);
            if (currentPath) params.set("path", currentPath);
        } else {
            const effectiveBucket = getEffectiveBucket();
            if (effectiveBucket) params.set("bucket", effectiveBucket);
            if (currentPath) params.set("path", currentPath);
            params.set("scope", currentScope);
        }

        var files = await apiRequest("/list?" + params.toString());

        if (currentBackend !== "local" && window.DriveOfflineSync) DriveOfflineSync.cacheFileListing(getEffectiveBucket(), currentPath, files);
        renderFiles(files);
    } catch (err) {
        var msg = err.message || '';
        // Show a more user-friendly message for missing buckets
        if (msg.indexOf('NoSuchBucket') !== -1 || msg.indexOf('bucket does not exist') !== -1) {
            content.innerHTML = '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg><h3 style="color:#f8fafc;">Welcome to your Drive</h3><p style="color:#94a3b8;">Drag files here or click Upload to get started.</p></div>';
        } else if (currentBackend === "local") {
            content.innerHTML = '<div class="empty-state"><h3>Could not read this folder</h3><p>' + escapeHtml(humanizeError(msg)) + '</p><button class="btn-primary" onclick="DriveModule.loadFiles(\'' + escapeJs(currentPath || "") + '\', \'' + escapeJs(localBucket(currentLocalRoot)) + '\')">Try again</button></div>';
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

// Human-readable reason for a failed upload. The auth/CSRF/RBAC middleware
// answers JSON `{error,message}` and ApiClient attaches both `status` and
// `body`, so surface them instead of silently counting failures (#1840).
function uploadFailureReason(err) {
    if (!err) return "unknown error";
    var detail = (err.body && (err.body.message || err.body.error)) || err.message || "request failed";
    return err.status ? "HTTP " + err.status + ": " + detail : detail;
}

async function uploadFiles(files) {
    var total = files.length;
    var uploaded = 0;
    var failed = 0;
    var failedFiles = [];
    var failures = [];
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
            console.error("Upload error for " + file.name + ":", err);
            failed++;
            failedFiles.push(file);
            failures.push(file.name + " (" + uploadFailureReason(err) + ")");
        }
    }
    if (failed === 0) {
        hideUploadProgress(progressEl);
        showNotification("Uploaded " + uploaded + " file(s)", "success");
    } else {
        // #1550: failed uploads are retryable instead of being lost.
        // #1840: the reason (HTTP status + server message) must be visible.
        var reason = failures.slice(0, 2).join("; ");
        if (failures.length > 2) reason += " +" + (failures.length - 2) + " more";
        var label = document.getElementById("gb-upload-label");
        var detail = document.getElementById("gb-upload-detail");
        if (label) label.textContent = failed + " upload(s) failed";
        if (detail) detail.textContent = uploaded + " uploaded: " + reason;
        var retry = document.createElement("button");
        retry.type = "button";
        retry.textContent = "Retry failed";
        retry.style.cssText = "margin-top:10px;padding:6px 10px;border:0;border-radius:6px;background:#3b82f6;color:#fff;font-size:12px;cursor:pointer;";
        retry.addEventListener("click", function () {
            if (progressEl && progressEl.parentNode) progressEl.remove();
            uploadFiles(failedFiles.slice());
        });
        if (progressEl) progressEl.appendChild(retry);
        setTimeout(function () {
            if (progressEl && progressEl.parentNode) progressEl.remove();
        }, 15000);
        showNotification("Uploaded " + uploaded + ", " + failed + " failed: " + reason, "warning");
    }
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
    var doCreate = async function (name) {
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
    };
    var wm = window.WindowManager;
    if (wm && wm.promptFloating) {
        wm.promptFloating("New folder", "Enter folder name:", "", function (name) {
            doCreate(name);
        });
        return;
    }
    var name = (window.GBModal && window.GBModal.prompt)
        ? window.GBModal.prompt("Enter folder name:", "")
        : prompt("Enter folder name:");
    await doCreate(name);
}

/* #1974 — Recent is "everything I opened", not "what happens to be in the tab
 * I am standing in": bot/workspace files and the caller's own user-scope files
 * are fetched separately and merged, newest first. One unavailable scope never
 * empties the list. */
async function fetchRecentEntries() {
    var scopes = ["bot", "user"];
    var entries = [];
    var seen = {};
    for (var i = 0; i < scopes.length; i++) {
        var scope = scopes[i];
        var params = new URLSearchParams();
        params.set("scope", scope);
        if (currentBucket) params.set("bucket", getEffectiveBucket());
        try {
            var files = await apiRequest("/recent?" + params.toString());
            (files || []).forEach(function (file) {
                var path = file.path || file.name || "";
                if (!path) return;
                var key = ((file.bucket || currentBucket || "") + "|" + path).toLowerCase();
                if (seen[key]) return;
                seen[key] = true;
                entries.push({
                    is_dir: !!file.is_dir,
                    name: file.name || path.split("/").pop(),
                    path: path,
                    size: file.size || 0,
                    modified: file.modified,
                    is_kb: !!file.is_kb,
                    is_public: !!file.is_public,
                    bucket: file.bucket || null,
                    scope: scope,
                });
            });
        } catch (err) {
            if (window.console) console.warn("drive: recent " + scope + " unavailable", err);
        }
    }
    entries.sort(function (a, b) {
        var am = a.modified ? Date.parse(a.modified) : 0;
        var bm = b.modified ? Date.parse(b.modified) : 0;
        return bm - am;
    });
    return entries;
}

async function loadRecentFiles() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    var entries = await fetchRecentEntries();
    if (!entries.length) {
        content.innerHTML = '<div class="empty-state"><h3>No recent files</h3><p>Files you open or upload show up here, from every Drive location.</p></div>';
        return;
    }
    renderFiles(entries);
}

window.fetchRecentEntries = fetchRecentEntries;


// ── Trash and search (kept in the core file: the UI test harness loads
// this file alone and calls searchFiles) ──

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
            // #1640 — retention is visible: deleted when, and how long is left.
            var expires = item.expires_at ? new Date(item.expires_at) : null;
            var daysLeft = expires && isFinite(expires.getTime())
                ? Math.ceil((expires.getTime() - Date.now()) / 86400000)
                : null;
            var retention = daysLeft === null
                ? ""
                : (daysLeft > 0 ? " · " + daysLeft + " day(s) left" : " · purge due");
            var folderTag = item.is_dir ? " (folder)" : "";
            html += '<div class="drive-file-item" data-trash-id="' + escapeHtml(item.id) + '"><div class="file-col file-name-col">' + getFileIcon(name) + '<span>' + escapeHtml(name) + folderTag + '</span></div><div class="file-col file-modified-col">Deleted ' + escapeHtml(item.deleted_at) + retention + '</div><div class="file-col file-size-col">' + formatFileSize(item.size) + '</div><div class="file-col file-actions-col"><button class="btn-primary" onclick="window.restoreTrash(\'' + escapeJs(item.id) + '\')">&#8617; Restore</button></div></div>';
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
    const everywhere = allBuckets !== false;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Searching ' + (everywhere ? 'every folder and bucket' : 'this folder') + '...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("query", query);
        // #1638 — the toolbar box always searches everywhere; the server also
        // matches text extracted from the file body (bounded scan).
        params.set("all_buckets", everywhere ? "true" : "false");
        params.set("content", "true");
        if (currentBucket) params.set("bucket", getEffectiveBucket());
        params.set("scope", currentScope);
        const files = await apiRequest("/search?" + params.toString());
        if (!files || files.length === 0) {
            content.innerHTML = '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg><h3>No matches for "' + escapeHtml(query) + '"</h3><p>Search covers file names and the text inside text, PDF, DOCX and XLSX files.</p><button class="btn-primary" onclick="DriveModule.loadFiles()">Back to files</button></div>';
            return;
        }
        renderFiles(files);
        showNotification("Found " + files.length + " result(s)", "success");
    } catch (err) {
        // #1638 — distinguish “the server could not search” from “nothing
        // matched”, otherwise a broken search looks like an empty folder.
        content.innerHTML = '<div class="empty-state"><h3>Search unavailable</h3><p>' + escapeHtml(humanizeError(err.message)) + '</p><button class="btn-primary" onclick="DriveModule.loadFiles()">Back to files</button></div>';
    }
}
