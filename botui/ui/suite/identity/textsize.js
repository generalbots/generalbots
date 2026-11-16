"use strict";
/* Text-size manager — scales the root font size.
 * Sizes: small (14px), default (16px), large (18px), xlarge (20px).
 * Stored in localStorage, applied before first paint. Ctrl+= / Ctrl+- / Ctrl+0.
 * Exposes window.GBTextSize.
 */
(function () {
    var STORAGE_KEY = "gb-text-size";
    var STEPS = ["small", "default", "large", "xlarge"];
    var PX = { small: 14, "default": 16, large: 18, xlarge: 20 };

    function stored() {
        try {
            var v = localStorage.getItem(STORAGE_KEY);
            if (v && PX[v] !== undefined) { return v; }
        } catch (e) { /* */ }
        return "default";
    }

    function applySize(name) {
        if (!PX[name]) { name = "default"; }
        var px = PX[name];
        var scale = px / 14; /* base design is 14px */
        document.documentElement.style.setProperty("--id-scale", String(scale));
        document.documentElement.style.setProperty("--id-text-base", px + "px");
        document.documentElement.style.setProperty("--id-text-md", (px + 1) + "px");
        document.documentElement.style.setProperty("--id-text-lg", (px + 2) + "px");
        document.documentElement.style.setProperty("--id-text-xl", (px + 3) + "px");
        document.documentElement.style.setProperty("--id-text-sm", Math.max(11, px - 2) + "px");
        document.documentElement.style.setProperty("--id-text-xs", Math.max(10, px - 4) + "px");
    }

    function set(name) {
        if (!PX[name]) { name = "default"; }
        try { localStorage.setItem(STORAGE_KEY, name); } catch (e) { /* */ }
        applySize(name);
    }

    function get() { return stored(); }

    function step(direction) {
        var cur = stored();
        var idx = STEPS.indexOf(cur);
        if (idx === -1) { idx = 1; }
        var next = idx + direction;
        if (next < 0) { next = 0; }
        if (next >= STEPS.length) { next = STEPS.length - 1; }
        set(STEPS[next]);
        return STEPS[next];
    }

    /* Apply immediately */
    applySize(stored());

    /* Keyboard shortcuts: Ctrl+= (increase), Ctrl+- (decrease), Ctrl+0 (reset) */
    document.addEventListener("keydown", function (e) {
        if (!(e.ctrlKey || e.metaKey)) { return; }
        if (e.key === "=" || e.key === "+") {
            e.preventDefault();
            step(1);
        } else if (e.key === "-") {
            e.preventDefault();
            step(-1);
        } else if (e.key === "0") {
            e.preventDefault();
            set("default");
        }
    });

    window.GBTextSize = {
        set: set,
        get: get,
        step: step,
        steps: STEPS,
        px: PX
    };
})();
