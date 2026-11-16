"use strict";
/**
 * 13_attachpanel.js - Side panel for attachment preview.
 *
 * Handles: images (lightbox), text/code/CSV/JSON (inline viewer),
 * PDF/DOCX (server-rendered preview via /api/chat/attachments/preview),
 * other types (metadata + download).
 * Sources: in-memory File blobs for the current session; Drive download
 * for history items; server preview endpoint for binary docs.
 */
(function () {
var A = window.GBAssistant;
var I = window.GBIcon || function () { return ""; };

function esc(s) {
    return String(s == null ? "" : s)
        .replace(/&/g, "&amp;").replace(/</g, "&lt;")
        .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

/* ---- File blob cache (keyed by message index + filename) ---- */
var blobCache = {};

A.cacheFileBlob = function (key, file) { blobCache[key] = file; };
A.getCachedBlob = function (key) { return blobCache[key] || null; };

/* ---- Extension helpers ---- */
function ext(name) { return (name || "").split(".").pop().toLowerCase(); }
function isImage(name) { return "jpg jpeg png gif webp bmp svg ico".split(" ").indexOf(ext(name)) >= 0; }
function isText(name) { return "txt md csv json js ts py rs html css xml yaml toml sh bas log".split(" ").indexOf(ext(name)) >= 0; }
function isPdf(name) { return ext(name) === "pdf"; }
function isDocx(name) { return "docx xlsx pptx".split(" ").indexOf(ext(name)) >= 0; }
function isCode(name) { return "js ts py rs html css xml yaml toml sh bas json".split(" ").indexOf(ext(name)) >= 0; }
function isCsv(name) { return ext(name) === "csv"; }
/** True when the server preview endpoint can render this type. */
function hasServerPreview(name) { return isPdf(name) || isDocx(name); }

/* ---- Make attachment cards clickable ---- */
A.makeAttachClickable = function (card, fileOrInfo, opts) {
    opts = opts || {};
    card.style.cursor = "pointer";
    card.setAttribute("role", "button");
    card.setAttribute("tabindex", "0");
    card.addEventListener("click", function (e) {
        if (e.target.closest(".gb-attach-remove")) { return; }
        openAttachPanel(fileOrInfo, opts);
    });
    card.addEventListener("keydown", function (e) {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            openAttachPanel(fileOrInfo, opts);
        }
    });
};

/* ---- Panel ---- */
var panelEl = null;

function openAttachPanel(fileOrInfo, opts) {
    closePanel();
    var name = fileOrInfo.name || (typeof fileOrInfo === "string" ? fileOrInfo : "file");

    if (isImage(name) && (fileOrInfo instanceof File || opts.blobUrl)) {
        openLightbox(fileOrInfo, opts);
        return;
    }

    panelEl = document.createElement("div");
    panelEl.className = "gb-attach-panel";
    panelEl.setAttribute("role", "dialog");
    panelEl.setAttribute("aria-label", name);

    var header = '<div class="gb-attach-panel-header">' +
        '<span class="gb-attach-panel-title">' + esc(name) + '</span>' +
        '<button class="gb-attach-panel-close" aria-label="' + esc(A.t("panel.close", "Close panel")) + '">' +
        I("x", 16) + '</button></div>';
    panelEl.innerHTML = header + '<div class="gb-attach-panel-body" id="gbAttachBody"></div>';

    var main = document.getElementById("gb-main");
    if (main) { main.appendChild(panelEl); main.classList.add("gb-panel-open"); }

    panelEl.querySelector(".gb-attach-panel-close").addEventListener("click", closePanel);
    document.addEventListener("keydown", panelEsc);

    var body = document.getElementById("gbAttachBody");
    loadContent(body, fileOrInfo, opts);
}

function panelEsc(e) {
    if (e.key === "Escape") { closePanel(); }
}

function closePanel() {
    if (panelEl) { panelEl.remove(); panelEl = null; }
    var main = document.getElementById("gb-main");
    if (main) { main.classList.remove("gb-panel-open"); }
    closeLightbox();
    document.removeEventListener("keydown", panelEsc);
}
A.closeAttachPanel = closePanel;

/* ---- Content loaders ---- */
function loadContent(body, fileOrInfo, opts) {
    var name = fileOrInfo.name || (typeof fileOrInfo === "string" ? fileOrInfo : "");

    /* In-memory File blob */
    if (fileOrInfo instanceof File) {
        /* For PDF/DOCX blobs we cannot render them in WebView2;
           use the server preview endpoint if the file is from inbox. */
        if (hasServerPreview(name) && opts.drivePath) {
            loadServerPreview(body, name, fileOrInfo, opts);
            return;
        }
        loadFromBlob(body, fileOrInfo, name, opts);
        return;
    }

    /* Cached blob */
    if (opts.cacheKey) {
        var cached = A.getCachedBlob(opts.cacheKey);
        if (cached) {
            if (hasServerPreview(name) && opts.drivePath) {
                loadServerPreview(body, name, cached, opts);
                return;
            }
            loadFromBlob(body, cached, name, opts);
            return;
        }
    }

    /* Drive path: for PDF/DOCX use the server preview; for others download blob */
    if (opts.drivePath) {
        if (hasServerPreview(name)) {
            loadServerPreview(body, name, fileOrInfo, opts);
            return;
        }
        body.innerHTML = '<p class="gb-attach-panel-loading">' + esc(A.t("assistant.connecting", "Loading...")) + '</p>';
        fetchFromDrive(opts.drivePath, function (blob) {
            if (blob) { loadFromBlob(body, blob, name, opts); }
            else { showMeta(body, name, fileOrInfo, opts); }
        });
        return;
    }

    /* Fallback: show metadata */
    showMeta(body, name, fileOrInfo, opts);
}

/** Server-rendered preview for PDF, DOCX, XLSX, PPTX.
 *  Uses GET /api/chat/attachments/preview?bot=&name= which returns
 *  rendered HTML for office formats and raw bytes for PDF. */
function loadServerPreview(body, name, fileOrInfo, opts) {
    var drivePath = opts.drivePath || "";
    var inboxName = drivePath.indexOf("inbox/") === 0
        ? drivePath.substring(6) : (name || "");
    var bot = A.botName || "default";
    var url = A.origin + "/api/chat/attachments/preview?bot=" +
        encodeURIComponent(bot) + "&name=" + encodeURIComponent(inboxName);

    body.innerHTML = '<p class="gb-attach-panel-loading">' +
        esc(A.t("assistant.connecting", "Loading...")) + '</p>';

    var token = A.getToken();
    fetch(url, {
        headers: token ? { "Authorization": "Bearer " + token } : {}
    }).then(function (r) {
        if (!r.ok) { throw new Error("HTTP " + r.status); }
        var ct = r.headers.get("content-type") || "";
        if (ct.indexOf("text/html") >= 0) {
            return r.text().then(function (html) {
                renderDocPreview(body, name, html, fileOrInfo, opts);
            });
        }
        /* PDF raw bytes: try to show in iframe, with text fallback */
        if (ct.indexOf("pdf") >= 0) {
            return r.blob().then(function (blob) {
                renderPdfPreview(body, name, blob, fileOrInfo, opts);
            });
        }
        /* Image or other inline content */
        return r.blob().then(function (blob) {
            loadFromBlob(body, blob, name, opts);
        });
    }).then(function () {
        bindDownloadButtons(body, name, opts);
    }).catch(function () {
        showMeta(body, name, fileOrInfo, opts);
    });
}

/** Renders server-generated HTML preview (DOCX/XLSX) in a sandboxed iframe. */
function renderDocPreview(body, name, html, fileOrInfo, opts) {
    var size = (fileOrInfo && (fileOrInfo.size || fileOrInfo.size_bytes)) || 0;
    var type = (fileOrInfo && (fileOrInfo.type || fileOrInfo.content_type)) || "";
    var metaHtml = '<div class="gb-attach-panel-doc-meta">' +
        '<span class="gb-attach-panel-doc-icon">' + I(A.fileTypeIcon ? A.fileTypeIcon(name) : "file", 20) + '</span>' +
        '<span class="gb-attach-panel-doc-name">' + esc(name) + '</span>' +
        (type ? '<span class="gb-attach-panel-doc-type">' + esc(type) + '</span>' : '') +
        (size ? '<span class="gb-attach-panel-doc-size">' + esc(fmtBytes(size)) + '</span>' : '') +
        '</div>';
    body.innerHTML = metaHtml +
        '<iframe class="gb-attach-panel-doc-frame" sandbox="allow-same-origin" title="' +
        esc(name) + '"></iframe>' + downloadBar(name, opts);
    var frame = body.querySelector("iframe");
    if (frame) {
        frame.srcdoc = html;
    }
}

/** Renders a PDF preview: attempt iframe, fall back to text extraction message. */
function renderPdfPreview(body, name, blob, fileOrInfo, opts) {
    var size = (fileOrInfo && (fileOrInfo.size || fileOrInfo.size_bytes)) || 0;
    var metaHtml = '<div class="gb-attach-panel-doc-meta">' +
        '<span class="gb-attach-panel-doc-icon">' + I("file", 20) + '</span>' +
        '<span class="gb-attach-panel-doc-name">' + esc(name) + '</span>' +
        '<span class="gb-attach-panel-doc-type">PDF</span>' +
        (size ? '<span class="gb-attach-panel-doc-size">' + esc(fmtBytes(size)) + '</span>' : '') +
        '</div>';
    var pdfUrl = URL.createObjectURL(blob);
    body.innerHTML = metaHtml +
        '<iframe class="gb-attach-panel-pdf" src="' + pdfUrl + '#toolbar=0"></iframe>' +
        downloadBar(name, opts);
    /* If the iframe fails to load (WebView2 limitation), show a notice */
    var frame = body.querySelector("iframe");
    if (frame) {
        frame.addEventListener("load", function () {
            try {
                /* If the iframe loaded but is about:blank, it failed */
                if (frame.contentDocument && !frame.contentDocument.body.innerHTML) {
                    showPdfFallback(body, name, pdfUrl, fileOrInfo, opts);
                }
            } catch (e) {
                /* Cross-origin: PDF loaded successfully in the viewer */
            }
        });
        /* Timeout fallback: if after 2s the frame is still empty */
        setTimeout(function () {
            try {
                if (frame.contentDocument &&
                    frame.contentDocument.body &&
                    !frame.contentDocument.body.innerHTML) {
                    showPdfFallback(body, name, pdfUrl, fileOrInfo, opts);
                }
            } catch (e) { /* cross-origin = loaded */ }
        }, 2000);
    }
}

function showPdfFallback(body, name, pdfUrl, fileOrInfo, opts) {
    var size = (fileOrInfo && (fileOrInfo.size || fileOrInfo.size_bytes)) || 0;
    body.innerHTML =
        '<div class="gb-attach-panel-meta">' +
            '<p>' + I("file", 32) + '</p>' +
            '<p class="gb-attach-panel-name">' + esc(name) + '</p>' +
            '<p class="gb-attach-panel-info">PDF</p>' +
            (size ? '<p class="gb-attach-panel-info">' + esc(fmtBytes(size)) + '</p>' : '') +
            '<p class="gb-attach-panel-info" style="margin-top:8px;color:var(--id-warn,#a16207)">' +
                esc(A.t("attach.pdf-no-preview",
                    "A visualização de PDF não está disponível neste navegador.")) +
            '</p>' +
        '</div>' + downloadBar(name, opts);
    bindDownloadButtons(body, name, opts);
}

/** Download bar shown at the bottom of document previews. */
function downloadBar(name, opts) {
    return '<div class="gb-attach-panel-actions">' +
        '<button class="gb-attach-panel-download" data-download-name="' + esc(name) + '">' +
        I("download", 14) + ' <span>' +
        esc(A.t("drive.download", "Baixar")) + '</span></button>' +
        '</div>';
}

function bindDownloadButtons(body, name, opts) {
    var btns = body.querySelectorAll("[data-download-name]");
    for (var i = 0; i < btns.length; i++) {
        btns[i].addEventListener("click", function () {
            downloadAttachment(name, opts);
        });
    }
}

function downloadAttachment(name, opts) {
    var drivePath = (opts && opts.drivePath) || "";
    if (!drivePath) { return; }
    /* Inbox files live in the bot's Drive layout; the preview endpoint
       resolves the correct bucket and prefix. For binary types (PDF,
       images) it returns raw bytes suitable for download. */
    var isInbox = drivePath.indexOf("inbox/") >= 0;
    var fetchP;
    if (isInbox) {
        var inboxName = drivePath.substring(drivePath.indexOf("inbox/") + 6);
        var url = A.origin + "/api/chat/attachments/preview?bot=" +
            encodeURIComponent(A.botName || "default") +
            "&name=" + encodeURIComponent(inboxName);
        fetchP = fetch(url, {
            headers: A.getToken() ? { "Authorization": "Bearer " + A.getToken() } : {}
        });
    } else {
        fetchP = A.apiFetch("/api/files/download-binary", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ path: drivePath, scope: "bot" })
        });
    }
    fetchP.then(function (r) {
        if (!r.ok) { throw new Error("HTTP " + r.status); }
        return r.blob();
    }).then(function (blob) {
        var url = URL.createObjectURL(blob);
        var a = document.createElement("a");
        a.href = url; a.download = name;
        document.body.appendChild(a); a.click(); a.remove();
        URL.revokeObjectURL(url);
    }).catch(function (e) {
        console.warn("[attachpanel] download failed:", e);
    });
}

function loadFromBlob(body, blob, name, opts) {
    if (isImage(name)) {
        var url = URL.createObjectURL(blob);
        body.innerHTML = '<img class="gb-attach-panel-img" src="' + url + '" alt="' + esc(name) + '" />';
        return;
    }
    if (isPdf(name)) {
        renderPdfPreview(body, name, blob, blob, opts || {});
        return;
    }
    if (isText(name)) {
        var reader = new FileReader();
        reader.onload = function () {
            var text = reader.result || "";
            if (isCsv(name)) { renderCsv(body, text); }
            else { renderCode(body, text, name); }
        };
        reader.onerror = function () { showMeta(body, name); };
        reader.readAsText(blob);
        return;
    }
    showMeta(body, name, blob, opts);
}

function renderCode(body, text, name) {
    var lines = text.split("\n");
    var html = '<div class="gb-attach-panel-code">';
    for (var i = 0; i < lines.length; i++) {
        html += '<div class="gb-attach-code-line"><span class="gb-attach-line-num">' + (i + 1) + '</span><span class="gb-attach-line-text">' + esc(lines[i]) + '</span></div>';
    }
    html += '</div>';
    body.innerHTML = html;
}

function renderCsv(body, text) {
    var rows = text.split("\n").filter(function (r) { return r.trim(); });
    if (rows.length === 0) { body.innerHTML = '<p class="gb-attach-panel-empty">Empty</p>'; return; }
    var html = '<table class="gb-attach-panel-table"><thead><tr>';
    var header = rows[0].split(",");
    for (var h = 0; h < header.length; h++) { html += '<th>' + esc(header[h].trim()) + '</th>'; }
    html += '</tr></thead><tbody>';
    for (var r = 1; r < Math.min(rows.length, 200); r++) {
        var cols = rows[r].split(",");
        html += '<tr>';
        for (var c = 0; c < cols.length; c++) { html += '<td>' + esc(cols[c].trim()) + '</td>'; }
        html += '</tr>';
    }
    html += '</tbody></table>';
    body.innerHTML = html;
}

function showMeta(body, name, fileOrInfo, opts) {
    var size = (fileOrInfo && (fileOrInfo.size || fileOrInfo.size_bytes)) || 0;
    var type = (fileOrInfo && (fileOrInfo.type || fileOrInfo.content_type)) || "";
    var hasDrive = opts && opts.drivePath;
    body.innerHTML =
        '<div class="gb-attach-panel-meta">' +
            '<p>' + I(A.fileTypeIcon ? A.fileTypeIcon(name) : "file", 32) + '</p>' +
            '<p class="gb-attach-panel-name">' + esc(name) + '</p>' +
            (type ? '<p class="gb-attach-panel-info">' + esc(type) + '</p>' : '') +
            (size ? '<p class="gb-attach-panel-info">' + esc(fmtBytes(size)) + '</p>' : '') +
            (hasDrive ? '' : '<p class="gb-attach-panel-info" style="margin-top:8px">' + esc(A.t("attach.unavailable", "Content unavailable")) + '</p>') +
        '</div>' +
        (hasDrive ? downloadBar(name, opts) : '');
    if (hasDrive) { bindDownloadButtons(body, name, opts); }
}

function fmtBytes(b) {
    if (!b) { return "0 B"; }
    if (b < 1024) { return b + " B"; }
    if (b < 1048576) { return (b / 1024).toFixed(0) + " KB"; }
    return (b / 1048576).toFixed(1) + " MB";
}

/* ---- Drive fetch ---- */
function fetchFromDrive(path, cb) {
    A.apiFetch("/api/files/download", {
        method: "POST",
        body: JSON.stringify({ path: path, scope: "bot" })
    }).then(function (r) {
        if (!r.ok) { cb(null); return; }
        return r.blob();
    }).then(function (blob) {
        cb(blob || null);
    }).catch(function () { cb(null); });
}

/* ---- Lightbox (images) ---- */
var lightboxEl = null;
function openLightbox(fileOrInfo, opts) {
    closeLightbox();
    var url = opts.blobUrl || (fileOrInfo instanceof File ? URL.createObjectURL(fileOrInfo) : "");
    if (!url) { return; }
    lightboxEl = document.createElement("div");
    lightboxEl.className = "gb-lightbox";
    lightboxEl.innerHTML = '<div class="gb-lightbox-backdrop"></div>' +
        '<img class="gb-lightbox-img" src="' + url + '" alt="" />' +
        '<button class="gb-lightbox-close" aria-label="' + esc(A.t("panel.close", "Close")) + '">' + I("x", 20) + '</button>';
    document.body.appendChild(lightboxEl);
    lightboxEl.querySelector(".gb-lightbox-backdrop").addEventListener("click", closeLightbox);
    lightboxEl.querySelector(".gb-lightbox-close").addEventListener("click", closeLightbox);
    document.addEventListener("keydown", lightboxEsc);
}
function lightboxEsc(e) { if (e.key === "Escape") { closeLightbox(); } }
function closeLightbox() {
    if (lightboxEl) { lightboxEl.remove(); lightboxEl = null; }
    document.removeEventListener("keydown", lightboxEsc);
}
})();
