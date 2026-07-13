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
            // Pre-load sheet data from Drive — boot script inside injected HTML
            // can't read URL params (window.location.search belongs to main page).
            var bucket = getEffectiveBucket();
            try {
                var sheetResp = await fetch('/api/sheet/load-from-drive', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket: bucket, path: path })
                });
                if (sheetResp.ok) {
                    var sheetData = await sheetResp.json();
                    window.__LOADED_SHEET = sheetData;
                    window.__SHEET_INITIAL_ID = sheetData.id;
                    window.__SHEET_BOOT = Promise.resolve();
                }
            } catch(e) {
                console.warn('Pre-load sheet failed, boot script will retry via URL params:', e);
            }
            window.WindowManager.open("sheets-" + ts, fileName, "");
            var cleanUrl = url.split('?')[0];
            fetch(cleanUrl).then(function (r) { return r.text(); }).then(function (html) {
                var body = document.getElementById("window-body-sheets-" + ts);
                if (body) window.WindowManager._injectBodyContent("sheets-" + ts, html);
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
