"use strict";
/**
 * 10g_settings_agents.js - Settings > Modelo > "Agente por assinatura".
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
  { id:"claude-code", logo:"anthropic", name:"Claude Code", sub:"Claude", co:"Anthropic", paste:true },
  { id:"codex-cli", logo:"openai", name:"Codex", sub:"ChatGPT", co:"OpenAI", paste:false },
  { id:"antigravity", logo:"gemini", name:"Antigravity", sub:"Gemini", co:"Google", paste:true }
];

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
        .then(function () { if (input) { input.value = ""; } })
        .catch(function (e) {
          console.debug("gbAgents: code submission failed", e);
          box.insertAdjacentHTML("beforeend", '<p class="gb-set-warn">' + esc(tr("Não foi possível conectar:", "Could not connect:") + " " + e.message) + '</p>');
        })
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
      api("/api/ai-accounts/" + a.id + "/disconnect", {}).catch(function (e) {
        // Best-effort disconnect: refresh the list to reflect local state anyway.
        if (window.gbLogError) { window.gbLogError(e, "agent disconnect"); }
      }).then(function () { renderAgents(el); refreshPickers(); });
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
