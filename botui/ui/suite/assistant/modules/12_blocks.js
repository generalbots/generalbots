"use strict";
/**
 * 12_blocks.js - Block renderers for the conversation.
 *
 * Server event contract (gb.activity):
 * { gb_event:true, type:"CUSTOM", data:{ name:"gb.activity", payload:{
 *   id, kind, status:"started"|"completed"|"failed"|"needs_input",
 *   summary:{...}, error? }}}
 *
 * The same id arrives twice (started -> completed/failed). Update in place.
 * Activities of a turn group into one stack above the answer text.
 */
(function () {
var A = window.GBAssistant;
var I = window.GBIcon || function () { return ""; };
function esc(s) { return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;"); }
function fmtBytes(b) { return A.fmtBytes(b || 0); }
var pt = function () { return A.locale && A.locale.startsWith("pt"); };
function t(ptT, enT) { return pt() ? ptT : enT; }

/* ---- Status mapping ---- */
function mapStatus(s) {
    if (s === "started") return "running";
    if (s === "completed") return "done";
    if (s === "needs_input") return "needs-you";
    if (s === "failed") return "failed";
    return s || "done";
}
function chipHtml(status) {
    var labels = { running: t("Executando","Running"), done: t("Concluído","Done"),
        failed: t("Falhou","Failed"), "needs-you": t("Precisa de você","Needs you") };
    return '<span class="gb-block-chip gb-block-chip--' + esc(status) + '">' + esc(labels[status]||status) + '</span>';
}
function chevron() { return I("caret-right", 14, "gb-block-chevron"); }

/* ---- Generic block shell ---- */
function block(opts) {
    var status = opts.status || "done";
    var openByDefault = status === "running" || status === "needs-you";
    var cls = "gb-block gb-block--" + status + (openByDefault ? " open" : "");
    var el = document.createElement("div");
    el.className = cls;
    if (opts.activityId) el.dataset.activityId = opts.activityId;
    el.innerHTML =
        '<div class="gb-block-header" tabindex="0" role="button" aria-expanded="' + openByDefault + '">' +
            '<div class="gb-block-icon-wrap">' + I(opts.icon || "info", 16) + '</div>' +
            '<span class="gb-block-title">' + esc(opts.title) + '</span>' +
            '<span class="gb-block-summary">' + esc(opts.summary || "") + '</span>' +
            chipHtml(status) + chevron() +
        '</div>' +
        '<div class="gb-block-body">' + (opts.bodyHtml || "") + '</div>' +
        (opts.footerHtml ? '<div class="gb-block-footer">' + opts.footerHtml + '</div>' : '');
    var hdr = el.querySelector(".gb-block-header");
    hdr.addEventListener("click", function () {
        var open = el.classList.toggle("open");
        hdr.setAttribute("aria-expanded", String(open));
    });
    hdr.addEventListener("keydown", function (e) {
        if (e.key === "Enter" || e.key === " ") { e.preventDefault(); hdr.click(); }
    });
    return el;
}

/* ---- File-type icon with tint ---- */
function fileIcon(name) {
    if (!name) return "file";
    var ext = (name.split(".").pop() || "").toLowerCase();
    if ("jpg jpeg png gif webp bmp svg ico".split(" ").indexOf(ext) >= 0) return "image";
    if (ext === "pdf") return "file";
    if ("doc docx odt rtf".split(" ").indexOf(ext) >= 0) return "file";
    if ("xls xlsx ods csv".split(" ").indexOf(ext) >= 0) return "file";
    if ("ppt pptx odp".split(" ").indexOf(ext) >= 0) return "file";
    if ("mp3 wav ogg flac aac m4a".split(" ").indexOf(ext) >= 0) return "file";
    if ("mp4 webm mkv avi mov".split(" ").indexOf(ext) >= 0) return "file";
    if ("zip gz tar rar 7z".split(" ").indexOf(ext) >= 0) return "download";
    if ("js ts py rs html css json xml yaml toml sh bas".split(" ").indexOf(ext) >= 0) return "terminal";
    if ("txt md log".split(" ").indexOf(ext) >= 0) return "file";
    return "file";
}
function fileTint(name) {
    if (!name) return "";
    var ext = (name.split(".").pop() || "").toLowerCase();
    if ("jpg jpeg png gif webp bmp svg ico".split(" ").indexOf(ext) >= 0) return "color:var(--id-hl-keyword)";
    if (ext === "pdf") return "color:var(--id-hl-tag)";
    if ("doc docx odt rtf".split(" ").indexOf(ext) >= 0) return "color:var(--id-hl-function)";
    if ("xls xlsx ods csv".split(" ").indexOf(ext) >= 0) return "color:var(--id-hl-string)";
    if ("ppt pptx odp".split(" ").indexOf(ext) >= 0) return "color:var(--id-hl-number)";
    if ("zip gz tar rar 7z".split(" ").indexOf(ext) >= 0) return "color:var(--id-gray-500)";
    if ("js ts py rs html css json xml yaml toml sh bas".split(" ").indexOf(ext) >= 0) return "color:var(--id-hl-attr)";
    return "";
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
                ? '<div class="gb-block-row"><span class="gb-block-row-value" style="color:var(--id-warn)">' + esc(failLabel) + '</span></div>'
                : "") +
            (noTextLabel
                ? '<div class="gb-block-row"><span class="gb-block-row-value" style="color:var(--id-warn)">' + esc(noTextLabel) + '</span></div>'
                : "") +
            row(t("Arquivo","File"), s.name||s.file) +
            (s.type||s.content_type ? row(t("Tipo","Type"), s.type||s.content_type) : "") +
            (s.size||s.size_bytes ? row(t("Tamanho","Size"), fmtBytes(s.size||s.size_bytes)) : "") +
            (s.chars_read ? row(t("Lido","Read"), s.chars_read + " " + A.t("blocks.unit-chars", "chars")) : "") +
            (s.truncated ? '<div class="gb-block-row"><span class="gb-block-row-value" style="color:var(--id-warn)">' + t("Truncado","Truncated") + '</span></div>' : "") });
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

/* File the assistant created and saved to Drive. */
A.blockFileGenerated = function (ev) {
    var s = ev.summary || ev.data || ev;
    var name = s.name || s.file || "";
    var drivePath = s.drive_path || "";
    var el = block({ kind: "file-out", icon: "file", title: t("Arquivo gerado", "Generated file"),
        summary: name, status: mapStatus(ev.status), activityId: ev.id, open: true,
        bodyHtml: '<div class="gb-block-out">' +
            '<span class="gb-block-out-icon">' + I(fileIcon(name), 16) + '</span>' +
            '<div class="gb-block-out-text"><span class="gb-block-out-name">' + esc(name) + '</span>' +
            (s.detail ? '<span class="gb-block-out-meta">' + esc(s.detail) + '</span>' : "") + '</div>' +
            (drivePath ? '<button type="button" class="gb-block-btn" data-act="open">' + esc(t("Abrir", "Open")) + '</button>' +
                '<button type="button" class="gb-block-btn gb-block-btn--acc" data-act="drive">' + esc(t("Ver no Drive", "Show in Drive")) + '</button>' : "") +
            '</div>' });
    var open = el.querySelector('[data-act="open"]');
    if (open && typeof A.makeAttachClickable === "function" && name) {
        A.makeAttachClickable(open, { name: name, size: s.size }, { drivePath: drivePath });
    }
    var drive = el.querySelector('[data-act="drive"]');
    if (drive) { drive.addEventListener("click", function () { if (A.switchView) A.switchView("drive"); }); }
    return el;
};

/* A recurring task the assistant set up. */
A.blockTaskScheduled = function (ev) {
    var s = ev.summary || ev.data || ev;
    return block({ kind: "schedule", icon: "clock", title: t("Tarefa agendada", "Scheduled task"),
        summary: s.name || "", status: mapStatus(ev.status), activityId: ev.id, open: true,
        bodyHtml: (s.when ? row(t("Quando", "When"), s.when) : "") +
            (s.does ? row(t("Faz", "Does"), s.does) : "") +
            (s.where ? row(t("Onde", "Where"), s.where) : "") });
};

function row(label, value) {
    return '<div class="gb-block-row"><span class="gb-block-row-label">' + esc(label) + '</span><span class="gb-block-row-value">' + esc(value||"") + '</span></div>';
}

function updateBlockStatus(el, newStatus) {
    el.className = el.className.replace(/gb-block--\w+/g, "") + " gb-block--" + newStatus;
    var chip = el.querySelector(".gb-block-chip");
    if (chip) { chip.className = "gb-block-chip gb-block-chip--" + newStatus; chip.textContent = { running: t("Executando","Running"), done: t("Concluído","Done"), failed: t("Falhou","Failed") }[newStatus] || newStatus; }
}

/* Shared helpers for the split block modules (12b_block_live, 12d_block_vm). */
A._blk = {
    esc: esc, t: t, fmtBytes: fmtBytes, mapStatus: mapStatus,
    block: block, row: row, chipHtml: chipHtml,
    fileIcon: fileIcon, updateStatus: updateBlockStatus
};

/* Update a permission block's status when consent is resolved. */
A.setPermissionActivityStatus = function (requestId, status) {
    document.querySelectorAll(".gb-block[data-permission-id]").forEach(function (el) {
        if (el.dataset.permissionId === requestId) { updateBlockStatus(el, status); }
    });
};

/* ---- Render a stack of blocks ---- */
A.renderBlockStack = function (events) {
    if (!events || !events.length) return null;
    if (events.length === 1) return renderSingleBlock(events[0]);
    var stack = document.createElement("div");
    stack.className = "gb-block-stack";
    events.forEach(function (ev) { var el = renderSingleBlock(ev); if (el) stack.appendChild(el); });
    return stack;
};

function renderSingleBlock(ev) {
    var kind = ev.kind || ev.type || ev.event_type || "";
    var R = {
        "thinking": A.blockThinking, "attachment_read": A.blockAttachmentRead,
        "drive_list": A.blockDriveList, "drive_search": A.blockDriveSearch,
        "drive_read": A.blockDriveRead, "kb_search": A.blockKbSearch,
        "api_command": A.blockApiCommand, "tool_run": A.blockToolRun,
        "permission": A.blockPermission, "notice": A.blockNotice, "error": A.blockError,
        "file_generated": A.blockFileGenerated, "task_scheduled": A.blockTaskScheduled
    };
    var fn = R[kind]; if (fn) return fn(ev);
    return A.blockGeneric(ev);
}
A.renderSingleBlock = renderSingleBlock;

/* ---- Live activity event handler (from WS gb_event frames) ---- */
var pendingActivities = []; /* accumulate during a turn */

A.onActivityEvent = function (frame) {
    if (!frame.data || frame.data.name !== "gb.activity") return;
    var payload = frame.data.payload;
    if (!payload || !payload.id) return;
    /* Check if this activity already has a block rendered */
    var existing = document.querySelector('[data-activity-id="' + payload.id + '"]');
    if (existing) {
        /* Update in place */
        updateBlockStatus(existing, mapStatus(payload.status));
        if (payload.error) {
            var body = existing.querySelector(".gb-block-body");
            if (body) body.innerHTML = '<div class="gb-block-row"><span class="gb-block-row-value" style="color:var(--id-danger)">' + esc(payload.error) + '</span></div>';
        }
        /* Refresh the body on attachment_read completion with the final
           summary data (chars_read, drive_path, etc.). */
        if (payload.kind === "attachment_read" && payload.summary &&
            (payload.status === "completed" || payload.status === "failed")) {
            var ev2 = { id: payload.id, kind: payload.kind, status: payload.status,
                summary: payload.summary, data: payload.summary };
            if (payload.error) ev2.error = payload.error;
            var fresh = A.blockAttachmentRead(ev2);
            var freshBody = fresh.querySelector(".gb-block-body");
            var existBody = existing.querySelector(".gb-block-body");
            if (freshBody && existBody) existBody.innerHTML = freshBody.innerHTML;
        }
        return;
    }
    /* New activity: render and append to the current turn's stack */
    var ev = { id: payload.id, kind: payload.kind, status: payload.status,
        summary: payload.summary || {}, data: payload.summary || {} };
    if (payload.error) ev.data.error = payload.error;
    var blockEl = renderSingleBlock(ev);
    if (!blockEl) return;
    var msgs = document.getElementById("gbMessages");
    if (!msgs) return;
    /* Find or create the turn's activity stack (only after the last
       user message to avoid adopting a previous turn's stack). */
    var stack = null, children = msgs.children;
    var lastUserIdx = -1;
    for (var ui = children.length - 1; ui >= 0; ui--) { if (children[ui].classList.contains("gb-msg-user")) { lastUserIdx = ui; break; } }
    for (var ci = children.length - 1; ci > lastUserIdx && ci >= Math.max(0, children.length - 4); ci--) {
        var ch = children[ci];
        if (ch.classList.contains("gb-msg-assistant")) {
            var s = ch.querySelector(".gb-block-stack");
            if (s) { stack = s; break; }
        }
    }
    if (!stack) {
        stack = document.createElement("div");
        stack.className = "gb-block-stack";
        /* Insert BEFORE the streaming element if one exists, so blocks group above the answer */
        var streamingEl = msgs.querySelector(".gb-streaming");
        var turn = document.createElement("div");
        turn.className = "gb-msg-assistant";
        turn.appendChild(stack);
        if (streamingEl && streamingEl.parentElement === msgs) {
            msgs.insertBefore(turn, streamingEl);
        } else {
            msgs.appendChild(turn);
        }
    }
    stack.appendChild(blockEl);
    if (!A.isUserScrolledUp || !A.isUserScrolledUp()) { A.scrollToBottom(false); }
};

/* attachCard and addMsgToolbar live in 12b_blocks_activity.js (split for 450-line limit). */
})();
