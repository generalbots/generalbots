"use strict";

(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon;
    var drawer;
    var opener;

    function tr(pt, en) { return (A.locale || "").startsWith("pt") ? pt : en; }

    function syncFast() {
        var available = Boolean(A.productConfig && A.productConfig.fast_available);
        document.querySelectorAll(".gb-fast-toggle").forEach(function (button) {
            button.disabled = !available;
            button.title = available ? tr("Respostas mais rápidas e curtas", "Faster, shorter answers")
                : tr("Fast indisponível neste motor", "Fast is unavailable on this engine");
            button.setAttribute("aria-pressed", String(A.fastOn()));
        });
    }

    A.fastOn = function () {
        return Boolean(A.productConfig && A.productConfig.fast_available && A.fastEnabled);
    };

    function node(tag, cls, text) {
        var item = document.createElement(tag);
        if (cls) { item.className = cls; }
        if (text) { item.textContent = text; }
        return item;
    }

    function section(title) { return node("h3", "gb-engine-label", title); }

    function makeButton(cls, label, icon, action) {
        var button = node("button", cls);
        button.type = "button";
        button.setAttribute("aria-label", label);
        if (icon) { button.innerHTML = I(icon, 14); }
        button.addEventListener("click", action);
        return button;
    }

    function isLocal() {
        var value = A.selectedModel || A.defaultModel || "";
        return !/^(sub|acct):/.test(value) && A.engineMode !== "free" && (A.destination || {}).scope === "machine";
    }

    function engineInfo() {
        var value = A.selectedModel || A.defaultModel || "";
        var dest = A.destination || {};
        if (value.startsWith("sub:")) { return tr("Assinatura", "Subscription"); }
        if (value.startsWith("acct:")) { return tr("Chave API", "API key"); }
        if (A.engineMode === "free") { return tr("Gratuito", "Free"); }
        if (dest.scope === "machine") { return tr("Local", "Local"); }
        if (dest.scope === "network") { return tr("Rede", "Network"); }
        if (dest.scope === "external") { return tr("Externo", "External"); }
        return "";
    }

    function destinationInfo() {
        var value = A.selectedModel || A.defaultModel || "";
        var dest = A.destination || {};
        if (value.startsWith("sub:") || value.startsWith("acct:")) {
            return tr("Vai ao provedor selecionado.", "Sent to the selected provider.");
        }
        if (dest.scope === "machine") { return tr("Ficam neste computador.", "Stays on this computer."); }
        if (dest.host && (dest.scope === "network" || dest.scope === "external")) {
            return tr("Vai a ", "Sent to ") + dest.host;
        }
        return tr("Destino indisponível.", "Destination unavailable.");
    }

    function layoutComposer(select) {
        var row = select.parentElement;
        if (row.closest(".gb-motor")) { return; }
        var composer = row.parentElement;
        var input = composer.querySelector("textarea");
        var fields = node("div", "gb-composer-fields");
        composer.insertBefore(fields, composer.firstChild);
        var attachments = composer.querySelector(".gb-file-chips");
        if (attachments) { fields.appendChild(attachments); }
        fields.append(input, row);
        var motor = node("section", "gb-motor");
        var heading = section(tr("Motor", "Engine"));
        heading.appendChild(node("span", "gb-engine-mode"));
        var model = node("div", "gb-model-anchor");
        model.append(select, row.querySelector(".gb-model-btn"));
        var usage = makeButton("gb-usage-trigger", tr("Ver uso e limites", "View usage and limits"), "", function () { toggleDrawer(usage); });
        usage.setAttribute("aria-controls", "gbQuotaDrawer");
        usage.setAttribute("aria-expanded", "false");
        usage.append(node("span", "gb-usage-kind"), node("span", "gb-usage-bar"), node("span", "gb-usage-pct"));
        usage.querySelector(".gb-usage-bar").appendChild(node("span"));
        usage.insertAdjacentHTML("beforeend", '<svg class="gb-usage-chevron" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9 6l6 6-6 6"/></svg>');
        var toggles = node("div", "gb-engine-toggles");
        var fast = makeButton("gb-fast-toggle gb-engine-toggle", "Fast", "", function () {
            A.fastEnabled = !A.fastEnabled;
            syncFast();
        });
        fast.append(node("span", "", "Fast"), node("span", "gb-toggle-track"));
        toggles.appendChild(fast);
        motor.append(heading, model, usage, toggles);
        composer.appendChild(motor);
        var send = row.querySelector(".gb-send-btn");
        send.innerHTML = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14M13 6l6 6-6 6"/></svg>';
        var drive = makeButton("gb-btn-icon gb-composer-drive", tr("Usar arquivo do Drive", "Use a Drive file"), "folder", function () {
            A.switchView("drive");
        });
        drive.title = drive.getAttribute("aria-label");
        if ((A.compiledFeatures || []).includes("drive")) { row.insertBefore(drive, send); }
        row.insertBefore(node("span", "gb-composer-keyhint", tr("Enter envia · Shift+Enter quebra linha", "Enter sends · Shift+Enter adds a line")), send);
    }

    /* The drawer stops above the composer so the engine panel stays usable. */
    function placeDrawer() {
        if (!drawer || drawer.hidden) { return; }
        var chat = drawer.parentElement;
        var dock = document.getElementById("gbComposerDock");
        /* In the empty state the composer sits mid-page, outside the dock. */
        var below = dock && dock.offsetParent ? dock : Array.from(document.querySelectorAll(".gb-composer-inner, .gb-composer-card"))
            .filter(function (item) { return item.offsetParent; })[0];
        var gap = 0;
        if (chat && below) {
            var area = below.getBoundingClientRect();
            if (area.right > drawer.getBoundingClientRect().left) {
                var zf = window.GBZoom && GBZoom.factor ? GBZoom.factor() : 1;
                gap = Math.max(0, Math.round((chat.getBoundingClientRect().bottom - area.top) / zf));
            }
        }
        drawer.style.setProperty("--gb-drawer-bottom", gap + "px");
    }
    window.addEventListener("resize", placeDrawer);

    function setDrawer(open, returnFocus) {
        drawer.hidden = !open;
        placeDrawer();
        document.querySelectorAll(".gb-usage-trigger").forEach(function (button) {
            button.setAttribute("aria-expanded", String(open));
        });
        if (!open && returnFocus && opener) { opener.focus(); }
    }

    function toggleDrawer(button) {
        if (!drawer.hidden) { setDrawer(false, true); return; }
        opener = button;
        setDrawer(true);
        drawer.querySelector("button").focus();
        renderDrawer();
    }

    function ensureDrawer() {
        if (drawer) { return; }
        var chat = document.getElementById("gbChatView");
        if (!chat) { return; }
        drawer = node("aside", "gb-quota-drawer");
        drawer.id = "gbQuotaDrawer";
        drawer.hidden = true;
        var title = section(tr("Uso e limites", "Usage and limits"));
        title.id = "gbQuotaTitle";
        drawer.setAttribute("aria-labelledby", title.id);
        title.appendChild(makeButton("gb-btn-icon", tr("Fechar", "Close"), "x", function () { setDrawer(false, true); }));
        var body = node("div", "gb-quota-body");
        body.setAttribute("aria-live", "polite");
        drawer.append(title, body);
        chat.appendChild(drawer);
        drawer.addEventListener("keydown", function (event) {
            if (event.key === "Escape") {
                event.stopPropagation();
                setDrawer(false, true);
            }
        });
    }

    function label(text) { return node("div", "gb-quota-label", text); }

    function limitEngine(model) {
        return A.currentLimitEngine && A.cachedLimits && A.cachedLimits() ? A.currentLimitEngine(model, isLocal()) : null;
    }

    /* The bars /api/llm/limits reports for the current engine; without
       data the drawer says so and shows no number. */
    function fillLimits(box, model) {
        var engine = limitEngine(model);
        var bars = engine ? A.limitBars(engine) : [];
        var open = isLocal() || (engine && (engine.unlimited || engine.kind === "local"));
        if (open && !bars.length) {
            box.replaceChildren(node("p", "gb-quota-note", tr("Não há cota nem custo. O limite é a memória deste computador.", "No quota and no cost. The limit is this computer's memory.")));
            return;
        }
        if (!bars.length) {
            box.replaceChildren(node("p", "gb-quota-note", tr("Este motor não informa limites nem renovação. Confira com o provedor.", "This engine does not report limits or reset times. Check with the provider.")));
            return;
        }
        var rows = bars.map(function (b) {
            var row = node("div", "gb-quota-lim");
            var main = node("div", "gb-quota-lim-in");
            var top = node("div", "gb-quota-lim-top");
            var pct = node("b", "", b.pct + "%");
            pct.style.color = b.color;
            top.append(node("span", "", b.label), pct);
            var track = node("div", "gb-quota-track");
            var fill = node("i");
            fill.style.width = b.pct + "%";
            fill.style.background = b.color;
            track.appendChild(fill);
            var foot = node("div", "gb-quota-lim-foot");
            foot.append(node("span", "", b.value), node("span", "", b.reset));
            main.append(top, track, foot);
            row.append(node("span", "gb-quota-gutter"), main);
            return row;
        });
        var seen = A.limitSeen(engine);
        if (seen) { rows.push(node("p", "gb-quota-note gb-quota-seen", seen)); }
        box.replaceChildren.apply(box, rows);
    }

    function renderDrawer() {
        var body = drawer.querySelector(".gb-quota-body");
        var model = A.selectedModel || A.defaultModel || "";
        var head = node("div", "gb-quota-head");
        var kindRow = node("div", "gb-quota-kindrow");
        kindRow.append(node("span", "gb-quota-kind", engineInfo() || tr("Motor", "Engine")), node("span", "gb-quota-plan", destinationInfo()));
        head.append(kindRow, node("p", "gb-quota-model", A.shortModelName(model) || tr("Sem modelo ativo", "No active model")));
        var limits = node("div", "gb-quota-section");
        fillLimits(limits, model);
        if (A.llmLimits) {
            A.llmLimits().then(function () { if (limits.isConnected) { fillLimits(limits, model); } paintUsageLine(); });
        }
        var report = node("div", "gb-quota-report");
        report.appendChild(node("p", "gb-quota-note", tr("Carregando…", "Loading…")));
        body.replaceChildren(head, label(tr("Limites", "Limits")), limits, label(tr("Uso recente · 30 dias", "Recent usage · 30 days")), report);
        var footer = drawer.querySelector(".gb-quota-foot");
        if (!footer) {
            footer = makeButton("gb-quota-foot", tr("Ajustar limites na Oficina", "Adjust limits in Settings"), "gear", function () {
                setDrawer(false);
                A.switchView("settings");
                setTimeout(function () {
                    var tab = document.querySelector('.gb-set-tab[data-section="limits"]');
                    if (tab) { tab.click(); }
                }, 80);
            });
            footer.appendChild(node("span", "", tr("Ajustar limites na Oficina", "Adjust limits in Settings")));
            drawer.appendChild(footer);
        }
        A.apiFetch("/api/user/usage/summary?days=30").then(function (response) {
            if (!response.ok) { throw new Error("usage"); }
            return response.json();
        }).then(function (data) { renderReport(report, data); }).catch(function () {
            report.replaceChildren(node("p", "gb-quota-note", tr("O uso está indisponível.", "Usage is unavailable.")));
        });
    }

    /* /api/user/usage/summary: { days, models:{…}, per_day, totals:{…},
       pricing_available, engines:[{engine,tokens,messages,cost,providers:[…]}] }.
       When engines is present and non-empty, group by engine type then provider. */
    var ENGINE_NAMES = {
        "pt-BR": { subscription: "Agente por assinatura", api_key: "Chave própria", local: "Este computador", free: "Gratuito", unknown: "Sem registro" },
        en: { subscription: "Subscription agent", api_key: "Own key", local: "This computer", free: "Free", unknown: "Not recorded" }
    };
    function engineName(key) { var m = (A.locale || "").startsWith("pt") ? ENGINE_NAMES["pt-BR"] : ENGINE_NAMES.en; return m[key] || key; }
    function providerName(p, engine) {
        if (p.label) { return p.label; }
        if (engine === "local") { return tr("Este computador", "This computer"); }
        return p.provider || tr("Desconhecido", "Unknown");
    }

    function renderReport(report, data) {
        var fmt = new Intl.NumberFormat(A.locale || undefined);
        var money = new Intl.NumberFormat(A.locale || undefined, { style: "currency", currency: "USD", maximumFractionDigits: 2 });
        function metrics(item) {
            var meta = node("span", "gb-quota-metrics");
            var priced = typeof item.cost === "number";
            meta.append(node("strong", "", priced ? money.format(item.cost) : fmt.format(item.tokens || 0) + " tokens"),
                node("small", "", priced ? fmt.format(item.tokens || 0) + " tokens"
                    : fmt.format(item.messages || 0) + tr(" mensagens", " messages")));
            return meta;
        }
        function line(name, item, cls) {
            var row = node("div", "gb-quota-row" + (cls ? " " + cls : ""));
            row.append(node("span", "gb-quota-gutter"), node("span", "gb-quota-name", name), metrics(item));
            return row;
        }

        var engines = Array.isArray(data.engines) && data.engines.length ? data.engines : null;
        var rows = [];

        if (engines) {
            engines.forEach(function (eng) {
                var heading = node("div", "gb-quota-group-head");
                heading.append(node("span", "gb-quota-gutter"), node("strong", "gb-quota-group-title", engineName(eng.engine)), metrics(eng));
                rows.push(heading);
                (eng.providers || []).forEach(function (prov) {
                    var singleModel = (prov.models || []).length === 1;
                    var provLabel = providerName(prov, eng.engine);
                    if (singleModel) {
                        provLabel += " · " + A.shortModelName(prov.models[0].model);
                    }
                    rows.push(line(provLabel, prov, "gb-quota-provider"));
                    if (!singleModel) {
                        (prov.models || []).forEach(function (mdl) {
                            rows.push(line(A.shortModelName(mdl.model), mdl, "gb-quota-model-row"));
                        });
                    }
                });
            });
        } else {
            var entries = Object.entries(data.models || {}).sort(function (a, b) { return (b[1].tokens || 0) - (a[1].tokens || 0); });
            entries.forEach(function (entry) {
                var name = entry[0] === "unknown" ? tr("Sem modelo registrado", "No model recorded") : A.shortModelName(entry[0]);
                rows.push(line(name, entry[1]));
            });
        }

        if (!rows.length) { rows.push(node("p", "gb-quota-note", tr("Nenhum uso nos últimos 30 dias.", "No usage in the last 30 days."))); }
        rows.push(line(tr("Total", "Total"), data.totals || {}, "gb-quota-total"));
        if (!data.pricing_available) {
            rows.push(node("p", "gb-quota-note", tr("Sem preços publicados para estes modelos: o uso aparece em tokens.", "No published prices for these models: usage shows in tokens.")));
        }
        report.replaceChildren.apply(report, rows);
    }

    /* The engine panel's line: the share of the current engine's tightest window. */
    function paintUsageLine() {
        var model = A.selectedModel || A.defaultModel || "";
        var engine = limitEngine(model);
        var top = engine && A.limitTightest ? A.limitTightest(engine) : null;
        var local = !top && (isLocal() || Boolean(engine && (engine.unlimited || engine.kind === "local")));
        document.querySelectorAll(".gb-usage-trigger").forEach(function (button) {
            var kind = button.querySelector(".gb-usage-kind");
            if (!kind) { return; }
            kind.textContent = engineInfo() || tr("Uso", "Usage");
            button.classList.toggle("gb-usage--open-ended", local);
            var pct = button.querySelector(".gb-usage-pct"), fill = button.querySelector(".gb-usage-bar > span");
            pct.textContent = local ? "∞" : top ? top.pct + "%" : "—";
            pct.style.color = top ? A.limitBars(engine).filter(function (b) { return b.pct === top.pct; })[0].color : "";
            fill.style.width = top ? top.pct + "%" : "";
            fill.style.background = top ? pct.style.color : "";
            button.title = local ? tr("Sem cota: roda neste computador", "No quota: runs on this computer")
                : top ? A.limitWindowName(top.window.name) + ": " + top.pct + "%"
                    : tr("Este motor não informa limites", "This engine does not report limits");
        });
    }

    A.syncEngineUsage = function () {
        ensureDrawer();
        document.querySelectorAll(".gb-model-picker").forEach(layoutComposer);
        document.querySelectorAll(".gb-engine-mode").forEach(function (item) { item.textContent = engineInfo(); });
        paintUsageLine();
        if (A.llmLimits) { A.llmLimits().then(paintUsageLine); }
        syncFast();
        if (drawer && !drawer.hidden) { renderDrawer(); placeDrawer(); }
    };
})();
