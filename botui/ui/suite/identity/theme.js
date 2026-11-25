"use strict";
/* Light/dark/System adapter for standalone pages. The shell's ThemeManager owns
 * data-theme; here we delegate to window.GBTheme or apply data-color-scheme plus
 * inline root tokens. One storage key (gb-theme) everywhere. */
(function () {
    var STORAGE_KEY = "gb-theme";
    var VALID = { light: 1, dark: 1, system: 1 };
    var TOKENS = {
        light: { "--background": "0 0% 100%", "--foreground": "222 47% 11%", "--bg": "#f8fafc", "--surface": "#ffffff", "--text": "#1e293b" },
        dark: { "--background": "222 47% 11%", "--foreground": "210 40% 98%", "--bg": "#0f172a", "--surface": "#1e293b", "--text": "#f8fafc" },
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
        if (!mql && window.matchMedia) mql = window.matchMedia("(prefers-color-scheme: dark)");
        return !!(mql && mql.matches);
    }

    function resolve(pref) {
        if (pref === "system") return osDark() ? "dark" : "light";
        return VALID[pref] ? pref : (osDark() ? "dark" : "light");
    }

    function applyLocal(pref) {
        var effective = resolve(pref);
        var tokens = TOKENS[effective];
        document.documentElement.setAttribute("data-color-scheme", effective);
        Object.keys(tokens).forEach(function (name) {
            document.documentElement.style.setProperty(name, tokens[name]);
        });
        if (window.GBIdentity && typeof window.GBIdentity.reapply === "function") window.GBIdentity.reapply();
        window.dispatchEvent(new CustomEvent("gb:themechange", { detail: { themeId: effective, preference: pref } }));
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
        set(owner.get() || stored());
    } else {
        window.GBTheme = { set: set, get: get, current: current, resolve: resolve, apply: applyLocal };
        applyLocal(stored());
    }

    // Follow the OS live only while the preference is "system".
    try {
        if (!mql && window.matchMedia) mql = window.matchMedia("(prefers-color-scheme: dark)");
        if (mql && mql.addEventListener) {
            mql.addEventListener("change", function () {
                if (stored() === "system") set("system");
            });
        }
    } catch (e) { /* matchMedia or addEventListener not supported */ }

    // Cross-tab changes sync without writing data-theme.
    window.addEventListener("storage", function (e) {
        if (e.key === STORAGE_KEY && e.newValue && VALID[e.newValue]) {
            if (owner && typeof owner.set === "function") owner.set(e.newValue);
            else applyLocal(e.newValue);
        }
    });
})();
