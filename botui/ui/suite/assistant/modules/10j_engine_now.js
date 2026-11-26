"use strict";
/**
 * 10j_engine_now.js - The engine the chat sends the next message to, the one
 * the composer's model button names (A.selectedModel, else A.defaultModel):
 * its kind, where the text goes, the privacy level and what it can do in this
 * version. Settings > Modelo draws "Motor em uso" from it (10c) and redraws
 * that card in place whenever the choice changes: on the page, in the
 * composer's menu or through the engine confirm's "Continuar".
 */
(function () {
var A = window.GBAssistant;
function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }
function tr(p, e) { return (A.locale || "").startsWith("pt") ? p : e; }
function S() { return A._modelShared || {}; }
function st(key, fallback) { return S().t ? S().t(key) : fallback; }
function isWindows() { return /Windows/i.test(navigator.userAgent || ""); }

/* Hosts whose models receive pictures (botllm::media_parts::sees_images). */
var SEEING = ["anthropic.com", "openai.com", "x.ai", "googleapis.com"];
function sees(host) {
  var h = String(host || "").toLowerCase().replace(/:\d+$/, "");
  return SEEING.some(function (d) { return h === d || h.slice(-d.length - 1) === "." + d; });
}
function scopeOf(host) {
  var h = String(host || "").toLowerCase().replace(/:\d+$/, "");
  if (!h) { return ""; }
  if (h === "localhost" || h === "::1" || /^127\./.test(h)) { return "machine"; }
  if (/^10\.|^192\.168\.|^172\.(1[6-9]|2\d|3[01])\.|\.local$|\.lan$/.test(h)) { return "network"; }
  return "external";
}

/* /api/llm/accounts names the host of each key; read once, again on demand. */
var accountHosts = null, hostsAsked = false;
function loadHosts() {
  hostsAsked = true;
  return A.apiFetch("/api/llm/accounts").then(function (r) { return r.ok ? r.json() : {}; }).then(function (d) {
    accountHosts = {};
    (d.accounts || []).forEach(function (a) { accountHosts[a.id] = a.host || a.name || ""; });
  }).catch(function () { accountHosts = accountHosts || {}; }).then(redraw);
}

/** The engine in use: {value, name, kind, dest, host, scope, level, agent}. */
function describe(cfg) {
  var value = A.selectedModel || A.defaultModel || (cfg && cfg.model) || "";
  var name = (A.shortModelName ? A.shortModelName(value) : value) || "—";
  if (/^sub:/.test(value)) {
    var provider = value.slice(4).split("/")[0];
    var meta = A._agentMeta ? A._agentMeta(provider) : null;
    return { value: value, name: name, kind: "acp", agent: meta || { name: provider, co: "" },
      dest: meta ? meta.name + " · " + meta.co : provider, host: meta ? meta.co : provider, scope: "external", level: 2 };
  }
  var acct = /^acct:([^/]+)\//.exec(value);
  if (acct) {
    if (!accountHosts && !hostsAsked) { loadHosts(); }
    var h = accountHosts ? accountHosts[acct[1]] || "" : "";
    var sc = scopeOf(h) || "external";
    return { value: value, name: name, kind: "own_key", dest: h, host: h, scope: sc, level: sc === "machine" ? 5 : sc === "network" ? 4 : 3 };
  }
  var dest = A.destination || {};
  var mode = cfg ? cfg.mode || "" : A.engineMode || (dest.scope === "machine" ? "local" : "");
  var host = cfg ? cfg.endpoint_host || "" : dest.host || "";
  var scope = cfg ? cfg.endpoint_scope || "" : dest.scope || "";
  var level = mode === "local" || scope === "machine" ? 5 : scope === "network" ? 4 : mode === "own_key" ? 3 : mode === "acp" ? 2 : 1;
  return { value: value, name: name.replace(/\.gguf$/i, ""), kind: mode, dest: mode === "local" ? "" : host, host: host, scope: scope, level: level };
}

/** The privacy sentence of an engine, already escaped. */
function privacy(info) {
  if (info.scope === "machine") { return esc(st("privLocal", tr("Suas conversas ficam neste computador. Nada é enviado para fora.", "Your conversations stay on this computer. Nothing is sent outside."))); }
  if (info.scope === "network") { return esc(st("privNet", "%h").replace("%h", info.host)); }
  var text = info.kind === "acp"
    ? tr("Suas conversas vão ao agente %a (%c), pela sua assinatura. O agente trabalha numa máquina virtual isolada neste computador.",
        "Your conversations go to the %a agent (%c), through your subscription. The agent works in an isolated virtual machine on this computer.")
        .replace("%a", info.agent.name).replace("%c", info.agent.co)
    : st("privExt", "%h").replace("%h", info.host || st("freeHost", tr("um provedor externo", "an external provider")));
  return info.kind || info.host ? '<span class="gb-set-external-warn">' + esc(text) + '</span>' : '';
}

/** What the engine does in this version: [kind ("yes"|"part"|"no"), label, reason]. */
function caps(info) {
  var k = info.kind, acp = k === "acp", local = k === "local";
  var eyes = acp || ((k === "own_key" || k === "free") && sees(info.host));
  var ocr = isWindows();
  var vmOff = A.productConfig && A.productConfig.agent_backend_unavailable;
  var out = [["yes", tr("Documentos", "Documents"), tr("PDF, Word, planilhas e texto, lidos neste computador.", "PDF, Word, spreadsheets and text, read on this computer.")]];
  out.push(eyes || ocr ? ["yes", tr("Texto em imagens", "Text in images"), eyes ? tr("O motor vê a imagem e lê o texto dela.", "The engine sees the picture and reads its text.")
    : tr("O texto da imagem é lido neste computador antes de ir ao motor.", "The picture's text is read on this computer before it goes to the engine.")]
    : ["no", tr("Texto em imagens", "Text in images"), tr("Este motor não vê imagens e a leitura de texto em imagens não existe neste sistema.", "This engine does not see pictures and reading text in images is not available on this system.")]);
  out.push(eyes ? ["yes", tr("Fotos e gráficos", "Photos and charts"), tr("O motor recebe a imagem e entende o que ela mostra.", "The engine receives the picture and understands what it shows.")]
    : ["no", tr("Fotos e gráficos", "Photos and charts"), local ? tr("Os modelos locais desta versão não veem imagens; leem só o texto delas.", "Local models in this version do not see pictures; they read only their text.")
      : tr("Este motor não recebe imagens; só o texto delas chega a ele.", "This engine does not receive pictures; only their text reaches it.")]);
  out.push(["yes", tr("Páginas da web", "Web pages"), tr("Lê as páginas cujo endereço você escreve na mensagem.", "Reads the pages whose address you write in the message.")]);
  out.push(local ? ["part", tr("Textos longos", "Long texts"), tr("Um modelo local guarda menos texto de uma vez: lê a parte que cabe na memória dele.", "A local model holds less text at once: it reads the part that fits its memory.")]
    : ["yes", tr("Textos longos", "Long texts"), tr("Um texto longo colado vai inteiro, como anexo.", "A long pasted text goes whole, as an attachment.")]);
  var vmLabel = tr("Máquina Virtual", "Virtual Machine");
  out.push(vmOff ? ["no", vmLabel, tr("A máquina virtual não está disponível neste computador.", "The virtual machine is not available on this computer.")]
    : acp ? ["yes", vmLabel, tr("Comandos e arquivos na máquina virtual isolada, com cartões de permissão. A chave Agente fica ligada com este motor.", "Commands and files in the isolated virtual machine, with permission cards. The Agent switch stays on with this engine.")]
    : A.agentEnabled ? ["yes", vmLabel, tr("Com a chave Agente ligada: comandos e arquivos na máquina virtual isolada, com cartões de permissão.", "With the Agent switch on: commands and files in the isolated virtual machine, with permission cards.")]
    : ["part", vmLabel, tr("Ligue a chave Agente na conversa para usar comandos e arquivos na máquina virtual.", "Turn the Agent switch on in the conversation to use commands and files in the virtual machine.")]);
  out.push(["yes", tr("Resposta ao vivo", "Live answer"), tr("A resposta aparece enquanto é escrita.", "The answer appears while it is written.")]);
  return out;
}

/* ---- Live redraw of Settings > Modelo ---- */
var drawn = "";
function panel() { return document.querySelector('#gbSetPanel[data-section="model"]'); }
function redraw() {
  var p = panel(), G = A._modelGarage, run = p && p.querySelector(".gb-of-run");
  if (!run || !G || !G.runCard) { return; }
  var cfg = S().cfg, info = describe(cfg);
  drawn = info.value;
  var holder = document.createElement("div");
  holder.innerHTML = G.runCard(cfg);
  if (holder.firstElementChild) { run.replaceWith(holder.firstElementChild); }
  if (G.markInUse) { G.markInUse(p, info.kind, cfg); }
  var agent = info.kind === "acp" ? info.value.slice(4).split("/")[0] : "";
  p.querySelectorAll(".gb-of-picked").forEach(function (n) {
    var row = n.closest("[data-agent]");
    if (!row || row.getAttribute("data-agent") !== agent) { n.remove(); }
  });
}
function changed() { if ((A.selectedModel || A.defaultModel || "") !== drawn) { redraw(); } }

var baseRender = A.renderModelPickers;
if (typeof baseRender === "function") {
  A.renderModelPickers = function () { var out = baseRender.apply(this, arguments); changed(); return out; };
}
var baseSync = A.syncAgentToggle;
if (typeof baseSync === "function") {
  A.syncAgentToggle = function () { var out = baseSync.apply(this, arguments); redraw(); return out; };
}

A._engineNow = {
  describe: function (cfg) { var info = describe(cfg); drawn = info.value; return info; },
  privacy: privacy, caps: caps, redraw: redraw,
  refreshHosts: function () { hostsAsked = false; accountHosts = null; }
};
})();
