"use strict";
/**
 * 09_settings.js - Settings frame for the assistant shell: the numbered section
 * list in the rail, the page heading and the section router. The pages live in
 * 09c (Persona), 09f (Memória), 09d (Conta, Sobre), 09e (Idioma, Atalhos), 10* (Modelo) and
 * 10i (Limites). Appearance is a window next to the gear (09b_patterns.js).
 */

(function () {
    var A = window.GBAssistant;
    var currentSection = "account";
    var EXTRA_SECTIONS = [];
    A.registerSettingsSection = function (section) { EXTRA_SECTIONS.push(section); };

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
        var sections = [
            { id: "account",      icon: "user",     key: "settings.account" },
            { id: "instructions", icon: "message",  key: "settings.instructions" },
            { id: "appearance",   icon: "palette",  key: "settings.appearance" },
            { id: "language",     icon: "globe",    key: "settings.language" },
            { id: "model",        icon: "cpu",      key: "settings.model" },
            { id: "shortcuts",    icon: "keyboard", key: "shortcut.title" },
            { id: "about",        icon: "info",     key: "settings.about" },
        ].concat(EXTRA_SECTIONS);
        var nav = '<nav class="gb-set-nav" role="tablist">';
        nav += '<button class="gb-set-back" id="gbSetBack" aria-label="' +
            esc(A.t("settings.back", "Back to conversation")) + '">' +
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>' +
            '<span>' + esc(A.t("settings.back", "Back to conversation")) + '</span></button>';
        for (var i = 0; i < sections.length; i++) {
            var s = sections[i];
            nav += '<button class="gb-set-tab' + (s.id === currentSection ? ' active' : '') +
                '" data-section="' + s.id + '" role="tab" aria-selected="' +
                (s.id === currentSection) + '">' +
                sectionIcon(s.icon) +
                '<span>' + esc(A.t(s.key, s.id)) + '</span></button>';
        }
        nav += '</nav>';
        container.innerHTML = nav + '<div class="gb-set-panel" id="gbSetPanel"></div>';

        /* Bind nav */
        container.querySelectorAll(".gb-set-tab").forEach(function (btn) {
            btn.addEventListener("click", function () {
                currentSection = btn.getAttribute("data-section");
                container.querySelectorAll(".gb-set-tab").forEach(function (b) {
                    b.classList.toggle("active", b === btn);
                    b.setAttribute("aria-selected", b === btn);
                });
                renderSection(currentSection);
            });
        });
        renderSection(currentSection);
    };

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
        EXTRA_SECTIONS.forEach(function (s) { renderers[s.id] = s.render; });
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

    /* -- Account -- */
    function renderAccount(panel) {
        var name = A.userName || "";
        var email = A.userEmail || "";
        panel.innerHTML =
            '<div class="gb-set-card">' +
                '<h2 class="gb-set-title">' + esc(A.t("settings.account", "Account")) + '</h2>' +
                '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.name", "Name")) + '</span>' +
                    '<span class="gb-set-value">' + esc(name || "—") + '</span></div>' +
                '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.email", "E-mail")) + '</span>' +
                    '<span class="gb-set-value">' + esc(email || "—") + '</span></div>' +
                '<button class="gb-set-signout" id="gbSetSignOut">' +
                    esc(A.t("assistant.sign-out", "Sign out")) + '</button>' +
            '</div>';
        var btn = document.getElementById("gbSetSignOut");
        if (btn) { btn.addEventListener("click", function () { A.signOut(); }); }
    }

    /* -- Custom instructions (delegated to 09c_settings_instructions.js) -- */
    function renderInstructions(panel) {
        if (typeof A.renderInstructions === "function") {
            A.renderInstructions(panel);
        } else {
            panel.innerHTML = '<p class="gb-set-hint">' + (typeof window.gbT === "function" ? window.gbT("label-loading", "Loading...") : "Loading...") + '</p>';
        }
    }

    /* -- Language -- */
    function renderLanguage(panel) {
        var langs = [
            { code: "pt-BR", label: "Português (Brasil)" },
            { code: "en",    label: "English" },
        ];
        var html = '<div class="gb-set-card">' +
            '<h2 class="gb-set-title">' + esc(A.t("settings.language", "Language")) + '</h2>';
        for (var i = 0; i < langs.length; i++) {
            var l = langs[i];
            var active = A.locale === l.code || (l.code === "pt-BR" && A.locale.startsWith("pt"));
            html += '<button class="gb-set-lang-btn' + (active ? " active" : "") +
                '" data-lang="' + l.code + '">' + esc(l.label) + '</button>';
        }
        html += '</div>';
        panel.innerHTML = html;
        panel.querySelectorAll(".gb-set-lang-btn").forEach(function (btn) {
            btn.addEventListener("click", function () {
                var code = btn.getAttribute("data-lang");
                A.locale = code;
                try { localStorage.setItem("gb-locale", code); } catch (e) { /* */ }
                /* Re-translate every data-i18n element in the shell, re-render
                   dynamic components and settings so their labels update. */
                if (typeof A.translatePage === "function") { A.translatePage(); }
                if (typeof A.renderStatus === "function") { A.renderStatus(); }
                if (typeof A.renderGreeting === "function") { A.renderGreeting(); }
                A.renderSettings();
            });
        });
    }

    /* -- Model (delegated to 10_settings_model.js) -- */
    function renderModel(panel) {
        if (typeof A.renderModelFull === "function") {
            /* renderModelFull is async; catch rejections so they show in-panel */
            Promise.resolve(A.renderModelFull(panel)).catch(function (e) {
                panel.innerHTML = sectionError(e);
            });
        } else {
            panel.innerHTML = '<p class="gb-set-hint">' + (typeof window.gbT === "function" ? window.gbT("label-loading", "Loading...") : "Loading...") + '</p>';
        }
    }

    /* -- Keyboard shortcuts -- */
    function renderShortcuts(panel) {
        var isMac = /mac/i.test(navigator.platform || "");
        var mod = isMac ? "⌘" : "Ctrl";
        var shortcuts = [
            [mod + "+Shift+O", "shortcut.new-chat"],
            [mod + "+K", "shortcut.search"],
            [mod + "+B", "shortcut.sidebar"],
            [mod + "+,", "shortcut.settings"],
            ["Esc", "shortcut.stop"],
            ["↑", "shortcut.edit-last"],
        ];
        var html = '<div class="gb-set-card"><h2 class="gb-set-title">' +
            esc(A.t("shortcut.title", "Keyboard shortcuts")) + '</h2>' +
            '<ul class="gb-shortcut-list">';
        for (var i = 0; i < shortcuts.length; i++) {
            html += '<li><span>' + esc(A.t(shortcuts[i][1], shortcuts[i][1])) +
                '</span><kbd>' + esc(shortcuts[i][0]) + '</kbd></li>';
        }
        html += '</ul></div>';
        panel.innerHTML = html;
    }

    /* -- About -- */
    function renderAbout(panel) {
        var pc = A.productConfig || {};
        var name = pc.name || "General Bots";
        var version = pc.version || "";
        var buildDate = pc.build_date || pc.built_at || "";
        var commit = pc.commit || pc.git_commit || "";
        if (commit && commit.length > 8) { commit = commit.substring(0, 8); }
        panel.innerHTML =
            '<div class="gb-set-card">' +
                '<h2 class="gb-set-title">' + esc(A.t("settings.about", "About")) + '</h2>' +
                '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.product", "Product")) + '</span>' +
                    '<span class="gb-set-value">' + esc(name) + '</span></div>' +
                (version ? '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.version", "Version")) + '</span>' +
                    '<span class="gb-set-value">' + esc(version) + '</span></div>' : '') +
                (buildDate ? '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.build-date", "Build date")) + '</span>' +
                    '<span class="gb-set-value">' + esc(buildDate) + '</span></div>' : '') +
                (commit ? '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.commit", "Commit")) + '</span>' +
                    '<span class="gb-set-value" style="font-family:monospace">' + esc(commit) + '</span></div>' : '') +
                '<div class="gb-set-field"><span class="gb-set-label">' +
                    esc(A.t("settings.licence", "Licence")) + '</span>' +
                    '<span class="gb-set-value">MIT</span></div>' +
                '<p class="gb-set-hint">' +
                    esc(A.t("settings.open-source", "Open source")) +
                '</p>' +
            '</div>';
    }

    /* ================================================================
       Open settings to a specific section (e.g. from error card)
       ================================================================ */
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
