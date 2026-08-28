/* Drive Module v2.0 — 02 API */
"use strict";

async function apiRequest(endpoint, options = {}) {
    const url = API_BASE + endpoint;
    if (window.ApiClient) {
        try {
            return await window.ApiClient.request(url, options);
        } catch (err) {
            console.error("API Error [" + endpoint + "]:", err);
            throw err;
        }
    }
    const defaultHeaders = { "Content-Type": "application/json" };
    const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (token) defaultHeaders["Authorization"] = "Bearer " + token;
    try {
        const response = await fetch(url, {
            headers: Object.assign({}, defaultHeaders, options.headers || {}),
            ...options,
        });
        if (!response.ok) {
            const error = await response.json().catch(() => ({ error: response.statusText }));
            throw new Error(error.error || "Request failed");
        }
        return response.json();
    } catch (err) {
        console.error("API Error [" + endpoint + "]:", err);
        throw err;
    }
}

async function fetchUserInfo() {
    try {
        const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        const headers = {};
        if (token) headers["Authorization"] = "Bearer " + token;
        const res = await fetch("/api/auth/me", { headers });
        if (res.ok) {
            userInfo = await res.json();
            isAdmin = userInfo.roles && userInfo.roles.some(function(r) { return r.toLowerCase() === "admin" || r.toLowerCase() === "superadmin"; });
        }
    } catch (e) {
        console.warn("Failed to fetch user info:", e);
    }
}

async function retryWithBackoff() {
    if (retryCount >= MAX_RETRIES) {
        showNotification("Max retries reached. Please refresh the page.", "error");
        return;
    }
    const delay = RETRY_DELAYS[retryCount] || RETRY_DELAYS[RETRY_DELAYS.length - 1];
    retryCount++;
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (content) {
        content.innerHTML = '<div class="empty-state"><div class="spinner"></div><p>Retrying in ' + (delay / 1000) + 's... (attempt ' + retryCount + '/' + MAX_RETRIES + ')</p></div>';
    }
    await new Promise((resolve) => setTimeout(resolve, delay));
    await init();
}

async function downloadFile(path) {
    try {
        const fileName = path.split("/").pop() || "download";
        const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        const headers = { "Content-Type": "application/json" };
        if (token) headers["Authorization"] = "Bearer " + token;
        const res = await fetch(API_BASE + "/download-binary", {
            method: "POST",
            headers: headers,
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, scope: currentScope })
        });
        if (!res.ok) throw new Error("HTTP " + res.status);
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = fileName;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        showNotification("Downloaded " + fileName, "success");
    } catch (err) {
        showNotification("Download failed: " + err.message, "error");
    }
}

async function previewFile(path) {
    try {
        showNotification("Loading preview...", "info");
        const response = await apiRequest("/download", {
            method: "POST",
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, scope: currentScope }),
        });
        const content = response.content;
        const fileName = path.split("/").pop() || "preview";
        const ext = fileName.split('.').pop().toLowerCase();
        const byteCharacters = atob(content);
        const byteNumbers = new Array(byteCharacters.length);
        for (let i = 0; i < byteCharacters.length; i++) {
            byteNumbers[i] = byteCharacters.charCodeAt(i);
        }
        const byteArray = new Uint8Array(byteNumbers);
        let mimeType = "application/octet-stream";
        if (ext === "pdf") mimeType = "application/pdf";
        else if (ext === "png") mimeType = "image/png";
        else if (ext === "jpg" || ext === "jpeg") mimeType = "image/jpeg";
        else if (ext === "gif") mimeType = "image/gif";
        else if (ext === "mp3") mimeType = "audio/mpeg";
        else if (ext === "wav") mimeType = "audio/wav";
        else if (ext === "mp4") mimeType = "video/mp4";
        const blob = new Blob([byteArray], { type: mimeType });
        showPreviewModal(fileName, ext, blob);
    } catch (err) {
        showNotification("Preview failed: " + err.message, "error");
    }
}

    async function openFile(path) {
    try {
        const response = await apiRequest("/open", {
            method: "POST",
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, scope: currentScope }),
        });
        const { app, url } = response;
        if (app === "sheets" && window.WindowManager) {
            const fileName = path.split("/").pop() || "Spreadsheet";
            const ts = Date.now();
            var bucket = getEffectiveBucket();
            var winId = "sheets-" + ts;
            var qs = url.split('?')[1] || '';
            // Per-window sheet data map — avoids race when multiple files open rapidly
            window.__SHEET_DATA_MAP = window.__SHEET_DATA_MAP || {};
            window.__SHEET_DATA_MAP[winId] = { urlParams: qs, fileName: fileName };
            try {
                var sheetResp = await fetch('/api/sheet/load-from-drive', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket: bucket, path: path })
                });
                if (sheetResp.ok) {
                    var sheetData = await sheetResp.json();
                    // Keep the server-assigned sheet id (deterministic per
                    // source file) so edits persist to the same document and
                    // link back to the source xlsx for save-back.
                    window.__SHEET_DATA_MAP[winId].loadedSheet = sheetData;
                    window.__SHEET_DATA_MAP[winId].boot = Promise.resolve();
                } else {
                    window.__SHEET_DATA_MAP[winId].loadedSheet = {
                        id: 'new-' + ts, name: fileName,
                        worksheets: [{ name: 'Sheet1', data: {} }]
                    };
                    window.__SHEET_DATA_MAP[winId].boot = Promise.resolve();
                }
            } catch(e) {
                console.warn('Pre-load sheet failed:', e);
                window.__SHEET_DATA_MAP[winId].loadedSheet = {
                    id: 'new-' + ts, name: fileName,
                    worksheets: [{ name: 'Sheet1', data: {} }]
                };
                window.__SHEET_DATA_MAP[winId].boot = Promise.resolve();
            }
            window.WindowManager.open(winId, fileName, "");
            var cleanUrl = url.split('?')[0];
            fetch(cleanUrl).then(function (r) { return r.text(); }).then(function (html) {
                var body = document.getElementById("window-body-" + winId);
                if (body) window.WindowManager._injectBodyContent(winId, html);
            });
            return;
        }
        if (app === "designer" && window.WindowManager) {
            var bucket = getEffectiveBucket();
            var fileName = path.split("/").pop() || "Script";
            var ts = Date.now();
            window.__EDITOR_FILE_BUCKET = bucket;
            window.__EDITOR_FILE_PATH = path;
            window.__EDITOR_FILE_SCOPE = currentScope;
            window.__EDITOR_BOOT = Promise.resolve();
            window.WindowManager.open("designer-" + ts, fileName, "");
            fetch("/suite/designer.html").then(function(r){return r.text();}).then(function(html){
                window.WindowManager._injectBodyContent("designer-" + ts, html);
            });
            return;
        }
        if (app === "canvas" && window.WindowManager) {
            var bucket = getEffectiveBucket();
            var fileName = path.split("/").pop() || "Canvas";
            var ts = Date.now();
            // Canvas app context: the .draw file is loaded/saved by the app
            // through the drive API using these globals (bucket + folder).
            window.__CANVAS_FILE_BUCKET = bucket;
            window.__CANVAS_FILE_PATH = path;
            window.__CANVAS_FILE_SCOPE = currentScope;
            var folder = path.indexOf("/") === -1 ? "" : path.substring(0, path.lastIndexOf("/"));
            window.__CANVAS_FOLDER = folder;
            window.WindowManager.open("canvas-" + ts, fileName, "");
            fetch("/suite/canvas/canvas.html").then(function(r){return r.text();}).then(function(html){
                window.WindowManager._injectBodyContent("canvas-" + ts, html);
            });
            return;
        }
        if (app === "editor" && window.WindowManager) {
            var bucket = getEffectiveBucket();
            var fileName = path.split("/").pop() || "Untitled";
            var ts = Date.now();
            window.WindowManager.open("editor-" + ts, fileName, "");
            window.__EDITOR_FILE_BUCKET = bucket;
            window.__EDITOR_FILE_PATH = path;
            window.__EDITOR_FILE_SCOPE = currentScope;
            window.__EDITOR_BOOT = Promise.resolve();
            fetch("/suite/editor.html").then(function(r){return r.text();}).then(function(html){
                window.WindowManager._injectBodyContent("editor-" + ts, html);
            });
            return;
        }
        if (window.htmx) {
            htmx.ajax("GET", url, { target: "#main-content", swap: "innerHTML" });
            window.history.pushState({}, "", "/#" + app + "?bucket=" + encodeURIComponent(currentBucket) + "&path=" + encodeURIComponent(path));
        } else {
            window.location.href = url;
        }
    } catch (err) {
        console.error("Failed to open file:", err);
        showNotification("Failed to open file: " + err.message, "error");
    }
}
