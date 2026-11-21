"use strict";
/**
 * 12a_block_kinds.js - One renderer per activity kind.
 *
 * Shapes come from botserver pipeline/activity.rs. `file_generated` and
 * `task_scheduled` have no server event yet: they render when an event with
 * that kind arrives (summary: name, size, drive_path, detail / name, when,
 * does, where) and are otherwise unused.
 */
(function () {
var A = window.GBAssistant;
var B = A._blk;
var I = window.GBIcon || function () { return ""; };
var esc = B.esc, t = B.t, row = B.row, block = B.block, fmtBytes = B.fmtBytes;

function sum(ev) { return ev.summary || ev.data || ev; }
function plural(n, one, many) { return n + " " + (n === 1 ? one : many); }

function fileRows(items, sizeKey) {
    return '<div class="gb-block-list">' + items.map(function (f) {
        var name = f.name || f.path || "";
        var lead = f.kind === "folder" ? I("folder", 14, "gb-block-file-icon") : B.fileBadge(name);
        return '<div class="gb-block-file">' + lead +
            '<span class="gb-block-file-name">' + esc(name) + '</span>' +
            '<span class="gb-block-file-meta">' + (f[sizeKey || "size"] ? fmtBytes(f[sizeKey || "size"]) : "") + '</span></div>';
    }).join("") + '</div>';
}

A.blockThinking = function (evOrElapsed, reasoning) {
    var elapsed = evOrElapsed, reason = reasoning;
    if (evOrElapsed && typeof evOrElapsed === "object") {
        var d = sum(evOrElapsed);
        elapsed = d.elapsed || 0; reason = d.reasoning || "";
    }
    return block({ kind: "thinking", icon: "brain", title: t("Raciocínio", "Reasoning"),
        summary: elapsed ? t("Pensou por ", "Thought for ") + elapsed + " s" : "", status: reason ? "done" : "running",
        bodyHtml: reason ? '<div class="gb-block-prose">' + esc(reason) + '</div>' : "" });
};

A.blockAttachmentRead = function (ev) {
    var s = sum(ev);
    var isFailed = ev.status === "failed";
    var failReason = isFailed ? (ev.error || s.error || "") : "";
    var failLabel = "";
    if (failReason === "image_reader_unavailable") {
        failLabel = t("Não foi possível ler a imagem: a leitura de imagens não está instalada",
            "Could not read the image: image reading is not installed");
    } else if (isFailed && failReason) {
        failLabel = failReason;
    }
    var noText = ev.status === "completed" && !s.chars_read && !s.seen_by_model;
    var name = s.name || s.file || "";
    var el = block({ kind: "file-read", icon: "paperclip", title: t("Leitura de anexo", "Attachment read"),
        summary: name || failLabel, status: B.mapStatus(ev.status), activityId: ev.id,
        bodyHtml: (failLabel ? B.noteRow(failLabel, "warn") : "") +
            (noText ? B.noteRow(t("Formato não legível: o texto não pôde ser extraído",
                "Format not readable: text could not be extracted"), "warn") : "") +
            row(t("Arquivo", "File"), name) +
            (s.type || s.content_type ? row(t("Tipo", "Type"), s.type || s.content_type) : "") +
            (s.size || s.size_bytes ? row(t("Tamanho", "Size"), fmtBytes(s.size || s.size_bytes)) : "") +
            (s.chars_read ? row(t("Lido", "Read"), s.chars_read + " " + t("caracteres", "characters")) : "") +
            (s.seen_by_model ? row(t("Enviado ao modelo", "Sent to the model"), t("o próprio arquivo, para ele ver", "the file itself, for it to see")) : "") +
            (s.truncated ? B.noteRow(t("Truncado: só o começo foi ao modelo", "Truncated: only the start went to the model"), "warn") : "") });
    if (typeof A.makeAttachClickable === "function" && name) {
        var hdr = el.querySelector(".gb-block-header");
        hdr.addEventListener("dblclick", function () {
            A.makeAttachClickable(el, { name: name, size: s.size || s.size_bytes, type: s.type || s.content_type },
                { drivePath: s.drive_path || "" });
        });
    }
    return el;
};

A.blockDriveList = function (ev) {
    var s = sum(ev);
    var items = s.entries || s.items || [];
    var total = s.total || items.length;
    return block({ kind: "drive", icon: "folder-open", title: t("Listagem no Drive", "Drive listing"),
        summary: s.path || s.query || "", count: ev.status === "started" ? "" : plural(total, t("item", "item"), t("itens", "items")),
        status: B.mapStatus(ev.status), activityId: ev.id, bodyHtml: items.length ? fileRows(items) : "" });
};

A.blockDriveSearch = function (ev) {
    var s = sum(ev);
    var items = s.entries || s.results || [];
    var total = s.total || items.length;
    var q = s.query || s.path || "";
    return block({ kind: "drive", icon: "magnifying-glass", title: t("Pesquisa no Drive", "Drive search"),
        summary: q ? "“" + q + "”" : "", count: ev.status === "started" ? "" : plural(total, t("resultado", "result"), t("resultados", "results")),
        status: B.mapStatus(ev.status), activityId: ev.id, bodyHtml: items.length ? fileRows(items) : "" });
};

/* A file read from the Drive by the assistant (not an attachment). The
   server sends the command name today; name, size and lines when present. */
A.blockDriveRead = function (ev) {
    var s = sum(ev);
    var name = s.name || s.path || s.file || "";
    var read = s.lines ? plural(s.lines, t("linha", "line"), t("linhas", "lines")) :
        (s.chars_read ? s.chars_read + " " + t("caracteres", "characters") : "");
    var body = (name ? row(t("Arquivo", "File"), name) : "") +
        (s.size ? row(t("Tamanho", "Size"), fmtBytes(s.size)) : "") +
        (read ? row(t("Lido", "Read"), read) : "") +
        (s.sent_to_model ? row(t("Enviado ao modelo", "Sent to the model"), s.sent_to_model) : "");
    return block({ kind: "file-read", icon: "file", title: t("Leitura de arquivo", "File read"),
        summary: name || t("arquivo do Drive", "Drive file"), status: B.mapStatus(ev.status), activityId: ev.id,
        bodyHtml: body });
};

A.blockKbSearch = function (ev) {
    var s = sum(ev);
    var sources = s.results || s.sources || [];
    var total = s.total || sources.length;
    var rows = sources.map(function (r, i) {
        return '<div class="gb-block-source"><span class="gb-block-source-n">' + (i + 1) + '</span>' +
            '<div class="gb-block-source-text"><div class="gb-block-source-title">' + esc(r.document || r.title || "") + '</div>' +
            (r.excerpt ? '<div class="gb-block-source-excerpt">' + esc(r.excerpt) + '</div>' : "") + '</div></div>';
    }).join("");
    return block({ kind: "kb", icon: "magnifying-glass", title: t("Base de conhecimento", "Knowledge base"),
        summary: s.query || "", count: ev.status === "started" ? "" : plural(total, t("trecho", "excerpt"), t("trechos", "excerpts")),
        status: B.mapStatus(ev.status), activityId: ev.id, bodyHtml: rows ? '<div class="gb-block-list">' + rows + '</div>' : "" });
};

A.blockApiCommand = function (ev) {
    var s = sum(ev);
    return block({ kind: "command", icon: "terminal", title: s.command || t("Comando", "Command"),
        summary: "", status: B.mapStatus(ev.status), activityId: ev.id });
};

A.blockToolRun = function (ev) {
    var s = sum(ev);
    return block({ kind: "command", icon: "terminal", title: s.tool || s.name || t("Ferramenta", "Tool"),
        summary: s.outcome || s.result || "", status: B.mapStatus(ev.status), activityId: ev.id });
};

A.blockPermission = function (ev) {
    var s = sum(ev);
    var el = block({ kind: "permission", icon: "shield-check", title: t("Permissão", "Permission"),
        summary: (s.action || s.app_id || "") + (s.action_class ? " · " + s.action_class : ""),
        status: B.mapStatus(ev.status), activityId: ev.id,
        bodyHtml: s.description ? B.noteRow(s.description) : "" });
    if (s.request_id) { el.dataset.permissionId = s.request_id; }
    return el;
};

A.blockNotice = function (ev) {
    var s = sum(ev);
    return block({ kind: "notice", icon: "info", title: t("Indisponível", "Unavailable"),
        summary: s.message || t("nesta edição", "in this edition"), status: "done", activityId: ev.id });
};

A.blockError = function (ev) {
    var s = sum(ev);
    var el = block({ kind: "error", icon: "warning-circle", title: t("Erro", "Error"),
        summary: s.message || s.error || "", status: "failed", activityId: ev.id,
        footerHtml: '<button type="button" class="gb-block-btn gb-msg-retry">' + esc(t("Tentar de novo", "Try again")) + '</button>' });
    el.querySelector(".gb-msg-retry").addEventListener("click", function () {
        A.regenerateFrom(el.closest(".gb-msg-assistant"));
    });
    return el;
};

/* A file the assistant produced (workspace or Drive). No server event yet. */
A.blockFileGenerated = function (ev) {
    var s = sum(ev);
    var name = s.name || s.file || "";
    var drivePath = s.drive_path || "";
    var el = block({ kind: "file-out", icon: "file", title: t("Arquivo gerado", "Generated file"),
        summary: name, count: s.size ? fmtBytes(s.size) : "", status: B.mapStatus(ev.status), activityId: ev.id, open: true,
        bodyHtml: '<div class="gb-block-out">' + B.fileBadge(name) +
            '<div class="gb-block-out-text"><span class="gb-block-out-name">' + esc(name) + '</span>' +
            (s.detail ? '<span class="gb-block-out-meta">' + esc(s.detail) + '</span>' : "") + '</div>' +
            (drivePath ? '<button type="button" class="gb-block-btn" data-act="open">' + esc(t("Abrir", "Open")) + '</button>' +
                '<button type="button" class="gb-block-btn gb-block-btn--acc" data-act="drive">' + esc(t("Ver no Drive", "Show in Drive")) + '</button>' : "") +
            '</div>' });
    var open = el.querySelector('[data-act="open"]');
    if (open && typeof A.makeAttachClickable === "function") {
        A.makeAttachClickable(open, { name: name, size: s.size }, { drivePath: drivePath });
    }
    var drive = el.querySelector('[data-act="drive"]');
    if (drive) { drive.addEventListener("click", function () { A.switchView("drive"); }); }
    return el;
};

/* A recurring task the assistant set up. No server event yet. */
A.blockTaskScheduled = function (ev) {
    var s = sum(ev);
    return block({ kind: "schedule", icon: "clock", title: t("Tarefa agendada", "Scheduled task"),
        summary: s.name || "", status: B.mapStatus(ev.status), activityId: ev.id, open: true,
        bodyHtml: (s.when ? row(t("Quando", "When"), s.when) : "") +
            (s.does ? row(t("Faz", "Does"), s.does) : "") +
            (s.where ? row(t("Onde", "Where"), s.where) : "") });
};

/* Unknown kind: generic step block */
A.blockGeneric = function (ev) {
    var s = sum(ev);
    return block({ kind: "generic", icon: "info", title: ev.kind || t("Etapa", "Step"),
        summary: JSON.stringify(s).substring(0, 80), status: B.mapStatus(ev.status), activityId: ev.id });
};
})();
