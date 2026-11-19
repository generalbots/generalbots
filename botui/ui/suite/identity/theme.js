"use strict";
/* Theme manager — light, dark, system. Applies data-theme on <html>,
 * persists to localStorage, follows prefers-color-scheme live.
 * Plain script, no modules. Exposes window.GBTheme.
 *
 * MUST run BEFORE first paint (loaded in <head> after tokens.css).
 * Reads gb-theme from localStorage and applies data-theme immediately
 * so the page never flashes white in dark mode.
 */
(function () {
    var STORAGE_KEY = "gb-theme";
    var VALID = { light: 1, dark: 1, system: 1 };
    var mql = null;

    /** Read the persisted preference or fall back to "system". */
    function stored() {
        try {
            var v = localStorage.getItem(STORAGE_KEY);
            if (v && VALID[v]) { return v; }
        } catch (e) { /* storage unavailable */ }
        return "system";
    }

    /** Whether the OS prefers dark. */
    function osDark() {
        if (!mql) { mql = window.matchMedia("(prefers-color-scheme: dark)"); }
        return mql.matches;
    }

    /** Resolve "system" into "light" or "dark". */
    function resolve(pref) {
        if (pref === "system") { return osDark() ? "dark" : "light"; }
        return pref;
    }

    /** Set data-theme on <html> and re-derive accent for the new surface. */
    function applyTheme(pref) {
        var effective = resolve(pref);
        document.documentElement.setAttribute("data-theme", effective);
        /* Re-derive accent so contrast checks use the correct paper. The
           accent in use may be the product's default (never stored): the
           one last applied counts, the stored choice is the fallback. */
        if (window.GBIdentity) {
            var hex = null;
            try { hex = localStorage.getItem("gb-accent"); } catch (e) { /* */ }
            if (typeof window.GBIdentity.reapply === "function" && window.GBIdentity.reapply()) {
                return;
            }
            if (hex) {
                window.GBIdentity.apply(hex);
            }
        }
    }

    /** Public: set and persist the preference ("light" | "dark" | "system"). */
    function set(pref) {
        if (!VALID[pref]) { pref = "system"; }
        try { localStorage.setItem(STORAGE_KEY, pref); } catch (e) { /* */ }
        applyTheme(pref);
    }

    /** Public: current preference as stored. */
    function get() { return stored(); }

    /** Public: current resolved theme ("light" or "dark"). */
    function current() { return resolve(stored()); }

    /* Apply immediately so the first paint matches. */
    applyTheme(stored());

    /* Follow the OS live when preference is "system". */
    try {
        if (!mql) { mql = window.matchMedia("(prefers-color-scheme: dark)"); }
        mql.addEventListener("change", function () {
            if (stored() === "system") { applyTheme("system"); }
        });
    } catch (e) { /* matchMedia or addEventListener not supported */ }

    window.GBTheme = {
        set: set,
        get: get,
        current: current,
        resolve: resolve
    };
})();
