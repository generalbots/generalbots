/* slides diagrams — toolbar hook and lifecycle (#2488). Local only. */
"use strict";

(function () {
    "use strict";

    var GB = window.GBSlidesDiagrams;
    var TRIGGER_ID = "slides-diagrams-btn";

    function build() {
        var toolbar = sd$(".slides-toolbar-row-main .toolbar-right") ||
            sd$(".slides-toolbar-row-main") || sd$(".toolbar-right");
        if (!toolbar || document.getElementById(TRIGGER_ID)) { return; }
        var trigger = sdEl("button", "btn-icon slides-diagrams-btn",
            sdT("slides-diagrams-open", "Diagrams"), "slides-diagrams-open");
        trigger.type = "button";
        trigger.id = TRIGGER_ID;
        trigger.setAttribute("data-i18n-aria-label", "slides-diagrams-open-aria");
        trigger.setAttribute("aria-label", sdT("slides-diagrams-open-aria", "Diagram editor (local only)"));
        trigger.addEventListener("click", function (event) {
            event.stopPropagation();
            if (window.GBSlidesDiagramsDialog) { window.GBSlidesDiagramsDialog.open(); }
        });
        toolbar.insertBefore(trigger, toolbar.firstChild);
    }

    function refresh() {
        if (window.GBSlidesDiagramsCanvas) { window.GBSlidesDiagramsCanvas.refresh(); }
    }

    function onKeys(event) {
        if (!window.GBSlidesDiagramsDialog || !window.GBSlidesDiagramsDialog.isOpen()) { return; }
        var key = String(event.key || "").toLowerCase();
        if (!(event.ctrlKey || event.metaKey) || (key !== "z" && key !== "y")) { return; }
        event.preventDefault();
        event.stopPropagation();
        var redo = key === "y" || event.shiftKey;
        if (redo) { GB.redo(); } else { GB.undo(); }
        if (window.GBSlidesDiagramsDialog.refreshFromState) {
            window.GBSlidesDiagramsDialog.refreshFromState();
        }
        refresh();
    }

    function init() {
        GB.init();
        GB.on("change", refresh);
        build();
        refresh();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
    window.addEventListener("keydown", onKeys, true);
    document.addEventListener("htmx:afterSwap", function (event) {
        if (event.target && event.target.id !== "slides-content") { return; }
        if (window.GBSlidesDiagramsCanvas) { window.GBSlidesDiagramsCanvas.refresh(); }
    });
}());
