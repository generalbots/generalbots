"use strict";
/**
 * 12c_attach_card.js - Attachment cards (composer and history) and the
 * answer toolbar. Split from 12_blocks.js.
 */
(function () {
var A = window.GBAssistant;
var B = A._blk;
var I = window.GBIcon || function () { return ""; };
var esc = B.esc, t = B.t;

function truncMiddle(s, max) {
    if (!s || s.length <= max) { return s; }
    var h = Math.floor((max - 3) / 2);
    return s.substring(0, h) + "..." + s.substring(s.length - h);
}

A.attachCard = function (file, opts) {
    opts = opts || {};
    var name = file.name || (typeof file === "string" ? file : "");
    var size = file.size || file.size_bytes || 0;
    var isImage = /\.(jpg|jpeg|png|gif|webp|bmp|svg)$/i.test(name);
    var overLimit = size > 50 * 1024 * 1024;
    var card = document.createElement("div");
    card.className = "gb-attach-card" + (overLimit ? " gb-attach-error" : "");
    if (isImage && file instanceof File && !overLimit) {
        var thumb = document.createElement("img");
        thumb.className = "gb-attach-thumb";
        try { thumb.src = URL.createObjectURL(file); } catch (e) { thumb.remove(); }
        card.appendChild(thumb);
    }
    var displayName = truncMiddle(name, 28);
    var html = '<span class="gb-attach-icon ' + A.fileTint(name) + '">' + I(A.fileTypeIcon(name), 16) + '</span>';
    html += '<span class="gb-attach-name"' + (name.length > 28 ? ' title="' + esc(name) + '"' : '') + '>' + esc(displayName) + '</span>';
    html += '<span class="gb-attach-size">' + (overLimit ? esc(t("Limite: 50 MB", "Limit: 50 MB")) : (size > 0 ? B.fmtBytes(size) : '')) + '</span>';
    if (opts.removable !== false) {
        html += '<button class="gb-attach-remove" aria-label="' + esc(t("Remover", "Remove")) + '">' + I("x-circle", 16) + '</button>';
    }
    card.insertAdjacentHTML("beforeend", html);
    if (opts.onRemove) { var rm = card.querySelector(".gb-attach-remove"); if (rm) { rm.addEventListener("click", opts.onRemove); } }
    /* Auto-detect drivePath from history attachment objects. */
    if (!opts.drivePath && file && typeof file === "object" && !(file instanceof File)) {
        var dp = file.drive_path || file.drivePath || (file.name ? "inbox/" + file.name : "");
        if (dp) { opts = Object.assign({}, opts, { drivePath: dp }); }
    }
    /* Make clickable for preview (unless removable = composer) */
    if (typeof A.makeAttachClickable === "function" && !opts.removable) {
        A.makeAttachClickable(card, file, opts);
    }
    return card;
};

/* ---- Answer toolbar ---- */
A.addMsgToolbar = function (el, content, retryable) {
    var tb = document.createElement("div");
    tb.className = "gb-msg-toolbar";
    if (content) {
        var cp = document.createElement("button");
        cp.className = "gb-msg-toolbar-btn";
        cp.title = t("Copiar", "Copy");
        cp.innerHTML = I("copy", 14);
        cp.addEventListener("click", function () {
          navigator.clipboard.writeText(content).catch(function (e) {
            // Clipboard can be blocked by the WebView; copy is best-effort.
            if (window.gbLogError) { window.gbLogError(e, "copy message"); }
          });
        });
        tb.appendChild(cp);
    }
    if (retryable) {
        var rt = document.createElement("button");
        rt.className = "gb-msg-toolbar-btn";
        rt.title = t("Tentar de novo", "Try again");
        rt.innerHTML = I("arrows-clockwise", 14);
        rt.addEventListener("click", function () { A.regenerateFrom(el); });
        tb.appendChild(rt);
    }
    el.appendChild(tb);
};
})();
