"use strict";
(function() {
    function getTabs() {
        return Array.prototype.slice.call(document.querySelectorAll('#tax-tabs [data-tab-trigger]'));
    }
    function activateTab(btn, focus) {
        var tabs = getTabs();
        tabs.forEach(function(b) {
            var isActive = b === btn;
            b.classList.toggle("active", isActive);
            b.setAttribute("aria-selected", isActive ? "true" : "false");
            b.tabIndex = isActive ? 0 : -1;
        });
        var panel = btn.getAttribute("data-panel");
        var main = document.getElementById("gb-brazil-content");
        var validators = document.getElementById("gb-validators");
        if (panel === "validators") {
            if (main) main.hidden = true;
            if (validators) {
                validators.hidden = false;
                validators.setAttribute("aria-labelledby", btn.id || "tax-tabs");
            }
        } else {
            if (main) {
                main.hidden = false;
                if (btn.id) main.setAttribute("aria-labelledby", btn.id);
            }
            if (validators) validators.hidden = true;
        }
        if (focus && btn) btn.focus();
    }
    document.body.addEventListener("click", function(e) {
        var t = e.target.closest("#tax-tabs [data-tab-trigger]");
        if (t) activateTab(t, false);
    });
    document.body.addEventListener("keydown", function(e) {
        var current = e.target.closest("#tax-tabs [data-tab-trigger]");
        if (!current) return;
        var tabs = getTabs();
        var idx = tabs.indexOf(current);
        if (idx === -1) return;
        var next = null;
        if (e.key === "ArrowRight") next = tabs[(idx + 1) % tabs.length];
        else if (e.key === "ArrowLeft") next = tabs[(idx - 1 + tabs.length) % tabs.length];
        else if (e.key === "Home") next = tabs[0];
        else if (e.key === "End") next = tabs[tabs.length - 1];
        if (next) {
            e.preventDefault();
            next.click();
            activateTab(next, true);
        }
    });
    // ensure initial tabindex state
    (function init() {
        var tabs = getTabs();
        if (!tabs.length) return;
        var active = tabs.filter(function(b){ return b.classList.contains('active') || b.getAttribute('aria-selected') === 'true'; })[0] || tabs[0];
        activateTab(active, false);
    })();
})();
