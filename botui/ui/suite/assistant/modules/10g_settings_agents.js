"use strict";
/**
 * 10g_settings_agents.js - Settings > Modelo > "Agente": subscription agents.
 *
 * Claude Code, Codex and Antigravity connect through the agent workspace: an
 * isolated VM where the agent's own program runs and the person signs in
 * with their subscription (/api/ai-accounts). Nothing starts until the
 * person clicks Conectar. The other agents are listed as coming soon.
 *
 * Uses main's existing /api/ai-accounts surface. The status endpoint wraps
 * the account as { account, checked_at, note }, so the poll unwraps it.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function tr(ptText, enText) { return A.locale && A.locale.startsWith("pt") ? ptText : enText; }

var AGENTS = [
  { id:"claude-code", logo:"anthropic", name:"Claude Code", co:"Anthropic", paste:true },
  { id:"codex-cli", logo:"openai", name:"Codex", co:"OpenAI", paste:false },
  { id:"antigravity", logo:"gemini", name:"Antigravity", co:"Google", paste:true },
  { logo:"copilot", name:"GitHub Copilot", co:"GitHub" }, { logo:"cursor", name:"Cursor Agent", co:"Cursor" },
  { logo:"qwen", name:"Qwen Code", co:"Alibaba" }, { logo:"kimi", name:"Kimi CLI", co:"Moonshot AI" },
  { logo:"mistral", name:"Mistral Vibe", co:"Mistral" }, { logo:"zhipu", name:"GLM Agent", co:"Zhipu" },
  { logo:"ibm", name:"Bob", co:"IBM" }, { logo:"", name:"Junie", co:"JetBrains" }
];

function logo(id) {
  var svg = (A._providerLogos || {})[id];
  return '<span class="gb-prov-logo">' + (svg || '') + '</span>';
}
function api(path, body) {
  return A.apiFetch(path, body === undefined ? undefined : { method: "POST", body: JSON.stringify(body) })
    .then(function (r) {
      return r.json().catch(function () { return {}; }).then(function (d) {
        if (!r.ok) { throw new Error(d.message || d.error || ("HTTP " + r.status)); }
        return d;
      });
    });
}

var poll = null;
function stopPoll() { if (poll) { clearInterval(poll); poll = null; } }

function rowHtml(a, st) {
  if (!a.id) {
    return '<div class="gb-prov-row is-soon" role="listitem">' + logo(a.logo) +
      '<div class="gb-prov-text"><div class="gb-prov-name">' + esc(a.name) + ' <span class="gb-prov-co">' + esc(a.co) + '</span></div></div>' +
      '<span class="gb-badge gb-gar-soon">' + esc(tr("Em breve", "Coming soon")) + '</span></div>';
  }
  var status = (st && st.status) || "unlinked";
  var badge = status === "linked" ? '<span class="gb-prov-badge">' + esc(tr("Conectado", "Connected")) + '</span>'
    : status === "connecting" ? '<span class="gb-badge gb-gar-soon">' + esc(tr("Conectando…", "Connecting…")) + '</span>' : '';
  var btn = status === "linked"
    ? '<button type="button" class="gb-prov-btn" data-agent-off="' + a.id + '">' + esc(tr("Desconectar", "Disconnect")) + '</button>'
    : '<button type="button" class="gb-prov-btn" data-agent-on="' + a.id + '">' + esc(tr("Conectar", "Connect")) + '</button>';
  return '<div class="gb-prov-agent" data-agent="' + a.id + '"><div class="gb-prov-row is-soon" role="listitem">' + logo(a.logo) +
    '<div class="gb-prov-text"><div class="gb-prov-name">' + esc(a.name) + ' <span class="gb-prov-co">' + esc(a.co) + '</span> ' + badge + '</div></div>' +
    btn + '</div><div class="gb-prov-flow" data-flow="' + a.id + '"></div></div>';
}

/** The sign-in step: link, code to type there, or a box for the code it returns. */
function showFlow(el, a, d) {
  var box = el.querySelector('[data-flow="' + a.id + '"]');
  if (!box) { return; }
  var code = d.device_code || d.auth_code || "";
  var html = '<p class="gb-set-hint">' + esc(d.message || tr(
    "Preparando a máquina virtual e instalando o agente. Na primeira vez pode levar alguns minutos.",
    "Preparing the virtual machine and installing the agent. The first time can take a few minutes.")) + '</p>';
  if (d.url) {
    html += '<a class="gb-set-btn-sm gb-set-btn-accent gb-prov-open" href="' + esc(d.url) + '" target="_blank" rel="noopener noreferrer">' +
      esc(tr("Abrir a página de login", "Open the sign-in page")) + ' ↗</a>';
  }
  if (code && !a.paste) {
    html += '<p class="gb-set-hint">' + esc(tr("Na página de login, entre com este código.", "On the sign-in page, enter this code.")) +
      '</p><div class="gb-prov-url gb-prov-code">' + esc(code) + '</div>';
  }
  if (d.url && a.paste) {
    html += '<div class="gb-set-form-actions"><input class="gb-set-input" data-paste="' + a.id + '" placeholder="' +
      esc(tr("Cole aqui o código que a página mostrar", "Paste the code the page shows")) + '" autocomplete="off"/>' +
      '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" data-send="' + a.id + '">' +
      esc(tr("Enviar código", "Send code")) + '</button></div>';
  }
  var focused = document.activeElement && document.activeElement.getAttribute("data-paste") === a.id;
  if (!focused) { box.innerHTML = html; }
  var send = box.querySelector('[data-send="' + a.id + '"]');
  if (send) {
    send.addEventListener("click", function () {
      var input = box.querySelector('[data-paste="' + a.id + '"]');
      var value = input ? input.value.trim() : "";
      if (!value) { return; }
      send.disabled = true;
      api("/api/ai-accounts/" + a.id + "/code", { code: value })
        .then(function () { if (input) { input.value = ""; } })
        .catch(function (e) {
          console.debug("gbAgents: code submission failed", e);
          box.insertAdjacentHTML("beforeend", '<p class="gb-set-warn">' + esc(tr("Não foi possível conectar:", "Could not connect:") + " " + e.message) + '</p>');
        })
        .then(function () { send.disabled = false; });
    });
  }
}

function watch(el, a) {
  stopPoll();
  poll = setInterval(function () {
    if (!document.body.contains(el)) { stopPoll(); return; }
    api("/api/ai-accounts/" + a.id + "/status", {}).then(function (d) {
      /* main returns { account: {...}, checked_at, note } — not a flat account. */
      var s = d.account || d;
      if (s.status === "linked" || s.status === "error") { stopPoll(); renderAgents(el); return; }
      showFlow(el, a, s);
    }).catch(function (e) {
      console.debug("gbAgents: status poll failed, retrying while the page is open", e);
    });
  }, 3000);
}

function renderAgents(el) {
  var reason = A.productConfig && A.productConfig.agent_backend_unavailable;
  api("/api/ai-accounts").catch(function (e) {
    console.debug("gbAgents: could not list accounts", e);
    return { accounts: [] };
  }).then(function (data) {
    var byId = {};
    (data.accounts || []).forEach(function (x) { byId[x.provider] = x; });
    var useHint = tr("Para usar: no chat, escolha o agente no menu de modelos. O botão Agente da conversa fica ligado.",
      "To use it: in the chat, pick the agent in the model menu. The conversation's Agent switch turns on.");
    el.innerHTML = '<div class="gb-prov">' +
      '<p class="gb-set-desc">' + esc(tr(
        "Use um agente que você já assina. Ele roda dentro de uma máquina virtual isolada, criada para ele neste computador: sua conta e o que ele executa ficam lá dentro, não no seu Windows.",
        "Use an agent you already subscribe to. It runs inside an isolated virtual machine created for it on this computer: your account and what it runs stay there, not on your Windows.")) + '</p>' +
      (reason ? '<p class="gb-set-warn">' + esc(tr("A máquina virtual de agentes não está disponível neste computador:",
        "The agent virtual machine is not available on this computer:")) + ' ' + esc(reason) + '</p>' : '') +
      '<div class="gb-prov-list" role="list">' + AGENTS.map(function (a) { return rowHtml(a, byId[a.id]); }).join("") + '</div>' +
      (AGENTS.some(function (a) { return a.id && byId[a.id] && byId[a.id].status === "linked"; })
        ? '<p class="gb-set-hint">' + esc(useHint) + '</p>' : '') + '</div>';
    if (reason) { el.querySelectorAll("[data-agent-on]").forEach(function (b) { b.disabled = true; }); }
    AGENTS.forEach(function (a) {
      if (!a.id) { return; }
      var on = el.querySelector('[data-agent-on="' + a.id + '"]');
      if (on) {
        on.addEventListener("click", function () {
          on.disabled = true;
          showFlow(el, a, {});
          api("/api/ai-accounts/" + a.id + "/connect", {})
            .then(function (d) { showFlow(el, a, d); watch(el, a); })
            .catch(function (e) {
              on.disabled = false;
              var box = el.querySelector('[data-flow="' + a.id + '"]');
              if (box) { box.innerHTML = '<p class="gb-set-warn">' +
                esc(tr("Não foi possível conectar:", "Could not connect:") + " " + e.message) + '</p>'; }
            });
        });
      }
      var off = el.querySelector('[data-agent-off="' + a.id + '"]');
      if (off) {
        off.addEventListener("click", function () {
          off.disabled = true;
          /* Disconnect is best-effort: refresh the card either way. */
          api("/api/ai-accounts/" + a.id + "/disconnect", {})
            .catch(function (e) { console.debug("gbAgents: disconnect failed, refreshing status", e); })
            .then(function () { renderAgents(el); });
        });
      }
      if (byId[a.id] && byId[a.id].status === "connecting") { watch(el, a); }
    });
  });
}

/* The chat model menu lists connected agents (05b_model_picker.js). */
A.loadLinkedAgents = function () {
  return api("/api/ai-accounts").then(function (data) {
    A.linkedAgents = (data.accounts || []).filter(function (x) { return x.status === "linked"; })
      .map(function (x) {
        var meta = AGENTS.filter(function (a) { return a.id === x.provider; })[0] || {};
        return { id: x.provider, name: meta.name || x.provider, logo: meta.logo || "" };
      });
  }).catch(function () { A.linkedAgents = []; });
};

/* Wire the ACP detail pane; tolerate either script load order. */
if (A._modelProviders) { A._modelProviders.renderAcp = renderAgents; }
else { A._acpRenderPending = renderAgents; }
})();
