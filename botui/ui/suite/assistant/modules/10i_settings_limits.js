"use strict";
/* Settings > Limites: the four engine types and what each one reports. */

(function () {
    var A = window.GBAssistant;
    var selected = "acp";
    var engines = [
        ["acp", "Assinaturas", "Subscriptions", "Cota do plano, renova sozinha", "Plan quota, renews by itself", "M4 4h16v16H4zM8 9l3 3-3 3M13 15h3"],
        ["own_key", "Chaves API", "API keys", "Gasto em dólar, você define", "Spending in dollars, you decide",
            "M15.5 7.5 19 4M21 2l-2 2m-3.5 3.5 3 3L22 7l-3-3M11.4 11.6a5.5 5.5 0 1 1-7.8 7.8 5.5 5.5 0 0 1 7.8-7.8z"],
        ["free", "Modelo Gratuito", "Free model", "Limite diário do serviço", "Daily limit of the service", "M13 2 3 14h9l-1 8 10-12h-9z"],
        ["local", "Modelos Locais", "Local models", "Capacidade deste computador", "Capacity of this computer",
            "M4 4h16v16H4zM9 9h6v6H9zM15 2v2M15 20v2M2 15h2M2 9h2M20 15h2M20 9h2M9 2v2M9 20v2"]
    ];
    function tr(pt, en) { return A.settingsText(pt, en); }
    function esc(value) { return A.settingsEsc(value); }
    function counts(value) { return typeof value === "number" ? value.toLocaleString(A.locale) : "—"; }
    function tone(pct) { return pct >= 85 ? "var(--id-danger-text)" : pct >= 60 ? "var(--id-warn-text)" : "var(--id-ink)"; }
    async function read(path) {
        var response = await A.apiFetch(path);
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
    }
    function section(letter, title, summary, body) {
        return '<section class="gb-lim-sec"><div class="gb-lim-sec-head"><span class="gb-set-letter">' + letter + '</span><strong>' + esc(title) +
            '</strong>' + (summary ? '<span class="gb-lim-sum">' + esc(summary) + '</span>' : '') + '</div>' + body + '</section>';
    }
    function bar(label, pct, value, reset) {
        return '<div class="gb-lim-bar"><i aria-hidden="true"></i><div><div class="gb-lim-bar-top"><span>' + esc(label) +
            '</span><b style="color:' + tone(pct) + '">' + pct + '%</b></div><div class="gb-lim-track"><i style="width:' + pct + '%;background:' + tone(pct) +
            '"></i></div><div class="gb-lim-bar-foot"><span>' + esc(value) + '</span><span>' + esc(reset) + '</span></div></div></div>';
    }
    function shortDevice(name) {
        return String(name || "").replace(/\b(NVIDIA|GeForce|AMD|Radeon|Intel\(R\)|Intel|Laptop|GPU|Graphics)\b/gi, "").replace(/\s+/g, " ").trim();
    }

    A.renderLimits = async function (panel) {
        panel.innerHTML = A.settingsHeading(tr("Limites", "Limits"), tr("Quanto cada motor pode usar.", "How much each engine can use."),
            tr("Cada tipo de motor tem um limite diferente: a cota da assinatura, o gasto da sua chave, o limite do serviço gratuito ou a capacidade deste computador.",
                "Each type of engine has a different limit: the subscription quota, your key’s spending, the free service limit or the capacity of this computer.")) +
            '<div class="gb-sp-card gb-lim"><div class="gb-lim-types" role="group" aria-label="' + tr("Tipos de motor", "Engine types") + '">' +
            engines.map(function (engine, index) {
                var local = engine[0] === "local";
                return '<button type="button" data-limit="' + engine[0] + '"><span class="gb-lim-top"><span class="gb-lim-icon">' +
                    A.settingsIcon(engine[5], 15) + '</span><span class="gb-lim-no">0' + (index + 1) + '</span><span class="gb-lim-pct"' +
                    (local ? ' style="color:var(--id-success-text)"' : '') + '>' + (local ? "∞" : "—") + '</span></span><span class="gb-lim-name"><strong>' +
                    tr(engine[1], engine[2]) + '</strong><span>' + tr(engine[3], engine[4]) + '</span></span><span class="gb-lim-meter"><i' +
                    (local ? ' style="width:100%;background:var(--id-success-text)"' : '') + '></i></span></button>';
            }).join("") + '</div><div class="gb-lim-detail-head"><span class="gb-sp-mark gb-lim-detail-k">' + tr("Detalhes", "Details") +
            '</span><strong id="gbLimitsName"></strong></div><div class="gb-lim-tray"><i class="gb-lim-gutter" aria-hidden="true"></i>' +
            '<div class="gb-lim-tray-in" id="gbLimitsDetail"></div></div></div>';
        var details = panel.querySelector("#gbLimitsDetail");
        var cards = panel.querySelectorAll("[data-limit]");
        var hardware;
        var config;
        function note(text) { return '<p class="gb-lim-note">' + esc(text) + '</p>'; }
        function drawDetails() {
            var selection = engines.find(function (engine) { return engine[0] === selected; });
            cards.forEach(function (button) { button.setAttribute("aria-pressed", button.dataset.limit === selected); });
            panel.querySelector("#gbLimitsName").textContent = tr(selection[1], selection[2]);
            var html;
            if (selected === "local") {
                var vram = hardware && hardware.vram_gb > 0 ? hardware.vram_gb : 0;
                var free = hardware && typeof hardware.vram_free_gb === "number" ? hardware.vram_free_gb : null;
                var summary = hardware && hardware.device ? shortDevice(hardware.device) + (vram ? " · " + counts(vram) + " GB" : "") : "";
                var used = vram && free !== null ? Math.max(0, vram - free) : null;
                html = section("A", tr("Sem cota, sem custo", "No quota, no cost"), summary,
                    note(tr("O modelo local não tem cota de uso. O limite é a capacidade deste computador.",
                        "The local model has no usage quota. The limit is the capacity of this computer.")) +
                    (used !== null ? bar(tr("Memória de vídeo em uso", "Video memory in use"), Math.round(used / vram * 100),
                        counts(used) + tr(" DE ", " OF ") + counts(vram) + " GB", tr("AGORA", "NOW")) : ""));
            } else if (selected === "own_key") {
                var key = config && config.mode === "own_key" && config.endpoint_host ? config : null;
                html = section("A", tr("Gasto por chave", "Spending per key"), "",
                    (key ? '<div class="gb-lim-key"><span class="gb-lim-ini">' + esc(String(key.provider || key.endpoint_host).slice(0, 2).toUpperCase()) +
                        '</span><span><b>' + esc(key.provider || key.endpoint_host) + '</b><small>' + esc(String(key.endpoint_host).toUpperCase()) +
                        '</small></span></div>' : '') +
                    note(tr("O provedor não informa o gasto da chave. Consulte o painel do provedor para definir limites e ver cobranças.",
                        "The provider does not report the key’s spending. Use the provider’s dashboard to set limits and check charges.")));
            } else if (selected === "free") {
                html = section("A", tr("Limites do serviço", "Service limits"), "",
                    note(tr("O serviço gratuito define o consumo permitido e o período de renovação, e não os informa.",
                        "The free service sets its allowance and reset period and does not report them.")));
            } else {
                html = section("A", tr("Limites de cada plano", "Limits of each plan"), "",
                    note(tr("O agente não informa a cota do plano nem a próxima renovação. Confira na conta do agente.",
                        "The agent does not report its plan quota or next reset. Check the agent’s account.")));
            }
            details.innerHTML = html;
        }
        cards.forEach(function (button) {
            button.onclick = function () { selected = button.dataset.limit; drawDetails(); };
        });
        drawDetails();

        await Promise.allSettled([
            read("/api/llm/config/modes").then(function (data) {
                var local = (data.modes || []).find(function (mode) { return mode.id === "local"; });
                hardware = local && local.hardware;
                if (panel.isConnected) { drawDetails(); }
            }),
            read("/api/llm/config").then(function (data) {
                config = data;
                if (panel.isConnected) { drawDetails(); }
            })
        ]);
    };
})();
