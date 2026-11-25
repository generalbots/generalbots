"use strict";
/**
 * 19a_file_cards.js - The files a turn produced, as cards in the answer.
 *
 * Frame, at the end of a turn: {type:"files_produced", files:[{name, path,
 * size, mime, drive_path}]}; `path` is in the conversation's workspace
 * (`output/<name>`), `drive_path` the copy in the person's Drive, or null.
 * Each card shows the name, type and size, a thumbnail (images) or the first
 * page (PDF, through the webview's own viewer), "Abrir" (the file panel,
 * 18_file_panel.js) and "Baixar". Bytes come through the file store
 * (18a_file_store.js), which falls back to the Drive copy when the Virtual
 * Machine is off; `drive_path` is the copy in the conversation's Drive folder.
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
        var typed = blob.type ? blob : new Blob([blob], { type: kind === "pdf" ? "application/pdf" : (f.mime || "image/" + F.ext(src.name)) });
        var url = URL.createObjectURL(typed);
        box.classList.add("gb-fcard-thumb--" + kind);
        box.innerHTML = kind === "pdf"
            ? '<iframe src="' + esc(url) + '#page=1&toolbar=0&navpanes=0&scrollbar=0&view=FitH" title="' +
                esc(t("Primeira página de ", "First page of ") + src.name) + '" tabindex="-1" loading="lazy"></iframe>'
            : '<img src="' + esc(url) + '" alt="' + esc(src.name) + '">';
    }).catch(function () { /* the type icon stays */ });
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
            (f.drive_path ? '<div class="gb-fcard-drive">' + I("folder", 12) + '<span>' + esc(t("No Drive: Meus arquivos › Conversas", "In Drive: My files › Conversations")) + '</span></div>' : '') +
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

/* ---- Drive: "Meus arquivos" › "Conversas" ----
   The files stay in one place, the conversation folders. "Meus arquivos"
   shows them as a "Conversas" folder at its root; opening it lists
   GET /api/agent/drive/conversations, one group per conversation, with the
   actions of the "Arquivos das conversas" tab (06d). */
var D = window.GBDrive;
function atMineRoot() {
    if (!D || D.scope !== "user" || D.inboxMode || D.convMode || D.searching) { return false; }
    var path = D.path || "";
    return !(D.pathPrefix ? path.slice(D.pathPrefix(path).length) : path).replace(/\/$/, "");
}
function closeMineConversations() {
    D.convFromMine = false;
    D.convMode = false;
    var container = document.getElementById("gbDriveContainer");
    if (container) { container.classList.remove("gb-drv-inbox", "gb-drv-convs"); }
}
function openMineConversations() {
    D.convFromMine = true;
    var container = document.getElementById("gbDriveContainer");
    if (container) { container.classList.add("gb-drv-inbox", "gb-drv-convs"); }
    D.loadConversationFiles(true);
}
/* The crumb while "Conversas" is open from "Meus arquivos" (06d asks). */
D && (D.mineConversationsCrumb = function (crumb) {
    crumb.innerHTML = '<button class="gb-drv-crumb" data-path="">' + esc(A.t("drive.root", "Drive")) + '</button>' +
        '<button class="gb-drv-crumb gb-drv-crumb-current" data-path="">' + esc(t("Conversas", "Conversations")) + '</button>';
});
var render = D && D.renderFileList;
if (render) {
    D.renderFileList = function () {
        render.apply(D, arguments);
        var list = document.getElementById("gbDrvList");
        if (!list || !atMineRoot() || list.querySelector(".gb-drv-conv-root")) { return; }
        var state = list.querySelector(".gb-drv-state");
        if (state) { state.remove(); }
        var container = document.getElementById("gbDriveContainer");
        if (container) { container.classList.remove("gb-drv-empty"); }
        list.insertAdjacentHTML("afterbegin", '<div class="gb-drv-row gb-drv-conv-root" data-name="Conversas" data-is-dir="true" tabindex="0">' +
            '<div class="gb-drv-name-cell">' + D.fileBadge("Conversas", true) + '<span class="gb-drv-name-text">' + esc(t("Conversas", "Conversations")) + '</span>' +
            '<span class="gb-drv-conv-mark">' + esc(t("Arquivos das conversas", "Conversation files")) + '</span></div>' +
            '<div class="gb-drv-size-cell">—</div></div>');
        var row = list.querySelector(".gb-drv-conv-root");
        row.addEventListener("dblclick", openMineConversations);
        row.addEventListener("keydown", function (e) { if (e.key === "Enter") { e.preventDefault(); openMineConversations(); } });
    };
    var navigate = D.navigateTo;
    D.navigateTo = function () {
        if (D.convFromMine) { closeMineConversations(); }
        return navigate.apply(D, arguments);
    };
    /* Another Drive tab leaves "Conversas" (its own switch sets the mode). */
    document.addEventListener("click", function (e) {
        if (D.convFromMine && e.target.closest && e.target.closest("#gbDriveNav [data-tab], #gbDrvTabs [data-tab]")) { D.convFromMine = false; }
    }, true);
}
})();
