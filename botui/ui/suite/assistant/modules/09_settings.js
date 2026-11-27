"use strict";
/**
 * 09_settings.js - Settings frame for the assistant shell: the numbered section
 * list in the rail, the page heading and the section router. The pages live in
 * 09c (Persona), 09f (Memória), 09d (Conta, Sobre), 09e (Idioma, Atalhos), 10* (Modelo) and
 * 10i (Limites). Appearance is a window next to the gear (09b_patterns.js).
 */

(function () {
    var A = window.GBAssistant;
    var currentSection = "model";
    var sections = [
        ["account", "Conta", "Account", "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"],
        ["instructions", "Persona", "Persona", "M12 8V4H8M4 8h16v12H4zM2 14h2M20 14h2M9 13v2M15 13v2"],
        ["memory", "Memória", "Memory", "M6 3h12v18l-6-4-6 4zM9 8h6M9 12h4"],
        ["language", "Idioma", "Language", "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20M2 12h20"],
        ["model", "Modelo", "Model", "M4 4h16v16H4zM9 9h6v6H9zM15 2v2M15 20v2M2 15h2M2 9h2M20 15h2M20 9h2M9 2v2M9 20v2"],
        ["limits", "Limites", "Limits", "M12 14l4-4M3.34 19a10 10 0 1 1 17.32 0"],
        ["shortcuts", "Atalhos de teclado", "Keyboard shortcuts", "M2 6h20v12H2zM6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"],
        ["about", "Sobre", "About", "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM12 16v-4M12 8h.01"]
    ];
    var signOutPath = "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9";

    function tr(pt, en) { return A.locale && A.locale.startsWith("pt") ? pt : en; }
    A.settingsText = tr;

    function esc(s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }
    A.settingsEsc = esc;

    /* Stroke icon used by the settings pages: one path, Mantis line weight. */
    A.settingsIcon = function (path, size, stroke) {
        return '<svg aria-hidden="true" width="' + size + '" height="' + size + '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="' +
            (stroke || 1.7) + '" stroke-linecap="round" stroke-linejoin="round"><path d="' + path + '"/></svg>';
    };

    A.settingsHeading = function (kicker, title, description) {
        return '<header class="gb-set-heading"><span class="gb-set-kicker">' +
            esc(kicker) + '</span><h1>' + esc(title) + '</h1>' +
            (description ? '<p>' + esc(description) + '</p>' : '') + '</header>';
    };

    A.renderSettings = function () {
        var container = document.getElementById("gbSettingsContainer");
        if (!container) { return; }
        var navHost = document.getElementById("gbSettingsNav");
        container.innerHTML = '<div class="gb-set-scroll" id="gbSettingsScroll"></div>';
        if (navHost) {
            var nav = '<div class="gb-set-nav"><button type="button" class="gb-set-back" id="gbSetBack">' +
                A.settingsIcon("m15 6-6 6 6 6", 14, 3) + '<span>' + esc(A.t("settings.back", "Back to conversation")) + '</span></button>';
            sections.forEach(function (section, index) {
                nav += '<button type="button" class="gb-set-tab" data-section="' + section[0] +
                    '"><i class="gb-set-gutter" aria-hidden="true"></i><span class="gb-set-tab-in" data-num="0' + (index + 1) + '">' + A.settingsIcon(section[3], 15) +
                    '<span class="gb-set-tab-label">' + esc(tr(section[1], section[2])) + '</span></span></button>';
            });
            nav += '<button type="button" class="gb-set-tab gb-set-nav-signout" id="gbSetNavSignOut"><i class="gb-set-gutter" aria-hidden="true"></i>' +
                '<span class="gb-set-tab-in">' + A.settingsIcon(signOutPath, 15, 1.8) + '<span class="gb-set-tab-label">' +
                esc(A.t("assistant.sign-out", "Sign out")) + '</span></span></button></div>';
            navHost.innerHTML = nav;
            navHost.querySelector("#gbSetBack").onclick = function () { A.switchView("chat"); };
            navHost.querySelector("#gbSetNavSignOut").onclick = function () { A.signOut(); };
            navHost.querySelectorAll("[data-section]").forEach(function (button) {
                button.onclick = function () { renderSection(button.dataset.section); };
            });
        }
        renderSection(currentSection);
    };

    function sectionIcon(name) {
        var icons = {
            user: '<circle cx="12" cy="8" r="4"/><path d="M20 21a8 8 0 1 0-16 0"/>',
            palette: '<circle cx="13.5" cy="6.5" r=".5" fill="currentColor"/><circle cx="17.5" cy="10.5" r=".5" fill="currentColor"/><circle cx="8.5" cy="7.5" r=".5" fill="currentColor"/><circle cx="6.5" cy="12" r=".5" fill="currentColor"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.93 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-1 0-.83.67-1.5 1.5-1.5H16c3.31 0 6-2.69 6-6 0-5.17-4.36-9-10-9z"/>',
            globe: '<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
            cpu: '<rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/>',
            info: '<circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>',
            keyboard: '<rect x="2" y="6" width="20" height="12" rx="2"/><line x1="6" y1="10" x2="6" y2="10.01"/><line x1="10" y1="10" x2="10" y2="10.01"/><line x1="14" y1="10" x2="14" y2="10.01"/><line x1="18" y1="10" x2="18" y2="10.01"/><line x1="8" y1="14" x2="16" y2="14"/>',
            message: '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>',
            server: '<rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/>',
        };
        return '<svg class="gb-set-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">' + (icons[name] || '') + '</svg>';
    }

    /* ================================================================
       Section renderers
       ================================================================ */
    function renderSection(id) {
        var scroller = document.getElementById("gbSettingsScroll");
        if (!scroller) { return; }
        currentSection = id;
        document.querySelectorAll("#gbSettingsNav [data-section]").forEach(function (button) {
            var active = button.dataset.section === id;
            button.classList.toggle("active", active);
            if (active) { button.setAttribute("aria-current", "page"); }
            else { button.removeAttribute("aria-current"); }
        });
        if (A.updateBreadcrumb) { A.updateBreadcrumb(); }
        var panel = document.createElement("div");
        panel.id = "gbSetPanel";
        panel.dataset.section = id;
        panel.className = "gb-set-panel";
        scroller.replaceChildren(panel);
        scroller.scrollTop = 0;
        var renderers = {
            account: A.renderAccount,
            instructions: A.renderInstructions,
            memory: A.renderMemory,
            language: A.renderLanguage,
            model: A.renderModelFull,
            shortcuts: A.renderShortcuts,
            about: A.renderAbout,
            limits: A.renderLimits
        };
        var fn = renderers[id];
        if (typeof fn !== "function") { panel.innerHTML = ""; return; }
        panel.innerHTML = '<div class="gb-set-loading"><span class="gb-set-loading-dot"></span>' +
            esc(A.t("assistant.connecting", "Loading...")) + '</div>';
        try {
            Promise.resolve(fn(panel)).catch(function (e) {
                if (panel.isConnected) { panel.innerHTML = sectionError(e); }
            });
        } catch (e) {
            panel.innerHTML = sectionError(e);
        }
    }

    function sectionError(e) {
        var msg = e && e.message ? e.message : String(e);
        return '<div class="gb-set-error"><p>' + esc(msg) + '</p>' +
            '<button class="gb-set-btn-sm" onclick="var A=window.GBAssistant;if(A&&A.renderSettings)A.renderSettings();">' +
            esc(A.t("settings.retry", "Tentar novamente")) + '</button></div>';
    }

    /* Open settings at a section. "appearance" opens the Appearance window. */
    A.openSettings = function (section) {
        if (section === "appearance") {
            var opener = document.getElementById("gbNavAppearance");
            if (opener) { opener.click(); }
            return;
        }
        if (sections.some(function (entry) { return entry[0] === section; })) { currentSection = section; }
        if (A.currentView === "settings") { renderSection(currentSection); }
        else { A.switchView("settings"); }
    };

    /* Accent boot: restore from server or localStorage */
    A.restoreAccent = function () {
        var serverAccent = A.productConfig && A.productConfig.accent;
        if (serverAccent && window.GBIdentity) { window.GBIdentity.apply(serverAccent); return; }
        try {
            var stored = localStorage.getItem("gb-accent");
            if (stored && window.GBIdentity) { window.GBIdentity.apply(stored); }
        } catch (e) { /* storage unavailable */ }
    };
})();
