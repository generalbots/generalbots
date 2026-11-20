"use strict";
/**
 * 10g_settings_agents.js - Settings > Modelo > "Agente por assinatura".
 *
 * Claude Code, Codex and Antigravity connect through the agent workspace: an
 * isolated VM where the agent's own program runs and the person signs in
 * with their subscription (/api/ai-accounts). Nothing starts until the
 * person clicks Conectar. The other agents are listed as coming soon.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}

var AGENTS = [
  { id:"claude-code", logo:"anthropic", name:"Claude Code", co:"Anthropic", paste:true },
  { id:"codex-cli", logo:"openai", name:"Codex", co:"OpenAI", paste:false },
  { id:"antigravity", logo:"gemini", name:"Antigravity", co:"Google", paste:true },
  { logo:"copilot", name:"GitHub Copilot", co:"GitHub" }, { logo:"cursor", name:"Cursor Agent", co:"Cursor" },
  { logo:"qwen", name:"Qwen Code", co:"Alibaba" }, { logo:"kimi", name:"Kimi CLI", co:"Moonshot AI" },
  { logo:"mistral", name:"Mistral Vibe", co:"Mistral" }, { logo:"zhipu", name:"GLM Agent", co:"Zhipu" },
  { logo:"ibm", name:"Bob", co:"IBM" }, { logo:"", name:"Junie", co:"JetBrains" }
];

var L = { "pt-BR": {
  intro:"Use a sua própria assinatura do Claude, ChatGPT ou Google. O agente roda dentro de uma máquina virtual isolada, criada para ele neste computador: sua conta e o que ele executa ficam lá dentro, não no seu Windows.",
  connect:"Conectar", disconnect:"Desconectar", soon:"Em breve", linked:"Conectado", connecting:"Conectando…", off:"Não conectado",
  preparing:"Preparando a máquina virtual e instalando o agente. Na primeira vez pode levar alguns minutos.",
  open:"Abrir a página de login", code:"Código", codeHelp:"Na página de login, entre com este código.",
  paste:"Cole aqui o código que a página mostrar", send:"Enviar código",
  use:"Para usar: no chat, escolha o agente no menu de modelos. O botão Agente da conversa fica ligado.",
  unavailable:"A máquina virtual de agentes não está disponível neste computador:", failed:"Não foi possível conectar:" },
en: {
  intro:"Use your own Claude, ChatGPT or Google subscription. The agent runs inside an isolated virtual machine created for it on this computer: your account and what it runs stay there, not on your Windows.",
  connect:"Connect", disconnect:"Disconnect", soon:"Coming soon", linked:"Connected", connecting:"Connecting…", off:"Not connected",
  preparing:"Preparing the virtual machine and installing the agent. The first time can take a few minutes.",
  open:"Open the sign-in page", code:"Code", codeHelp:"On the sign-in page, enter this code.",
  paste:"Paste the code the page shows", send:"Send code",
  use:"To use it: in the chat, pick the agent in the model menu. The conversation's Agent switch turns on.",
  unavailable:"The agent virtual machine is not available on this computer:", failed:"Could not connect:" } };
function g(k) { var l = A.locale && A.locale.startsWith("pt") ? L["pt-BR"] : L.en; return l[k] || L.en[k] || k; }

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

/* One status poll per agent whose sign-in is in progress. */
var polls = {};
function stopPoll(id) { if (polls[id]) { clearInterval(polls[id]); delete polls[id]; } }
function stopAll() { Object.keys(polls).forEach(stopPoll); }
var PENDING = ["connecting", "waiting", "verifying", "starting"];
/* Sign-ins this page started. A sign-in left half-done earlier (status
 * "connecting": the CLI is starting, waiting or verifying) is offered again. */
var started = {};
function markRow(el, a, connecting) {
  var b = el.querySelector('[data-agent="' + a.id + '"] .gb-prov-name .gb-badge');
  if (b) { b.className = connecting ? "gb-badge gb-gar-soon" : "gb-badge"; b.textContent = g(connecting ? "connecting" : "off"); }
}
function statusOf(a, st) {
  var s = (st && st.status) || "unlinked";
  return s === "connecting" && !started[a.id] ? "unlinked" : s;
}

function rowHtml(a, st) {
  if (!a.id) {
    return '<div class="gb-prov-row is-soon" role="listitem">' + logo(a.logo) +
      '<div class="gb-prov-text"><div class="gb-prov-name">' + esc(a.name) + ' <span class="gb-prov-co">' + esc(a.co) + '</span></div></div>' +
      '<span class="gb-badge gb-gar-soon">' + esc(g("soon")) + '</span></div>';
  }
  var status = statusOf(a, st);
  var badge = status === "linked" ? '<span class="gb-prov-badge">' + esc(g("linked")) + '</span>'
    : status === "connecting" ? '<span class="gb-badge gb-gar-soon">' + esc(g("connecting")) + '</span>'
    : '<span class="gb-badge">' + esc(g("off")) + '</span>';
  var btn = status === "linked"
    ? '<button type="button" class="gb-prov-btn" data-agent-off="' + a.id + '">' + esc(g("disconnect")) + '</button>'
    : '<button type="button" class="gb-prov-btn" data-agent-on="' + a.id + '">' + esc(g("connect")) + '</button>';
  return '<div class="gb-prov-agent" data-agent="' + a.id + '"><div class="gb-prov-row is-soon" role="listitem">' + logo(a.logo) +
    '<div class="gb-prov-text"><div class="gb-prov-name">' + esc(a.name) + ' <span class="gb-prov-co">' + esc(a.co) + '</span> ' + badge + '</div></div>' +
    btn + '</div><div class="gb-prov-flow" data-flow="' + a.id + '"></div></div>';
}

/* What the sign-in has shown so far, per agent: a status poll that comes
 * back without the link or the code never takes them off the screen. */
var flows = {};
function show(node, on) { if (node) { node.style.display = on ? "" : "none"; } }
function setText(node, text) { if (node && node.textContent !== text) { node.textContent = text; } }

/** The box is built once per sign-in; later polls only change what differs. */
function flowParts(el, a) {
  var box = el.querySelector('[data-flow="' + a.id + '"]');
  if (!box) { return null; }
  if (!box.querySelector('[data-role="msg"]')) {
    box.innerHTML = '<p class="gb-set-hint" data-role="msg"></p>' +
      '<a class="gb-set-btn-sm gb-set-btn-accent gb-prov-open" data-role="open" target="_blank" rel="noopener noreferrer" style="display:none">' + esc(g("open")) + ' ↗</a>' +
      '<div data-role="codewrap" style="display:none"><p class="gb-set-hint">' + esc(g("codeHelp")) + '</p><div class="gb-prov-url gb-prov-code" data-role="code"></div></div>' +
      '<div class="gb-set-form-actions" data-role="pastewrap" style="display:none"><input class="gb-set-input" data-paste="' + a.id + '" placeholder="' + esc(g("paste")) + '" autocomplete="off"/>' +
        '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" data-send="' + a.id + '">' + esc(g("send")) + '</button></div>' +
      '<div data-role="err"></div>';
    var send = box.querySelector('[data-send="' + a.id + '"]');
    send.addEventListener("click", function () {
      var input = box.querySelector('[data-paste="' + a.id + '"]');
      var value = input ? input.value.trim() : "";
      if (!value) { return; }
      send.disabled = true;
      api("/api/ai-accounts/" + a.id + "/code", { code: value })
        .then(function () { if (input) { input.value = ""; } box.querySelector('[data-role="err"]').innerHTML = ""; })
        .catch(function (e) { box.querySelector('[data-role="err"]').innerHTML = '<p class="gb-set-warn">' + esc(g("failed") + " " + e.message) + '</p>'; })
        .then(function () { send.disabled = false; });
    });
  }
  return box;
}

/** The sign-in step: link, code to type there, or a box for the code it returns. */
function showFlow(el, a, d) {
  var f = flows[a.id] || (flows[a.id] = { url: "", code: "", msg: "" });
  if (d.url) { f.url = d.url; }
  if (d.device_code || d.auth_code) { f.code = d.device_code || d.auth_code; }
  if (d.message) { f.msg = d.message; }
  var box = flowParts(el, a);
  if (!box) { return; }
  setText(box.querySelector('[data-role="msg"]'), f.msg || g("preparing"));
  var open = box.querySelector('[data-role="open"]');
  if (f.url && open.getAttribute("href") !== f.url) { open.setAttribute("href", f.url); }
  show(open, !!f.url);
  setText(box.querySelector('[data-role="code"]'), f.code);
  show(box.querySelector('[data-role="codewrap"]'), !!f.code && !a.paste);
  show(box.querySelector('[data-role="pastewrap"]'), !!f.url && !!a.paste);
}

function watch(el, a) {
  stopPoll(a.id);
  polls[a.id] = setInterval(function () {
    if (!document.body.contains(el)) { stopAll(); return; }
    api("/api/ai-accounts/" + a.id + "/status", {}).then(function (r) {
      var d = (r && r.account) || r || {};
      if (!polls[a.id]) { return; }
      if (PENDING.indexOf(d.status) >= 0) { showFlow(el, a, d); return; }
      stopPoll(a.id);
      delete started[a.id];
      delete flows[a.id];
      redrawRow(el, a, d);
    }).catch(function () { /* keep trying while the page is open */ });
  }, 3000);
}

/** The sign-in ended (linked, error or back to unlinked): this row only is drawn again. */
function redrawRow(el, a, d) {
  var row = el.querySelector('[data-agent="' + a.id + '"]');
  if (!row) { return; }
  row.outerHTML = rowHtml(a, d);
  bindRow(el, a);
  var box = el.querySelector('[data-flow="' + a.id + '"]');
  if (d.status === "error" && box) {
    box.innerHTML = '<p class="gb-set-warn">' + esc(g("failed") + (d.message ? " " + d.message : "")) + '</p>';
  }
  if (d.status === "linked") { refreshPickers(); }
  if (d.status === "linked" && !el.querySelector("[data-agent-use]")) {
    var list = el.querySelector(".gb-prov-list");
    if (list) { list.insertAdjacentHTML("afterend", '<p class="gb-set-hint" data-agent-use>' + esc(g("use")) + '</p>'); }
  }
}

function bindRow(el, a) {
  var on = el.querySelector('[data-agent-on="' + a.id + '"]');
  if (on && A.productConfig && A.productConfig.agent_backend_unavailable) { on.disabled = true; }
  if (on) {
    on.addEventListener("click", function () {
      on.disabled = true;
      started[a.id] = true;
      delete flows[a.id];
      markRow(el, a, true);
      var fresh = el.querySelector('[data-flow="' + a.id + '"]');
      if (fresh) { fresh.innerHTML = ""; }
      showFlow(el, a, {});
      api("/api/ai-accounts/" + a.id + "/connect", {})
        .then(function (d) { showFlow(el, a, d); watch(el, a); })
        .catch(function (e) {
          delete started[a.id];
          markRow(el, a, false);
          on.disabled = false;
          var box = el.querySelector('[data-flow="' + a.id + '"]');
          if (box) { box.innerHTML = '<p class="gb-set-warn">' + esc(g("failed") + " " + e.message) + '</p>'; }
        });
    });
  }
  var off = el.querySelector('[data-agent-off="' + a.id + '"]');
  if (off) {
    off.addEventListener("click", function () {
      off.disabled = true;
      delete started[a.id];
      api("/api/ai-accounts/" + a.id + "/disconnect", {}).catch(function () {}).then(function () { renderAgents(el); });
    });
  }
}

function renderAgents(el) {
  var reason = A.productConfig && A.productConfig.agent_backend_unavailable;
  stopAll();
  api("/api/ai-accounts").catch(function () { return { accounts: [] }; }).then(function (data) {
    var byId = {};
    (data.accounts || []).forEach(function (x) { byId[x.provider] = x; });
    el.innerHTML = '<div class="gb-prov">' +
      '<p class="gb-set-desc">' + esc(g("intro")) + '</p>' +
      (reason ? '<p class="gb-set-warn">' + esc(g("unavailable")) + ' ' + esc(reason) + '</p>' : '') +
      '<div class="gb-prov-list" role="list">' + AGENTS.map(function (a) { return rowHtml(a, byId[a.id]); }).join("") + '</div>' +
      (AGENTS.some(function (a) { return a.id && byId[a.id] && byId[a.id].status === "linked"; })
        ? '<p class="gb-set-hint" data-agent-use>' + esc(g("use")) + '</p>' : '') + '</div>';
    AGENTS.forEach(function (a) {
      if (!a.id) { return; }
      bindRow(el, a);
      var row = byId[a.id];
      if (statusOf(a, row) === "connecting") { watch(el, a); }
      else if (row && row.vm_name && row.status !== "linked") { checkOnce(el, a); }
    });
  });
}

/* A sign-in finished elsewhere: one status call lets the server see the
 * credentials in the workspace and mark the account linked. */
function checkOnce(el, a) {
  api("/api/ai-accounts/" + a.id + "/status", {}).then(function (r) {
    var d = (r && r.account) || r || {};
    if (d.status === "linked" && !started[a.id]) { redrawRow(el, a, d); }
  }).catch(function () { /* the row keeps what the list said */ });
}
/* The chat's model menu lists linked agents: read them again after a change. */
function refreshPickers() {
  if (!A.loadLinkedAgents) { return; }
  A.loadLinkedAgents().then(function () { if (A.renderModelPickers) { A.renderModelPickers(); } });
}

if (A._modelProviders) { A._modelProviders.renderAcp = renderAgents; }
/* The chat's model menu lists the connected agents (05b_model_picker.js). */
A.loadLinkedAgents = function () {
  return api("/api/ai-accounts").then(function (data) {
    A.linkedAgents = (data.accounts || []).filter(function (x) { return x.status === "linked"; })
      .map(function (x) {
        var meta = AGENTS.filter(function (a) { return a.id === x.provider; })[0] || {};
        return { id: x.provider, name: meta.name || x.provider, logo: meta.logo || "" };
      });
  }).catch(function () { A.linkedAgents = []; });
};
})();
