"use strict";
/**
 * 10g_settings_agents.js - Settings > Modelo > "Agente por assinatura".
 *
 * Claude Code, Codex and Antigravity connect through the agent workspace: an
 * isolated VM where the agent's own program runs and the person signs in
 * with their subscription (/api/ai-accounts). Nothing starts until the
 * person clicks Conectar.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}

var AGENTS = [
  { id:"claude-code", logo:"anthropic", name:"Claude Code", sub:"Claude", co:"Anthropic", paste:true },
  { id:"codex-cli", logo:"openai", name:"Codex", sub:"ChatGPT", co:"OpenAI", paste:false },
  { id:"antigravity", logo:"gemini", name:"Antigravity", sub:"Gemini", co:"Google", paste:true }
];

var L = { "pt-BR": {
  title:"Agentes por assinatura", tip:"O agente roda separado do resto do sistema, numa máquina virtual isolada neste computador. Sem ela, o modo agente fica desligado.",
  space:"ESPAÇO ISOLADO", spaceOff:"INDISPONÍVEL",
  picked:"Em uso no chat. As conversas com este agente ligam a chave Agente: ele trabalha na máquina virtual isolada deste computador.",
  connect:"Conectar", disconnect:"Sair da conta", useNow:"Usar no chat", linked:"CONECTADO", connecting:"CONECTANDO…", off:"NÃO CONECTADO",
  preparing:"Preparando a máquina virtual e instalando o agente. Na primeira vez pode levar alguns minutos.",
  open:"Abrir página de login", code:"Código", codeHelp:"Na página de login, entre com este código.",
  signIn:"Entre na sua conta na página de login.", signInPaste:"Entre na sua conta na página de login e cole aqui o código que ela mostrar.", verifying:"Conferindo o código…",
  paste:"Cole aqui o código que a página mostrar", send:"Enviar código",
  unavailable:"A máquina virtual de agentes não está disponível neste computador:", failed:"Não foi possível conectar:",
  models:"MODELOS NO MENU DO CHAT", modelsTip:"Ligue os modelos deste agente que devem aparecer no menu do chat.",
  count:"%n DE %t LIGADOS", def:"PADRÃO", saving:"Salvando…", saved:"Salvo.",
  loading:"Lendo os modelos do agente…", none:"O agente não informou modelos.", readFail:"Não foi possível ler os modelos do agente:", saveFail:"Não foi possível salvar:",
  vmStopped:"MÁQUINA DESLIGADA", vmStarting:"MÁQUINA INICIANDO", vmUnavailable:"MÁQUINA INDISPONÍVEL",
  vmStoppedMsg:"A máquina virtual está desligada. O login continua quando ela ligar.",
  vmStartingMsg:"A máquina virtual está iniciando. O login continua em instantes.",
  vmUnavailableMsg:"A máquina virtual não está disponível agora. Tentaremos de novo em instantes." },
en: {
  title:"Subscription agents", tip:"The agent runs apart from the rest of the system, in an isolated virtual machine on this computer. Without it, agent mode stays off.",
  space:"ISOLATED SPACE", spaceOff:"UNAVAILABLE",
  picked:"In use in the chat. Conversations with this agent turn the Agent switch on: it works in this computer’s isolated virtual machine.",
  connect:"Connect", disconnect:"Sign out", useNow:"Use in chat", linked:"CONNECTED", connecting:"CONNECTING…", off:"NOT CONNECTED",
  preparing:"Preparing the virtual machine and installing the agent. The first time can take a few minutes.",
  open:"Open sign-in page", code:"Code", codeHelp:"On the sign-in page, enter this code.",
  signIn:"Sign in to your account on the sign-in page.", signInPaste:"Sign in on the sign-in page, then paste here the code it shows.", verifying:"Checking the code…",
  paste:"Paste the code the page shows", send:"Send code",
  unavailable:"The agent virtual machine is not available on this computer:", failed:"Could not connect:",
  models:"MODELS IN THE CHAT MENU", modelsTip:"Switch on the models of this agent that should appear in the chat menu.",
  count:"%n OF %t ON", def:"DEFAULT", saving:"Saving…", saved:"Saved.",
  loading:"Reading the agent's models…", none:"The agent reported no models.", readFail:"Could not read the agent's models:", saveFail:"Could not save:",
  vmStopped:"MACHINE OFF", vmStarting:"MACHINE STARTING", vmUnavailable:"MACHINE UNAVAILABLE",
  vmStoppedMsg:"The virtual machine is off. The sign-in continues when it starts.",
  vmStartingMsg:"The virtual machine is starting. The sign-in continues shortly.",
  vmUnavailableMsg:"The virtual machine is not available right now. Trying again shortly." } };
function g(k) { var l = A.locale && A.locale.startsWith("pt") ? L["pt-BR"] : L.en; return l[k] || L.en[k] || k; }

function logo(id) {
  var svg = (A._providerLogos || {})[id];
  return '<span class="gb-of-logo" aria-hidden="true">' + (svg || '') + '</span>';
}
/* Reads carry a 15 s deadline (02b_requests.js); connect and sign-in calls
 * run as long as the server needs. */
function api(path, body, method) {
  if (body === undefined) {
    return A.fetchJSON(path, null, 15000).then(function (res) {
      var d = res.data || {};
      if (!res.ok) { throw new Error(d.message || d.error || ("HTTP " + res.status)); }
      return d;
    });
  }
  return A.apiFetch(path, { method: method || "POST", body: JSON.stringify(body) })
    .then(function (r) {
      return r.json().catch(function () { return {}; }).then(function (d) {
        if (!r.ok) { throw new Error(d.message || d.error || ("HTTP " + r.status)); }
        return d;
      });
    });
}
/* The sign-in status, as a background request: 8 s deadline, never two at
 * once for the same agent. A quick {"status": "stopped" | "starting" |
 * "unavailable"} (or a 503) is the workspace's state, not the account's. */
function statusPoll(a) {
  return A.pollJSON("/api/ai-accounts/" + a.id + "/status", { init: { method: "POST", body: "{}" } }).then(function (res) {
    var d = res.data || {};
    res.vm = d.account || d.provider ? null : A.vmState(res);
    res.account = d.account || d;
    return res;
  });
}

/* One status poll per agent whose sign-in is in progress. */
var polls = {};
function stopPoll(id) { if (polls[id]) { polls[id].stop(); delete polls[id]; } }
var WATCH_MS = 3000, WATCH_BACKOFF = 30000;
var VM_BADGE = { stopped: ["gb-of-muc", "vmStopped"], starting: ["gb-of-warnc", "vmStarting"], unavailable: ["gb-of-muc", "vmUnavailable"] };
function stopAll() { Object.keys(polls).forEach(stopPoll); }
var PENDING = ["connecting", "waiting", "verifying", "starting"];
/* Sign-ins this page started. A sign-in left half-done earlier (status
 * "connecting": the CLI is starting, waiting or verifying) is offered again. */
var started = {};
function badge(status, vm) {
  var k = VM_BADGE[vm] || (status === "linked" ? ["gb-of-ok", "linked"] : status === "connecting" ? ["gb-of-warnc", "connecting"] : ["gb-of-muc", "off"]);
  return '<span class="gb-of-ag-st ' + k[0] + '"><span class="gb-of-sq"></span>' + esc(g(k[1])) + '</span>';
}
function markRow(el, a, connecting, vm) {
  var b = el.querySelector('[data-agent="' + a.id + '"] .gb-of-ag-st');
  var html = badge(connecting ? "connecting" : "unlinked", vm);
  if (b && b.outerHTML !== html) { b.outerHTML = html; }
}
function statusOf(a, st) {
  var s = (st && st.status) || "unlinked";
  return s === "connecting" && !started[a.id] ? "unlinked" : s;
}

function rowHtml(a, st) {
  var status = statusOf(a, st);
  var btn = status === "linked"
    ? '<button type="button" class="gb-of-btn" data-agent-use="' + a.id + '">' + esc(g("useNow")) + '</button>' +
      '<button type="button" class="gb-of-btn" data-agent-off="' + a.id + '">' + esc(g("disconnect")) + '</button>'
    : '<button type="button" class="gb-of-btn" data-agent-on="' + a.id + '">' + esc(g("connect")) + '</button>';
  return '<div class="gb-of-ag" data-agent="' + a.id + '" role="listitem"><div class="gb-of-ag-r">' + logo(a.logo) +
    '<div><b>' + esc(a.sub) + '</b><span class="tool">' + esc(a.name + " · " + a.co) + '</span></div>' + badge(status) + btn + '</div>' +
    '<div class="gb-prov-flow" data-flow="' + a.id + '"></div>' + (status === "linked" ? '<div data-agm="' + a.id + '"></div>' : '') + '</div>';
}

/* The agent's models (/api/ai-accounts/:provider/models): the ones switched
 * on appear in the chat's model menu; the list folds under its header. */
var openModels = {};
var HINT = 'class="gb-of-hint" style="margin:0;padding:10px 16px;border-top:1px solid var(--id-line)"';
function mountModels(el, a) {
  var box = el.querySelector('[data-agm="' + a.id + '"]');
  if (!box) { return; }
  var G = A._modelGarage, list = [], on = {}, savedOn = {};
  var saveTimer = null, saving = false, pendingSave = false;
  box.innerHTML = '<section class="gb-of-km"><button type="button" class="gb-of-km-h" aria-expanded="false" style="width:100%;border:0;font:inherit;color:inherit;text-align:start;cursor:pointer">' +
    '<span class="gb-of-k">' + (G ? G.sl() : '') + esc(g("models")) + '</span>' + (G ? G.help(g("modelsTip")) : '') +
    '<span class="gb-of-sp"></span><span class="gb-of-meta" data-role="count"></span>' +
    '<svg data-role="chev" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="margin:0 6px"><path d="M6 9l6 6 6-6"/></svg></button>' +
    '<div data-role="body"></div><div class="gb-of-say" role="status" data-role="msg"></div></section>';
  var head = box.querySelector(".gb-of-km-h"), body = box.querySelector('[data-role="body"]');
  function say(cls, text) {
    box.querySelector('[data-role="msg"]').innerHTML = text ? '<p class="' + cls + '" style="padding:10px 16px;border-top:1px solid var(--id-line)">' + esc(text) + '</p>' : "";
  }
  function count() {
    var n = list.filter(function (m) { return on[m.id]; }).length;
    box.querySelector('[data-role="count"]').textContent = list.length ? g("count").replace("%n", String(n)).replace("%t", String(list.length)) : "";
  }
  function fold() {
    var open = !!openModels[a.id];
    head.setAttribute("aria-expanded", String(open));
    box.querySelector('[data-role="chev"]').style.transform = open ? "rotate(180deg)" : "";
    body.style.display = open ? "" : "none";
  }
  function draw() {
    count();
    if (!list.length) { body.innerHTML = '<p ' + HINT + '>' + esc(g("none")) + '</p>'; return; }
    body.innerHTML = '<div class="gb-of-km-list" role="group" aria-label="' + esc(g("models")) + '">' + list.map(function (m) {
      return '<label class="gb-of-km-r' + (on[m.id] ? ' on' : '') + '"><i></i><span><b>' + esc(A.agentModelLabel(m)) + '</b>' +
        (m["default"] ? '<span class="gb-of-chip xs mute">' + esc(g("def")) + '</span>' : '') +
        '<input type="checkbox" class="gb-of-tg" data-model="' + esc(m.id) + '"' + (on[m.id] ? ' checked' : '') + '/></span></label>';
    }).join("") + '</div>';
    body.querySelectorAll(".gb-of-km-r input").forEach(function (cb) {
      cb.addEventListener("change", function () {
        on[cb.getAttribute("data-model")] = cb.checked;
        cb.closest(".gb-of-km-r").classList.toggle("on", cb.checked);
        count();
        scheduleSave();
      });
    });
  }
  function scheduleSave() {
    clearTimeout(saveTimer);
    if (saving) { pendingSave = true; return; }
    saveTimer = setTimeout(doSave, 400);
  }
  function revertSwitches() {
    list.forEach(function (m) { on[m.id] = !!savedOn[m.id]; });
    draw(); fold();
  }
  function doSave() {
    saving = true;
    say("gb-set-hint", g("saving"));
    var enabled = list.filter(function (m) { return on[m.id]; }).map(function (m) { return m.id; });
    var snapshot = {};
    list.forEach(function (m) { snapshot[m.id] = !!on[m.id]; });
    api("/api/ai-accounts/" + a.id + "/models", { enabled: enabled }, "PUT")
      .then(function () {
        list.forEach(function (m) { savedOn[m.id] = !!snapshot[m.id]; });
        say("gb-set-hint gb-set-ok", g("saved"));
        if (A.loadModels) { return A.loadModels().then(function () { if (A.renderModelPickers) { A.renderModelPickers(); } }); }
      })
      .catch(function (e) {
        say("gb-set-warn", g("saveFail") + " " + e.message);
        pendingSave = false;
        revertSwitches();
      })
      .then(function () {
        saving = false;
        if (pendingSave) { pendingSave = false; scheduleSave(); }
      });
  }
  head.addEventListener("click", function () { openModels[a.id] = !openModels[a.id]; fold(); });
  fold();
  body.innerHTML = '<p ' + HINT + '>' + esc(g("loading")) + '</p>';
  api("/api/ai-accounts/" + a.id + "/models").then(function (d) {
    list = (d && d.models) || [];
    list.forEach(function (m) { on[m.id] = !!m.enabled; savedOn[m.id] = !!m.enabled; });
    draw();
  }).catch(function (e) { body.innerHTML = '<p class="gb-set-warn" style="margin:0;padding:10px 16px;border-top:1px solid var(--id-line)">' + esc(g("readFail") + " " + e.message) + '</p>'; });
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
    box.innerHTML = '<div class="gb-ag-wait" data-role="wait" role="status"><span class="gb-ag-spin" data-role="spin" aria-hidden="true"></span><span data-role="msg"></span></div>' +
      '<a class="gb-of-btn acc gb-prov-open" data-role="open" target="_blank" rel="noopener noreferrer" style="display:none">' + esc(g("open")) + ' ↗</a>' +
      '<div data-role="codewrap" style="display:none"><p class="gb-set-hint">' + esc(g("codeHelp")) + '</p><div class="gb-prov-url gb-prov-code" data-role="code"></div></div>' +
      '<div class="gb-set-form-actions" data-role="pastewrap" style="display:none"><input class="gb-of-in" data-paste="' + a.id + '" placeholder="' + esc(g("paste")) + '" autocomplete="off" aria-label="' + esc(g("paste")) + '"/>' +
        '<button type="button" class="gb-of-btn pri" data-send="' + a.id + '">' + esc(g("send")) + '</button></div>' +
      '<div data-role="err"></div>';
    box.querySelector('[data-role="open"]').addEventListener("click", function (e) {
      var href = e.currentTarget.getAttribute("href");
      if (href && openOutside(href)) { e.preventDefault(); }
    });
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

/* The desktop app opens the page in the person's own browser (opener
 * plugin); a plain browser keeps the visible button. */
function openOutside(url) {
  var tauri = window.__TAURI_INTERNALS__;
  if (!tauri || typeof tauri.invoke !== "function" || !/^https:\/\//.test(url)) { return false; }
  tauri.invoke("plugin:opener|open_url", { url: url }).catch(function () { window.open(url, "_blank", "noopener"); });
  return true;
}

/** The sign-in step: link, code to type there, or a box for the code it returns.
 *  The words are the shell's own per step; a workspace message (vm*Msg)
 *  replaces them while the machine is off or starting. */
function showFlow(el, a, d) {
  var f = flows[a.id] || (flows[a.id] = { url: "", code: "", msg: "", opened: false });
  if (d.url) { f.url = d.url; }
  if (d.device_code || d.auth_code) { f.code = d.device_code || d.auth_code; }
  f.msg = d.vmMsg || "";
  var box = flowParts(el, a);
  if (!box) { return; }
  var busy = !f.url || d.state === "verifying" || !!d.vmMsg;
  var text = f.msg || (d.state === "verifying" ? g("verifying") : !f.url ? g("preparing") : a.paste ? g("signInPaste") : g("signIn"));
  setText(box.querySelector('[data-role="msg"]'), text);
  show(box.querySelector('[data-role="spin"]'), busy);
  box.querySelector('[data-role="wait"]').classList.toggle("idle", !busy);
  if (f.url && !f.opened && started[a.id]) { f.opened = true; openOutside(f.url); }
  var open = box.querySelector('[data-role="open"]');
  if (f.url && open.getAttribute("href") !== f.url) { open.setAttribute("href", f.url); }
  show(open, !!f.url);
  setText(box.querySelector('[data-role="code"]'), f.code);
  show(box.querySelector('[data-role="codewrap"]'), !!f.code && !a.paste);
  show(box.querySelector('[data-role="pastewrap"]'), !!f.url && !!a.paste);
}

function watch(el, a) {
  stopPoll(a.id);
  var p = polls[a.id] = A.poller(function () {
    if (!document.body.contains(el)) { stopAll(); return; }
    return statusPoll(a).then(function (res) {
      if (polls[a.id] !== p || res.skipped) { return WATCH_MS; }
      if (res.vm) {
        markRow(el, a, true, res.vm);
        showFlow(el, a, { vmMsg: g("vm" + res.vm.charAt(0).toUpperCase() + res.vm.slice(1) + "Msg") });
        return res.vm === "starting" ? WATCH_MS : WATCH_BACKOFF;
      }
      if (res.failed) { return WATCH_BACKOFF; }
      var d = res.account;
      if (PENDING.indexOf(d.status) >= 0) { markRow(el, a, true); showFlow(el, a, d); return; }
      stopPoll(a.id);
      delete started[a.id];
      delete flows[a.id];
      redrawRow(el, a, d);
    });
  }, { every: WATCH_MS, backoff: WATCH_BACKOFF });
  p.start(WATCH_MS);
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
}

function bindRow(el, a) {
  mountModels(el, a);
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
  /* Same as picking the agent in the chat's model menu (the conversation's
   * Agent switch turns on and the next message goes to this agent), but the
   * page stays open: the choice shows in "Motor em uso" and in the row. */
  var use = el.querySelector('[data-agent-use="' + a.id + '"]');
  if (use) {
    use.addEventListener("click", function () {
      var box = el.querySelector('[data-flow="' + a.id + '"]');
      var off = A.productConfig && A.productConfig.agent_backend_unavailable;
      if (off) { if (box) { box.innerHTML = '<p class="gb-set-warn">' + esc(g("unavailable") + " " + off) + '</p>'; } return; }
      var current = A.selectedModel || "";
      var value = current === "sub:" + a.id || current.indexOf("sub:" + a.id + "/") === 0 ? current : "sub:" + a.id;
      A.pickEngine(value, use, function () { if (box) { box.innerHTML = '<p class="gb-set-hint gb-set-ok gb-of-picked">' + esc(g("picked")) + '</p>'; } });
    });
  }
  var off = el.querySelector('[data-agent-off="' + a.id + '"]');
  if (off) {
    off.addEventListener("click", function () {
      off.disabled = true;
      delete started[a.id];
      api("/api/ai-accounts/" + a.id + "/disconnect", {}).catch(function () {}).then(function () { renderAgents(el); refreshPickers(); });
    });
  }
}

function renderAgents(el) {
  var reason = A.productConfig && A.productConfig.agent_backend_unavailable;
  stopAll();
  api("/api/ai-accounts").catch(function () { return { accounts: [] }; }).then(function (data) {
    var byId = {};
    (data.accounts || []).forEach(function (x) { byId[x.provider] = x; });
    var G = A._modelGarage;
    el.innerHTML = '<div class="gb-of-sec"><div class="gb-of-sec-h"><span class="gb-of-letter">A</span><span class="gb-of-sec-t">' + esc(g("title")) + '</span>' +
      (G ? G.help(g("tip")) : '') + '<span class="gb-of-sp"></span><span class="gb-of-ag-st ' + (reason ? 'gb-of-dngc' : 'gb-of-ok') + '" style="padding-right:6px">' +
      '<span class="gb-of-sq"></span>' + esc(reason ? g("spaceOff") : g("space") + (/Windows/i.test(navigator.userAgent || "") ? " · WSL2" : "")) + '</span></div>' +
      (reason ? '<div class="gb-of-sec-b" style="border-bottom:1px solid var(--id-line)"><p class="gb-set-warn" style="margin:0">' + esc(g("unavailable")) + ' ' + esc(reason) + '</p></div>' : '') +
      '<div role="list">' + AGENTS.map(function (a) { return rowHtml(a, byId[a.id]); }).join("") + '</div></div>';
    AGENTS.forEach(function (a) {
      bindRow(el, a);
      var row = byId[a.id];
      if (statusOf(a, row) === "connecting") { watch(el, a); }
      else if (row && row.vm_name && row.status !== "linked") { checkOnce(el, a); }
    });
  });
}

/* A sign-in finished elsewhere: one status call lets the server see the
 * credentials in the workspace and mark the account linked. */
function checkOnce(el, a, tries) {
  statusPoll(a).then(function (res) {
    if (res.skipped) {
      tries = (tries || 0) + 1;
      if (tries < 5) { setTimeout(function () { if (document.body.contains(el)) { checkOnce(el, a, tries); } }, 1500); }
      return;
    }
    if (res.vm) { if (!started[a.id]) { markRow(el, a, false, res.vm); } return; }
    var d = res.account || {};
    if (res.ok && d.status === "linked" && !started[a.id]) { redrawRow(el, a, d); }
  });
}
/* The chat's model menu lists linked agents: read them again after a change. */
function refreshPickers() {
  if (!A.loadLinkedAgents) { return; }
  Promise.all([A.loadLinkedAgents(), A.loadModels ? A.loadModels() : null])
    .then(function () { if (A.renderModelPickers) { A.renderModelPickers(); } });
}

/** Look up an agent's display metadata by provider id. */
A._agentMeta = function (provider) {
    for (var i = 0; i < AGENTS.length; i++) {
        if (AGENTS[i].id === provider) { return AGENTS[i]; }
    }
    return null;
};
if (A._modelProviders) { A._modelProviders.renderAcp = renderAgents; }
else { A._acpRenderPending = renderAgents; }
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
