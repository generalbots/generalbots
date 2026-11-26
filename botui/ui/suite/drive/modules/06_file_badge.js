/* Drive parity with the gbgov assistant shell: compact file-type badge.
 *
 * A short format label (PDF, DOC, XLS, JPG, MD) next to the name makes the
 * file type visible at a glance in both grid and list views. Extension labels
 * are format names, not translatable copy. No new server call is involved.
 */
"use strict";

var DRIVE_BADGE_LABEL = {
    docx: "DOC", odt: "ODT", doc: "DOC", txt: "TXT", md: "MD",
    xlsx: "XLS", xls: "XLS", ods: "ODS", csv: "CSV",
    pptx: "PPT", ppt: "PPT", jpeg: "JPG", jpg: "JPG",
    png: "PNG", gif: "GIF", webp: "WEB", svg: "SVG", pdf: "PDF"
};

var DRIVE_BADGE_TONE = {
    pdf: "danger",
    jpg: "image", jpeg: "image", png: "image", gif: "image", webp: "image", svg: "image",
    doc: "ink", odt: "ink", txt: "ink", md: "ink",
    xls: "ok", ods: "ok", csv: "ok", ppt: "ok", pptx: "ok"
};

function driveFileBadge(name, isDir) {
    var ext = "";
    if (!isDir && name && name.indexOf(".") > 0) {
        ext = String(name).split(".").pop().toLowerCase();
    }
    var label = isDir ? "\u25A2" : ((DRIVE_BADGE_LABEL[ext] || ext.slice(0, 4)).toUpperCase() || "\u00B7");
    var tone = isDir ? "" : (DRIVE_BADGE_TONE[ext] || "");
    return '<span class="drive-file-badge' + (isDir ? " drive-file-badge-dir" : "") +
        (tone ? " drive-file-badge-" + tone : "") + '" aria-hidden="true">' +
        escapeHtml(label) + "</span>";
}

function driveDecorateFileBadges() {
    var content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content || !content.querySelectorAll) return;
    var rows = content.querySelectorAll(".file-card, .drive-file-item");
    for (var i = 0; i < rows.length; i++) {
        var row = rows[i];
        if (row.getAttribute("data-drive-badge") === "1") continue;
        row.setAttribute("data-drive-badge", "1");
        var host = row.querySelector(".file-card-preview") || row.querySelector(".file-name-col");
        if (!host) continue;
        var label = host.querySelector(".file-card-name") || host.querySelector("span:not(.kb-tag)");
        var html = driveFileBadge(row.getAttribute("data-name") || "", row.getAttribute("data-type") === "folder");
        if (label) label.insertAdjacentHTML("beforebegin", html);
        else host.insertAdjacentHTML("beforeend", html);
    }
}

// Decorate after every ordinary listing render. The grouped conversation and
// attachment views insert their own badges, so they are left untouched.
if (typeof renderFiles === "function") {
    var driveBaseRenderFiles = renderFiles;
    renderFiles = function (files) {
        driveBaseRenderFiles(files);
        driveDecorateFileBadges();
    };
}

window.DriveFileBadge = { render: driveFileBadge, decorate: driveDecorateFileBadges };
