"use strict";
/**
 * 06c_drive_meta.js - Drive figures shown around the listing: space used in
 * the breadcrumb, item counts beside the two tabs and the top-level folders
 * in the sidebar. Every figure comes from the server; nothing is shown until
 * it has been read.
 */
(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };
    var D = window.GBDrive;
    if (!D) return;
    var esc = D.esc;
    var inboxRead = false;
    var rootPrefix = null;

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

    function setCount(tab, value) {
        document.querySelectorAll('.gb-drv-tab[data-tab="' + tab + '"]').forEach(function (button) {
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
                node.textContent = D.fmtBytes(used) + " " + A.t("drive.used", "used");
                node.title = A.t("drive.usage-hint", "Space taken by your files in Drive");
            })
            .catch(function () { var node = crumbMeta(); if (node) node.hidden = true; });
    };

    function renderFolders(files) {
        var box = document.getElementById("gbDrvRailFolders");
        if (!box) return;
        var dirs = files.filter(function (f) { return f.is_dir; });
        box.hidden = !dirs.length;
        box.innerHTML = '<span class="gb-strip">' + esc(A.t("drive.folders", "Folders")) + '</span>' +
            dirs.map(function (f) {
                var path = f.path || f.name;
                var name = f.name || path.replace(/\/$/, "").split("/").pop();
                var active = !D.inboxMode && D.path === path ? " active" : "";
                return '<button type="button" class="gb-drv-tab gb-drv-folder' + active + '" data-folder="' +
                    esc(path) + '">' + I("folder", 15) + '<span>' + esc(name) + '</span></button>';
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
            setCount("inbox", D.files.length);
            return;
        }
        if (D.scope === "user" && !D.path) {
            learnPrefix(D.files);
            setCount("files", D.files.filter(function (f) { return !f.is_dir; }).length);
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
