"use strict";
/**
 * 12b_blocks_activity.js - Attachment card and hover toolbar for the
 * conversation blocks declared in 12_blocks.js. Split out so every module
 * stays under the 450-line limit; behaviour is unchanged.
 */
(function () {
var A = window.GBAssistant;
var I = window.GBIcon || function () { return ""; };
function esc(s) { return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;"); }
function fmtBytes(b) { return A.fmtBytes(b || 0); }
var pt = function () { return A.locale && A.locale.startsWith("pt"); };
function t(ptT, enT) { return pt() ? ptT : enT; }
/* 12_blocks.js owns the file-type helpers; reuse them instead of duplicating. */
function fileIcon(name) { return A.fileTypeIcon ? A.fileTypeIcon(name) : "file"; }
function fileTint(name) { return A.fileTint ? A.fileTint(name) : ""; }

/* ---- Attachment card ---- */
function truncMiddle(s, max) { if (!s || s.length <= max) return s; var h = Math.floor((max-3)/2); return s.substring(0,h) + "..." + s.substring(s.length-h); }

A.attachCard = function (file, opts) {
    opts = opts || {};
    var name = file.name || (typeof file === "string" ? file : "");
    var size = file.size || file.size_bytes || 0;
    var isImage = /\.(jpg|jpeg|png|gif|webp|bmp|svg)$/i.test(name);
    var icon = fileIcon(name); var tint = fileTint(name);
    var overLimit = size > 50 * 1024 * 1024;
    var card = document.createElement("div");
    card.className = "gb-attach-card" + (overLimit ? " gb-attach-error" : "");
    if (isImage && file instanceof File && !overLimit) {
        var thumb = document.createElement("img");
        thumb.className = "gb-attach-thumb";
        try { thumb.src = URL.createObjectURL(file); } catch (e) { /* */ }
        card.appendChild(thumb);
    }
    var displayName = truncMiddle(name, 28);
    var html = '<span class="gb-attach-icon"' + (tint ? ' style="' + tint + '"' : '') + '>' + I(icon, 16) + '</span>';
    html += '<span class="gb-attach-name"' + (name.length > 28 ? ' title="' + esc(name) + '"' : '') + '>' + esc(displayName) + '</span>';
    html += '<span class="gb-attach-size">' + (overLimit ? esc(t("Limite: 50 MB","Limit: 50 MB")) : (size > 0 ? fmtBytes(size) : '')) + '</span>';
    if (opts.removable !== false) {
        html += '<button class="gb-attach-remove" aria-label="' + esc(t("Remover","Remove")) + '">' + I("x-circle", 16) + '</button>';
    }
    card.insertAdjacentHTML("beforeend", html);
    if (opts.onRemove) { var rm = card.querySelector(".gb-attach-remove"); if (rm) rm.addEventListener("click", opts.onRemove); }
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

/* ---- Hover toolbar ---- */
A.addMsgToolbar = function (el, content, retryable) {
    var tb = document.createElement("div"); tb.className = "gb-msg-toolbar";
    if (content) {
        var cp = document.createElement("button"); cp.className = "gb-msg-toolbar-btn"; cp.title = t("Copiar","Copy");
        cp.innerHTML = I("copy", 14);
        cp.addEventListener("click", function () {
            navigator.clipboard.writeText(content).catch(function (err) {
                console.warn("[assistant] toolbar copy failed", err);
            });
        });
        tb.appendChild(cp);
    }
    if (retryable) {
        var rt = document.createElement("button"); rt.className = "gb-msg-toolbar-btn"; rt.title = t("Tentar de novo","Try again");
        rt.innerHTML = I("arrows-clockwise", 14); rt.addEventListener("click", function () {
            var u = document.querySelectorAll("#gbMessages .gb-msg-user"), l = u[u.length-1];
            var txt = l && (l.dataset.originalContent||l.textContent||"").trim();
            if (!txt) return;
            el.remove(); A.lastCompletedContent = ""; A.lastCompletedTime = 0; A.showThinking();
            if (!A.sendMessage(txt)) { A.hideThinking(); A.renderErrorCode("model_unreachable"); }
        }); tb.appendChild(rt);
    }
    el.appendChild(tb);
};
})();
