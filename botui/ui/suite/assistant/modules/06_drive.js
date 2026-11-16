"use strict";
/**
 * 06_drive.js - Native Drive file browser: state, API, shell, rendering.
 *
 * Split partner: 06a_drive_actions.js (upload, rename, delete, download,
 * preview, search, create-folder, context menu).
 *
 * Scope: user's own file space (scope "user"). The person never sees the
 * local Windows file system or the assistant's package files.
 */
(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };
    var API = "/api/files";

    /* ---- Shared Drive state (exposed to 06a via window.GBDrive) ---- */
    var D = window.GBDrive = {
        bucket: "", path: "", scope: "user",
        files: [], sort: { col: "name", asc: true },
        selected: null, searching: false, initialised: false,
        ctxEl: null, dragCount: 0, API: API
    };

    /* ---- Helpers (shared) ---- */
    D.esc = function (s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    };
    var esc = D.esc;

    D.fmtBytes = function (b) {
        if (!b || b <= 0) return "";
        if (b < 1024) return b + " B";
        if (b < 1048576) return (b / 1024).toFixed(0) + " KB";
        if (b < 1073741824) return (b / 1048576).toFixed(1) + " MB";
        return (b / 1073741824).toFixed(1) + " GB";
    };

    D.fmtDate = function (iso) {
        if (!iso) return "";
        try {
            var d = new Date(iso);
            if (isNaN(d.getTime())) return "";
            var now = new Date();
            var sameYear = d.getFullYear() === now.getFullYear();
            var opts = sameYear
                ? { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" }
                : { day: "numeric", month: "short", year: "numeric" };
            return d.toLocaleDateString(A.locale || "pt-BR", opts);
        } catch (e) { return ""; }
    };

    D.fileExt = function (name) {
        if (!name || name.indexOf(".") === -1) return "";
        return name.split(".").pop().toLowerCase();
    };

    D.fileIcon = function (name, isDir) {
        if (isDir) return I("folder", 18);
        var ext = D.fileExt(name);
        if ("jpg jpeg png gif webp bmp svg ico avif".split(" ").indexOf(ext) >= 0) return I("image", 18);
        return I("file", 18);
    };

    /* ---- JSON request via A.apiFetch ---- */
    D.req = function (path, opts) {
        return A.apiFetch(API + path, opts).then(function (r) {
            if (!r.ok) {
                return r.json().catch(function () { return {}; }).then(function (body) {
                    var msg = (body && (body.message || body.error)) || "HTTP " + r.status;
                    var err = new Error(msg);
                    err.status = r.status;
                    throw err;
                });
            }
            var ct = r.headers.get("content-type") || "";
            if (ct.indexOf("json") >= 0) return r.json();
            return r.text().then(function (t) {
                try { return JSON.parse(t); } catch (e) { return t; }
            });
        });
    };

    /* ---- Sort ---- */
    D.sortFiles = function (files) {
        var col = D.sort.col;
        var asc = D.sort.asc;
        return files.slice().sort(function (a, b) {
            if (a.is_dir && !b.is_dir) return -1;
            if (!a.is_dir && b.is_dir) return 1;
            var av, bv;
            if (col === "name") {
                av = (a.name || "").toLowerCase();
                bv = (b.name || "").toLowerCase();
                return asc ? av.localeCompare(bv) : bv.localeCompare(av);
            }
            if (col === "size") {
                av = a.size || 0; bv = b.size || 0;
                return asc ? av - bv : bv - av;
            }
            if (col === "modified") {
                av = a.modified ? new Date(a.modified).getTime() : 0;
                bv = b.modified ? new Date(b.modified).getTime() : 0;
                return asc ? av - bv : bv - av;
            }
            return 0;
        });
    };

    /* ================================================================
       States
       ================================================================ */
    D.showLoading = function () {
        var list = document.getElementById("gbDrvList");
        if (!list) return;
        list.innerHTML = '<div class="gb-drv-state"><div class="gb-drv-spinner"></div><p>' +
            esc(A.t("drive.loading", "Carregando...")) + '</p></div>';
    };

    D.showEmpty = function () {
        var list = document.getElementById("gbDrvList");
        if (!list) return;
        list.innerHTML = '<div class="gb-drv-state">' +
            I("folder-open", 48) +
            '<h3>' + esc(A.t("drive.empty-title", "Nenhum arquivo ainda")) + '</h3>' +
            '<p>' + esc(A.t("drive.empty-desc", "Envie arquivos usando o botão acima ou arraste-os para cá.")) + '</p>' +
            '</div>';
    };

    D.showError = function (msg) {
        var list = document.getElementById("gbDrvList");
        if (!list) return;
        list.innerHTML = '<div class="gb-drv-state">' +
            I("warning-circle", 48) +
            '<h3>' + esc(msg || A.t("drive.error-title", "Erro ao carregar")) + '</h3>' +
            '<p>' + esc(A.t("drive.error-desc", "Verifique a conexão e tente novamente.")) + '</p>' +
            '<button class="gb-drv-btn" id="gbDrvRetry">' + esc(A.t("assistant.retry", "Tentar novamente")) + '</button>' +
            '</div>';
        var retry = document.getElementById("gbDrvRetry");
        if (retry) retry.addEventListener("click", function () { D.refreshFiles(); });
    };

    /* ================================================================
       Navigate / refresh
       ================================================================ */
    D.navigateTo = function (path) {
        D.path = path || "";
        D.selected = null;
        D.searching = false;
        var search = document.getElementById("gbDrvSearchInput");
        if (search) search.value = "";
        D.refreshFiles();
    };

    D.refreshFiles = function () {
        D.showLoading();
        var params = new URLSearchParams();
        params.set("scope", D.scope);
        if (D.bucket) params.set("bucket", D.bucket);
        if (D.path) params.set("path", D.path);

        D.req("/list?" + params.toString())
            .then(function (files) {
                D.files = D.sortFiles(files || []);
                D.renderFileList();
            })
            .catch(function (e) {
                console.error("[drive] list failed:", e);
                D.showError(e.message);
            });
    };

    /* ================================================================
       View switching
       ================================================================ */
    A.switchView = function (view) {
        A.currentView = view;
        var chatView = document.getElementById("gbChatView");
        var driveView = document.getElementById("gbDriveView");
        var settingsView = document.getElementById("gbSettingsView");

        if (chatView) chatView.classList.toggle("gb-view-active", view === "chat");
        if (driveView) driveView.classList.toggle("gb-view-active", view === "drive");
        if (settingsView) settingsView.classList.toggle("gb-view-active", view === "settings");

        var navItems = document.querySelectorAll(".gb-nav-item[data-view]");
        for (var i = 0; i < navItems.length; i++) {
            var v = navItems[i].getAttribute("data-view");
            navItems[i].classList.toggle("active", v === view);
        }

        if (view === "drive" && !D.initialised) loadDrive();
        if (view === "settings" && typeof A.renderSettings === "function") A.renderSettings();
    };

    /* ================================================================
       Load Drive (first time)
       ================================================================ */
    function loadDrive() {
        var container = document.getElementById("gbDriveContainer");
        if (!container) return;
        D.renderShell(container);
        D.showLoading();

        D.req("/list?scope=user").then(function (files) {
            D.files = D.sortFiles(files || []);
            D.initialised = true;
            D.renderFileList();
        }).catch(function (e) {
            console.error("[drive] init failed:", e);
            D.showError(e.message || String(e));
        });
    }

    /* renderShell lives in 06a_drive_actions.js (D.renderShell) */

    /* ================================================================
       Breadcrumb
       ================================================================ */
    D.renderBreadcrumb = function () {
        var el = document.getElementById("gbDrvBreadcrumb");
        if (!el) return;
        var parts = D.path ? D.path.split("/").filter(Boolean) : [];
        var html = '<button class="gb-drv-crumb' + (parts.length === 0 ? " gb-drv-crumb-current" : "") +
            '" data-path="">' + esc(A.t("drive.root", "Drive")) + '</button>';
        var running = "";
        for (var i = 0; i < parts.length; i++) {
            running += (running ? "/" : "") + parts[i];
            var isCurrent = i === parts.length - 1;
            html += '<span class="gb-drv-crumb-sep">/</span>' +
                '<button class="gb-drv-crumb' + (isCurrent ? " gb-drv-crumb-current" : "") +
                '" data-path="' + esc(running) + '">' + esc(parts[i]) + '</button>';
        }
        el.innerHTML = html;
        el.addEventListener("click", function (e) {
            var btn = e.target.closest("[data-path]");
            if (!btn || btn.classList.contains("gb-drv-crumb-current")) return;
            D.navigateTo(btn.getAttribute("data-path"));
        });
    };

    /* ================================================================
       File list rendering
       ================================================================ */
    D.renderFileList = function () {
        var list = document.getElementById("gbDrvList");
        if (!list) return;
        D.renderBreadcrumb();
        /* Update column headers */
        var header = document.getElementById("gbDrvColHeader");
        if (header) {
            var btns = header.querySelectorAll("[data-col]");
            for (var j = 0; j < btns.length; j++) {
                var c = btns[j].getAttribute("data-col");
                var act = c === D.sort.col;
                btns[j].classList.toggle("gb-drv-col-active", act);
                var arrow = btns[j].querySelector(".gb-drv-sort-arrow");
                if (arrow) arrow.textContent = act ? (D.sort.asc ? "▲" : "▼") : "";
            }
        }

        if (!D.files || D.files.length === 0) { D.showEmpty(); return; }

        var html = "";
        for (var i = 0; i < D.files.length; i++) {
            var f = D.files[i];
            var name = f.name || (f.path || "").split("/").pop() || "?";
            var isDir = !!f.is_dir;
            var path = f.path || name;
            var selected = path === D.selected;
            html += '<div class="gb-drv-row' + (selected ? " gb-drv-selected" : "") +
                '" data-path="' + esc(path) + '" data-name="' + esc(name) +
                '" data-is-dir="' + isDir + '" tabindex="0">' +
                '<div class="gb-drv-name-cell">' + D.fileIcon(name, isDir) +
                '<span class="gb-drv-name-text">' + esc(name) + '</span></div>' +
                '<div class="gb-drv-size-cell">' + (isDir ? "" : D.fmtBytes(f.size)) + '</div>' +
                '<div class="gb-drv-date-cell">' + D.fmtDate(f.modified) + '</div>' +
                '<div class="gb-drv-action-cell">' +
                '<button class="gb-drv-row-menu" aria-label="Menu" title="Menu">' +
                '<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="12" cy="19" r="2"/></svg>' +
                '</button></div></div>';
        }
        list.innerHTML = html;
        D.bindRows(list);
    };

    D.bindRows = function (list) {
        var rows = list.querySelectorAll(".gb-drv-row");
        for (var r = 0; r < rows.length; r++) {
            D.bindOneRow(rows[r]);
        }
    };

    D.bindOneRow = function (row) {
        row.addEventListener("click", function (e) {
            if (e.target.closest(".gb-drv-row-menu")) return;
            D.selected = row.getAttribute("data-path");
            D.highlightRow(D.selected);
        });
        row.addEventListener("dblclick", function () {
            var path = row.getAttribute("data-path");
            var isDir = row.getAttribute("data-is-dir") === "true";
            if (isDir) D.navigateTo(D.path ? D.path + "/" + row.getAttribute("data-name") : row.getAttribute("data-name"));
            else D.previewFile(path, row.getAttribute("data-name"));
        });
        row.addEventListener("keydown", function (e) {
            if (e.key === "Enter") {
                var path = row.getAttribute("data-path");
                var isDir = row.getAttribute("data-is-dir") === "true";
                if (isDir) D.navigateTo(D.path ? D.path + "/" + row.getAttribute("data-name") : row.getAttribute("data-name"));
                else D.previewFile(path, row.getAttribute("data-name"));
            }
        });
        var menuBtn = row.querySelector(".gb-drv-row-menu");
        if (menuBtn) {
            menuBtn.addEventListener("click", function (e) {
                e.stopPropagation();
                D.showCtxMenu(e, row.getAttribute("data-path"), row.getAttribute("data-name"),
                    row.getAttribute("data-is-dir") === "true");
            });
        }
    };

    D.highlightRow = function (path) {
        var rows = document.querySelectorAll(".gb-drv-row");
        for (var i = 0; i < rows.length; i++) {
            rows[i].classList.toggle("gb-drv-selected", rows[i].getAttribute("data-path") === path);
        }
    };

    /* Context menu close (shared with 06a) */
    D.closeCtx = function () {
        if (D.ctxEl) { D.ctxEl.remove(); D.ctxEl = null; }
    };

    /* ================================================================
       Navigation bindings
       ================================================================ */
    A.initNavigation = function () {
        var navItems = document.querySelectorAll(".gb-nav-item[data-view]");
        for (var i = 0; i < navItems.length; i++) {
            (function (item) {
                item.addEventListener("click", function () {
                    A.switchView(item.getAttribute("data-view"));
                });
            })(navItems[i]);
        }
        var brand = document.getElementById("gbWordmark");
        if (brand) {
            brand.style.cursor = "pointer";
            brand.addEventListener("click", function () { A.switchView("chat"); });
        }
    };

    A.filterFeatures = function () {
        var apps = A.apps || [];
        var hasDrive = apps.indexOf("drive") !== -1;
        var driveNav = document.getElementById("gbNavDrive");
        if (driveNav && !hasDrive) driveNav.style.display = "none";
    };
})();
