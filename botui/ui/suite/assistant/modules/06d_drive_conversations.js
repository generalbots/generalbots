"use strict";
/**
 * 06d_drive_conversations.js - Drive tab "Arquivos das conversas".
 *
 * Lists the files the assistant saved for each conversation
 * (`<bot>.gbdrive/conversations/<folder>/`, see 18a_file_store.js), grouped
 * by conversation. The server names each folder's conversation and title;
 * an older server's folder names (title slug or `<slug>-<id8>`) are matched
 * to the conversation list instead. A row opens the same file panel as a
 * link in an answer.
 */
(function () {
    var A = window.GBAssistant;
    var D = window.GBDrive;
    var F = A.files;
    if (!D || !F) { return; }
    var t = F.t;
    var esc = D.esc;
    var groups = [];
    var request = 0;

    function title(g) {
        if (g.session && g.session.title) { return g.session.title; }
        if (g.title) { return g.title; }
        var name = g.folder.replace(/-[0-9a-f]{8}$/i, "").replace(/^conversation$/i, "");
        return name ? name.replace(/-/g, " ") : t("Conversa sem título", "Untitled conversation");
    }

    function render(query) {
        var list = document.getElementById("gbDrvList");
        if (!list || !D.convMode) { return; }
        var crumb = document.getElementById("gbDrvBreadcrumb");
        if (crumb && D.convFromMine && D.mineConversationsCrumb) { D.mineConversationsCrumb(crumb); }
        else if (crumb) { crumb.textContent = t("Arquivos das conversas", "Conversation files"); }
        var q = String(query || "").trim().toLowerCase();
        var shown = groups.map(function (g) {
            var files = q ? g.files.filter(function (f) { return f.name.toLowerCase().indexOf(q) !== -1; }) : g.files;
            return { g: g, files: files };
        }).filter(function (x) { return x.files.length; });
        var container = document.getElementById("gbDriveContainer");
        if (container) {
            container.classList.add("gb-drv-no-date");
            container.classList.toggle("gb-drv-empty", !shown.length);
        }
        if (!shown.length) {
            list.innerHTML = '<div class="gb-drv-state">' + (window.GBIcon ? window.GBIcon("folder-open", 40) : "") +
                '<h3>' + esc(q ? t("Nada encontrado", "Nothing found") : t("Nenhum arquivo de conversa ainda", "No conversation files yet")) + '</h3>' +
                '<p>' + esc(t("Os arquivos que o assistente cria e salva no Drive aparecem aqui, por conversa.",
                    "Files the assistant makes and saves to Drive appear here, by conversation.")) + '</p></div>';
            return;
        }
        list.innerHTML = shown.map(function (x, gi) {
            var head = '<div class="gb-drv-group" role="heading" aria-level="3">' +
                '<span class="gb-drv-group-title">' + esc(title(x.g)) + '</span>' +
                '<span class="gb-drv-group-count">' + x.files.length + " " +
                esc(x.files.length === 1 ? t("arquivo", "file") : t("arquivos", "files")) + '</span></div>';
            return head + x.files.map(function (f) {
                return '<div class="gb-drv-row gb-drv-conv-row" tabindex="0" data-group="' + gi + '" data-path="' + esc(f.path) +
                    '" data-name="' + esc(f.name) + '" data-size="' + (f.size || 0) + '" data-folder="' + esc(x.g.folder) + '"' + (f.api ? ' data-api="1"' : '') + '>' +
                    '<div class="gb-drv-name-cell">' + D.fileBadge(f.name, false) +
                    '<span class="gb-drv-name-text">' + esc(f.name) + '</span></div>' +
                    '<div class="gb-drv-size-cell">' + D.fmtBytes(f.size) + '</div>' +
                    '<div class="gb-drv-action-cell"><button type="button" class="gb-drv-row-menu gb-drv-conv-dl" aria-label="' +
                    esc(t("Baixar", "Download") + " " + f.name) + '">' + (window.GBIcon ? window.GBIcon("download", 14) : "") + '</button></div></div>';
            }).join("");
        }).join("");
    }

    function srcOf(row) {
        return { kind: "drive", path: row.getAttribute("data-path"), name: row.getAttribute("data-name"),
            size: Number(row.getAttribute("data-size")) || 0, folder: row.getAttribute("data-folder"), api: row.getAttribute("data-api") === "1" };
    }

    function bind(list) {
        if (list.dataset.convBound === "1") { return; }
        list.dataset.convBound = "1";
        list.addEventListener("click", function (e) {
            var row = D.convMode && e.target.closest(".gb-drv-conv-row");
            if (!row) { return; }
            e.stopPropagation();
            var src = srcOf(row);
            if (e.target.closest(".gb-drv-conv-dl")) {
                F.fetchBlob(src).then(function (blob) { return F.saveBlob(blob, src.name); }).catch(function (err) {
                    D.showToast(t("Erro ao baixar", "Download failed") + (err && err.status ? " (HTTP " + err.status + ")" : ""), "error");
                });
                return;
            }
            F.open(src, row);
        }, true);
        list.addEventListener("keydown", function (e) {
            var row = D.convMode && e.target.classList && e.target.classList.contains("gb-drv-conv-row") ? e.target : null;
            if (!row || (e.key !== "Enter" && e.key !== " ")) { return; }
            e.preventDefault();
            e.stopPropagation();
            F.open(srcOf(row), row);
        }, true);
    }

    D.loadConversationFiles = function (force) {
        D.convMode = true;
        D.inboxMode = false;
        var mine = ++request;
        var list = document.getElementById("gbDrvList");
        if (list) { bind(list); }
        D.showLoading();
        F.conversationFolders(force).then(function (found) {
            if (mine !== request || !D.convMode) { return; }
            groups = found.slice().sort(function (a, b) {
                var ia = (A.sessions || []).indexOf(a.session), ib = (A.sessions || []).indexOf(b.session);
                return (ia === -1 ? 1e9 : ia) - (ib === -1 ? 1e9 : ib) || a.folder.localeCompare(b.folder);
            });
            render(document.getElementById("gbDrvSearchInput") ? document.getElementById("gbDrvSearchInput").value : "");
        }).catch(function (err) {
            if (mine !== request || !D.convMode) { return; }
            /* No conversation folder yet reads as an empty list. */
            if (err && err.status === 404) { groups = []; render(""); return; }
            D.showError(t("Não foi possível listar os arquivos das conversas", "Could not list the conversation files"));
        });
    };

    var baseRefresh = D.refreshFiles;
    D.refreshFiles = function () {
        if (D.convMode) { D.loadConversationFiles(true); return; }
        return baseRefresh.apply(this, arguments);
    };
    var baseSearch = D.searchFiles;
    D.searchFiles = function (query) {
        if (D.convMode) { D.searching = true; render(query); return; }
        return baseSearch.apply(this, arguments);
    };
})();
