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

    window.openMinModal = function(id) {
        var m = document.getElementById(id);
        if (m) m.hidden = false;
    };
    window.closeMinModal = function(id) {
        var m = document.getElementById(id);
        if (m) m.hidden = true;
    };
    document.body.addEventListener("click", function(e) {
        var backdrop = e.target.closest(".min-modal");
        if (backdrop && e.target === backdrop) backdrop.hidden = true;
    });

    function collectForm(form) {
        var data = {};
        Array.prototype.forEach.call(form.elements, function(el) {
            if (!el.name || el.disabled) return;
            if (el.type === "checkbox" || el.type === "radio") {
                if (el.checked) data[el.name] = el.value;
            } else if (el.name) {
                data[el.name] = el.value;
            }
        });
        return data;
    }

    function showMinuteResult(ok, msg) {
        var el = document.getElementById("min-content");
        if (el) {
            var note = document.createElement("div");
            note.className = "min-toast " + (ok ? "min-toast-ok" : "min-toast-err");
            note.textContent = msg;
            el.parentNode.insertBefore(note, el);
            setTimeout(function() { note.remove(); }, 4000);
        }
    }

    window.submitMinuteForm = function(event, endpoint) {
        event.preventDefault();
        var form = event.target;
        var data = collectForm(form);
        fetch(endpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data)
        })
            .then(function(r) { return r.json(); })
            .then(function(resp) {
                if (resp && resp.ok) {
                    showMinuteResult(true, "Salvo com sucesso.");
                    form.closest(".min-modal").hidden = true;
                    form.reset();
                    if (window.htmx) htmx.trigger("#min-content", "load");
                } else {
                    showMinuteResult(false, "Erro: " + ((resp && resp.error) || "falha"));
                }
            })
            .catch(function(err) {
                console.error(err);
                showMinuteResult(false, "Erro de rede.");
            });
        return false;
    };

    window.submitMinuteFormWithId = function(event, idFieldId, endpointPrefix) {
        event.preventDefault();
        var form = event.target;
        var idVal = document.getElementById(idFieldId).value.trim();
        if (!idVal) { showMinuteResult(false, "Informe o ID."); return false; }
        var data = collectForm(form);
        delete data.meeting_id;
        delete data.document_id;
        fetch(endpointPrefix + encodeURIComponent(idVal), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data)
        })
            .then(function(r) { return r.json(); })
            .then(function(resp) {
                if (resp && resp.ok) {
                    showMinuteResult(true, "Salvo com sucesso.");
                    form.closest(".min-modal").hidden = true;
                    form.reset();
                    if (window.htmx) htmx.trigger("#min-content", "load");
                } else {
                    showMinuteResult(false, "Erro: " + ((resp && resp.error) || "falha"));
                }
            })
            .catch(function(err) {
                console.error(err);
                showMinuteResult(false, "Erro de rede.");
            });
        return false;
    };
})();
