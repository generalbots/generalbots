"use strict";
/**
 * 13_attachpanel.js - Side panel for attachment preview.
 *
 * Handles: images (lightbox), text/code/CSV/JSON (inline viewer),
 * PDF (embedded if CSP allows, otherwise fallback), other types (metadata).
 * Sources: in-memory File blobs for the current session; Drive download
 * for history items.
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
function isCode(name) { return "js ts py rs html css xml yaml toml sh bas json".split(" ").indexOf(ext(name)) >= 0; }
function isCsv(name) { return ext(name) === "csv"; }

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
        loadFromBlob(body, fileOrInfo, name);
        return;
    }

    /* Cached blob */
    if (opts.cacheKey) {
        var cached = A.getCachedBlob(opts.cacheKey);
        if (cached) { loadFromBlob(body, cached, name); return; }
    }

    /* Drive path (from activity summary) */
    if (opts.drivePath) {
        body.innerHTML = '<p class="gb-attach-panel-loading">' + esc(A.t("assistant.connecting", "Loading...")) + '</p>';
        fetchFromDrive(opts.drivePath, function (blob) {
            if (blob) { loadFromBlob(body, blob, name); }
            else { showUnavailable(body, name, fileOrInfo); }
        });
        return;
    }

    /* Fallback: show metadata */
    showUnavailable(body, name, fileOrInfo);
}

function loadFromBlob(body, blob, name) {
    if (isImage(name)) {
        var url = URL.createObjectURL(blob);
        body.innerHTML = '<img class="gb-attach-panel-img" src="' + url + '" alt="' + esc(name) + '" />';
        return;
    }
    if (isPdf(name)) {
        var pdfUrl = URL.createObjectURL(blob);
        body.innerHTML = '<iframe class="gb-attach-panel-pdf" src="' + pdfUrl + '#toolbar=0"></iframe>';
        return;
    }
    if (isText(name)) {
        var reader = new FileReader();
        reader.onload = function () {
            var text = reader.result || "";
            if (isCsv(name)) { renderCsv(body, text); }
            else { renderCode(body, text, name); }
        };
        reader.onerror = function () { showUnavailable(body, name); };
        reader.readAsText(blob);
        return;
    }
    showUnavailable(body, name);
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

function showUnavailable(body, name, fileOrInfo) {
    var size = (fileOrInfo && (fileOrInfo.size || fileOrInfo.size_bytes)) || 0;
    var type = (fileOrInfo && (fileOrInfo.type || fileOrInfo.content_type)) || "";
    body.innerHTML =
        '<div class="gb-attach-panel-meta">' +
            '<p>' + I(A.fileTypeIcon ? A.fileTypeIcon(name) : "file", 32) + '</p>' +
            '<p class="gb-attach-panel-name">' + esc(name) + '</p>' +
            (type ? '<p class="gb-attach-panel-info">' + esc(type) + '</p>' : '') +
            (size ? '<p class="gb-attach-panel-info">' + esc(fmtBytes(size)) + '</p>' : '') +
            '<p class="gb-attach-panel-info" style="margin-top:8px">' + esc(A.t("attach.unavailable", "Content unavailable")) + '</p>' +
        '</div>';
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
