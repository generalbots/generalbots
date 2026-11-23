"use strict";
/**
 * 05d_engine_guard.js - The engine the chat names, and the first send out.
 *
 * 1. /api/chat/models names the bot's stored model, while a turn runs on the
 *    model the runtime is set to (the one Settings > Modelo shows, from
 *    /api/llm/config). After every model load the shell takes the runtime's
 *    model, so the menu, the status line and the answer tag name the engine
 *    that really answers.
 * 2. The first time the person picks an engine outside this computer, a
 *    `.gb-confirm` card above the composer says where the text goes.
 */
(function () {
var A = window.GBAssistant;
var KEY = "gb-external-ok";

function tr(ptT, enT) { return (A.locale || "").startsWith("pt") ? ptT : enT; }

/* The empty state and the conversation each have a composer; only one
   shows. Cards that belong above the composer go above the visible one. */
A.visibleComposer = function () {
    var all = document.querySelectorAll(".gb-composer-inner, .gb-composer-card");
    for (var i = 0; i < all.length; i++) {
        if (all[i].offsetParent !== null) { return all[i]; }
    }
    return all[0] || null;
};

var load = A.loadModels;
if (typeof load === "function") {
    A.loadModels = function () {
        return Promise.resolve(load.apply(A, arguments)).then(reconcile);
    };
}

function reconcile() {
    return A.apiFetch("/api/llm/config").then(function (r) { return r.ok ? r.json() : null; }).then(function (cfg) {
        A.engineMode = cfg && cfg.mode ? cfg.mode : "";
        var live = cfg && typeof cfg.model === "string" ? cfg.model.trim() : "";
        var stale = A.defaultModel || "";
        if (!live || live === stale) { return; }
        A.staleModel = stale;
        A.defaultModel = live;
        (A.tiers || []).forEach(function (tier) { if (!tier.model || tier.model === stale) { tier.model = live; } });
        if (!A.selectedModel || A.selectedModel === stale) { A.selectedModel = live; }
    }).catch(function () { /* the stored names stay */ });
}

function accepted() {
    try { return JSON.parse(localStorage.getItem(KEY) || "[]") || []; } catch (e) { return []; }
}
function accept(host) {
    var list = accepted();
    if (list.indexOf(host) < 0) { list.push(host); }
    try { localStorage.setItem(KEY, JSON.stringify(list)); } catch (e) { /* storage blocked: ask again next time */ }
}

/** Where a picked value sends the conversation, or null when it stays here. */
A.externalTarget = function (value) {
    var v = String(value || "");
    var name = A.shortModelName ? A.shortModelName(v) : v;
    if (/^sub:/.test(v)) {
        var provider = v.slice(4).split("/")[0];
        var agentMeta = A._agentMeta ? A._agentMeta(provider) : null;
        var agentName = agentMeta ? agentMeta.name : name;
        var agentHost = agentMeta ? (agentMeta.name + ", " + agentMeta.co) : name;
        return Promise.resolve({ name: agentName, host: agentHost });
    }
    var acct = /^acct:([^/]+)\//.exec(v);
    if (!acct) { return Promise.resolve(null); }
    return A.apiFetch("/api/llm/accounts").then(function (r) { return r.ok ? r.json() : {}; }).then(function (data) {
        var hit = (data.accounts || []).filter(function (a) { return a.id === acct[1]; })[0] || {};
        return { name: name, host: hit.host || hit.name || acct[1] };
    }).catch(function () { return { name: name, host: acct[1] }; });
};

function closeCards() { document.querySelectorAll(".gb-confirm").forEach(function (c) { c.remove(); }); }

/**
 * Asks once per destination. `target` is {name, host} or null (stays here);
 * `anchor` is any element inside the composer.
 */
A.confirmExternal = function (target, anchor, onYes) {
    if (!target || !target.host || accepted().indexOf(target.host) >= 0) { onYes(); return; }
    closeCards();
    var shared = A._modelShared || {};
    var sentence = shared.t ? shared.t("confirmExt") : tr("O que você escrever e anexar será enviado a %h, fora deste computador. Continuar?",
        "What you write and attach will be sent to %h, outside this computer. Continue?");
    var parts = sentence.split("%h");
    var card = document.createElement("div");
    card.className = "gb-confirm";
    card.setAttribute("role", "alertdialog");
    card.setAttribute("aria-label", tr("Confirmação", "Confirmation"));
    var head = document.createElement("div");
    head.className = "gb-confirm-head";
    head.innerHTML = '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
        '<path d="M12 3l9 16H3z"/><path d="M12 10v4M12 17h.01"/></svg>';
    var label = document.createElement("span");
    label.textContent = tr("Confirmação", "Confirmation");
    head.appendChild(label);
    var text = document.createElement("p");
    text.className = "gb-confirm-text";
    var model = document.createElement("strong");
    model.textContent = target.name;
    var host = document.createElement("strong");
    host.textContent = target.host;
    text.append(tr("Você escolheu ", "You picked "), model, ". " + parts[0], host, parts.slice(1).join(target.host));
    var actions = document.createElement("div");
    actions.className = "gb-confirm-actions";
    var stay = document.createElement("button");
    stay.type = "button";
    stay.className = "gb-block-btn";
    stay.textContent = (A.destination || {}).scope === "machine"
        ? tr("Ficar no modelo local", "Stay on the local model") : tr("Cancelar", "Cancel");
    var go = document.createElement("button");
    go.type = "button";
    go.className = "gb-block-btn gb-block-btn--acc";
    go.textContent = tr("Continuar", "Continue");
    actions.append(stay, go);
    card.append(head, text, actions);
    stay.addEventListener("click", function () { card.remove(); });
    go.addEventListener("click", function () { card.remove(); accept(target.host); onYes(); });
    card.addEventListener("keydown", function (e) { if (e.key === "Escape") { e.stopPropagation(); card.remove(); } });
    var composer = anchor && anchor.closest(".gb-composer-inner, .gb-composer-card");
    if (!composer || composer.offsetParent === null) { composer = A.visibleComposer(); }
    if (composer && composer.parentNode) { composer.parentNode.insertBefore(card, composer); }
    else { document.body.appendChild(card); }
    go.focus();
};
})();
