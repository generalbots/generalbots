"use strict";
/* App zoom: scales the whole interface, like the browser's own zoom.
 * 80 % to 150 % in steps of 10, stored per device, applied before first paint.
 * Inside the desktop app the webview's own zoom is used when the app grants
 * it; otherwise (and in a browser) CSS zoom on the root element.
 * CSS zoom also scales viewport units and the pixel values scripts write, so
 * the factor is published as --gb-zoom (stylesheets divide vh/vw by it) and as
 * GBZoom.factor() (scripts divide client coordinates by it before placing a
 * fixed element). Both are 1 when the webview zooms natively.
 * Emits "gb-zoom-change" on window. Exposes window.GBZoom.
 */
(function () {
    var KEY = "gb-app-zoom", MIN = 80, MAX = 150, STEP = 10, DEF = 100;
    var root = document.documentElement;
    var native = false;

    function clamp(value) {
        var n = Math.round(Number(value) / STEP) * STEP;
        if (!isFinite(n)) { return DEF; }
        return Math.min(MAX, Math.max(MIN, n));
    }

    function stored() {
        try {
            var v = localStorage.getItem(KEY);
            if (v !== null) { return clamp(parseInt(v, 10)); }
        } catch (e) { /* storage unavailable */ }
        return DEF;
    }

    function cssZoom(pct) {
        root.style.zoom = pct === DEF ? "" : String(pct / 100);
        if (pct === DEF) { root.style.removeProperty("--gb-zoom"); } else { root.style.setProperty("--gb-zoom", String(pct / 100)); }
    }

    function factor() { return native ? 1 : stored() / 100; }

    function webview() {
        var tauri = window.__TAURI__;
        var api = tauri && tauri.webview;
        var view = api && typeof api.getCurrentWebview === "function" ? api.getCurrentWebview() : null;
        return view && typeof view.setZoom === "function" ? view : null;
    }

    /* The CSS zoom is always set first so the page never paints unscaled;
       a granted webview zoom then takes over and the CSS one is removed. */
    function apply(pct) {
        if (!native) { cssZoom(pct); }
        var view = webview();
        if (!view) { return; }
        view.setZoom(pct / 100).then(function () {
            native = true;
            cssZoom(DEF);
        }, function () {
            native = false;
            cssZoom(pct);
        });
    }

    function set(value) {
        var pct = clamp(value);
        try { localStorage.setItem(KEY, String(pct)); } catch (e) { /* storage unavailable */ }
        apply(pct);
        window.dispatchEvent(new CustomEvent("gb-zoom-change", { detail: { value: pct } }));
        return pct;
    }

    function step(direction) {
        return set(stored() + direction * STEP);
    }

    function mode() { return native ? "webview" : "css"; }

    apply(stored());

    window.GBZoom = { get: stored, set: set, step: step, reset: function () { return set(DEF); },
        mode: mode, factor: factor, min: MIN, max: MAX, increment: STEP, def: DEF };
})();
