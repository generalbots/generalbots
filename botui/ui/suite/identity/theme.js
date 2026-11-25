"use strict";
/* Light/dark/System preference adapter for pages that do not load the shell's
 * ThemeManager (assistant.html, login.html).
 *
 * #2930 — js/theme-manager.js is the SINGLE owner of
 * document.documentElement's data-theme attribute. This script never writes
 * that attribute: where the owner is present it delegates to window.GBTheme,
 * and on standalone pages it applies the preference through the state-free
 * data-color-scheme axis plus inline root tokens, which the identity CSS and
 * accent derivation already consume. The storage key is gb-theme everywhere.
 *
 * Loaded before first paint so the standalone pages never flash the wrong
 * surface.
 */
(function () {
    var STORAGE_KEY = "gb-theme";
    var VALID = { light: 1, dark: 1, system: 1 };
    var TOKENS = {
        light: {
            "--background": "0 0% 100%",
            "--foreground": "222 47% 11%",
            "--bg": "#f8fafc",
            "--surface": "#ffffff",
            "--text": "#1e293b",
        },
        dark: {
            "--background": "222 47% 11%",
            "--foreground": "210 40% 98%",
            "--bg": "#0f172a",
            "--surface": "#1e293b",
            "--text": "#f8fafc",
        },
    };
    var mql = null;
    var owner = window.GBTheme;

    function stored() {
        try {
            var v = localStorage.getItem(STORAGE_KEY);
            if (v && VALID[v]) return v;
        } catch (e) { /* storage unavailable */ }
        return "system";
    }

    function osDark() {
        if (!mql && window.matchMedia) {
            mql = window.matchMedia("(prefers-color-scheme: dark)");
        }
        return !!(mql && mql.matches);
    }

    function resolve(pref) {
        if (pref === "system") return osDark() ? "dark" : "light";
        return VALID[pref] ? pref : (osDark() ? "dark" : "light");
    }

    /* Standalone apply path — data-color-scheme + inline tokens only. */
    function applyLocal(pref) {
        var effective = resolve(pref);
        var tokens = TOKENS[effective];
        document.documentElement.setAttribute("data-color-scheme", effective);
        Object.keys(tokens).forEach(function (name) {
            document.documentElement.style.setProperty(name, tokens[name]);
        });
        if (window.GBIdentity && typeof window.GBIdentity.reapply === "function") {
            window.GBIdentity.reapply();
        }
        window.dispatchEvent(new CustomEvent("gb:themechange", {
            detail: { themeId: effective, preference: pref },
        }));
    }

    function set(pref) {
        var value = VALID[pref] ? pref : "system";
        try { localStorage.setItem(STORAGE_KEY, value); } catch (e) { /* */ }
        if (owner && typeof owner.set === "function") {
            owner.set(value);
            return;
        }
        applyLocal(value);
    }

    function get() { return stored(); }
    function current() { return resolve(stored()); }

    if (owner && typeof owner.set === "function") {
        // The shell owns data-theme; just mirror the stored choice into it.
        set(owner.get() || stored());
    } else {
        window.GBTheme = {
            set: set,
            get: get,
            current: current,
            resolve: resolve,
            apply: applyLocal,
        };
        applyLocal(stored());
    }

    // Follow the OS live only while the preference is "system".
    try {
        if (!mql && window.matchMedia) {
            mql = window.matchMedia("(prefers-color-scheme: dark)");
        }
        if (mql && mql.addEventListener) {
            mql.addEventListener("change", function () {
                if (stored() === "system") set("system");
            });
        }
    } catch (e) { /* matchMedia or addEventListener not supported */ }

    // Keep a cross-tab preference change in sync without writing data-theme.
    window.addEventListener("storage", function (e) {
        if (e.key === STORAGE_KEY && e.newValue && VALID[e.newValue]) {
            if (owner && typeof owner.set === "function") owner.set(e.newValue);
            else applyLocal(e.newValue);
        }
    });
})();
