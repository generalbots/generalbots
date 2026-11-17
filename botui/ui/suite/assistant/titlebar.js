/**
 * titlebar.js - Custom window titlebar for the undecorated desktop window.
 *
 * Standalone: no dependency on the assistant shell scripts, i18n or identity.
 * Included in both assistant.html and login.html.
 *
 * Contract with the Tauri shell:
 *   - Drag region: elements with data-tauri-drag-region (buttons must NOT have it)
 *   - Window calls: window.__TAURI__.window.getCurrentWindow().minimize() etc.
 *   - When __TAURI__ / __TAURI_INTERNALS__ is absent: no window buttons rendered.
 *
 * Test flag: ?titlebar=1 forces the window buttons visible (for headless tests
 * without Tauri); the buttons click but the calls are no-ops.
 */
(function () {
    "use strict";

    var IS_TAURI = !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
    var FORCE = false;
    try {
        FORCE = new URLSearchParams(window.location.search).get("titlebar") === "1";
    } catch (e) { /* search unavailable */ }
    var SHOW_CONTROLS = IS_TAURI || FORCE;

    var lang = "en";
    try {
        lang = (navigator.language || "en").toLowerCase().startsWith("pt") ? "pt" : "en";
    } catch (e) { /* */ }

    var L = {
        pt: { min: "Minimizar", max: "Maximizar", restore: "Restaurar", close: "Fechar", toggle: "Barra lateral" },
        en: { min: "Minimize", max: "Maximize", restore: "Restore", close: "Close", toggle: "Sidebar" }
    };
    var t = L[lang] || L.en;

    /* ---- Build the titlebar ---- */
    function build() {
        var bar = document.createElement("div");
        bar.className = "gb-titlebar" + (isDark() ? " gb-titlebar--dark" : "");
        bar.setAttribute("data-tauri-drag-region", "");

        /* Left */
        var left = document.createElement("div");
        left.className = "gb-titlebar-left";
        left.setAttribute("data-tauri-drag-region", "");

        var toggle = document.createElement("button");
        toggle.className = "gb-titlebar-toggle";
        toggle.setAttribute("aria-label", t.toggle);
        toggle.title = t.toggle;
        toggle.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/></svg>';
        toggle.addEventListener("click", function () {
            var shell = window.GBAssistant;
            if (shell && typeof shell.toggleSidebar === "function") { shell.toggleSidebar(); return; }
            var sidebar = document.getElementById("gb-sidebar");
            if (sidebar) { sidebar.classList.toggle("collapsed"); }
        });
        left.appendChild(toggle);

        var wm = document.createElement("span");
        wm.className = "gb-titlebar-wordmark";
        wm.setAttribute("data-tauri-drag-region", "");
        wm.innerHTML = 'General Bots<span class="gb-titlebar-dot">.</span>';
        left.appendChild(wm);

        bar.appendChild(left);

        /* Centre: conversation title */
        var title = document.createElement("span");
        title.className = "gb-titlebar-title";
        title.id = "gbTitlebarTitle";
        title.setAttribute("data-tauri-drag-region", "");
        bar.appendChild(title);

        /* Right: window controls (only in Tauri or forced) */
        var controls = document.createElement("div");
        controls.className = "gb-titlebar-controls";
        if (!SHOW_CONTROLS) { controls.hidden = true; }

        controls.appendChild(makeBtn("minimize", t.min,
            '<svg width="12" height="12" viewBox="0 0 12 12"><rect x="1" y="5.5" width="10" height="1" fill="currentColor"/></svg>'));
        controls.appendChild(makeBtn("maximize", t.max,
            '<svg width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1"/></svg>'));
        controls.appendChild(makeBtn("close", t.close,
            '<svg width="12" height="12" viewBox="0 0 12 12"><line x1="2" y1="2" x2="10" y2="10" stroke="currentColor" stroke-width="1.2"/><line x1="10" y1="2" x2="2" y2="10" stroke="currentColor" stroke-width="1.2"/></svg>'));

        bar.appendChild(controls);

        /* Insert at the very top of body */
        document.body.insertBefore(bar, document.body.firstChild);

        /* Listen for maximize state changes to toggle icon */
        if (IS_TAURI) { watchMaximized(controls); }

        /* Expose the title setter for the assistant shell */
        window.gbSetTitlebarTitle = function (text) {
            title.textContent = text || "";
        };
    }

    function makeBtn(action, label, iconHtml) {
        var btn = document.createElement("button");
        btn.className = "gb-titlebar-btn" + (action === "close" ? " gb-titlebar-btn--close" : "");
        btn.setAttribute("aria-label", label);
        btn.title = label;
        btn.dataset.action = action;
        btn.innerHTML = iconHtml;
        btn.addEventListener("click", function () { doAction(action, btn); });
        return btn;
    }

    function doAction(action, btn) {
        if (!IS_TAURI) { return; }
        try {
            var win = window.__TAURI__.window.getCurrentWindow();
            if (action === "minimize") { win.minimize(); }
            else if (action === "maximize") { win.toggleMaximize(); }
            else if (action === "close") { win.close(); }
        } catch (e) { /* permission refused or API unavailable */ }
    }

    function watchMaximized(controls) {
        var maxBtn = controls.querySelector('[data-action="maximize"]');
        if (!maxBtn) { return; }
        var restoreIcon = '<svg width="12" height="12" viewBox="0 0 12 12"><rect x="2.5" y="3.5" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1"/><polyline points="4 3.5 4 1.5 10.5 1.5 10.5 8 8.5 8" fill="none" stroke="currentColor" stroke-width="1"/></svg>';
        var maxIcon = maxBtn.innerHTML;

        function update() {
            try {
                window.__TAURI__.window.getCurrentWindow().isMaximized().then(function (m) {
                    maxBtn.innerHTML = m ? restoreIcon : maxIcon;
                    maxBtn.title = m ? t.restore : t.max;
                    maxBtn.setAttribute("aria-label", m ? t.restore : t.max);
                });
            } catch (e) { /* */ }
        }
        window.addEventListener("resize", update);
        update();
    }

    function isDark() {
        /* login.html has a dark bg; detect by body bg or class */
        var bg = getComputedStyle(document.body).backgroundColor || "";
        if (/rgba?\((\d+)/.test(bg)) {
            var r = parseInt(RegExp.$1, 10);
            return r < 40;
        }
        return document.body.classList.contains("dark") || document.body.dataset.theme === "dark";
    }

    /* Boot: run as soon as body is available */
    if (document.body) {
        build();
    } else {
        document.addEventListener("DOMContentLoaded", build);
    }
})();
