/* #2006 — accept a drawing streamed in chat as the canvas background.
 *
 * The chat live preview dispatches `gb:canvas-set-background` with the SVG it
 * has drawn so far ("Open in Canvas"). The canvas background layer already
 * exists for project.draw / drive images (`#canvas-bg-svg`, z-index 0 under the
 * interactive canvas), so the handoff is the same path: put the markup in the
 * layer and hide the opaque canvas fill so the drawing shows through.
 */
(function () {
    "use strict";

    function setBackground(svg, title) {
        var bg = document.getElementById("canvas-bg-svg");
        if (!bg) return false;
        bg.innerHTML = svg;
        var app = window._canvasApp;
        if (app && typeof app.setCanvasBackgroundHidden === "function") {
            app.setCanvasBackgroundHidden();
        }
        var name = document.getElementById("canvas-name");
        if (name && title) name.value = title;
        return true;
    }

    function isMounted() {
        return !!document.getElementById("canvas-bg-svg");
    }

    // The desktop window can be closed and reopened: `window` outlives the
    // window content, so the listener stays bound to this document exactly
    // once and always resolves the *current* background layer.
    if (!document.__gbLiveCanvasListening) {
        document.__gbLiveCanvasListening = true;
        document.addEventListener("gb:canvas-set-background", function (ev) {
            var detail = (ev && ev.detail) || {};
            var svg = typeof detail.svg === "string" ? detail.svg : "";
            if (!svg || !/<svg[\s>]/i.test(svg)) return;
            if (!setBackground(svg, detail.title)) return;
            if (window.GBToasts && typeof window.GBToasts.show === "function") {
                window.GBToasts.show("Canvas", "Drawing loaded from chat", "success");
            }
            // The canvas may live in another desktop window that is not focused.
            if (typeof detail.focus === "function") {
                try { detail.focus(); } catch (e) { /* optional callback */ }
            }
        });
    }

    window.GBLiveCanvas = { setBackground: setBackground, isMounted: isMounted };
})();
