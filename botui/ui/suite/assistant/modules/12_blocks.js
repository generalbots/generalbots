"use strict";
/**
 * 12_blocks.js - Activity block shell and registry.
 *
 * Server event contract (gb.activity):
 * { gb_event:true, type:"CUSTOM", data:{ name:"gb.activity", payload:{
 *   id, kind, status:"started"|"completed"|"failed"|"needs_input",
 *   summary:{...}, error? }}}
 *
 * The same id arrives twice (started -> completed/failed). Update in place.
 * Activities of a turn group into one stack above the answer text.
 * Renderers per kind live in 12a_block_kinds.js, the live frame handler and
 * the workspace run block in 12b_block_live.js, attachment cards and the
 * answer toolbar in 12c_attach_card.js.
 */
(function () {
var A = window.GBAssistant;
var I = window.GBIcon || function () { return ""; };
function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }
function fmtBytes(b) {
    if (!b && b !== 0) { return "0 B"; }
    if (b < 1024) { return b + " B"; }
    if (b < 1048576) { return (b / 1024).toFixed(0) + " KB"; }
    if (b < 1073741824) { return (b / 1048576).toFixed(1) + " MB"; }
    return (b / 1073741824).toFixed(1) + " GB";
}
function t(ptT, enT) { return A.locale && A.locale.startsWith("pt") ? ptT : enT; }

/* ---- Status mapping ---- */
function mapStatus(s) {
    if (s === "started") { return "running"; }
    if (s === "completed") { return "done"; }
    if (s === "needs_input") { return "needs-you"; }
    if (s === "failed") { return "failed"; }
    return s || "done";
}
function statusLabel(status) {
    var labels = { running: t("Executando", "Running"), done: t("Concluído", "Done"),
        failed: t("Falhou", "Failed"), "needs-you": t("Precisa de você", "Needs you") };
    return labels[status] || status;
}
function chipHtml(status) {
    return '<span class="gb-block-chip gb-block-chip--' + esc(status) + '">' + esc(statusLabel(status)) + '</span>';
}
function countHtml(text) {
    return text ? '<span class="gb-block-count">' + esc(text) + '</span>' : "";
}

/* ---- Generic block shell ---- */
function block(opts) {
    var status = opts.status || "done";
    var openByDefault = opts.open !== undefined ? opts.open : (status === "running" || status === "needs-you");
    var el = document.createElement("div");
    el.className = "gb-block gb-block--" + status + (opts.kind ? " gb-block-kind--" + opts.kind : "") + (openByDefault ? " open" : "");
    if (opts.activityId) { el.dataset.activityId = opts.activityId; }
    el.innerHTML =
        '<div class="gb-block-header" tabindex="0" role="button" aria-expanded="' + openByDefault + '">' +
            I("caret-right", 14, "gb-block-chevron") + '<div class="gb-block-icon-wrap">' + I(opts.icon || "info", 16) + '</div>' +
            '<span class="gb-block-title">' + esc(opts.title) + '</span>' +
            '<span class="gb-block-summary">' + esc(opts.summary || "") + '</span>' +
            countHtml(opts.count) + chipHtml(status) +
        '</div>' +
        '<div class="gb-block-body">' + (opts.bodyHtml || "") + '</div>' +
        (opts.footerHtml ? '<div class="gb-block-footer">' + opts.footerHtml + '</div>' : '');
    var hdr = el.querySelector(".gb-block-header");
    hdr.addEventListener("click", function () {
        el.dataset.userToggled = "1";
        var open = el.classList.toggle("open");
        hdr.setAttribute("aria-expanded", String(open));
    });
    hdr.addEventListener("keydown", function (e) {
        if (e.key === "Enter" || e.key === " ") { e.preventDefault(); hdr.click(); }
    });
    return el;
}

function row(label, value) {
    return '<div class="gb-block-row"><span class="gb-block-row-label">' + esc(label) + '</span><span class="gb-block-row-value">' + esc(value || "") + '</span></div>';
}
function noteRow(text, tone) {
    return '<div class="gb-block-row gb-block-note' + (tone ? " gb-block-note--" + tone : "") + '"><span class="gb-block-row-value">' + esc(text) + '</span></div>';
}

/* ---- File types: icon and a short mono badge (PDF, XLS, DOC...) ---- */
var TYPES = [
    ["image", "IMG", "jpg jpeg png gif webp bmp svg ico"],
    ["pdf", "PDF", "pdf"],
    ["doc", "DOC", "doc docx odt rtf"],
    ["sheet", "XLS", "xls xlsx ods csv"],
    ["slides", "PPT", "ppt pptx odp"],
    ["audio", "AUD", "mp3 wav ogg flac aac m4a"],
    ["video", "VID", "mp4 webm mkv avi mov"],
    ["archive", "ZIP", "zip gz tar rar 7z"],
    ["code", "COD", "js ts py rs html css json xml yaml toml sh bas"],
    ["text", "TXT", "txt md log"]
];
function fileType(name) {
    var ext = (String(name || "").split(".").pop() || "").toLowerCase();
    for (var i = 0; i < TYPES.length; i++) {
        if (TYPES[i][2].split(" ").indexOf(ext) >= 0) { return { kind: TYPES[i][0], badge: TYPES[i][1] }; }
    }
    return { kind: "file", badge: ext && ext.length <= 4 && ext !== String(name || "").toLowerCase() ? ext.toUpperCase() : t("ARQ", "FILE") };
}
function fileIcon(name) {
    var kind = fileType(name).kind;
    if (kind === "image") { return "image"; }
    if (kind === "archive") { return "download"; }
    if (kind === "code") { return "terminal"; }
    return "file";
}
/** Class that tints a file icon or badge with a state token by type. */
function fileTint(name) { return "gb-ft--" + fileType(name).kind; }
function fileBadge(name) {
    var type = fileType(name);
    return '<span class="gb-file-badge gb-ft--' + type.kind + '" aria-hidden="true">' + esc(type.badge) + '</span>';
}
A.fileTypeIcon = fileIcon;
A.fileTint = fileTint;

function updateBlockStatus(el, newStatus) {
    Array.from(el.classList).filter(function (name) {
        return name.indexOf("gb-block--") === 0;
    }).forEach(function (name) { el.classList.remove(name); });
    el.classList.add("gb-block--" + newStatus);
    var chip = el.querySelector(".gb-block-chip");
    if (chip) { chip.outerHTML = chipHtml(newStatus); }
}

/* Shared helpers for the split block modules. */
A._blk = {
    esc: esc, t: t, fmtBytes: fmtBytes, mapStatus: mapStatus, statusLabel: statusLabel,
    block: block, row: row, noteRow: noteRow, chipHtml: chipHtml,
    fileIcon: fileIcon, fileBadge: fileBadge, updateStatus: updateBlockStatus
};

/* ---- Registry ---- */
A.setPermissionActivityStatus = function (requestId, status) {
    document.querySelectorAll(".gb-block[data-permission-id]").forEach(function (el) {
        if (el.dataset.permissionId === requestId) { updateBlockStatus(el, status); }
    });
};

A.renderSingleBlock = function (ev) {
    var kind = ev.kind || ev.type || ev.event_type || "";
    var R = {
        "thinking": A.blockThinking, "attachment_read": A.blockAttachmentRead,
        "drive_list": A.blockDriveList, "drive_search": A.blockDriveSearch,
        "drive_read": A.blockDriveRead, "kb_search": A.blockKbSearch,
        "api_command": A.blockApiCommand, "tool_run": A.blockToolRun,
        "permission": A.blockPermission, "notice": A.blockNotice, "error": A.blockError,
        "file_generated": A.blockFileGenerated, "task_scheduled": A.blockTaskScheduled
    };
    var fn = R[kind];
    if (typeof fn === "function") { return fn(ev); }
    return A.blockGeneric ? A.blockGeneric(ev) : null;
};

A.renderBlockStack = function (events) {
    if (!events || !events.length) { return null; }
    if (events.length === 1) { return A.renderSingleBlock(events[0]); }
    var stack = document.createElement("div");
    stack.className = "gb-block-stack";
    events.forEach(function (ev) { var el = A.renderSingleBlock(ev); if (el) { stack.appendChild(el); } });
    return stack;
};
})();
