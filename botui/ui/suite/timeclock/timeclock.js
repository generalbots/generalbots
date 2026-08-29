if (window.GBAppLifecycle) GBAppLifecycle.begin("timeclock");
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

    window.openTcOtModal = function() {
        var m = document.getElementById("tc-ot-modal");
        if (m) m.hidden = false;
    };
    window.closeTcOtModal = function() {
        var m = document.getElementById("tc-ot-modal");
        if (m) m.hidden = true;
    };
    document.body.addEventListener("click", function(e) {
        var backdrop = e.target.closest(".tc-modal");
        if (backdrop && e.target === backdrop) backdrop.hidden = true;
    });

    window.submitTcOtForm = function(event) {
        event.preventDefault();
        var form = event.target;
        var data = {};
        Array.prototype.forEach.call(form.elements, function(el) {
            if (el.name && !el.disabled) data[el.name] = el.value;
        });
        fetch("/api/timeclock/forms/overtime", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data)
        })
            .then(function(r) { return r.json(); })
            .then(function(resp) {
                if (resp && resp.ok) {
                    form.closest(".tc-modal").hidden = true;
                    form.reset();
                    if (window.htmx) htmx.trigger("#tc-content", "load");
                } else {
                    alert("Erro: " + ((resp && resp.error) || "falha ao enviar"));
                }
            })
            .catch(function(err) {
                console.error(err);
                alert("Erro de rede.");
            });
        return false;
    };
})();
