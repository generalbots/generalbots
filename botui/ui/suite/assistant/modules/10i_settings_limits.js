"use strict";
/* Settings > Limites: the four engine types and what each one reports.
 * Also the shared reader of /api/llm/limits used by the chat's usage drawer
 * (17_usage.js): A.llmLimits(), A.limitBars(), A.currentLimitEngine(). */

(function () {
    var A = window.GBAssistant;
    var selected = "acp";
    var KIND = { acp: "subscription", own_key: "api_key", free: "free", local: "local" };
    var LOGO = { "claude-code": "anthropic", "codex-cli": "openai", "antigravity": "gemini" };
    var engines = [
        ["acp", "Assinaturas", "Subscriptions", "Cota do plano, renova sozinha", "Plan quota, renews by itself", "M4 4h16v16H4zM8 9l3 3-3 3M13 15h3"],
        ["own_key", "Chaves API", "API keys", "Gasto em dólar, você define", "Spending in dollars, you decide",
            "M15.5 7.5 19 4M21 2l-2 2m-3.5 3.5 3 3L22 7l-3-3M11.4 11.6a5.5 5.5 0 1 1-7.8 7.8 5.5 5.5 0 0 1 7.8-7.8z"],
        ["free", "Modelo Gratuito", "Free model", "Limite diário do serviço", "Daily limit of the service", "M13 2 3 14h9l-1 8 10-12h-9z"],
        ["local", "Modelos Locais", "Local models", "Capacidade deste computador", "Capacity of this computer",
            "M4 4h16v16H4zM9 9h6v6H9zM15 2v2M15 20v2M2 15h2M2 9h2M20 15h2M20 9h2M9 2v2M9 20v2"]
    ];
    function tr(pt, en) { return (A.locale || "").startsWith("pt") ? pt : en; }
    function esc(value) { return String(value == null ? "" : value).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }
    function counts(value) { return typeof value === "number" ? value.toLocaleString(A.locale) : "—"; }
    function tone(pct) { return pct >= 85 ? "var(--id-danger-text)" : pct >= 60 ? "var(--id-warn-text)" : "var(--id-ink)"; }
    async function read(path) {
        var response = await A.apiFetch(path);
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.json();
    }

    /* ---- /api/llm/limits, read at most once a minute ---- */
    var cache = null, asked = 0, pending = null;
    A.llmLimits = function (fresh) {
        if (pending) { return pending; }
        if (cache && !fresh && Date.now() - asked < 60000) { return Promise.resolve(cache); }
        asked = Date.now();
        pending = Promise.all([read("/api/llm/limits"), read("/api/llm/config").catch(function () { return null; })])
            .then(function (got) { cache = { engines: (got[0] && got[0].engines) || [], config: got[1] }; return cache; })
            .catch(function () { cache = cache || { engines: [], config: null }; return cache; })
            .then(function (value) { pending = null; return value; });
        return pending;
    };
    A.cachedLimits = function () { return cache; };

    function pctOf(w) {
        if (typeof w.used_percent === "number") { return Math.max(0, Math.min(100, Math.round(w.used_percent))); }
        if (typeof w.limit === "number" && w.limit > 0 && typeof w.remaining === "number") {
            return Math.max(0, Math.min(100, Math.round((w.limit - w.remaining) / w.limit * 100)));
        }
        return null;
    }
    /** The highest used share among the engine's windows, or null without data. */
    A.limitTightest = function (engine) {
        var best = null;
        ((engine && engine.windows) || []).forEach(function (w) {
            var p = pctOf(w);
            if (p !== null && (best === null || p > best.pct)) { best = { pct: p, window: w }; }
        });
        return best;
    };
    var NAMES = { "5h": ["Janela de 5 horas", "5-hour window"], weekly: ["Limite semanal", "Weekly limit"], requests: ["Pedidos", "Requests"],
        tokens: ["Tokens", "Tokens"], input_tokens: ["Tokens de entrada", "Input tokens"], output_tokens: ["Tokens de saída", "Output tokens"],
        daily: ["Limite diário", "Daily limit"] };
    A.limitWindowName = function (name) { var n = NAMES[name]; return n ? tr(n[0], n[1]) : String(name || ""); };
    function hhmm(d) { return String(d.getHours()).padStart(2, "0") + ":" + String(d.getMinutes()).padStart(2, "0"); }
    function resetText(iso) {
        var d = iso ? new Date(iso) : null, now = new Date();
        if (!d || isNaN(d.getTime()) || d <= now) { return ""; }
        if (d.getHours() === 0 && d.getMinutes() === 0 && d - now <= 86400000) { return tr("RENOVA À MEIA-NOITE", "RENEWS AT MIDNIGHT"); }
        if (d.toDateString() === now.toDateString()) { return tr("RENOVA ÀS ", "RENEWS AT ") + hhmm(d); }
        if (d - now < 6 * 86400000) {
            var day = tr("DOM SEG TER QUA QUI SEX SÁB", "SUN MON TUE WED THU FRI SAT").split(" ")[d.getDay()];
            return tr("RENOVA ", "RENEWS ") + day + " " + hhmm(d);
        }
        return tr("RENOVA ", "RENEWS ") + d.toLocaleDateString(A.locale, { day: "2-digit", month: "2-digit" }) + " " + hhmm(d);
    }
    function valueText(w) {
        var lim = typeof w.limit === "number", rem = typeof w.remaining === "number";
        if (lim && rem) { return tr("RESTAM " + counts(w.remaining) + " DE " + counts(w.limit), counts(w.remaining) + " OF " + counts(w.limit) + " LEFT"); }
        if (rem) { return tr("RESTAM ", "") + counts(w.remaining) + tr("", " LEFT"); }
        if (lim) { return tr("LIMITE ", "LIMIT ") + counts(w.limit); }
        return "";
    }
    /** One bar per window that reports a share: [{label, pct, color, value, reset}]. */
    A.limitBars = function (engine) {
        return ((engine && engine.windows) || []).map(function (w) {
            var p = pctOf(w);
            return p === null ? null : { label: A.limitWindowName(w.name), pct: p, color: tone(p), value: valueText(w), reset: resetText(w.resets_at) };
        }).filter(Boolean);
    };
    A.limitSeen = function (engine) {
        var src = { "codex-session": tr("SESSÃO DO AGENTE", "AGENT SESSION"), "response-headers": tr("RESPOSTA DO PROVEDOR", "PROVIDER RESPONSE") }[engine && engine.source] || "";
        var d = engine && engine.observed_at ? new Date(engine.observed_at) : null;
        var at = d && !isNaN(d.getTime()) ? tr("VISTO ", "SEEN ") + (d.toDateString() === new Date().toDateString() ? tr("ÀS ", "AT ") + hhmm(d)
            : d.toLocaleDateString(A.locale, { day: "2-digit", month: "2-digit" }) + " " + hhmm(d)) : "";
        return [src, at].filter(Boolean).join(" · ");
    };
    /** The engine entry for the chat's current engine, or null when the server has none. */
    A.currentLimitEngine = function (value, local) {
        var list = (cache && cache.engines) || [];
        function find(kind, test) { return list.filter(function (e) { return e.kind === kind && (!test || test(e)); })[0] || null; }
        var v = String(value || "");
        var sub = /^sub:([^/]+)/.exec(v), acct = /^acct:([^/]+)\//.exec(v);
        if (sub) { return find("subscription", function (e) { return e.provider === sub[1]; }); }
        if (acct) { return find("api_key", function (e) { return e.provider === acct[1]; }); }
        if (A.engineMode === "free") { return find("free"); }
        if (local) { return find("local") || { kind: "local", unlimited: true, windows: [] }; }
        var cfg = (cache && cache.config) || {};
        var host = String(cfg.endpoint_host || "").toLowerCase(), prov = String(cfg.provider || "").toLowerCase();
        return find("api_key", function (e) {
            var p = String(e.provider || "").toLowerCase();
            return p && (p === prov || (host && host.indexOf(p) >= 0));
        });
    };

    function section(letter, title, summary, body) {
        return '<section class="gb-lim-sec"><div class="gb-lim-sec-head"><span class="gb-set-letter">' + letter + '</span><strong>' + esc(title) +
            '</strong>' + (summary ? '<span class="gb-lim-sum">' + esc(summary) + '</span>' : '') + '</div>' + body + '</section>';
    }
    function bar(label, pct, value, reset, color) {
        var c = color || tone(pct);
        return '<div class="gb-lim-bar"><i aria-hidden="true"></i><div><div class="gb-lim-bar-top"><span>' + esc(label) +
            '</span><b style="color:' + c + '">' + pct + '%</b></div><div class="gb-lim-track"><i style="width:' + pct + '%;background:' + c +
            '"></i></div><div class="gb-lim-bar-foot"><span>' + esc(value) + '</span><span>' + esc(reset) + '</span></div></div></div>';
    }
    function shortDevice(name) {
        return String(name || "").replace(/\b(NVIDIA|GeForce|AMD|Radeon|Intel\(R\)|Intel|Laptop|GPU|Graphics)\b/gi, "").replace(/\s+/g, " ").trim();
    }
    function note(text) { return '<p class="gb-lim-note">' + esc(text) + '</p>'; }
    /* One block per engine: who it is, where the numbers came from, its bars. */
    function engineBlock(e) {
        var svg = (A._providerLogos || {})[LOGO[e.provider] || e.provider];
        var mark = svg ? '<span class="gb-lim-ini gb-lim-logo">' + svg + '</span>'
            : '<span class="gb-lim-ini">' + esc(String(e.label || e.provider || "?").slice(0, 2).toUpperCase()) + '</span>';
        var bars = A.limitBars(e);
        var right = e.unlimited ? '<b class="gb-lim-inf">∞</b>' : '';
        return '<div class="gb-lim-eng"><div class="gb-lim-key">' + mark + '<span class="gb-lim-who"><b>' + esc(e.label || e.provider) + '</b><small>' +
            esc(A.limitSeen(e) || (bars.length || e.unlimited ? "" : tr("SEM DADOS DE USO", "NO USAGE DATA"))) + '</small></span>' + right + '</div>' +
            (bars.length ? bars.map(function (b) { return bar(b.label, b.pct, b.value, b.reset); }).join("")
                : e.unlimited ? "" : note(tr("Este motor ainda não informou limites.", "This engine has not reported limits yet."))) + '</div>';
    }

    A.renderLimits = async function (panel) {
        panel.innerHTML = A.settingsHeading(tr("Limites", "Limits"), tr("Quanto cada motor pode usar.", "How much each engine can use."),
            tr("Cada tipo de motor tem um limite diferente: a cota da assinatura, o gasto da sua chave, o limite do serviço gratuito ou a capacidade deste computador.",
                "Each type of engine has a different limit: the subscription quota, your key’s spending, the free service limit or the capacity of this computer.")) +
            '<div class="gb-sp-card gb-lim"><div class="gb-lim-types" role="group" aria-label="' + tr("Tipos de motor", "Engine types") + '">' +
            engines.map(function (engine, index) {
                return '<button type="button" data-limit="' + engine[0] + '"><span class="gb-lim-top"><span class="gb-lim-icon">' +
                    A.settingsIcon(engine[5], 15) + '</span><span class="gb-lim-no">0' + (index + 1) + '</span><span class="gb-lim-pct">—</span></span><span class="gb-lim-name"><strong>' +
                    tr(engine[1], engine[2]) + '</strong><span>' + tr(engine[3], engine[4]) + '</span></span><span class="gb-lim-meter"><i></i></span></button>';
            }).join("") + '</div><div class="gb-lim-detail-head"><span class="gb-sp-mark gb-lim-detail-k">' + tr("Detalhes", "Details") +
            '</span><strong id="gbLimitsName"></strong></div><div class="gb-lim-tray"><i class="gb-lim-gutter" aria-hidden="true"></i>' +
            '<div class="gb-lim-tray-in" id="gbLimitsDetail"></div></div></div>';
        var details = panel.querySelector("#gbLimitsDetail");
        var cards = panel.querySelectorAll("[data-limit]");
        var hardware;
        var config;
        var reported = [];
        function ofKind(id) { return reported.filter(function (e) { return e.kind === KIND[id]; }); }
        function drawCards() {
            cards.forEach(function (button) {
                var id = button.dataset.limit, list = ofKind(id);
                var top = null;
                list.forEach(function (e) { var t = A.limitTightest(e); if (t && (top === null || t.pct > top)) { top = t.pct; } });
                var inf = id === "local" || (list.length && list.every(function (e) { return e.unlimited; }));
                var pct = button.querySelector(".gb-lim-pct"), meter = button.querySelector(".gb-lim-meter i");
                pct.textContent = inf ? "∞" : top === null ? "—" : top + "%";
                pct.style.color = inf ? "var(--id-success-text)" : top === null ? "" : tone(top);
                meter.style.width = inf ? "100%" : top === null ? "0" : top + "%";
                meter.style.background = inf ? "var(--id-success-text)" : top === null ? "" : tone(top);
            });
        }
        function drawDetails() {
            var selection = engines.find(function (engine) { return engine[0] === selected; });
            cards.forEach(function (button) { button.setAttribute("aria-pressed", button.dataset.limit === selected); });
            panel.querySelector("#gbLimitsName").textContent = tr(selection[1], selection[2]);
            var list = ofKind(selected).filter(function (e) { return selected !== "local" || (e.windows || []).length; });
            var blocks = list.map(engineBlock).join("");
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
                        counts(used) + tr(" DE ", " OF ") + counts(vram) + " GB", tr("AGORA", "NOW")) : "") + blocks);
            } else if (selected === "own_key") {
                var key = config && config.mode === "own_key" && config.endpoint_host ? config : null;
                html = section("A", tr("Gasto por chave", "Spending per key"), "", blocks ||
                    ((key ? '<div class="gb-lim-key"><span class="gb-lim-ini">' + esc(String(key.provider || key.endpoint_host).slice(0, 2).toUpperCase()) +
                        '</span><span><b>' + esc(key.provider || key.endpoint_host) + '</b><small>' + esc(String(key.endpoint_host).toUpperCase()) +
                        '</small></span></div>' : '') +
                    note(tr("O provedor não informa o gasto da chave. Consulte o painel do provedor para definir limites e ver cobranças.",
                        "The provider does not report the key’s spending. Use the provider’s dashboard to set limits and check charges."))));
            } else if (selected === "free") {
                html = section("A", tr("Limites do serviço", "Service limits"), "", blocks ||
                    note(tr("O serviço gratuito define o consumo permitido e o período de renovação, e não os informa.",
                        "The free service sets its allowance and reset period and does not report them.")));
            } else {
                html = section("A", tr("Limites de cada plano", "Limits of each plan"), blocks ? tr("RENOVAM SOZINHOS", "RENEW BY THEMSELVES") : "", blocks ||
                    note(tr("O agente não informa a cota do plano nem a próxima renovação. Confira na conta do agente.",
                        "The agent does not report its plan quota or next reset. Check the agent’s account.")));
            }
            details.innerHTML = html;
        }
        cards.forEach(function (button) {
            button.onclick = function () { selected = button.dataset.limit; drawDetails(); };
        });
        drawCards();
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
            }),
            A.llmLimits(true).then(function (data) {
                reported = data.engines;
                if (panel.isConnected) { drawCards(); drawDetails(); }
            })
        ]);
    };
})();
