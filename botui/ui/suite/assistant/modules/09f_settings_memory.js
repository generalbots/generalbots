"use strict";
/* Settings > Memória: what the assistant keeps about the person across
 * conversations. GET /api/user/memory lists it newest first, PUT adds
 * ({value}) or edits ({value, key}), DELETE /api/user/memory/:key removes one
 * and DELETE /api/user/memory removes all; /api/user/memory-settings switches
 * it on and off. Keys starting with "m-" are internal: only the value shows. */

(function () {
    var A = window.GBAssistant;
    var ENDPOINT = "/api/user/memory";
    function tr(pt, en) { return A.settingsText(pt, en); }
    function gt(key, en) { return window.gbT ? window.gbT(key, en) : en; }

    function toast(text, error) {
        var D = window.GBDrive;
        if (D && D.showToast) { D.showToast(text, error ? "error" : ""); }
        if (A.announce) { A.announce(text); }
    }
    function failed() { toast(tr("Não foi possível salvar. Tente de novo.", "Could not save. Try again."), true); }

    async function call(path, method, body) {
        var options = { method: method || "GET" };
        if (body) { options.body = JSON.stringify(body); options.headers = { "Content-Type": "application/json" }; }
        var response = await A.apiFetch(path, options);
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json().catch(function () { return {}; });
    }

    function when(iso) {
        var d = iso ? new Date(iso) : null;
        if (!d || isNaN(d.getTime())) { return ""; }
        return gbFormatDate(d, { locale: A.locale, day: "2-digit", month: "short", year: d.getFullYear() === new Date().getFullYear() ? undefined : "numeric" });
    }

    A.renderMemory = async function (panel) {
        var esc = A.settingsEsc;
        var data = await call(ENDPOINT).catch(function () {
            throw new Error(tr("Não foi possível carregar a memória.", "Could not load the memory."));
        });
        var items = Array.isArray(data.memories) ? data.memories : [];
        var enabled = data.enabled !== false;
        panel.innerHTML = A.settingsHeading(tr("Configurações", "Settings"), tr("Memória", "Memory"),
            tr("O assistente lembra do que você pedir para guardar e usa isso em todas as conversas.",
                "The assistant remembers what you ask it to keep and uses it in every conversation.")) +
            '<section class="gb-sp-card gb-sp-clip"><button type="button" class="gb-sp-switchrow gb-mem-switch" role="switch" id="gbMemEnabled" aria-checked="' +
            enabled + '" aria-describedby="gbMemHint"><span>' + esc(tr("Usar memória", "Use memory")) + '</span><span class="gb-sp-state" id="gbMemState"></span>' +
            '<i class="gb-sp-toggle" aria-hidden="true"></i></button><p class="gb-mem-hint" id="gbMemHint">' +
            esc(tr("Desligada, o assistente não lembra nem guarda nada novo.", "When off, the assistant neither remembers nor saves anything new.")) +
            '</p></section><section class="gb-sp-card gb-sp-clip"><form class="gb-mem-add" id="gbMemAdd"><label class="gb-mem-sr" for="gbMemNew">' +
            esc(tr("Algo que o assistente deve lembrar", "Something the assistant should remember")) + '</label><input class="gb-sp-input" id="gbMemNew" maxlength="1000" autocomplete="off" placeholder="' +
            esc(tr("Algo que o assistente deve lembrar", "Something the assistant should remember")) + '"><button type="submit" class="gb-sp-btn gb-sp-btn-pri" disabled>' +
            A.settingsIcon("M12 5v14M5 12h14", 14, 2) + esc(tr("Adicionar", "Add")) + '</button></form><div id="gbMemBody"></div></section>';

        var toggle = panel.querySelector("#gbMemEnabled");
        var body = panel.querySelector("#gbMemBody");
        var form = panel.querySelector("#gbMemAdd");
        var input = form.querySelector("input");
        var add = form.querySelector("[type=submit]");

        function paintToggle() {
            panel.querySelector("#gbMemState").textContent = toggle.getAttribute("aria-checked") === "true" ? tr("Ligada", "On") : tr("Desligada", "Off");
        }
        function shown(item) { return String(item.key || "").indexOf("m-") === 0 ? "" : String(item.key || ""); }
        function row(item, index) {
            var label = String(item.value || "");
            var short = label.length > 60 ? label.slice(0, 57) + "…" : label;
            var key = shown(item);
            return '<li class="gb-mem-row" data-index="' + index + '"><div class="gb-mem-text">' + (key ? '<span class="gb-sp-label">' + esc(key) + '</span>' : '') +
                '<p>' + esc(label) + '</p></div><time class="gb-mem-when" datetime="' + esc(item.updated_at || "") + '">' + esc(when(item.updated_at)) + '</time>' +
                '<span class="gb-mem-acts"><button type="button" class="gb-mem-act" data-act="edit" aria-label="' + esc(tr("Editar", "Edit") + ": " + short) + '">' +
                esc(tr("Editar", "Edit")) + '</button><button type="button" class="gb-mem-act gb-mem-del" data-act="delete" aria-label="' +
                esc(gt("memory-delete", "Delete") + ": " + short) + '">' + esc(gt("memory-delete", "Delete")) + '</button></span></li>';
        }
        function paint(focusKey) {
            if (!items.length) {
                body.innerHTML = '<p class="gb-mem-empty">' + esc(tr("Nada guardado ainda. Diga, por exemplo, “Lembre que prefiro respostas curtas”.",
                    "Nothing saved yet. Say, for example, “Remember that I prefer short answers”.")) + '</p>';
                return;
            }
            body.innerHTML = '<ul class="gb-mem-list" aria-label="' + esc(tr("Memória", "Memory")) + '">' + items.map(row).join("") + '</ul>' +
                '<div class="gb-sp-foot gb-mem-foot"><button type="button" class="gb-sp-btn gb-sp-btn-danger" id="gbMemClear">' +
                A.settingsIcon("M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14", 14, 1.8) + esc(tr("Apagar tudo", "Delete all")) + '</button></div>';
            if (focusKey) {
                var at = items.findIndex(function (item) { return item.key === focusKey; });
                var target = at >= 0 ? body.querySelector('[data-index="' + at + '"] [data-act="edit"]') : null;
                if (target) { target.focus(); }
            }
        }
        async function reload(focusKey) {
            var fresh = await call(ENDPOINT);
            items = Array.isArray(fresh.memories) ? fresh.memories : [];
            if (panel.isConnected) { paint(focusKey); }
        }
        async function removeAt(index) {
            var item = items[index];
            if (!item) { return; }
            try {
                await call(ENDPOINT + "/" + encodeURIComponent(item.key), "DELETE");
                items.splice(index, 1);
                paint();
                var next = body.querySelector('[data-index="' + Math.min(index, items.length - 1) + '"] [data-act="edit"]');
                (next || input).focus();
                toast(tr("Memória excluída", "Memory deleted"));
            } catch (e) { failed(); }
        }
        function edit(index) {
            var item = items[index], li = body.querySelector('[data-index="' + index + '"]');
            if (!item || !li) { return; }
            var id = "gbMemEdit" + index;
            li.classList.add("gb-mem-editing");
            li.innerHTML = '<form class="gb-mem-edit"><label class="gb-mem-sr" for="' + id + '">' + esc(tr("Editar", "Edit")) + '</label><input class="gb-sp-input" id="' + id +
                '" maxlength="1000" autocomplete="off"><button type="button" class="gb-sp-btn" data-act="cancel">' + esc(gt("cancel", "Cancel")) +
                '</button><button type="submit" class="gb-sp-btn gb-sp-btn-pri">' + esc(A.t("settings.save", gt("save", "Save"))) + '</button></form>';
            var box = li.querySelector("input"), editor = li.querySelector("form");
            box.value = item.value || "";
            box.focus();
            box.select();
            function back() { paint(item.key); }
            li.querySelector('[data-act="cancel"]').onclick = back;
            box.addEventListener("keydown", function (event) { if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); back(); } });
            editor.onsubmit = async function (event) {
                event.preventDefault();
                var value = box.value.trim();
                if (!value) { box.focus(); return; }
                if (value === item.value) { back(); return; }
                editor.inert = true;
                try {
                    await call(ENDPOINT, "PUT", { value: value, key: item.key });
                    await reload(item.key);
                } catch (e) {
                    editor.inert = false;
                    box.focus();
                    failed();
                }
            };
        }
        function confirmClear() {
            var foot = body.querySelector(".gb-mem-foot");
            foot.classList.add("gb-mem-confirm");
            foot.innerHTML = '<p class="gb-mem-ask" id="gbMemAsk">' + esc(tr("Apagar todas as memórias? Isso não pode ser desfeito.",
                "Delete all memories? This cannot be undone.")) + '</p><button type="button" class="gb-sp-btn" data-act="keep">' + esc(gt("cancel", "Cancel")) +
                '</button><button type="button" class="gb-sp-btn gb-sp-btn-danger gb-mem-clear-yes" data-act="clear" aria-describedby="gbMemAsk">' +
                esc(tr("Apagar tudo", "Delete all")) + '</button>';
            foot.querySelector('[data-act="keep"]').onclick = function () { paint(); body.querySelector("#gbMemClear").focus(); };
            foot.querySelector('[data-act="clear"]').onclick = async function () {
                foot.inert = true;
                try {
                    await call(ENDPOINT, "DELETE");
                    items = [];
                    paint();
                    input.focus();
                    toast(tr("Memória apagada", "Memory cleared"));
                } catch (e) { foot.inert = false; failed(); }
            };
            foot.addEventListener("keydown", function (event) {
                if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); foot.querySelector('[data-act="keep"]').click(); }
            });
            foot.querySelector('[data-act="keep"]').focus();
        }

        body.addEventListener("click", function (event) {
            var button = event.target.closest("button");
            if (!button) { return; }
            if (button.id === "gbMemClear") { confirmClear(); return; }
            var li = button.closest(".gb-mem-row:not(.gb-mem-editing)");
            if (!li) { return; }
            var index = Number(li.dataset.index);
            if (button.dataset.act === "edit") { edit(index); }
            else if (button.dataset.act === "delete") { removeAt(index); }
        });
        input.addEventListener("input", function () { add.disabled = !input.value.trim(); });
        form.onsubmit = async function (event) {
            event.preventDefault();
            var value = input.value.trim();
            if (!value) { return; }
            form.inert = true;
            try {
                await call(ENDPOINT, "PUT", { value: value });
                input.value = "";
                add.disabled = true;
                await reload();
            } catch (e) { failed(); }
            finally { form.inert = false; input.focus(); }
        };
        toggle.onclick = async function () {
            var next = toggle.getAttribute("aria-checked") !== "true";
            toggle.setAttribute("aria-checked", String(next));
            paintToggle();
            toggle.disabled = true;
            try { await call("/api/user/memory-settings", "PUT", { enabled: next }); }
            catch (e) {
                toggle.setAttribute("aria-checked", String(!next));
                paintToggle();
                failed();
            } finally { toggle.disabled = false; toggle.focus(); }
        };
        paintToggle();
        paint();
    };
})();
