if (window.GBAppLifecycle) GBAppLifecycle.begin("minutes");
"use strict";

(function() {
    function activateTab(btn) {
        document.querySelectorAll("[data-tab-trigger]").forEach(function(b) {
            b.classList.toggle("active", b === btn);
        });
    }
    document.body.addEventListener("click", function(e) {
        var t = e.target.closest("[data-tab-trigger]");
        if (t) activateTab(t);
    });
})();
