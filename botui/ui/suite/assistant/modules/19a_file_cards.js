"use strict";
/**
 * 19a_file_cards.js - The files a turn produced, as cards in the answer.
 *
 * Frame, at the end of a turn: {type:"files_produced", files:[{name, path,
 * size, mime, drive_path}]}; `path` is in the conversation's workspace
 * (`output/<name>`), `drive_path` the file in the person's Drive (relative to
 * "Meus arquivos", e.g. `Conversas/Relatorio-1a2b3c4d/relatorio.pdf`), or null.
 * Each card shows the name, type and size, a thumbnail (images) or the first
 * page (PDF, drawn by 18b_pdf_view.js), "Abrir" (the file panel,
 * 18_file_panel.js) and "Baixar"; the Drive line opens the file's folder in
 * Drive. Bytes come through the file store (18a_file_store.js), which falls
 * back to the Drive copy when the Virtual Machine is off.
 *
 * On reload the cards come from the meta of the message that lists the file
 * links (saved right after the answer); they go into that answer.
 */
(function () {
var A = window.GBAssistant;
var F = A.files;
if (!F || !A.frameHandlers) { return; }
var t = F.t;
var I = window.GBIcon || function () { return ""; };
var THUMB_MAX = 15 * 1024 * 1024;
function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }

function sourceOf(f, conversationId) {
    return { kind: "vm", conversationId: conversationId || String(A.currentSessionId || A.sessionId || ""),
        path: String(f.path || ("output/" + f.name)), name: String(f.name || String(f.path || "").split("/").pop()),
        size: f.size != null ? Number(f.size) : null };
}

function thumb(card, src, f) {
    var box = card.querySelector(".gb-fcard-thumb");
    var kind = F.kind(src.name);
    if ((kind !== "image" && kind !== "pdf") || (src.size && src.size > THUMB_MAX)) { return; }
    F.fetchBlob(src).then(function (blob) {
        if (kind === "pdf") {
            return F.pdfThumb(box, blob, src.name).then(function () { box.classList.add("gb-fcard-thumb--pdf"); });
        }
        var typed = blob.type ? blob : new Blob([blob], { type: f.mime || "image/" + F.ext(src.name) });
        box.classList.add("gb-fcard-thumb--image");
        box.innerHTML = '<img src="' + esc(URL.createObjectURL(typed)) + '" alt="' + esc(src.name) + '">';
    }).catch(function () { /* the type icon stays */ });
}

/* "Meus arquivos › Conversas › <conversa>": the folder of `drive_path`
   (relative to the person's Drive, e.g. Conversas/Relatorio-1a2b3c4d/x.pdf). */
function driveFolder(drivePath) {
    var parts = String(drivePath || "").replace(/^\/+/, "").replace(/^users\/[^/]+\//, "").split("/");
    parts.pop();
    return parts;
}
function driveLine(f) {
    var parts = driveFolder(f.drive_path);
    var label = [t("Meus arquivos", "My files")].concat(parts).join(" › ");
    return '<button type="button" class="gb-fcard-drive" data-act="drive" title="' + esc(t("Abrir a pasta no Drive", "Open the folder in Drive")) + '">' +
        I("folder", 12) + '<span>' + esc(t("No Drive: ", "In Drive: ") + label) + '</span></button>';
}

/* Opens the person's Drive at the folder that holds `drivePath`. Paths in
   the listing carry the account folder ("users/<id>/"): it is read from the
   root listing when Drive has not been opened yet. */
function openDriveFolder(drivePath) {
    var D = window.GBDrive;
    if (!D || typeof A.switchView !== "function") { return; }
    var rel = driveFolder(drivePath).join("/");
    A.switchView("drive");
    var tab = document.querySelector('#gbDrvTabs [data-tab="files"]');
    if (tab && tab.getAttribute("aria-pressed") !== "true") { tab.click(); }
    D.scope = "user";
    D.convMode = false;
    var known = (D.files || []).map(function (x) { return /^users\/[^/]+\//.exec(x.path || ""); }).filter(Boolean)[0];
    var prefix = known ? Promise.resolve(known[0]) : D.req("/list?scope=user").then(function (files) {
        var hit = (files || []).map(function (x) { return /^users\/[^/]+\//.exec(x.path || ""); }).filter(Boolean)[0];
        return hit ? hit[0] : "";
    }).catch(function () { return ""; });
    prefix.then(function (p) { D.navigateTo(p + rel); });
}

function fileCard(f, conversationId) {
    var src = sourceOf(f, conversationId);
    var card = document.createElement("div");
    card.className = "gb-fcard";
    card.dataset.path = src.path;
    card.setAttribute("role", "group");
    card.setAttribute("aria-label", src.name);
    var meta = [F.typeName(src.name), src.size != null ? F.fmtBytes(src.size) : ""].filter(Boolean).join(" · ");
    card.innerHTML =
        '<div class="gb-fcard-thumb" aria-hidden="true">' + I(A.fileTypeIcon ? A.fileTypeIcon(src.name) : "file", 26) + '</div>' +
        '<div class="gb-fcard-info">' +
            '<div class="gb-fcard-name" title="' + esc(src.name) + '">' + esc(src.name) + '</div>' +
            '<div class="gb-fcard-meta">' + esc(meta) + '</div>' +
            (f.drive_path ? driveLine(f) : '') +
            '<div class="gb-fcard-actions">' +
                '<button type="button" class="gb-block-btn gb-block-btn--acc" data-act="open">' + I("eye", 14) + '<span>' + esc(t("Abrir", "Open")) + '</span></button>' +
                '<button type="button" class="gb-block-btn" data-act="download">' + I("download", 14) + '<span>' + esc(t("Baixar", "Download")) + '</span></button>' +
            '</div>' +
            '<p class="gb-fcard-status" role="status"></p>' +
        '</div>';
    card.addEventListener("click", function (e) {
        var btn = e.target.closest("[data-act]");
        if (!btn || btn.disabled) { return; }
        if (btn.getAttribute("data-act") === "open") { F.open(sourceOf(f, conversationId), btn); return; }
        if (btn.getAttribute("data-act") === "drive") { openDriveFolder(f.drive_path); return; }
        var status = card.querySelector(".gb-fcard-status");
        btn.disabled = true;
        F.fetchBlob(sourceOf(f, conversationId)).then(function (blob) { return F.saveBlob(blob, src.name); }).then(function (path) {
            status.textContent = path ? t("Salvo em ", "Saved to ") + path : "";
        }).catch(function (err) {
            status.textContent = t("Não foi possível baixar o arquivo.", "Could not download the file.") + (err && err.status ? " (HTTP " + err.status + ")" : "");
        }).then(function () { btn.disabled = false; });
    });
    thumb(card, src, f);
    return card;
}

/* One list per answer; a file already shown is not added twice. */
function fill(list, files, conversationId) {
    files.forEach(function (f) {
        if (!f || (!f.name && !f.path)) { return; }
        var path = sourceOf(f, conversationId).path;
        var seen = Array.prototype.some.call(list.children, function (c) { return c.dataset.path === path; });
        if (!seen) { list.appendChild(fileCard(f, conversationId)); }
    });
}

function newList() {
    var list = document.createElement("div");
    list.className = "gb-fcards";
    list.setAttribute("role", "list");
    list.setAttribute("aria-label", t("Arquivos gerados", "Files created"));
    return list;
}

A.frameHandlers.files_produced = function (d) {
    var files = Array.isArray(d.files) ? d.files : [];
    if (!files.length) { return; }
    var msgs = document.getElementById("gbMessages");
    var lastUser = msgs ? msgs.querySelectorAll(".gb-msg-user") : [];
    var answers = msgs ? msgs.querySelectorAll(":scope > .gb-msg-assistant") : [];
    var answer = answers.length ? answers[answers.length - 1] : null;
    var current = answer && (!lastUser.length || (lastUser[lastUser.length - 1].compareDocumentPosition(answer) & Node.DOCUMENT_POSITION_FOLLOWING));
    var list = current ? answer.querySelector(".gb-fcards") : null;
    if (!list) {
        list = newList();
        if (!A.placeTurnCard(list)) { return; }
    }
    fill(list, files);
    if (files.some(function (f) { return f.drive_path; })) {
        F.invalidateFolders();
        var D = window.GBDrive;
        if (D && D.initialised && A.currentView === "drive" && !D.inboxMode && typeof D.refreshFiles === "function") { D.refreshFiles(); }
    }
    A.announce(files.length === 1 ? t("Arquivo gerado: ", "File created: ") + (files[0].name || "")
        : files.length + " " + t("arquivos gerados", "files created"));
};

var extras = A.renderTurnExtras;
A.renderTurnExtras = function (el, m) {
    if (extras) { extras(el, m); }
    var stored = A.turnMeta ? A.turnMeta(m, "files_produced") : null;
    var files = Array.isArray(stored) ? stored : stored && Array.isArray(stored.files) ? stored.files : [];
    if (!files.length) { return; }
    var list = newList();
    fill(list, files, A.currentSessionId || A.sessionId || "");
    /* The links message follows its answer: the cards go into that answer. */
    var msgs = document.getElementById("gbMessages");
    var prev = msgs && msgs.lastElementChild;
    var bar = prev && prev.classList.contains("gb-msg-assistant") ? prev.querySelector(".gb-msg-toolbar") : null;
    if (bar) { bar.before(list); } else { el.appendChild(list); }
};
})();
