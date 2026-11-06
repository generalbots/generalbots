(function () {
    "use strict";

    var MAX_FAST_ATTEMPTS = 6; // ~9s of eager retrying every 1.5s
    var attempts = 0;
    var slow = false;

    // Capture the expected stylesheet URLs while the <link>s are still in the
    // DOM so a missing/broken link can be recreated with the same version.
    var EXPECTED_CSS = Array.from(document.querySelectorAll('link[href*="/suite/canvas/canvas.p"]'))
        .map(function (l) { return l.href; });

    function canvasRootGone() {
        return !document.getElementById("canvas-app-root");
    }

    function canvasFullyMounted() {
        return !!document.getElementById("main-canvas")
            && !!document.querySelector(".canvas-toolbar")
            && !!document.querySelector(".canvas-main")
            && !!document.querySelector(".properties-panel")
            && !!document.getElementById("export-modal");
    }

    function cssLoaded() {
        var links = Array.from(document.querySelectorAll('link[href*="canvas.p"]'));
        return links.length > 0 && links.every(function (l) { return l.sheet; });
    }

    // A placeholder that never got swapped (htmx failure) is still an empty
    // [hx-get] element under the root. Re-processing it re-fires its
    // hx-trigger="load" and re-issues the request (htmx.trigger does NOT).
    function reloadMissingPartials() {
        if (canvasFullyMounted()) return;
        Array.from(document.querySelectorAll("#canvas-app-root > [hx-get]")).forEach(function (el) {
            if (window.htmx) htmx.process(el);
        });
    }

    // Broken stylesheets are replaced; links that are missing entirely
    // (removed or never injected) are recreated from the captured URLs.
    function reloadMissingCss() {
        if (cssLoaded()) return;
        Array.from(document.querySelectorAll('link[href*="canvas.p"]')).forEach(function (link) {
            if (link.sheet) return;
            var fresh = document.createElement("link");
            fresh.rel = "stylesheet";
            fresh.href = link.href;
            if (link.parentNode) link.parentNode.replaceChild(fresh, link);
        });
        EXPECTED_CSS.forEach(function (href) {
            var exists = Array.from(document.querySelectorAll('link[href*="canvas.p"]'))
                .some(function (l) { return l.href === href; });
            if (!exists) {
                var fresh = document.createElement("link");
                fresh.rel = "stylesheet";
                fresh.href = href;
                document.body.appendChild(fresh);
            }
        });
    }

    // Keeps watching while the app window is open: broken → retry eagerly
    // (1.5s), then settle into a slow watch (5s) so a window opened during a
    // server blip still repairs itself once the server returns. Healthy →
    // stay on the slow watch so late breakage is also caught. Stops when the
    // window is closed. All actions are no-ops when everything is present.
    function heal() {
        if (canvasRootGone()) return;
        if (canvasFullyMounted() && cssLoaded()) {
            setTimeout(heal, 5000);
            return;
        }
        attempts++;
        if (!slow && attempts >= MAX_FAST_ATTEMPTS) slow = true;
        reloadMissingPartials();
        reloadMissingCss();
        setTimeout(heal, slow ? 5000 : 1500);
    }

    setTimeout(heal, 1000);
})();
