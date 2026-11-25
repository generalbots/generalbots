"use strict";
/**
 * 19_turn_cards.js - What the server reports about a turn as structured
 * frames, shown inside the answer:
 * - {type:"agent_enable_request", reason, request}: the request needs the
 *   Virtual Machine and the conversation's Agent switch is off. A permission
 *   card in the look of the VM cards (08b) offers "Ativar e continuar" (the
 *   switch goes on and the same request is sent again through the composer)
 *   and "Agora não" (the text answer stays).
 * - {type:"memory_saved", key, value}, {type:"memory_forgot", count},
 *   {type:"memory_off"}: a small block in the style of the run blocks, with
 *   "Ver em Configurações › Memória" and, for a saved memory, "Desfazer"
 *   (DELETE /api/user/memory/:key).
 * On reload the answer's message meta carries the same objects under
 * `agent_enable_request` (shown answered, without buttons) and
 * `memory_event` (one object or a list).
 */
(function () {
var A = window.GBAssistant;
var B = A._blk;
if (!B) { return; }
var esc = B.esc;
function t(pt, en) { return A.locale && A.locale.indexOf("pt") === 0 ? pt : en; }
var SHIELD = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>';
var seq = 0;

A.frameHandlers = A.frameHandlers || {};

/* The last answer after the last message the person sent, if any. */
function turnAnswer() {
    var msgs = document.getElementById("gbMessages");
    if (!msgs) { return null; }
    for (var i = msgs.children.length - 1; i >= 0; i--) {
        var ch = msgs.children[i];
        if (ch.classList.contains("gb-msg-user")) { return null; }
        if (ch.classList.contains("gb-msg-assistant")) { return ch; }
    }
    return null;
}

/* A finished answer takes the card above its action row; a turn still
   running takes it among its steps (12b), above the text to come. */
A.placeTurnCard = function (el) {
    var answer = turnAnswer();
    var done = answer && answer.querySelector(".gb-msg-toolbar");
    if (done) { done.before(el); }
    else if (A.placeInTurn) { A.placeInTurn(el); }
    else if (answer) { answer.appendChild(el); }
    else { return false; }
    if (!A.isUserScrolledUp || !A.isUserScrolledUp()) { A.scrollToBottom(false); }
    return true;
};

/* Meta of a stored answer: flat on the message, or under `meta`. */
A.turnMeta = function (m, key) {
    if (!m) { return null; }
    if (m[key] != null) { return m[key]; }
    return m.meta && m.meta[key] != null ? m.meta[key] : null;
};

/* ---- Agent switch ---- */
function resolveAgentCard(card, text, granted) {
    card.classList.add("gb-consent-resolved");
    card.innerHTML =
        '<div class="gb-consent-outcome">' +
            '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">' +
                (granted ? '<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>'
                         : '<circle cx="12" cy="12" r="10"/><line x1="8" y1="12" x2="16" y2="12"/>') +
            '</svg><span>' + esc(text) + '</span>' +
        '</div>';
}

function agentCard(d, answered) {
    var card = document.createElement("div");
    var id = "gbAgentAsk" + (++seq);
    card.className = "gb-consent-card gb-consent-vm gb-consent-agent";
    card.setAttribute("role", "group");
    card.setAttribute("aria-labelledby", id);
    card.tabIndex = -1;
    if (answered) {
        resolveAgentCard(card, t("Pedido para ativar o Agente", "Asked to turn on the Agent") +
            (d.reason ? ": " + d.reason : ""), false);
        return card;
    }
    card.innerHTML =
        '<div class="gb-consent-head">' +
            '<span class="gb-consent-icon">' + SHIELD + '</span>' +
            '<div class="gb-consent-headings">' +
                '<div class="gb-consent-kicker">' + esc(t("Precisa da sua permissão", "Needs your permission")) + '</div>' +
                '<div class="gb-consent-title" id="' + id + '">' + esc(t("Ativar o Agente nesta conversa?", "Turn on the Agent in this conversation?")) + '</div>' +
            '</div>' +
        '</div>' +
        '<div class="gb-consent-body">' +
            '<p class="gb-consent-agent-text">' + esc(t("O assistente poderá executar comandos e gerar arquivos na Máquina Virtual; cada ação ainda pede sua permissão.",
                "The assistant will be able to run commands and create files in the Virtual Machine; each action still asks for your permission.")) + '</p>' +
            (d.reason ? '<div class="gb-consent-meta"><span>' + esc(t("Motivo:", "Reason:")) + '</span> ' + esc(d.reason) + '</div>' : '') +
        '</div>' +
        '<div class="gb-consent-actions">' +
            '<button type="button" class="gb-consent-btn gb-consent-primary" data-act="enable">' + esc(t("Ativar e continuar", "Turn on and continue")) + '</button>' +
            '<button type="button" class="gb-consent-btn" data-act="later">' + esc(t("Agora não", "Not now")) + '</button>' +
        '</div>';
    card.addEventListener("click", function (e) {
        var btn = e.target.closest("[data-act]");
        if (!btn) { return; }
        if (btn.getAttribute("data-act") === "later") {
            resolveAgentCard(card, t("Agente não ativado: a resposta em texto continua abaixo.", "Agent not turned on: the text answer stays below."), false);
            return;
        }
        if (typeof A.setAgentEnabled === "function") { A.setAgentEnabled(true); }
        resolveAgentCard(card, t("Agente ativado nesta conversa", "Agent turned on in this conversation"), true);
        A.announce(t("Agente ativado. Enviando o pedido de novo.", "Agent turned on. Sending the request again."));
        var request = String(d.request || "").trim();
        if (request && typeof A.sendFromComposer === "function") { A.sendFromComposer(request); }
    });
    return card;
}

A.frameHandlers.agent_enable_request = function (d) {
    var card = agentCard(d, false);
    if (!A.placeTurnCard(card)) { return; }
    A.announce(t("Pedido de permissão: ativar o Agente nesta conversa.", "Permission request: turn on the Agent in this conversation."));
};

/* ---- Memory ---- */
function memoryBlock(ev) {
    var kind = ev.type;
    var title = kind === "memory_saved" ? t("Memória salva", "Memory saved") :
        kind === "memory_forgot" ? t("Memória removida", "Memory removed") : t("Memória desligada", "Memory off");
    var summary = kind === "memory_saved" ? String(ev.value || "") :
        kind === "memory_forgot" ? (Number(ev.count) === 1 ? t("1 lembrança", "1 memory") : (Number(ev.count) || 0) + " " + t("lembranças", "memories")) :
        t("Nada desta conversa foi guardado", "Nothing from this conversation was kept");
    var undo = kind === "memory_saved" && ev.key
        ? '<button type="button" class="gb-block-btn" data-act="undo">' + esc(t("Desfazer", "Undo")) + '</button>' : '';
    var el = B.block({ kind: "memory", icon: "brain", title: title, summary: summary, status: "done", open: true,
        bodyHtml: '<div class="gb-memory-actions">' +
            '<button type="button" class="gb-block-btn" data-act="settings">' + esc(t("Ver em Configurações › Memória", "See in Settings › Memory")) + '</button>' +
            undo + '</div><p class="gb-memory-status" role="status"></p>' });
    el.addEventListener("click", function (e) {
        var btn = e.target.closest("[data-act]");
        if (!btn || btn.disabled) { return; }
        if (btn.getAttribute("data-act") === "settings") {
            if (typeof A.openSettings === "function") { A.openSettings("memory"); }
            return;
        }
        btn.disabled = true;
        var status = el.querySelector(".gb-memory-status");
        A.apiFetch("/api/user/memory/" + encodeURIComponent(ev.key), { method: "DELETE" }).then(function (r) {
            if (!r.ok && r.status !== 404) { throw new Error("HTTP " + r.status); }
            el.querySelector(".gb-block-title").textContent = t("Memória desfeita", "Memory undone");
            btn.remove();
            status.textContent = t("Esta lembrança não será usada.", "This memory will not be used.");
        }).catch(function () {
            btn.disabled = false;
            status.textContent = t("Não foi possível desfazer agora.", "Could not undo now.");
        });
    });
    return el;
}

function onMemory(ev) {
    var el = memoryBlock(ev);
    if (A.placeTurnCard(el)) {
        A.announce(el.querySelector(".gb-block-title").textContent + ": " + el.querySelector(".gb-block-summary").textContent);
    }
}
A.frameHandlers.memory_saved = onMemory;
A.frameHandlers.memory_forgot = onMemory;
A.frameHandlers.memory_off = onMemory;

/* ---- Reload: the cards stored with an answer ---- */
var extras = A.renderTurnExtras;
A.renderTurnExtras = function (el, m) {
    if (extras) { extras(el, m); }
    var stack = el.querySelector(".gb-block-stack");
    if (!stack) {
        stack = document.createElement("div");
        stack.className = "gb-block-stack";
        el.insertBefore(stack, el.firstChild);
    }
    var ask = A.turnMeta(m, "agent_enable_request");
    if (ask) { stack.appendChild(agentCard(ask, true)); }
    var mem = A.turnMeta(m, "memory_event");
    (Array.isArray(mem) ? mem : mem ? [mem] : []).forEach(function (ev) {
        if (ev && /^memory_(saved|forgot|off)$/.test(ev.type)) { stack.appendChild(memoryBlock(ev)); }
    });
    if (!stack.children.length) { stack.remove(); }
};
})();
