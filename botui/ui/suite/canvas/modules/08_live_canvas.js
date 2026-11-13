/* #2006/#2155 — accept a drawing streamed/sent from chat. Prefer importing as
 * editable elements when possible (flowcharts); fall back to the background
 * layer so the drawing is always visible.
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

    function applyDrawing(svg, title) {
        if (!document.getElementById("canvas-bg-svg")) return false;
        var name = document.getElementById("canvas-name");
        if (name && title) name.value = title;
        if (window.GBCanvasSvgImport && typeof window.GBCanvasSvgImport.importAsElements === "function") {
            window.GBCanvasSvgImport.importAsElements(svg, {
                app: window._canvasApp,
                bg: document.getElementById("canvas-bg-svg"),
                setStatus: function () {},
            });
            return true;
        }
        return setBackground(svg, title);
    }

    function isMounted() {
        return !!document.getElementById("canvas-bg-svg");
    }

    if (!document.__gbLiveCanvasListening) {
        document.__gbLiveCanvasListening = true;
        document.addEventListener("gb:canvas-set-background", function (ev) {
            var detail = (ev && ev.detail) || {};
            var svg = typeof detail.svg === "string" ? detail.svg : "";
            if (!svg || !/<svg[\s>]/i.test(svg)) return;
            if (!applyDrawing(svg, detail.title)) return;
            if (window.GBToasts && typeof window.GBToasts.show === "function") {
                window.GBToasts.show("Canvas", "Drawing loaded from chat", "success");
            }
            if (typeof detail.focus === "function") {
                try { detail.focus(); } catch (e) { /* optional callback */ }
            }
        });
    }

    window.GBLiveCanvas = { setBackground: setBackground, isMounted: isMounted };
})();
