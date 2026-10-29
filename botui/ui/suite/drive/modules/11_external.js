/* Drive Module v2.0 — 11 External Drives */
"use strict";

// OneDrive and Google Drive accounts connected by this tenant. The listing is
// mirrored server-side (metadata only); the bytes are streamed from the
// provider on download, so a connected account never doubles our storage.

const EXT_BASE = "/api/external-drives";

let externalConnections = [];
let externalProvider = null;   // provider currently being browsed, or null for the account list

async function extApiRequest(endpoint, options = {}) {
    if (window.ApiClient) {
        return await window.ApiClient.request(EXT_BASE + endpoint, options);
    }
    const headers = { "Content-Type": "application/json" };
    const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (token) headers["Authorization"] = "Bearer " + token;
    const response = await fetch(EXT_BASE + endpoint, {
        headers: Object.assign({}, headers, options.headers || {}),
        ...options,
    });
    if (!response.ok) {
        const error = await response.json().catch(function () { return { error: "HTTP " + response.status }; });
        throw new Error(error.error || "Request failed");
    }
    return response.json();
}

function extContent() {
    return document.getElementById("drive-content") || document.getElementById("file-grid");
}

function extEscape(value) {
    return String(value == null ? "" : value)
        .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

// Render the connection cards: one per provider, with its state and actions.
function renderExternalConnections() {
    var content = extContent();
    if (!content) return;
    if (!externalConnections.length) {
        content.innerHTML = '<div class="empty-state"><h3>No external drive configured</h3></div>';
        return;
    }
    var cards = externalConnections.map(function (p) {
        var actions;
        if (p.connected) {
            actions =
                '<button class="btn-secondary-full" data-ext-action="browse" data-provider="' + extEscape(p.provider) + '">Browse files</button>' +
                '<button class="btn-secondary-full" data-ext-action="sync" data-provider="' + extEscape(p.provider) + '">Sync now</button>' +
                '<button class="btn-secondary-full" data-ext-action="disconnect" data-provider="' + extEscape(p.provider) + '">Disconnect</button>';
        } else if (p.configured) {
            actions = '<button class="btn-primary-full" data-ext-action="connect" data-provider="' + extEscape(p.provider) + '">Connect ' + extEscape(p.display_name) + "</button>";
        } else {
            actions = '<div class="storage-detail">No OAuth client is configured for this provider on this deployment.</div>';
        }
        var lastSync = p.last_sync ? formatDate(p.last_sync) : "never";
        var error = p.last_error ? '<div class="storage-detail" style="color:var(--danger)">' + extEscape(p.last_error) + "</div>" : "";
        return (
            '<div class="drive-card" data-provider-card="' + extEscape(p.provider) + '">' +
            '<h3>' + extEscape(p.display_name) + "</h3>" +
            '<div class="storage-detail">' + p.file_count + " file(s) mirrored &middot; last sync " + extEscape(lastSync) + "</div>" +
            '<div class="storage-detail">status: ' + extEscape(p.status) + "</div>" +
            error +
            '<div class="drive-card-actions">' + actions + "</div>" +
            "</div>"
        );
    }).join("");
    content.innerHTML =
        '<div class="empty-state" style="text-align:left">' +
        "<h3>External drives</h3>" +
        '<p>Connect OneDrive or Google Drive to browse their files alongside your Drive. Content is mirrored as metadata; downloads stream from the provider.</p>' +
        '<div class="drive-card-grid">' + cards + "</div>" +
        "</div>";
}

// Render the mirrored listing of one connected provider.
function renderExternalFiles(provider, payload) {
    var content = extContent();
    if (!content) return;
    var files = (payload && payload.files) || [];
    var header =
        '<div class="drive-toolbar-left">' +
        '<button class="breadcrumb-item" data-ext-action="back">External drives</button>' +
        '<span class="breadcrumb-separator">/</span>' +
        '<span class="breadcrumb-item">' + extEscape(provider.display_name || provider) + "</span>" +
        '<button class="btn-secondary-full" data-ext-action="sync" data-provider="' + extEscape(provider.provider || provider) + '">Sync now</button>' +
        "</div>";
    if (!files.length) {
        content.innerHTML = header + '<div class="empty-state"><h3>No files mirrored</h3><p>Run a sync to pull the account listing.</p></div>';
        return;
    }
    var rows = files.map(function (f) {
        return (
            '<div class="file-item" data-ext-file="' + extEscape(f.remote_id) + '">' +
            '<div class="file-name"><span class="file-icon">📄</span>' + extEscape(f.name) + "</div>" +
            '<div class="file-size">' + formatFileSize(f.size_bytes) + "</div>" +
            '<div class="file-date">' + (f.modified_at ? formatDate(f.modified_at) : "") + "</div>" +
            '<div class="file-actions"><button class="btn-icon" data-ext-action="download" data-provider="' +
            extEscape(f.provider) + '" data-remote-id="' + extEscape(f.remote_id) +
            '" title="Download from provider">⬇</button></div>' +
            "</div>"
        );
    }).join("");
    content.innerHTML = header + '<div class="file-list" id="external-file-list">' + rows + "</div>";
}

async function loadExternalTab() {
    var content = extContent();
    if (!content) return;
    if (externalProvider) {
        await browseExternalProvider(externalProvider);
        return;
    }
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading external drives...</p></div>';
    try {
        var payload = await extApiRequest("/connections");
        externalConnections = (payload && payload.providers) || [];
        renderExternalConnections();
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Could not load external drives</h3><p>' + extEscape(err.message) + "</p></div>";
    }
}

async function browseExternalProvider(providerId) {
    var content = extContent();
    if (content) content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading files...</p></div>';
    var known = externalConnections.find(function (p) { return p.provider === providerId; }) || { provider: providerId };
    try {
        var payload = await extApiRequest("/files?provider=" + encodeURIComponent(providerId));
        renderExternalFiles(known, payload);
    } catch (err) {
        if (content) {
            content.innerHTML = '<div class="empty-state"><h3>Could not load files</h3><p>' + extEscape(err.message) + "</p></div>";
        }
    }
}

var CONNECTOR_PERMISSIONS = {
    onedrive: { name: "OneDrive", permissions: ["Read your files and folders", "List directory contents", "Download files on demand"], icon: "📁" },
    googledrive: { name: "Google Drive", permissions: ["Read file metadata", "Download files", "List shared drives"], icon: "📁" },
    github: { name: "GitHub", permissions: ["Read repository contents", "List repositories", "Download files from repos"], icon: "🐙" },
    rest: { name: "REST API", permissions: ["Send requests to configured endpoint", "Read response data"], icon: "🔗" },
};

function showConnectorConsent(providerId) {
    var info = CONNECTOR_PERMISSIONS[providerId] || { name: providerId, permissions: ["Access your data"], icon: "🔌" };
    var modal = document.getElementById("consent-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "consent-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;backdrop-filter:blur(8px);";
    var permList = info.permissions.map(function (p) { return '<li style="padding:4px 0;color:#cbd5e1;">' + extEscape(p) + '</li>'; }).join("");
    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:440px;padding:24px;box-shadow:0 25px 50px -12px rgba(0,0,0,0.5);">'
        + '<div style="display:flex;align-items:center;gap:12px;margin-bottom:16px;"><span style="font-size:32px;">' + info.icon + '</span><h3 style="margin:0;color:#f8fafc;">Connect ' + extEscape(info.name) + '</h3></div>'
        + '<p style="color:#94a3b8;font-size:14px;margin-bottom:12px;">This connector will request the following permissions:</p>'
        + '<ul style="list-style:none;padding:0;margin:0 0 20px;">' + permList + '</ul>'
        + '<p style="color:#64748b;font-size:12px;margin-bottom:20px;">You can disconnect at any time from the External Drives tab.</p>'
        + '<div style="display:flex;gap:10px;justify-content:flex-end;">'
        + '<button onclick="document.getElementById(\'consent-modal\').remove()" style="background:#334155;border:none;border-radius:6px;color:#f8fafc;padding:10px 20px;cursor:pointer;font-weight:500;">Cancel</button>'
        + '<button onclick="document.getElementById(\'consent-modal\').remove();DriveExternal.doConnect(\'' + extEscape(providerId) + '\')" style="background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:10px 20px;cursor:pointer;font-weight:500;">Allow &amp; Connect</button>'
        + '</div></div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);
}

async function connectExternalProvider(providerId) {
    showConnectorConsent(providerId);
}

async function doConnectProvider(providerId) {
    try {
        showNotification("Opening " + providerId + "...", "info");
        var payload = await extApiRequest("/connect", {
            method: "POST",
            body: JSON.stringify({ provider: providerId }),
        });
        if (!payload || !payload.authorize_url) throw new Error("no authorization URL returned");
        window.location.href = payload.authorize_url;
    } catch (err) {
        showNotification("Connect failed: " + err.message, "error");
    }
}

async function syncExternalProvider(providerId) {
    try {
        showNotification("Syncing " + providerId + "...", "info");
        var report = await extApiRequest("/sync", {
            method: "POST",
            body: JSON.stringify({ provider: providerId }),
        });
        if (report && report.error) {
            showNotification("Sync reported: " + report.error, "warning");
        } else {
            showNotification("Synced " + (report ? report.upserted : 0) + " file(s)", "success");
        }
        await loadExternalTab();
    } catch (err) {
        showNotification("Sync failed: " + err.message, "error");
    }
}

async function disconnectExternalProvider(providerId) {
    if (!confirm("Disconnect " + providerId + "? The mirrored listing and tokens are deleted.")) return;
    try {
        await extApiRequest("/disconnect", {
            method: "POST",
            body: JSON.stringify({ provider: providerId }),
        });
        showNotification(providerId + " disconnected", "success");
        if (externalProvider === providerId) externalProvider = null;
        await loadExternalTab();
    } catch (err) {
        showNotification("Disconnect failed: " + err.message, "error");
    }
}

async function downloadExternalFile(providerId, remoteId) {
    try {
        const headers = {};
        const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        if (token) headers["Authorization"] = "Bearer " + token;
        const url = EXT_BASE + "/download/" + encodeURIComponent(providerId) + "/" + encodeURIComponent(remoteId);
        const response = await fetch(url, { headers: headers });
        if (!response.ok) throw new Error("HTTP " + response.status);
        const blob = await response.blob();
        const disposition = response.headers.get("Content-Disposition") || "";
        const match = /filename="([^"]+)"/.exec(disposition);
        const name = match ? match[1] : "download";
        const objectUrl = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = objectUrl;
        a.download = name;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(objectUrl);
        showNotification("Downloaded " + name, "success");
    } catch (err) {
        showNotification("Download failed: " + err.message, "error");
    }
}

// One delegated listener for every action in the tab: the listing is rebuilt on
// every render, so per-button wiring would have to be redone each time.
function bindExternalEvents() {
    document.addEventListener("click", function (event) {
        var trigger = event.target.closest("[data-ext-action]");
        if (!trigger) return;
        var action = trigger.dataset.extAction;
        var providerId = trigger.dataset.provider;
        if (action === "connect") connectExternalProvider(providerId);
        else if (action === "sync") syncExternalProvider(providerId);
        else if (action === "disconnect") disconnectExternalProvider(providerId);
        else if (action === "download") downloadExternalFile(providerId, trigger.dataset.remoteId);
        else if (action === "browse") {
            externalProvider = providerId;
            loadExternalTab();
        } else if (action === "back") {
            externalProvider = null;
            loadExternalTab();
        }
    });
}

// The OAuth callback redirects to /drive?external=...; land on the tab it
// refers to and clear the query so a reload does not replay it.
function handleExternalCallbackParams() {
    var params = new URLSearchParams(window.location.search);
    var outcome = params.get("external");
    if (!outcome) return;
    var provider = params.get("provider");
    var reason = params.get("reason");
    if (outcome === "connected") {
        showNotification((provider || "Account") + " connected", "success");
        externalProvider = null;
    } else if (outcome === "error") {
        showNotification("Connection failed: " + (reason || "unknown error"), "error");
    }
    params.delete("external");
    params.delete("provider");
    params.delete("reason");
    var rest = params.toString();
    window.history.replaceState({}, "", window.location.pathname + (rest ? "?" + rest : ""));
}

window.DriveExternal = {
    load: loadExternalTab,
    bind: bindExternalEvents,
    handleCallbackParams: handleExternalCallbackParams,
    doConnect: doConnectProvider,
};