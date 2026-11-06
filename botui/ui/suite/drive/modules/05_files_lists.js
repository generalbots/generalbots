/* Drive Module v2.0 — 05 Files (lists): starred and shared listings. */
"use strict";

// Split from 05_files.js for #1731. Loaded right after the core file;
// every function here is a global like the rest of the Drive modules.

async function loadStarredFiles() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        const items = await apiRequest("/favorite?" + params.toString());
        if (!items || items.length === 0) {
            content.innerHTML = '<div class="empty-state"><h3>No starred files</h3><p>Click the ☆ button on any file (or its ⋮ menu) to add it here.</p></div>';
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

var _starredPaths = window._starredPaths = null;

function starredPathSet() {
    if (!_starredPaths) _starredPaths = window._starredPaths = {};
    return _starredPaths;
}

async function loadStarredPathSet() {
    try {
        var params = new URLSearchParams();
        params.set("scope", currentScope);
        var items = await apiRequest("/favorite?" + params.toString());
        _starredPaths = window._starredPaths = {};
        (items || []).forEach(function (item) { _starredPaths[item.path] = true; });
    } catch (err) {
        _starredPaths = window._starredPaths = _starredPaths || {};
    }
    refreshRowStars();
}

function refreshRowStars() {
    var stars = document.querySelectorAll(".star-btn[data-star-path]");
    Array.prototype.forEach.call(stars, function (btn) {
        var path = btn.getAttribute("data-star-path");
        var on = !!starredPathSet()[path] ||
            sessionStorage.getItem("drive-tab") === "starred";
        btn.classList.toggle("active", on);
        btn.textContent = on ? "\u2605" : "\u2606";
        btn.title = on ? "Unstar" : "Star";
        btn.setAttribute("data-starred", on ? "1" : "0");
    });
}

function toggleStarFromButton(btn) {
    if (!btn) return;
    var path = btn.getAttribute("data-star-path") || "";
    var bucket = btn.getAttribute("data-star-bucket") || "";
    var starred = btn.getAttribute("data-starred") === "1";
    return toggleStar(path, bucket, !starred);
}

async function toggleStar(path, bucket, starred) {
    try {
        await apiRequest("/favorite/toggle", {
            method: "POST",
            body: JSON.stringify({
                bucket: bucket || getEffectiveBucket(),
                path: path,
                starred: starred !== false,
            }),
        });
        var set = starredPathSet();
        if (starred === false) delete set[path];
        else set[path] = true;
        window._starredPaths = set;
        refreshRowStars();
        showNotification(starred === false ? "Removed from starred" : "Added to starred", "success");
        if (sessionStorage.getItem("drive-tab") === "starred") loadStarredFiles();
    } catch (err) {
        showNotification("Star failed: " + err.message, "error");
    }
}
window.toggleStar = toggleStar;

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
