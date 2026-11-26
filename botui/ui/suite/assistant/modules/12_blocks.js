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

/* ---- Block types (consume server contract shapes) ---- */

A.blockThinking = function (evOrElapsed, reasoning) {
    var elapsed = evOrElapsed, reason = reasoning;
    if (evOrElapsed && typeof evOrElapsed === "object") {
        var d = evOrElapsed.summary || evOrElapsed.data || evOrElapsed;
        elapsed = d.elapsed || 0; reason = d.reasoning || "";
    }
    return block({ icon: "brain", title: t("Pensando","Thinking"),
        summary: elapsed ? elapsed + " s" : "", status: reason ? "done" : "running",
        bodyHtml: reason ? '<div class="gb-block-row" style="white-space:pre-wrap;max-height:200px;overflow-y:auto">' + esc(reason) + '</div>' : '' });
};

A.blockAttachmentRead = function (ev) {
    var s = ev.summary || ev.data || ev;
    var isFailed = ev.status === "failed";
    var failReason = isFailed ? (ev.error || "") : "";
    /* #2561 — show a clear reason when image reading is not installed */
    var failLabel = "";
    if (failReason === "image_reader_unavailable") {
        failLabel = t(
            "Não foi possível ler a imagem: a leitura de imagens não está instalada",
            "Could not read the image: image reading is not installed"
        );
    } else if (isFailed && failReason) {
        failLabel = failReason;
    }
    /* Completed but no text extracted — file format not readable */
    var isCompleted = ev.status === "completed";
    var noTextExtracted = isCompleted && !s.chars_read;
    var noTextLabel = noTextExtracted
        ? t("Formato não legível: o texto não pôde ser extraído",
            "Format not readable: text could not be extracted")
        : "";
    var el = block({ icon: "paperclip", title: t("Leitura de anexo","Attachment read"),
        summary: s.name || s.file || failLabel || "", status: mapStatus(ev.status), activityId: ev.id,
        bodyHtml: (isFailed && failLabel
                ? '<div class="gb-block-row"><span class="gb-block-row-value" style="color:var(--id-warn,#a16207)">' + esc(failLabel) + '</span></div>'
                : "") +
            (noTextLabel
                ? '<div class="gb-block-row"><span class="gb-block-row-value" style="color:var(--id-warn,#a16207)">' + esc(noTextLabel) + '</span></div>'
                : "") +
            row(t("Arquivo","File"), s.name||s.file) +
            (s.type||s.content_type ? row(t("Tipo","Type"), s.type||s.content_type) : "") +
            (s.size||s.size_bytes ? row(t("Tamanho","Size"), fmtBytes(s.size||s.size_bytes)) : "") +
            (s.chars_read ? row(t("Lido","Read"), s.chars_read + " " + A.t("blocks.unit-chars", "chars")) : "") +
            (s.truncated ? '<div class="gb-block-row"><span class="gb-block-row-value" style="color:#a16207">' + t("Truncado","Truncated") + '</span></div>' : "") });
    /* Make the block clickable to open the file preview */
    if (typeof A.makeAttachClickable === "function" && (s.name || s.file)) {
        var hdr = el.querySelector(".gb-block-header");
        if (hdr) {
            hdr.style.cursor = "pointer";
            hdr.addEventListener("dblclick", function () {
                A.makeAttachClickable(el, { name: s.name || s.file, size: s.size || s.size_bytes, type: s.type || s.content_type },
                    { drivePath: s.drive_path || "" });
            });
        }
    }
    return el;
};

A.blockDriveList = function (ev) {
    var s = ev.summary || ev.data || ev;
    var items = s.entries || s.items || [];
    var rows = items.map(function (f) {
        var ic = f.kind === "folder" ? "folder" : fileIcon(f.name||"");
        var tint = f.kind === "folder" ? "" : fileTint(f.name||"");
        return '<div class="gb-block-file">' + I(ic, 14, "gb-block-file-icon") +
            '<span class="gb-block-file-name">' + esc(f.name||f.path||"") + '</span>' +
            '<span class="gb-block-file-meta">' + (f.size ? fmtBytes(f.size) : "") + '</span></div>';
    }).join("");
    var total = s.total || items.length;
    return block({ icon: "folder-open", title: t("Listagem no Drive","Drive listing"),
        summary: (s.path||s.query||"") + " · " + total + " " + A.t("blocks.unit-items", "items"),
        status: mapStatus(ev.status), activityId: ev.id, bodyHtml: rows });
};

A.blockDriveSearch = function (ev) {
    var s = ev.summary || ev.data || ev;
    var items = s.entries || s.results || [];
    var rows = items.map(function (f) {
        return '<div class="gb-block-file">' + I(fileIcon(f.name||""), 14, "gb-block-file-icon") +
            '<span class="gb-block-file-name">' + esc(f.name||"") + '</span>' +
            '<span class="gb-block-file-meta">' + (f.size ? fmtBytes(f.size) : "") + '</span></div>';
    }).join("");
    var total = s.total || items.length;
    return block({ icon: "magnifying-glass", title: t("Pesquisa no Drive","Drive search"),
        summary: (s.query||"") + " · " + total + " " + A.t("blocks.unit-results", "results"),
        status: mapStatus(ev.status), activityId: ev.id, bodyHtml: rows });
};

A.blockDriveRead = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ icon: "eye", title: t("Leitura de arquivo","File read"),
        summary: s.name || s.path || "", status: mapStatus(ev.status), activityId: ev.id,
        bodyHtml: row(t("Arquivo","File"), s.name||s.path) +
            (s.size ? row(t("Tamanho","Size"), fmtBytes(s.size)) : "") });
};

A.blockKbSearch = function (ev) {
    var s = ev.summary || ev.data || ev;
    var sources = s.results || s.sources || [];
    var total = s.total || sources.length;
    var rows = sources.map(function (r) {
        return '<div class="gb-block-source"><div class="gb-block-source-title">' + esc(r.document||r.title||"") + '</div>' +
            (r.excerpt ? '<div class="gb-block-source-excerpt">' + esc(r.excerpt) + '</div>' : '') + '</div>';
    }).join("");
    return block({ icon: "magnifying-glass", title: t("Base de conhecimento","Knowledge base"),
        summary: total + " " + A.t("blocks.unit-sources", "sources"), status: mapStatus(ev.status),
        activityId: ev.id, bodyHtml: rows });
};

A.blockApiCommand = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ icon: "terminal", title: s.command || t("Comando","Command"),
        summary: s.status || "", status: mapStatus(ev.status), activityId: ev.id });
};

A.blockToolRun = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ icon: "terminal", title: s.tool || s.name || t("Ferramenta","Tool"),
        summary: s.outcome || s.result || "", status: mapStatus(ev.status), activityId: ev.id });
};

A.blockPermission = function (ev) {
    var s = ev.summary || ev.data || ev;
    var pending = mapStatus(ev.status) === "needs-you";
    var footerHtml = "";
    if (pending && typeof A.consentBuildCard === "function" && s.request_id) {
        footerHtml = '<button class="gb-set-btn-sm gb-set-btn-accent" data-consent-allow="' + esc(s.request_id) + '">' +
            esc(t("Permitir uma vez","Allow once")) + '</button>' +
            '<button class="gb-set-btn-sm gb-set-btn-danger" data-consent-deny="' + esc(s.request_id) + '">' +
            esc(t("Negar","Deny")) + '</button>';
    }
    var b = block({ icon: "shield-check", title: t("Permissão","Permission"),
        summary: (s.action || s.app_id || "") + (s.action_class ? " · " + s.action_class : ""),
        status: pending ? "needs-you" : "done", activityId: ev.id,
        bodyHtml: s.detail || s.description ? '<div class="gb-block-row"><span class="gb-block-row-value">' + esc(s.detail||s.description) + '</span></div>' : '',
        footerHtml: footerHtml });
    if (pending) bindConsentButtons(b, s.request_id);
    return b;
};

function bindConsentButtons(el, requestId) {
    el.querySelectorAll("[data-consent-allow]").forEach(function (btn) {
        btn.addEventListener("click", function () { resolveConsent(el, requestId, "allow_once"); });
    });
    el.querySelectorAll("[data-consent-deny]").forEach(function (btn) {
        btn.addEventListener("click", function () { resolveConsent(el, requestId, "deny"); });
    });
}

function resolveConsent(el, requestId, decision) {
    el.querySelectorAll(".gb-set-btn-sm").forEach(function (b) { b.disabled = true; });
    A.apiFetch("/api/consent/resolve", {
        method: "POST", body: JSON.stringify({ request_id: requestId, decision: decision })
    }).then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        updateBlockStatus(el, decision === "deny" ? "done" : "done");
        if (decision !== "deny" && typeof A.sendMessage === "function") {
            setTimeout(function () { A.sendMessage("Approved. Continue."); }, 200);
        }
    }).catch(function () { updateBlockStatus(el, "failed"); });
}

A.blockNotice = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ icon: "info", title: t("Indisponível","Unavailable"),
        summary: s.message || t("nesta edição","in this edition"),
        status: "done", activityId: ev.id });
};

A.blockError = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ icon: "warning-circle", title: t("Erro","Error"),
        summary: s.message || s.error || "", status: "failed", activityId: ev.id,
        footerHtml: '<button class="gb-set-btn-sm gb-msg-retry" onclick="GBAssistant.regenerateFrom(this.closest(\'.gb-msg-assistant\'))">' +
            esc(t("Tentar de novo","Try again")) + '</button>' });
};

/* Unknown kind: generic step block */
A.blockGeneric = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ icon: "info", title: ev.kind || t("Etapa","Step"),
        summary: JSON.stringify(s).substring(0, 80), status: mapStatus(ev.status), activityId: ev.id });
};

function row(label, value) {
    return '<div class="gb-block-row"><span class="gb-block-row-label">' + esc(label) + '</span><span class="gb-block-row-value">' + esc(value||"") + '</span></div>';
}

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
        rt.innerHTML = I("arrows-clockwise", 14);
        rt.addEventListener("click", function () { A.regenerateFrom(el); }); tb.appendChild(rt);
    }
    el.appendChild(tb);
};
})();
