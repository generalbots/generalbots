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

    function tick() {
        var c = document.getElementById("tc-clock");
        var d = document.getElementById("tc-date");
        var now = new Date();
        if (c) {
            var h = String(now.getHours()).padStart(2, "0");
            var m = String(now.getMinutes()).padStart(2, "0");
            var s = String(now.getSeconds()).padStart(2, "0");
            c.textContent = h + ":" + m + ":" + s;
        }
        if (d) {
            var opts = { weekday: "long", year: "numeric", month: "long", day: "numeric" };
            d.textContent = now.toLocaleDateString("pt-BR", opts);
        }
    }
    tick();
    setInterval(tick, 1000);
})();
