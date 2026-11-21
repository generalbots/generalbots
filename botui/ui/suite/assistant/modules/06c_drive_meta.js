"use strict";
/**
 * 06c_drive_meta.js - Drive figures shown around the listing: space used in
 * the breadcrumb, item counts beside the two tabs and the top-level folders
 * in the sidebar; the file badge and the empty state of the listing. Every
 * figure comes from the server; nothing is shown until it has been read.
 */
(function () {
    var A = window.GBAssistant;
    var D = window.GBDrive;
    if (!D) return;
    var esc = D.esc;
    var inboxRead = false;
    var rootPrefix = null;
    var inboxAll = [];

    function portuguese() { return document.documentElement.lang.toLowerCase().startsWith("pt"); }

    /* Storage keys carry the account folder ("users/<id>/"); crumbs start below it. */
    D.pathPrefix = function (path) {
        path = path || "";
        var prefix = rootPrefix != null ? rootPrefix : ((/^users\/[^/]+\//.exec(path) || [""])[0]);
        return prefix && path.indexOf(prefix) === 0 ? prefix : "";
    };
    function learnPrefix(files) {
        var first = files.find(function (f) { return f.path; });
        if (!first) return;
        var key = first.path.replace(/\/$/, "");
        rootPrefix = key.slice(0, key.lastIndexOf("/") + 1);
    }

    /* Short type label in the row badge; a folder shows a square. */
    var BADGE = { docx: "DOC", xlsx: "XLS", pptx: "PPT", jpeg: "JPG", markdown: "MD" };
    var TONE = { pdf: "danger", jpg: "image", png: "image", gif: "image", webp: "image", svg: "image",
        doc: "ink", odt: "ink", txt: "ink", md: "ink", xls: "ok", ods: "ok", csv: "ok" };
    D.fileBadge = function (name, isDir) {
        var ext = !isDir && name.indexOf(".") > 0 ? name.split(".").pop().toLowerCase() : "";
        var label = isDir ? "▢" : (BADGE[ext] || ext.slice(0, 4)).toUpperCase();
        var tone = isDir ? "" : TONE[(BADGE[ext] || ext).toLowerCase()];
        return '<span class="gb-drv-file-badge' + (isDir ? " gb-drv-badge-dir" : tone ? " gb-drv-tone-" + tone : "") + '" aria-hidden="true">' +
            esc(label || "·") + '</span>';
    };

    D.showEmpty = function () {
        var list = document.getElementById("gbDrvList");
        if (!list) return;
        var pt = portuguese();
        var input = document.getElementById("gbDrvSearchInput");
        var q = D.searching && input ? input.value.trim() : "";
        var title = q ? (pt ? "Nada encontrado" : "Nothing found") : A.t("drive.empty-title", "No files yet");
        var text = q ? (pt ? "Nenhum arquivo com “" + q + "” no nome." : "No file with “" + q + "” in its name.") :
            D.inboxMode ? (pt ? "Os arquivos que você anexar numa conversa aparecem aqui." : "Files you attach in a conversation show up here.") :
            (pt ? "Envie arquivos pelo botão acima ou arraste para cá. Depois, use qualquer um deles numa conversa." :
                "Upload files with the button above or drag them here. Then use any of them in a conversation.");
        list.innerHTML = '<div class="gb-drv-empty-wrap"><button type="button" class="gb-drv-empty-upload">' +
            '<span class="gb-drv-empty-icon">' + D.svg(D.PATHS.upload, 22, 1.8) + '</span>' +
            '<strong>' + esc(title) + '</strong>' +
            '<span class="gb-drv-empty-text">' + esc(text) + '</span>' +
            '<span class="gb-drv-empty-limit">' + (pt ? "PDF · WORD · EXCEL · IMAGENS · ATÉ 25 MB" : "PDF · WORD · EXCEL · IMAGES · UP TO 25 MB") +
            '</span></button></div>';
        list.querySelector("button").onclick = function () {
            document.getElementById("gbDrvFileInput").click();
        };
    };

    /* Attachments are gathered from the conversations, so their search runs on the gathered list. */
    var search = D.searchFiles;
    D.searchFiles = function (query) {
        if (!D.inboxMode) { search.apply(D, arguments); return; }
        var q = String(query || "").toLowerCase();
        D.searching = true;
        D.files = D.sortFiles(inboxAll.filter(function (f) { return (f.name || "").toLowerCase().indexOf(q) >= 0; }));
        D.renderFileList();
    };

    function crumbMeta() {
        var crumb = document.querySelector(".gb-breadcrumb");
        if (!crumb) return null;
        var node = crumb.querySelector('.gb-crumb-meta[data-for="drive"]');
        if (!node) {
            node = document.createElement("small");
            node.className = "gb-crumb-meta";
            node.dataset.for = "drive";
            node.hidden = true;
            crumb.appendChild(node);
        }
        return node;
    }

    /* The sidebar count is drawn from data-count so the shell's breadcrumb, which reads the
       active sidebar entry's text, shows the tab name alone. */
    function setCount(tab, value) {
        document.querySelectorAll('#gbDrvTabs [data-tab="' + tab + '"] .gb-drv-count').forEach(function (node) {
            node.textContent = String(value);
        });
        document.querySelectorAll('#gbDriveNav [data-tab="' + tab + '"]').forEach(function (button) {
            button.dataset.count = String(value);
        });
    }

    D.refreshUsage = function () {
        A.apiFetch("/api/files/quota")
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (q) {
                var node = crumbMeta();
                if (!node) return;
                var used = q && typeof q.used_bytes === "number" ? q.used_bytes : null;
                node.hidden = used === null;
                if (used === null) return;
                node.textContent = (used ? D.fmtBytes(used) : "0 KB") + " " + A.t("drive.used", "used");
                node.title = A.t("drive.usage-hint", "Space taken by your files in Drive");
            })
            .catch(function () { var node = crumbMeta(); if (node) node.hidden = true; });
    };

    function renderFolders(files) {
        var box = document.getElementById("gbDrvRailFolders");
        if (!box) return;
        var dirs = files.filter(function (f) { return f.is_dir; });
        box.hidden = !dirs.length;
        box.innerHTML = '<span class="gb-strip">' + esc(A.t("drive.folders", "Folders").toUpperCase()) + '</span>' +
            dirs.map(function (f) {
                var path = f.path || f.name;
                var name = f.name || path.replace(/\/$/, "").split("/").pop();
                var active = !D.inboxMode && D.path === path ? " active" : "";
                return '<button type="button" class="gb-drv-tab gb-drv-folder' + active + '" data-folder="' +
                    esc(path) + '">' + D.railIcon(D.PATHS.folder) + '<span class="gb-drv-tab-label">' + esc(name) + '</span></button>';
            }).join("");
    }

    function openFolder(event) {
        var button = event.target.closest("[data-folder]");
        if (!button) return;
        event.stopPropagation();
        var path = button.getAttribute("data-folder");
        var files = document.querySelector('#gbDrvTabs [data-tab="files"]');
        if (D.inboxMode && files) files.click();
        D.navigateTo(path);
    }

    function countInbox() {
        D.collectInboxFiles().then(function (files) {
            inboxRead = true;
            setCount("inbox", files.length);
        }).catch(function () { inboxRead = false; });
    }

    var render = D.renderFileList;
    D.renderFileList = function () {
        render.apply(D, arguments);
        if (D.searching) return;
        if (D.inboxMode) {
            inboxRead = true;
            inboxAll = D.files.slice();
            setCount("inbox", D.files.length);
            return;
        }
        if (D.scope === "user" && !D.path) {
            learnPrefix(D.files);
            setCount("files", D.files.length);
            renderFolders(D.files);
            if (!inboxRead) countInbox();
        } else {
            document.querySelectorAll("#gbDrvRailFolders [data-folder]").forEach(function (button) {
                button.classList.toggle("active", D.path === button.getAttribute("data-folder"));
            });
        }
        D.refreshUsage();
    };

    var shell = D.renderShell;
    D.renderShell = function (container) {
        shell.apply(D, arguments);
        inboxRead = false;
        var rail = document.getElementById("gbDriveNav");
        if (rail) rail.addEventListener("click", openFolder, true);
    };
})();
