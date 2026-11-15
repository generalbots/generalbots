"use strict";
(function() {
    function activateTab(btn) {
        document.querySelectorAll("[data-tab-trigger]").forEach(function(b) {
            var isActive = b === btn;
            b.classList.toggle("active", isActive);
            b.setAttribute("aria-selected", isActive ? "true" : "false");
        });
        var panel = btn.getAttribute("data-panel");
        var main = document.getElementById("gb-brazil-content");
        var validators = document.getElementById("gb-validators");
        if (panel === "validators") {
            if (main) main.hidden = true;
            if (validators) validators.hidden = false;
        } else {
            if (main) main.hidden = false;
            if (validators) validators.hidden = true;
        }
    }
    document.body.addEventListener("click", function(e) {
        var t = e.target.closest("[data-tab-trigger]");
        if (t) activateTab(t);
    });
})();
