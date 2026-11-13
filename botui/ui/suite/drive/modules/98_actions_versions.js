/* Drive Module v2.0 — 98 File Actions (version history) */
"use strict";

// ── Version history (server-backed, GB7-14) ──────────────────────────
async function showVersionHistory(path) {
    var bucket = driveBucketForPath(path);
    var fileName = path.split("/").pop() || "file";
    var storageKey = "gb_file_versions_" + (bucket || "") + "_" + path;
    var localVersions = [];
    try { localVersions = gbStoreGet(storageKey, []); } catch (e) {}

    var modal = document.getElementById("version-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "version-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;backdrop-filter:blur(8px);";
    var container = document.createElement("div");
    container.style.cssText = "background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:560px;max-height:80vh;display:flex;flex-direction:column;box-shadow:0 25px 50px -12px rgba(0,0,0,0.5);overflow:hidden;";
    var header = '<div style="display:flex;justify-content:space-between;align-items:center;padding:16px 24px;border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;font-size:1.125rem;">Version History: ' + escapeHtml(fileName) + '</h3><button onclick="document.getElementById(\'version-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>';
    container.innerHTML = header + '<div style="padding:24px;overflow-y:auto;flex:1;color:#94a3b8;">Loading versions…</div>';
    modal.appendChild(container);
    modal.addEventListener("click", function(e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);

    var serverVersions = [];
    try {
        var payload = await apiRequest("/versions/list", {
            method: "POST",
            body: JSON.stringify({
                path: path,
                bucket: bucket,
                scope: currentScope,
            }),
        });
        serverVersions = (payload && payload.versions) || [];
    } catch (err) {
        console.warn("Version history unavailable on the server:", err);
    }

    var body = '<div style="padding:24px;overflow-y:auto;flex:1;">';
    if (serverVersions.length === 0 && localVersions.length === 0) {
        body += '<p style="color:#94a3b8;text-align:center;">No previous versions recorded.<br>Versions are saved automatically when you overwrite a file.</p>';
    } else {
        serverVersions.forEach(function(version) {
            var date = version.stamp || "";
            body += '<div style="display:flex;justify-content:space-between;align-items:center;padding:10px 12px;border:1px solid #334155;border-radius:8px;margin-bottom:8px;">'
                + '<div><div style="color:#f8fafc;font-size:14px;">' + escapeHtml(date) + '</div><div style="color:#64748b;font-size:12px;">' + formatFileSize(version.size || 0) + ' &middot; stored in Drive</div></div>'
                + '<button onclick="DriveModule.restoreServerVersion(\'' + escapeJs(path) + '\',\'' + escapeJs(version.version) + '\',\'' + escapeJs(bucket || '') + '\')" style="background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:6px 12px;cursor:pointer;font-size:12px;">Restore</button>'
                + '</div>';
        });
        localVersions.forEach(function(v, i) {
            var date = new Date(v.timestamp).toLocaleString();
            body += '<div style="display:flex;justify-content:space-between;align-items:center;padding:10px 12px;border:1px dashed #334155;border-radius:8px;margin-bottom:8px;">'
                + '<div><div style="color:#f8fafc;font-size:14px;">' + escapeHtml(v.label || "Local version") + '</div><div style="color:#64748b;font-size:12px;">' + escapeHtml(date) + ' &middot; ' + formatFileSize(v.size || 0) + ' &middot; this device</div></div>'
                + '<button onclick="DriveModule.restoreVersion(\'' + escapeJs(path) + '\',' + i + ',\'' + escapeJs(bucket || '') + '\')" style="background:#475569;border:none;border-radius:6px;color:#fff;padding:6px 12px;cursor:pointer;font-size:12px;">Restore</button>'
                + '</div>';
        });
    }
    body += '</div>';
    container.innerHTML = header + body;
}

async function restoreServerVersion(path, version, bucket) {
    var targetBucket = bucket || driveBucketForPath(path);
    try {
        await apiRequest("/versions/restore", {
            method: "POST",
            body: JSON.stringify({
                path: path,
                version: version,
                bucket: targetBucket,
                scope: currentScope,
            }),
        });
        showNotification("Version restored", "success");
        var modal = document.getElementById("version-modal");
        if (modal) modal.remove();
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Restore failed: " + humanizeError(err.message), "error");
    }
}

async function restoreVersion(path, index, bucket) {
    var targetBucket = bucket || driveBucketForPath(path);
    var storageKey = "gb_file_versions_" + (targetBucket || "") + "_" + path;
    var versions = [];
    try { versions = gbStoreGet(storageKey, []); } catch (e) {}
    if (!versions[index] || !versions[index].content) { showNotification("Version data not available", "error"); return; }
    try {
        await apiRequest("/write", {
            method: "POST",
            body: JSON.stringify({ bucket: targetBucket, path: path, content: versions[index].content, scope: currentScope }),
        });
        showNotification("Version restored", "success");
        var modal = document.getElementById("version-modal");
        if (modal) modal.remove();
        loadFiles(currentPath, currentBucket);
    } catch (err) {
        showNotification("Restore failed: " + humanizeError(err.message), "error");
    }
}

function saveFileVersion(path, content, bucket) {
    var versionBucket = bucket || driveBucketForPath(path);
    var storageKey = "gb_file_versions_" + (versionBucket || "") + "_" + path;
    var versions = [];
    try { versions = gbStoreGet(storageKey, []); } catch (e) {}
    versions.unshift({ timestamp: Date.now(), label: "Auto-save", size: content.length, content: content });
    if (versions.length > 20) versions = versions.slice(0, 20);
    try { localStorage.setItem(storageKey, JSON.stringify(versions)); } catch (e) {}
}
