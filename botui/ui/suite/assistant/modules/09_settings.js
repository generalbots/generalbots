"use strict";
/**
 * 09_settings.js - Native Settings view for the assistant shell.
 * Sections: Conta, Aparencia, Idioma, Modelo, Sobre.
 */

(function () {
    var A = window.GBAssistant;
    var currentSection = "account";

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
            { id: "account",    icon: "user",    key: "settings.account" },
            { id: "appearance", icon: "palette",  key: "settings.appearance" },
            { id: "language",   icon: "globe",    key: "settings.language" },
            { id: "model",      icon: "cpu",      key: "settings.model" },
            { id: "about",      icon: "info",     key: "settings.about" },
        ];
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
            appearance: renderAppearance,
            language: renderLanguage,
            model: renderModel,
            about: renderAbout,
        };
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

    /* -- Appearance -- */
    var PATTERNS = [
        { id: "none", css: "" },
        { id: "dots", css: "background-image:radial-gradient(var(--id-accent) 0.8px, var(--id-paper) 0.8px);background-size:8px 8px;" },
        { id: "microdots", css: "background:radial-gradient(var(--id-accent) 15%,transparent 16%) 0 0,radial-gradient(var(--id-accent) 15%,transparent 16%) 4px 4px,radial-gradient(var(--pat-accent20) 15%,transparent 20%) 0 1px,radial-gradient(var(--pat-accent20) 15%,transparent 20%) 4px 5px;background-color:var(--id-paper);background-size:8px 8px;" },
        { id: "grid", css: "background-image:linear-gradient(var(--id-accent) 0.8px,transparent 0.8px),linear-gradient(90deg,var(--id-accent) 0.8px,transparent 0.8px),linear-gradient(var(--id-accent) 0.4px,transparent 0.4px),linear-gradient(90deg,var(--id-accent) 0.4px,var(--id-paper) 0.4px);background-size:40px 40px,40px 40px,8px 8px,8px 8px;background-position:-0.8px -0.8px,-0.8px -0.8px,-0.4px -0.4px,-0.4px -0.4px;" },
        { id: "blueprint", css: "background-image:repeating-linear-gradient(to right,transparent 0,transparent 1.333px,var(--id-paper) 1.333px,var(--id-paper) 4px,transparent 4px,transparent 5.333px),repeating-linear-gradient(to bottom,transparent 0,transparent 1.333px,var(--id-paper) 1.333px,var(--id-paper) 4px,transparent 4px,transparent 5.333px),linear-gradient(to bottom,var(--id-accent) 0.48px,transparent 0.48px),linear-gradient(to right,var(--id-accent) 0.48px,transparent 0.48px);background-size:100% 100%,100% 100%,16px 16px,16px 16px;background-position:0 0,0 0,0 -0.16px,-0.16px 0;" },
        { id: "weave", css: "background:radial-gradient(circle,transparent 20%,var(--id-paper) 20%,var(--id-paper) 80%,transparent 80%,transparent),radial-gradient(circle,transparent 20%,var(--id-paper) 20%,var(--id-paper) 80%,transparent 80%,transparent) 15px 15px,linear-gradient(var(--id-accent) 1.2px,transparent 1.2px) 0 -0.6px,linear-gradient(90deg,var(--id-accent) 1.2px,var(--id-paper) 1.2px) -0.6px 0;background-size:30px 30px,30px 30px,15px 15px,15px 15px;" },
        { id: "mosaic", css: "background-image:repeating-linear-gradient(0deg,transparent,transparent 11px,var(--id-accent) 11px,var(--id-accent) 12px,transparent 12px,transparent 23px,var(--id-accent) 23px,var(--id-accent) 24px),repeating-linear-gradient(90deg,transparent,transparent 11px,var(--id-accent) 11px,var(--id-accent) 12px,transparent 12px,transparent 23px,var(--id-accent) 23px,var(--id-accent) 24px),radial-gradient(circle at 12px 12px,var(--id-accent) 0.96px,transparent 1.46px),radial-gradient(circle at 0px 0px,var(--id-accent) 0.72px,transparent 1.22px),radial-gradient(circle at 24px 0px,var(--id-accent) 0.72px,transparent 1.22px),radial-gradient(circle at 0px 24px,var(--id-accent) 0.72px,transparent 1.22px),radial-gradient(circle at 24px 24px,var(--id-accent) 0.72px,transparent 1.22px);background-size:24px 24px;background-color:var(--id-paper);" },
    ];
    /* "image" = the shipped pattern.png (dot composition) */
    PATTERNS.splice(1, 0, { id: "image", css: "__IMAGE__" });

    function getCurrentPattern() {
        try { return localStorage.getItem("gb-chat-pattern") || "image"; } catch (e) { return "image"; }
    }
    function setCurrentPattern(id) {
        try { localStorage.setItem("gb-chat-pattern", id); } catch (e) { /* */ }
    }

    function applyPattern(id) {
        var main = document.querySelector(".gb-main");
        if (!main) { return; }
        /* Reset: remove all pattern overrides, revert to original */
        main.classList.remove("gb-pattern-none", "gb-pattern-css", "gb-pattern-image");
        main.style.removeProperty("--gb-pat-css");
        main.removeAttribute("data-pattern");

        if (id === "none") {
            main.classList.add("gb-pattern-none");
        } else if (id === "image") {
            main.classList.add("gb-pattern-image");
        } else {
            var pat = PATTERNS.filter(function (p) { return p.id === id; })[0];
            if (pat && pat.css) {
                main.classList.add("gb-pattern-css");
                main.setAttribute("data-pattern", id);
                /* Apply via a style element for the pattern pseudo-element */
                applyPatternCSS(pat.css);
            }
        }
    }

    var patStyleEl = null;
    function applyPatternCSS(css) {
        if (!patStyleEl) {
            patStyleEl = document.createElement("style");
            patStyleEl.id = "gbPatternStyle";
            document.head.appendChild(patStyleEl);
        }
        /* The accent at 20% alpha for patterns needing it.
           Must NOT override background-size — each pattern defines its own. */
        patStyleEl.textContent =
            ':root { --pat-accent20: color-mix(in srgb, var(--id-accent) 20%, transparent); }' +
            '.gb-pattern-css.gb-main::before { background: none; ' + css + ' opacity: 0.15; -webkit-mask: none !important; mask: none !important; animation: none; z-index: 1; pointer-events: none; }';
    }

    A.bootPattern = function () {
        var id = getCurrentPattern();
        applyPattern(id);
    };

    function renderAppearance(panel) {
        var presets = (window.GBIdentity && window.GBIdentity.presets) || [];
        var currentAccent = getComputedStyle(document.documentElement)
            .getPropertyValue("--id-accent").trim().toUpperCase();
        var html = '<div class="gb-set-card">' +
            '<h2 class="gb-set-title">' + esc(A.t("settings.appearance", "Appearance")) + '</h2>' +
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
        html += '</div></div>';

        panel.innerHTML = html;

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

        /* Apply live pattern previews to tiles */
        panel.querySelectorAll(".gb-set-bg-preview").forEach(function (div) {
            var patId = div.getAttribute("data-pat-id");
            if (!patId || patId === "none") { return; }
            if (patId === "image") {
                /* The shipped dot composition image is too detailed for a 64x40 tile;
                   show a suggestive dot cluster as a recognisable preview. */
                div.style.cssText = "background:" +
                    "radial-gradient(1.5px 1.5px at 20% 30%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(2px 2px at 55% 20%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(1px 1px at 40% 60%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(2.5px 2.5px at 70% 55%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(1.5px 1.5px at 30% 80%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(1px 1px at 80% 35%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(2px 2px at 15% 55%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(1.5px 1.5px at 60% 75%, var(--id-accent) 60%, transparent 100%)," +
                    "radial-gradient(1px 1px at 85% 70%, var(--id-accent) 60%, transparent 100%)," +
                    "var(--id-paper);opacity:0.6;";
                return;
            }
            var pat = PATTERNS.filter(function (p) { return p.id === patId; })[0];
            if (pat && pat.css) { div.style.cssText = pat.css + "opacity:0.3;"; }
        });

        /* Bind pattern tile clicks */
        panel.querySelectorAll(".gb-set-bg-tile").forEach(function (tile) {
            tile.addEventListener("click", function () {
                var id = tile.getAttribute("data-pat");
                setCurrentPattern(id);
                applyPattern(id);
                panel.querySelectorAll(".gb-set-bg-tile").forEach(function (t) {
                    t.classList.toggle("gb-set-bg-active", t.getAttribute("data-pat") === id);
                });
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
                /* Re-render settings in the new language */
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
            panel.innerHTML = '<p class="gb-set-hint">Loading...</p>';
        }
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
       Accent boot: restore from server or localStorage
       ================================================================ */
    A.restoreAccent = function () {
        /* Server value (from /api/product) takes priority */
        var serverAccent = A.productConfig && A.productConfig.accent;
        if (serverAccent && window.GBIdentity) {
            window.GBIdentity.apply(serverAccent);
            return;
        }
        /* Fall back to localStorage */
        try {
            var stored = localStorage.getItem("gb-accent");
            if (stored && window.GBIdentity) {
                window.GBIdentity.apply(stored);
            }
        } catch (e) { /* */ }
    };
})();
