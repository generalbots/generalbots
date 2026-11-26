"use strict";
/* Limits: the "Atualizar" button shared by the usage drawer (17_usage.js)
 * and Settings > Limites (10i_settings_limits.js), and what each engine can
 * report, said plainly when it reports nothing. */

(function () {
    var A = window.GBAssistant;
    function tr(pt, en) { return (A.locale || "").startsWith("pt") ? pt : en; }
    function hhmm(d) { return String(d.getHours()).padStart(2, "0") + ":" + String(d.getMinutes()).padStart(2, "0"); }
    var ICON = '<svg class="gb-lim-refresh-ic" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"' +
        ' stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 11a8 8 0 0 0-14.3-4.9L4 8"/><path d="M4 3v5h5"/>' +
        '<path d="M4 13a8 8 0 0 0 14.3 4.9L20 16"/><path d="M20 21v-5h-5"/></svg>';

    /** "Atualizado às HH:MM" for the last read, or "" before any. */
    A.limitsCheckedText = function () {
        var at = A.cachedLimits && A.cachedLimits() && A.cachedLimits().at;
        return at ? tr("Atualizado às ", "Updated at ") + hhmm(new Date(at)) : "";
    };

    /** A refresh control: button + status line. `onDone(data)` repaints. */
    A.limitsRefreshControl = function (onDone) {
        var wrap = document.createElement("div");
        wrap.className = "gb-lim-refresh";
        var button = document.createElement("button");
        button.type = "button";
        button.className = "gb-lim-refresh-btn";
        button.title = tr("Ler os limites de novo agora", "Read the limits again now");
        button.innerHTML = ICON + "<span>" + tr("Atualizar", "Refresh") + "</span>";
        var status = document.createElement("span");
        status.className = "gb-lim-refresh-at";
        status.setAttribute("role", "status");
        status.textContent = A.limitsCheckedText();
        button.addEventListener("click", function () {
            if (button.disabled) { return; }
            button.disabled = true;
            button.setAttribute("aria-busy", "true");
            wrap.classList.add("is-loading");
            status.textContent = tr("Atualizando…", "Refreshing…");
            A.llmLimits(true, true).then(function (data) {
                button.disabled = false;
                button.removeAttribute("aria-busy");
                wrap.classList.remove("is-loading");
                status.textContent = data && data.failed ? tr("Não foi possível atualizar agora.", "Could not refresh now.") : A.limitsCheckedText();
                if (onDone) { onDone(data); }
            });
        });
        wrap.append(button, status);
        wrap.sync = function () { if (!button.disabled) { status.textContent = A.limitsCheckedText(); } };
        return wrap;
    };

    var STATUS = { allowed: ["Dentro do limite", "Within the limit"], allowed_warning: ["Perto do limite", "Close to the limit"],
        rejected: ["Cota esgotada", "Quota used up"] };
    /** The words for a window that reports a status and no share. */
    A.limitStatusText = function (status) { var s = STATUS[status]; return s ? tr(s[0], s[1]) : ""; };

    /* What each subscription engine can tell, and where else to look. */
    var SILENT = {
        "codex-cli": ["O Codex ainda não informou a cota. Toque em Atualizar para ler agora, ou envie uma mensagem com o Codex.",
            "Codex has not reported its quota yet. Tap Refresh to read it now, or send a message with Codex."],
        "claude-code": ["O Claude Code informa a cota a cada resposta, e ainda não respondeu desde que o servidor iniciou. Também dá para ver em claude.ai, em Configurações › Uso.",
            "Claude Code reports its quota with each answer and has not answered since the server started. You can also see it on claude.ai, under Settings › Usage."],
        "antigravity": ["O Antigravity não informa limites por aqui. Veja a cota no próprio Antigravity, entrando com a mesma conta Google.",
            "Antigravity does not report limits here. Check the quota in Antigravity itself, signed in with the same Google account."],
        "chatgpt-plan": ["O plano informa os limites a cada resposta, e ainda não houve resposta desde que o servidor iniciou.",
            "The plan reports its limits with each answer, and there has been no answer since the server started."]
    };
    var HOW = {
        live: ["Lido do serviço ao atualizar", "Read from the service on refresh"],
        turns: ["Informado a cada resposta", "Reported with each answer"],
        headers: ["Informado a cada resposta", "Reported with each answer"],
        none: ["Só avisa quando a cota acaba", "Only says so when the quota runs out"]
    };
    /** The plain sentence for an engine with no data, or "". */
    A.limitSilentText = function (engine) {
        var s = engine && SILENT[engine.provider];
        if (s) { return tr(s[0], s[1]); }
        if (engine && engine.kind === "api_key") {
            return tr("Este provedor ainda não enviou limites. Eles chegam com cada resposta, se o provedor os informar.",
                "This provider has not sent limits yet. They arrive with each answer, when the provider reports them.");
        }
        return "";
    };
    /** How the engine reports, for the small caption under its name. */
    A.limitHowText = function (engine) {
        if (engine && engine.source === "codex-service") { return ""; }
        var h = engine && HOW[engine.reports || (engine.kind === "api_key" || engine.kind === "free" ? "headers" : "")];
        return h ? tr(h[0], h[1]) : "";
    };
    /** A note when the refresh could not re-read this engine. */
    A.limitRefreshNote = function (engine) {
        return engine && engine.refresh_failed ? tr("Não foi possível ler agora; mostrando o último valor conhecido.",
            "Could not read it now; showing the last known value.") : "";
    };
})();
