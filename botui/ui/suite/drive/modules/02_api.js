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
            const message = error.message || error.error || response.statusText || "Request failed";
            const failure = new Error("HTTP " + response.status + ": " + message);
            failure.status = response.status;
            failure.body = error;
            failure.response = response;
            throw failure;
        }
        return response.json();
    } catch (err) {
        console.error("API Error [" + endpoint + "]:", err);
        throw err;
    }
}

async function fetchUserInfo() {
    if (window.__BOT_IS_PUBLIC__) {
        userInfo = { is_anonymous: true, name: "Local User", roles: ["user"] };
        isAdmin = false;
        return;
    }
    try {
        const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        const headers = {};
        if (token) headers["Authorization"] = "Bearer " + token;
        const res = await fetch("/api/auth/me", { headers });
        if (res.ok) {
            userInfo = await res.json();
            isAdmin = userInfo.roles && userInfo.roles.some(function(r) { return r.toLowerCase() === "admin" || r.toLowerCase() === "superadmin"; });
        } else {
            userInfo = { is_anonymous: true, name: "Guest", roles: [] };
            isAdmin = false;
        }
    } catch (e) {
        console.warn("Failed to fetch user info:", e);
        userInfo = { is_anonymous: true, name: "Guest", roles: [] };
        isAdmin = false;
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

async function downloadFile(path, bucket) {
    try {
        const fileName = path.split("/").pop() || "download";
        const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        const headers = { "Content-Type": "application/json" };
        if (token) headers["Authorization"] = "Bearer " + token;
        const res = await fetch(API_BASE + "/download-binary", {
            method: "POST",
            headers: headers,
            body: JSON.stringify({ bucket: bucket || getEffectiveBucket(), path: path, scope: currentScope })
        });
        if (!res.ok) throw new Error("HTTP " + res.status);
        const blob = await res.blob();
        // #1834 — the desktop WebView ignores blob + `<a download>`; the
        // shared helper opens the native Save dialog there and keeps the
        // anchor for the web build.
        if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
            const saved = await window.GBShared.saveBlob(blob, fileName);
            if (!saved) showNotification("Downloaded " + fileName, "success");
        } else {
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = fileName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            showNotification("Downloaded " + fileName, "success");
        }
    } catch (err) {
        showNotification("Download failed: " + err.message, "error");
    }
}

// Media and documents Drive has no dedicated app for. `/api/files/open`
// maps these to the generic "preview" app (the docs viewer), which cannot
// play a video or show a player, so opening one from the grid used to do
// nothing visible. Double-clicking one now opens the built-in viewer modal.
var PREVIEWABLE_EXT = [
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "avif", "svg",
    "mp4", "webm", "mov", "m4v", "mkv", "avi", "ogv",
    "mp3", "wav", "ogg", "oga", "m4a", "flac",
    "pdf",
    "txt", "md", "csv", "json", "xml", "log", "yml", "yaml", "toml",
    "js", "ts", "py", "rs", "go", "java", "c", "cpp", "h", "html", "css",
    "bas", "sql", "sh", "bat", "ini", "cfg", "conf",
    "docx", "doc", "odt", "rtf", "xlsx", "xls", "xlsb", "ods", "pptx", "ppt"
];

// Office documents render through the server-side converter (docx-rs,
// calamine, slide XML) instead of the raw-bytes viewer.
// Extensions the server-side converter actually renders. .doc/.odt/.rtf go
// through the OOXML reader (no word/document.xml) and .ppt through the ZIP
// reader (OLE), so the endpoint answered 422 for them; they now use the
// regular blob preview instead of failing in the Office modal.
var OFFICE_PREVIEW_EXT = ["docx", "xlsx", "xls", "xlsb", "ods", "pptx"];

function getFileExt(path) {
    var name = (path || "").split("/").pop() || "";
    if (name.indexOf(".") === -1) return "";
    return name.split(".").pop().toLowerCase();
}

function isPreviewable(path) {
    return PREVIEWABLE_EXT.indexOf(getFileExt(path)) !== -1;
}

async function previewFile(path, bucket) {
    try {
        var fileName = path.split("/").pop() || "preview";
        var ext = getFileExt(path);
        // A search hit may live in a bucket other than the current one; the
        // caller passes the row's own bucket so the preview never reads the
        // wrong store (#1710).
        var previewBucket = bucket || getEffectiveBucket();
        showNotification("Loading preview...", "info");
        // Fetch the raw bytes instead of the base64 JSON envelope: the
        // base64 route inflated the payload by a third and decoded it
        // byte-by-byte in JS, which stalls the tab on a real video.
        const headers = { "Content-Type": "application/json" };
        const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        if (token) headers["Authorization"] = "Bearer " + token;
        if (OFFICE_PREVIEW_EXT.indexOf(ext) !== -1 && !parseLocalBucket(previewBucket)) {
            const office = await fetch(
                "/api/files/office-preview?bucket=" + encodeURIComponent(previewBucket) +
                "&path=" + encodeURIComponent(path),
                { headers: token ? { "Authorization": "Bearer " + token } : {} }
            );
            if (!office.ok) throw new Error("HTTP " + office.status);
            showOfficePreviewModal(fileName, await office.text());
            return;
        }
        const response = await fetch(API_BASE + "/download-inline", {
            method: "POST",
            headers: headers,
            body: JSON.stringify({ bucket: previewBucket, path: path, scope: currentScope }),
        });
        if (!response.ok) throw new Error("HTTP " + response.status);
        const blob = await response.blob();
        showPreviewModal(fileName, ext, blob, path);
    } catch (err) {
        showNotification("Preview failed: " + err.message, "error");
    }
}

// #2282 — bind a Drive file to ONE app window. The context lives in a
// per-window map and in data attributes on that window's body; the old page
// globals (__EDITOR_FILE_PATH, __CANVAS_FILE_BUCKET, …) are deliberately NOT
// written here because a second open would clobber the first window's target.
// The apps still read those globals as a fallback for direct app opens.
function stampWindowBodyFileCtx(winId, ctx) {
    var body = document.getElementById("window-body-" + winId);
    if (body) {
        body.dataset.gbFileBucket = ctx.bucket;
        body.dataset.gbFilePath = ctx.path;
        body.dataset.gbFileScope = ctx.scope;
    }
}

// Open an editor app with its context scoped to the new window. `ctxMapName`
// is the app-specific registry (__EDITOR_FILE_CTX, __CANVAS_FILE_CTX, …);
// `afterInject` runs once the app fragment has been mounted (photo editor).
function openDriveEditorWindow(winId, fileName, htmlUrl, bucket, path, scope, ctxMapName, afterInject) {
    var ctx = { bucket: bucket || "", path: path || "", scope: scope || "bot" };
    window.__GB_WINDOW_FILE_CTX = window.__GB_WINDOW_FILE_CTX || {};
    window.__GB_WINDOW_FILE_CTX[winId] = ctx;
    if (ctxMapName) {
        window[ctxMapName] = window[ctxMapName] || {};
        window[ctxMapName][winId] = ctx;
    }
    window.WindowManager.open(winId, fileName, "");
    // open() creates the window body; stamp after it exists so the attributes
    // stay on this window even when another editor is opened later.
    stampWindowBodyFileCtx(winId, ctx);
    fetch(htmlUrl).then(function (r) { return r.text(); }).then(function (html) {
        window.WindowManager._injectBodyContent(winId, html);
        if (typeof afterInject === "function") afterInject();
    })
      .catch(function (fetchError) {
        window.gbLogError ? gbLogError(fetchError, "openDriveEditorWindow")
                            : console.error("if", fetchError);
      });
    return ctx;
}

async function openFile(path, bucket) {
    try {
        // #2281 — resolve the target bucket once (param wins; fallback to the
        // effective bucket when absent) and use it in every branch so opening a
        // file from Shared/Public/search never reads/writes the wrong bucket.
        const resolvedBucket = bucket || getEffectiveBucket();
        // #1801 — local files have no editor-app integration yet; the preview
        // modal is the working viewer for every local type.
        if (parseLocalBucket(resolvedBucket)) {
            previewFile(path, resolvedBucket);
            return;
        }
        const response = await apiRequest("/open", {
            method: "POST",
            body: JSON.stringify({ bucket: resolvedBucket, path: path, scope: currentScope }),
        });
        const { app, url } = response;
        if (app === "sheets" && window.WindowManager) {
            const fileName = path.split("/").pop() || "Spreadsheet";
            const ts = Date.now();
            // #2268/#2281 — a file's bucket comes from the rendered row
            // (parameter -> resolvedBucket), never the global current bucket;
            // #2272 keeps the per-app window-id namespace (`sheet-`).
            var winId = "sheet-" + ts;
            var qs = url.split('?')[1] || '';
            // Per-window sheet data map — avoids race when multiple files open rapidly
            window.__SHEET_DATA_MAP = window.__SHEET_DATA_MAP || {};
            window.__SHEET_DATA_MAP[winId] = { urlParams: qs, fileName: fileName };
            try {
                var sheetResp = await fetch('/api/sheet/load-from-drive', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket: resolvedBucket, path: path })
                });
                if (sheetResp.ok) {
                    var sheetData = await sheetResp.json();
                    // Keep the server-assigned sheet id (deterministic per
                    // source file) so edits persist to the same document and
                    // link back to the source xlsx for save-back.
                    window.__SHEET_DATA_MAP[winId].loadedSheet = sheetData;
                    window.__SHEET_DATA_MAP[winId].boot = Promise.resolve();
                } else {
                    // #2269 — a failed load must not become a silent blank
                    // workbook: a synthetic `new-<ts>` document has no
                    // source_bucket/source_path, so its edits are stranded and
                    // Save still reports success. Hand the window an explicit
                    // error state instead.
                    window.__SHEET_DATA_MAP[winId].loadError =
                        'Could not open ' + fileName + ' from Drive (HTTP ' + sheetResp.status + ').';
                    window.__SHEET_DATA_MAP[winId].requestedBucket = resolvedBucket;
                    window.__SHEET_DATA_MAP[winId].requestedPath = path;
                    window.__SHEET_DATA_MAP[winId].boot = Promise.resolve();
                }
            } catch(e) {
                console.warn('Pre-load sheet failed:', e);
                window.__SHEET_DATA_MAP[winId].loadError =
                    'Could not open ' + fileName + ' from Drive (' + (e && e.message ? e.message : 'network error') + ').';
                window.__SHEET_DATA_MAP[winId].requestedBucket = resolvedBucket;
                window.__SHEET_DATA_MAP[winId].requestedPath = path;
                window.__SHEET_DATA_MAP[winId].boot = Promise.resolve();
            }
            window.WindowManager.open(winId, fileName, "");
            var cleanUrl = url.split('?')[0];
            fetch(cleanUrl).then(function (r) { return r.text(); }).then(function (html) {
                var body = document.getElementById("window-body-" + winId);
                if (body) window.WindowManager._injectBodyContent(winId, html);
            })
              .catch(function (fetchError) {
                window.gbLogError ? gbLogError(fetchError, "openFile")
                                    : console.error("if", fetchError);
              });
            return;
        }
        // docx/pptx open in the Docs/Slides app window. The shell URL has no
        // query string of its own, so the app reads file/bucket from
        // window.__gbAppParams__ (see docs 99_init / slides boot).
        if (app === "docs" && window.WindowManager) {
            window.WindowManager.openDeepLink("docs", { file: path, bucket: resolvedBucket || "" });
            return;
        }
        if (app === "slides" && window.WindowManager) {
            window.WindowManager.openDeepLink("slides", { file: path, bucket: resolvedBucket || "" });
            return;
        }
        if (app === "designer" && window.WindowManager) {
            var fileName = path.split("/").pop() || "Script";
            // #2282 — context is bound to this window only (no page globals).
            openDriveEditorWindow("designer-" + Date.now(), fileName,
                "/suite/designer.html", resolvedBucket, path, currentScope, null);
            return;
        }
        if (app === "canvas" && window.WindowManager) {
            var fileName = path.split("/").pop() || "Canvas";
            openDriveEditorWindow("canvas-" + Date.now(), fileName,
                "/suite/canvas/canvas.html", resolvedBucket, path, currentScope, "__CANVAS_FILE_CTX");
            return;
        }
        if (app === "photo-editor" && window.WindowManager) {
            // #1306 — image files open the Photo Editor as an APP WINDOW:
            // never window.open, never a raw fragment navigation.
            var fileName = path.split("/").pop() || "Photo";
            var winId = "photo-editor-" + Date.now();
            openDriveEditorWindow(winId, fileName,
                "/suite/photos/photos-editor.html", resolvedBucket, path, currentScope,
                "__PHOTO_FILE_CTX", function () {
                    if (window.GBPhotoEditor && typeof window.GBPhotoEditor.init === "function") {
                        window.GBPhotoEditor.init(winId);
                    }
                });
            return;
        }
        if (app === "editor" && window.WindowManager) {
            var fileName = path.split("/").pop() || "Untitled";
            var winId = "editor-" + Date.now();
            openDriveEditorWindow(winId, fileName,
                "/suite/editor.html", resolvedBucket, path, currentScope, "__EDITOR_FILE_CTX");
            // Per-window boot gate; the global kept for direct-app backward compat.
            window.__EDITOR_BOOTS = window.__EDITOR_BOOTS || {};
            window.__EDITOR_BOOTS[winId] = Promise.resolve();
            return;
        }
        // PDFs and unknown types have no app window; the Drive preview modal
        // is the working viewer (htmx has no #main-content target here).
        previewFile(path, resolvedBucket);
        window.history.pushState({}, "", "/#" + app + "?bucket=" + encodeURIComponent(resolvedBucket || "") + "&path=" + encodeURIComponent(path));
    } catch (err) {
        console.error("Failed to open file:", err);
        showNotification("Failed to open file: " + err.message, "error");
    }
}
