"use strict";
/**
 * 09_settings.js - Native Settings view for the assistant shell.
 * Sections: Conta, Aparencia, Idioma, Modelo, Sobre.
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

    /* ================================================================
       Build the settings shell (nav + panel)
       ================================================================ */
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
        var backBtn = document.getElementById("gbSetBack");
        if (backBtn) {
            backBtn.addEventListener("click", function () { A.switchView("chat"); });
        }
        /* Esc key */
        container._escHandler = function (e) {
            if (e.key === "Escape" && A.currentView === "settings") {
                A.switchView("chat");
            }
        };
        document.addEventListener("keydown", container._escHandler);
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
        };
        return '<svg class="gb-set-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">' + (icons[name] || '') + '</svg>';
    }

    /* ================================================================
       Section renderers
       ================================================================ */
    function renderSection(id) {
        var panel = document.getElementById("gbSetPanel");
        if (!panel) { return; }
        var renderers = {
            account: renderAccount,
            instructions: renderInstructions,
            appearance: renderAppearance,
            language: renderLanguage,
            model: renderModel,
            shortcuts: renderShortcuts,
            about: renderAbout,
        };
        EXTRA_SECTIONS.forEach(function (s) { renderers[s.id] = s.render; });
        var fn = renderers[id];
        if (fn) {
            /* Show a loading indicator; the renderer replaces it. */
            panel.innerHTML = '<div class="gb-set-loading">' +
                '<span class="gb-set-loading-dot"></span>' +
                esc(A.t("assistant.connecting", "Loading...")) + '</div>';
            try { fn(panel); } catch (e) {
                panel.innerHTML = sectionError(e);
            }
        } else {
            panel.innerHTML = "";
        }
    }

    function sectionError(e) {
        var msg = e && e.message ? e.message : String(e);
        return '<div class="gb-set-error">' +
            '<p>' + esc(msg) + '</p>' +
            '<button class="gb-set-btn-sm" onclick="' +
            "var A=window.GBAssistant;if(A&&A.renderSettings)A.renderSettings();" +
            '">' + esc(A.t("settings.retry", "Tentar novamente")) + '</button></div>';
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

    /* -- Appearance (pattern data in 09b_patterns.js) -- */
    var PATTERNS = A._patterns || [];
    var getCurrentPattern = A._getCurrentPattern || function () { return "image"; };
    var setCurrentPattern = A._setCurrentPattern || function () {};
    var applyPattern = A._applyPattern || function () {};

    function renderAppearance(panel) {
        var presets = (window.GBIdentity && window.GBIdentity.presets) || [];
        var currentAccent = getComputedStyle(document.documentElement)
            .getPropertyValue("--id-accent").trim().toUpperCase();

        /* Theme selector */
        var curTheme = window.GBTheme ? window.GBTheme.get() : "system";
        var themeOpts = [
            { id: "light", key: "settings.theme-light" },
            { id: "dark", key: "settings.theme-dark" },
            { id: "system", key: "settings.theme-system" },
        ];
        var html = '<div class="gb-set-card">' +
            '<h2 class="gb-set-title">' + esc(A.t("settings.appearance", "Appearance")) + '</h2>' +
            '<p class="gb-set-desc">' + esc(A.t("settings.theme-label", "Theme")) + '</p>' +
            '<div class="gb-set-theme-row" id="gbThemeRow">';
        for (var ti = 0; ti < themeOpts.length; ti++) {
            var to = themeOpts[ti];
            var tActive = to.id === curTheme ? " active" : "";
            html += '<button class="gb-set-lang-btn gb-set-theme-btn' + tActive +
                '" data-theme="' + to.id + '">' + esc(A.t(to.key, to.id)) + '</button>';
        }
        html += '</div>' + (A._sliderRow ? A._sliderRow("frost") : "") + '</div>';

        /* Text size selector */
        var curSize = window.GBTextSize ? window.GBTextSize.get() : "default";
        var sizeOpts = [
            { id: "small", key: "settings.text-small" },
            { id: "default", key: "settings.text-default" },
            { id: "large", key: "settings.text-large" },
            { id: "xlarge", key: "settings.text-xlarge" },
        ];
        html += '<div class="gb-set-card">' +
            '<p class="gb-set-desc">' + esc(A.t("settings.text-size-label", "Text size")) + '</p>' +
            '<div class="gb-set-theme-row" id="gbSizeRow">';
        for (var si = 0; si < sizeOpts.length; si++) {
            var so = sizeOpts[si];
            var sActive = so.id === curSize ? " active" : "";
            html += '<button class="gb-set-lang-btn gb-set-size-btn' + sActive +
                '" data-size="' + so.id + '">' + esc(A.t(so.key, so.id)) + '</button>';
        }
        html += '</div>' +
            '<p class="gb-set-hint">' + esc(A.t("settings.text-size-hint", "Ctrl+= / Ctrl+- / Ctrl+0")) + '</p>' +
            '</div>';

        /* Accent colour */
        html += '<div class="gb-set-card">' +
            '<p class="gb-set-desc">' + esc(A.t("settings.accent-label", "Accent colour")) + '</p>' +
            '<div class="gb-set-presets" id="gbAccentPresets">';
        for (var i = 0; i < presets.length; i++) {
            var p = presets[i];
            var label = A.locale.startsWith("pt") ? p.pt : p.en;
            var sel = p.hex.toUpperCase() === currentAccent ? " gb-set-swatch-active" : "";
            html += '<button class="gb-set-swatch' + sel +
                '" data-hex="' + esc(p.hex) + '" title="' + esc(label) +
                '" style="background:' + esc(p.hex) + '"><span class="gb-sr-only">' +
                esc(label) + '</span></button>';
        }
        html += '</div>' +
            '<div class="gb-set-custom-row">' +
                '<label class="gb-set-label" for="gbAccentCustom">' +
                    esc(A.t("settings.custom-colour", "Custom")) + '</label>' +
                '<input type="color" id="gbAccentCustom" class="gb-set-colour-input" value="' +
                    esc(currentAccent || "#F28C28") + '" />' +
            '</div>' +
            '<div class="gb-set-preview" id="gbAccentPreview">' +
                '<button class="gb-set-preview-btn">' + esc(A.t("settings.preview-button", "Button")) + '</button>' +
                '<a class="gb-set-preview-link" href="#">' + esc(A.t("settings.preview-link", "Link text")) + '</a>' +
                '<div class="gb-set-preview-row gb-set-preview-selected">' + esc(A.t("settings.preview-selected", "Selected row")) + '</div>' +
            '</div>' +
            '<p class="gb-set-hint" id="gbAccentHint"></p>' +
        '</div>';

        /* Background pattern chooser */
        var curPat = getCurrentPattern();
        html += '<div class="gb-set-card">' +
            '<h2 class="gb-set-title">' + esc(A.t("settings.bg-label", "Chat background")) + '</h2>' +
            '<div class="gb-set-bg-grid" id="gbBgGrid">';
        for (var pi = 0; pi < PATTERNS.length; pi++) {
            var pat = PATTERNS[pi];
            var nameKey = "settings.bg-" + pat.id;
            var name = A.t(nameKey, pat.id);
            var selCls = pat.id === curPat ? " gb-set-bg-active" : "";
            html += '<button class="gb-set-bg-tile' + selCls + '" data-pat="' + esc(pat.id) + '" title="' + esc(name) + '">' +
                '<div class="gb-set-bg-preview" data-pat-id="' + esc(pat.id) + '"></div>' +
                '<span class="gb-set-bg-name">' + esc(name) + '</span></button>';
        }
        html += '</div>' + (A._sliderRow ? A._sliderRow("strength") + A._sliderRow("scale") : "") + '</div>';

        panel.innerHTML = html;

        /* Bind theme selector */
        panel.querySelectorAll(".gb-set-theme-btn").forEach(function (btn) {
            btn.addEventListener("click", function () {
                var theme = btn.getAttribute("data-theme");
                if (window.GBTheme) { window.GBTheme.set(theme); }
                panel.querySelectorAll(".gb-set-theme-btn").forEach(function (b) {
                    b.classList.toggle("active", b.getAttribute("data-theme") === theme);
                });
            });
        });

        /* Bind text size selector */
        panel.querySelectorAll(".gb-set-size-btn").forEach(function (btn) {
            btn.addEventListener("click", function () {
                var size = btn.getAttribute("data-size");
                if (window.GBTextSize) { window.GBTextSize.set(size); }
                panel.querySelectorAll(".gb-set-size-btn").forEach(function (b) {
                    b.classList.toggle("active", b.getAttribute("data-size") === size);
                });
            });
        });

        /* Bind accent preset clicks */
        panel.querySelectorAll(".gb-set-swatch").forEach(function (sw) {
            sw.addEventListener("click", function () {
                applyAccent(sw.getAttribute("data-hex"), panel);
            });
        });
        var customInput = document.getElementById("gbAccentCustom");
        if (customInput) {
            customInput.addEventListener("input", function () {
                applyAccent(customInput.value, panel);
            });
        }

        /* Live pattern previews in the tiles; opacity, scale and frost sliders */
        panel.querySelectorAll(".gb-set-bg-preview").forEach(function (div) { if (A._previewPattern) { A._previewPattern(div); } });
        if (A._bindSliders) { A._bindSliders(panel); }

        /* Bind pattern tile clicks */
        panel.querySelectorAll(".gb-set-bg-tile").forEach(function (tile) {
            tile.addEventListener("click", function () {
                var id = tile.getAttribute("data-pat");
                setCurrentPattern(id);
                applyPattern(id);
                panel.querySelectorAll(".gb-set-bg-tile").forEach(function (t) {
                    t.classList.toggle("gb-set-bg-active", t.getAttribute("data-pat") === id);
                });
                if (A._syncSliders) { A._syncSliders(panel); }
            });
        });
    }

    function applyAccent(hex, panel) {
        if (!hex || !window.GBIdentity) { return; }
        window.GBIdentity.apply(hex);
        /* Persist locally */
        try { localStorage.setItem("gb-accent", hex); } catch (e) { /* */ }
        /* Try server */
        var hint = document.getElementById("gbAccentHint");
        A.apiFetch("/api/appearance", {
            method: "PUT",
            body: JSON.stringify({ accent: hex })
        }).then(function (r) {
            if (r.ok) {
                if (hint) { hint.textContent = A.t("settings.saved-server", "Saved."); }
            } else {
                if (hint) { hint.textContent = A.t("settings.saved-local", "Saved on this device."); }
            }
        }).catch(function () {
            if (hint) { hint.textContent = A.t("settings.saved-local", "Saved on this device."); }
        });
        /* Update swatch highlights */
        if (panel) {
            var upper = hex.toUpperCase();
            panel.querySelectorAll(".gb-set-swatch").forEach(function (sw) {
                sw.classList.toggle("gb-set-swatch-active",
                    sw.getAttribute("data-hex").toUpperCase() === upper);
            });
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
        if (section) { currentSection = section; }
        A.switchView("settings");
    };

    /* Accent boot: restore from server or localStorage */
    A.restoreAccent = function () {
        var serverAccent = A.productConfig && A.productConfig.accent;
        if (serverAccent && window.GBIdentity) { window.GBIdentity.apply(serverAccent); return; }
        try { var stored = localStorage.getItem("gb-accent");
            if (stored && window.GBIdentity) { window.GBIdentity.apply(stored); }
        } catch (e) { /* */ }
    };
})();
