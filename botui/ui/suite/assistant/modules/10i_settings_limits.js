"use strict";

(function () {
    var A = window.GBAssistant;
    var selected = "acp";
    var engines = [
        ["acp", "Assinaturas", "Subscriptions", "Cota definida pelo plano", "Quota set by your plan", "M4 4h16v16H4zM8 9l3 3-3 3M13 15h3"],
        ["own_key", "Chaves API", "API keys", "Gasto definido pelo provedor", "Spending managed by your provider", "M15 9l6-6m-3 3 3 3M6 12a4 4 0 1 0 0 8 4 4 0 0 0 0-8zm3 1 7-7"],
        ["free", "Modelo gratuito", "Free model", "Limites do serviço hospedado", "Hosted service limits", "M13 2l-9 12h8l-1 8 9-12h-8z"],
        ["local", "Modelos locais", "Local models", "Capacidade deste computador", "Capacity of this computer", "M6 6h12v12H6zM9 9h6v6H9zM9 3v3m6-3v3M9 18v3m6-3v3M3 9h3m-3 6h3m12-6h3m-3 6h3"]
    ];
    function tr(pt, en) { return A.settingsText(pt, en); }
    function esc(value) {
        return String(value == null ? "" : value).replace(/[&<>"]/g, function (c) {
            return {"&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;"}[c];
        });
    }
    function counts(value) { return typeof value === "number" ? value.toLocaleString(A.locale) : "—"; }
    function estimate(value) {
        return typeof value === "number" ? new Intl.NumberFormat(A.locale, {
            style: "currency", currency: "USD", maximumFractionDigits: 6
        }).format(value) : tr("Indisponível", "Unavailable");
    }
    async function read(path) {
        var response = await A.apiFetch(path);
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
    }

    A.renderLimits = async function (panel) {
        panel.innerHTML = A.settingsHeading(tr("Limites", "Limits"), tr("Quanto cada motor pode usar.", "How much each engine can use."),
            tr("Cada motor tem seu próprio limite: a cota da assinatura, o gasto da chave, o limite do serviço gratuito ou a capacidade deste computador.",
                "Each engine has its own limit: your subscription quota, API spending, the free service limit, or this computer’s capacity.")) +
            '<section class="gb-set-slab"><div class="gb-set-limits-grid" role="group" aria-label="' +
            tr("Tipos de motor", "Engine types") + '">' + engines.map(function (engine, index) {
                return '<button type="button" data-limit="' + engine[0] + '"><span class="gb-set-limit-top">' +
                    '<span class="gb-set-limit-icon"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">' +
                    '<path d="' + engine[5] + '"/></svg></span><span class="gb-set-limit-no">0' + (index + 1) +
                    '</span><span class="gb-set-limit-pct">—</span></span><strong>' + tr(engine[1], engine[2]) +
                    '</strong><span class="gb-set-limit-desc">' + tr(engine[3], engine[4]) +
                    '</span><span class="gb-set-limit-bar" aria-hidden="true"><i></i></span></button>';
            }).join("") + '</div></section><div id="gbLimitsDetail"></div><div id="gbLimitsUsage" role="status"></div>';
        var details = panel.querySelector("#gbLimitsDetail");
        var usage = panel.querySelector("#gbLimitsUsage");
        var cards = panel.querySelectorAll("[data-limit]");
        var hardware;
        var config;
        var summary;
        function drawShares() {
            var shares = typeShares(summary, config);
            cards.forEach(function (button) {
                var share = shares[button.dataset.limit];
                var known = typeof share === "number";
                var label = known ? Math.round(share * 100) + "%" : "—";
                button.querySelector(".gb-set-limit-pct").textContent = label;
                button.querySelector(".gb-set-limit-bar i").style.width = known ? (share * 100).toFixed(1) + "%" : "0";
                button.title = known ? tr("Parte dos tokens usados nos últimos 30 dias: ", "Share of tokens used over the last 30 days: ") + label :
                    tr("Nenhum uso atribuído a este tipo nos últimos 30 dias.", "No usage attributed to this type over the last 30 days.");
            });
        }
        function drawDetails() {
            var selection = engines.find(function (engine) { return engine[0] === selected; });
            cards.forEach(function (button) { button.setAttribute("aria-pressed", button.dataset.limit === selected); });
            var explanation = {
                acp: tr("Consulte a conta do agente para ver a cota do plano e a próxima renovação.",
                    "Check your agent’s account for its plan quota and next reset."),
                own_key: tr("Consulte o painel do provedor para definir limites de gasto e ver cobranças da chave.",
                    "Use your provider’s dashboard to set spending limits and check API charges."),
                free: tr("O serviço gratuito define o consumo permitido e o período de renovação.",
                    "The hosted free service sets its allowance and reset period."),
                local: tr("O motor local usa os recursos deste computador, sem cota de mensagens do provedor.",
                    "Local models use this computer’s resources and have no provider message quota.")
            };
            var facts = "";
            if (selected === "local" && hardware) {
                if (hardware.device) { facts += '<p>' + esc(hardware.device) + '</p>'; }
                if (hardware.vram_gb > 0) { facts += '<p>' + counts(hardware.vram_gb) + ' GB VRAM</p>'; }
            } else if (config && config.mode === selected && config.endpoint_host &&
                    selected !== "acp") {
                facts = '<p class="gb-set-meta">' + esc(config.endpoint_host) + '</p>';
            }
            details.innerHTML = '<section class="gb-set-slab"><h2><span class="gb-set-letter">A</span>' +
                tr(selection[1], selection[2]) + '</h2><div class="gb-set-slab-body"><p>' + explanation[selected] +
                '</p>' + facts + '</div></section><section class="gb-set-slab"><h2><span class="gb-set-letter">B</span>' +
                tr("Trocar de motor", "Switch engine") + '</h2><div class="gb-set-slab-body">' +
                '<button type="button" class="gb-set-btn-sm">' + tr("Abrir a oficina", "Open the workshop") +
                '</button></div></section>';
            details.querySelector("button").onclick = function () { A.openSettings("model"); };
        }
        cards.forEach(function (button) {
            button.onclick = function () { selected = button.dataset.limit; drawDetails(); };
        });
        drawDetails();

        async function loadUsage() {
            usage.textContent = tr("Carregando uso…", "Loading usage…");
            try {
                summary = await read("/api/user/usage/summary?days=30");
                if (panel.isConnected) { usage.innerHTML = usageTable(summary); drawShares(); }
            } catch (error) {
                usage.innerHTML = '<p>' + tr("Uso indisponível.", "Usage unavailable.") +
                    '</p><button type="button" class="gb-set-btn-sm">' + tr("Tentar novamente", "Try again") + '</button>';
                usage.querySelector("button").onclick = loadUsage;
            }
        }
        await Promise.allSettled([
            loadUsage(),
            read("/api/llm/config/modes").then(function (data) {
                var local = (data.modes || []).find(function (mode) { return mode.id === "local"; });
                hardware = local && local.hardware;
                if (panel.isConnected) { drawDetails(); }
            }),
            read("/api/llm/config").then(function (data) {
                config = data;
                if (panel.isConnected) { drawDetails(); drawShares(); }
            })
        ]);
    };

    function typeShares(data, cfg) {
        var shares = {};
        var total = data && data.totals && data.totals.tokens;
        if (!cfg || !cfg.model || !(total > 0)) { return shares; }
        var row = (data.models || {})[cfg.model];
        if (row && typeof row.tokens === "number" && engines.some(function (e) { return e[0] === cfg.mode; })) {
            shares[cfg.mode] = Math.min(1, row.tokens / total);
        }
        return shares;
    }

    function usageTable(data) {
        var totals = data.totals || {};
        var models = Object.entries(data.models || {});
        var tokens = tr("Tokens", "Tokens");
        var messages = tr("Mensagens", "Messages");
        var cost = tr("Custo estimado", "Estimated cost");
        var period = tr("Uso nos últimos 30 dias", "Usage over the last 30 days");
        var body = models.map(function (item) {
            return '<tr><th scope="row">' + esc(item[0]) + '</th><td>' + counts(item[1].messages) +
                '</td><td>' + counts(item[1].tokens) + '</td><td>' + estimate(item[1].cost) + '</td></tr>';
        }).join("");
        return '<section class="gb-set-slab"><h2>' + period + '</h2><div class="gb-set-table-scroll"><table class="gb-set-usage-table"><thead><tr><th scope="col">' +
            tr("Modelo", "Model") + '</th><th scope="col">' + messages + '</th><th scope="col">' + tokens +
            '</th><th scope="col">' + cost + '</th></tr></thead><tbody>' + body +
            '</tbody><tfoot><tr><th scope="row">Total</th><td>' + counts(totals.messages) + '</td><td>' +
            counts(totals.tokens) + '</td><td>' + estimate(totals.cost) + '</td></tr></tfoot></table></div></section>';
    }
})();
