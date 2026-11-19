"use strict";
/**
 * 10c_settings_garage.js - Settings > Modelo laid out as a workshop where the
 * person tunes the assistant: a dashboard of the setup in use, a two-question
 * guide for someone who does not know which option to pick, and the three
 * ways to run the assistant compared side by side. The column headers are the
 * mode buttons (data-mode) that 10_settings_model.js binds.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function S() { return A._modelShared || {}; }

var L = {"pt-BR":{
  title:"Oficina do assistente", lead:"Ajuste onde e como o seu assistente pensa.",
  dash:"Painel", engine:"Motor em uso", privacy:"Privacidade", features:"Recursos ligados",
  fDocs:"Documentos", fOcr:"Texto em imagens", fWeb:"Pesquisa na web", fWebOff:"Pesquisa na web (desligada)", fPhotos:"Fotos e gráficos",
  where:"Escolha o motor", whereSub:"Compare as três formas de usar o assistente. Clique no cartão de uma delas para ver os detalhes e ativar.",
  local:"Neste computador", own:"Servidor ou chave própria", free:"Modelo gratuito",
  subLocal:"Privado e sem internet", subOwn:"O da sua organização", subFree:"Pronto para usar",
  inUse:"Em uso", suggested:"Sugerido", ctaOpen:"Ver e ativar", ctaAdjust:"Ver ajustes",
  rPrivacy:"Seus textos", rOffline:"Sem internet", rCost:"Custo", rDocs:"PDF, Word e texto",
  rOcr:"Texto em imagens e digitalizações", rWeb:"Pesquisa na web", rPhotos:"Entender fotos e gráficos", rNeeds:"Você precisa de",
  stayHere:"Ficam neste computador", goServer:"Vão ao servidor que você indicar", goNet:"Ficam na rede da sua organização", goExt:"Vão a um provedor externo",
  works:"Funciona", onlyNet:"Só com servidor na sua rede", noWork:"Não funciona",
  costFree:"Grátis", costContract:"Conforme o contrato", costFreeLimit:"Grátis, com limite de uso",
  reads:"Lê", noRead:"Não lê", optional:"Opcional, você liga", unavailable:"Indisponível", notYet:"Ainda não",
  needLocal:"Placa de vídeo com 6 GB ou mais e um download de 4 a 6 GB", needOwn:"Endereço do servidor e uma chave", needFree:"Nada",
  machineOk:"Este computador tem: %d, %v GB", machineSlow:"Este computador não tem placa de vídeo adequada: funcionaria devagar",
  guide:"Não sabe qual escolher?", guideSub:"Responda duas perguntas.",
  q1:"O que você escreve pode sair deste computador?", q1no:"Não, precisa ficar aqui", q1yes:"Sim, pode sair",
  q2:"Sua organização fornece um servidor ou uma chave de IA?", q2yes:"Sim", q2no:"Não, ou não sei",
  recLocal:"Use Neste computador: nada sai daqui, e esta máquina tem placa de vídeo para isso.",
  recLocalSlow:"Esta máquina não tem placa de vídeo para um modelo local. Use o servidor da sua organização, dentro da sua rede.",
  recOwn:"Use o servidor ou a chave da sua organização: o contrato define o custo e os limites.",
  recFree:"Comece pelo modelo gratuito: não precisa configurar nada. Para assuntos sigilosos, prefira Neste computador.",
  showMe:"Ver como ativar", restart:"Recomeçar"},
en:{
  title:"Assistant workshop", lead:"Tune where and how your assistant thinks.",
  dash:"Dashboard", engine:"Engine in use", privacy:"Privacy", features:"Features on",
  fDocs:"Documents", fOcr:"Text in images", fWeb:"Web search", fWebOff:"Web search (off)", fPhotos:"Photos and charts",
  where:"Choose the engine", whereSub:"Compare the three ways to use the assistant. Click one to see the details and turn it on.",
  local:"On this computer", own:"Own server or key", free:"Free model",
  subLocal:"Private and offline", subOwn:"Your organization’s", subFree:"Ready to use",
  inUse:"In use", suggested:"Suggested", ctaOpen:"See and turn on", ctaAdjust:"See settings",
  rPrivacy:"Your text", rOffline:"Without internet", rCost:"Cost", rDocs:"PDF, Word and text",
  rOcr:"Text in images and scans", rWeb:"Web search", rPhotos:"Understand photos and charts", rNeeds:"You need",
  stayHere:"Stays on this computer", goServer:"Goes to the server you set", goNet:"Stays in your organization’s network", goExt:"Goes to an external provider",
  works:"Works", onlyNet:"Only with a server on your network", noWork:"Does not work",
  costFree:"Free", costContract:"Per your contract", costFreeLimit:"Free, with usage limits",
  reads:"Reads", noRead:"Does not read", optional:"Optional, you turn it on", unavailable:"Unavailable", notYet:"Not yet",
  needLocal:"A graphics card with 6 GB or more and a 4 to 6 GB download", needOwn:"The server address and a key", needFree:"Nothing",
  machineOk:"This computer has: %d, %v GB", machineSlow:"This computer has no suitable graphics card: it would be slow",
  guide:"Not sure which to pick?", guideSub:"Answer two questions.",
  q1:"Can what you write leave this computer?", q1no:"No, it must stay here", q1yes:"Yes, it can",
  q2:"Does your organization provide an AI server or key?", q2yes:"Yes", q2no:"No, or I don’t know",
  recLocal:"Use On this computer: nothing leaves it, and this machine has a graphics card for it.",
  recLocalSlow:"This machine has no graphics card for a local model. Use your organization’s server, inside your network.",
  recOwn:"Use your organization’s server or key: the contract sets the cost and the limits.",
  recFree:"Start with the free model: nothing to set up. For confidential matters, prefer On this computer.",
  showMe:"See how to turn it on", restart:"Start over"}};
function g(k) { var lang = A.locale && A.locale.startsWith("pt") ? "pt-BR" : "en"; return (L[lang] && L[lang][k]) || L.en[k] || k; }

var ICONS = {
  local: '<svg class="gb-set-mode-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>',
  own_key: '<svg class="gb-set-mode-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
  free: '<svg class="gb-set-mode-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>',
  wrench: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>'
};
var MODES = ["local", "own_key", "free"];

function isWindows() { return /Windows/i.test(navigator.userAgent || ""); }
function webAvailable() { return typeof A.webSearchAvailable === "function" && A.webSearchAvailable(); }
function webOn() { return typeof A.webSearchOn === "function" && A.webSearchOn(); }
function localHardware(modes) {
  var m = (modes || []).find(function (x) { return x.id === "local"; });
  return (m && m.hardware) || {};
}
/* A local model is realistic when a CUDA card has the 6 GB the smallest one needs. */
function machineFits(hw) { return !!(hw.cuda_usable && hw.vram_gb >= 6); }

/* ---- Dashboard ---- */
function privacyLevel(mode, scope) {
  if (mode === "local" || scope === "machine") return 5;
  if (scope === "network") return 4;
  if (mode === "own_key") return 2;
  return 1;
}
function gauge(level, label) {
  var segs = "";
  for (var i = 1; i <= 5; i++) { segs += '<i class="' + (i <= level ? "on" : "") + '"></i>'; }
  return '<span class="gb-gar-gauge gb-gar-level-' + level + '" role="img" aria-label="' + esc(label + ": " + level + "/5") + '">' + segs + '</span>';
}
function chip(on, text) {
  return '<span class="gb-gar-chip' + (on ? " on" : "") + '"><span class="gb-gar-mark ' + (on ? "yes" : "no") + '" aria-hidden="true"></span>' + esc(text) + '</span>';
}
function dashboard(cfg, shortName, privacyText) {
  var mode = cfg ? cfg.mode : "";
  var scope = cfg ? cfg.endpoint_scope : "";
  var name = shortName(cfg ? cfg.model : (A.defaultModel || A.selectedModel || "")).replace(/\.gguf$/i, "");
  var where = mode ? g(mode === "own_key" ? "own" : mode) : "";
  var html = '<div class="gb-gar-hero">' + ICONS.wrench +
    '<div><h2 class="gb-gar-title">' + esc(g("title")) + '</h2><p class="gb-gar-lead">' + esc(g("lead")) + '</p></div></div>';
  html += '<div class="gb-set-card gb-gar-dash">' +
    '<div class="gb-gar-dash-main"><div><span class="gb-gar-kicker">' + esc(g("engine")) + '</span>' +
    '<div class="gb-gar-engine gb-set-model-name">' + esc(name) + '</div></div>' +
    (where ? '<span class="gb-gar-where">' + (ICONS[mode] || "") + esc(where) + '</span>' : '') + '</div>';
  if (mode) {
    var level = privacyLevel(mode, scope);
    html += '<div class="gb-gar-dash-row"><span class="gb-gar-dash-label">' + esc(g("privacy")) + '</span>' +
      gauge(level, g("privacy")) + '<span class="gb-gar-dash-text">' + privacyText + '</span></div>';
  }
  html += '<div class="gb-gar-dash-row"><span class="gb-gar-dash-label">' + esc(g("features")) + '</span><span class="gb-gar-chips">' +
    chip(true, g("fDocs")) + chip(isWindows(), g("fOcr")) +
    (webAvailable() ? chip(webOn(), webOn() ? g("fWeb") : g("fWebOff")) : '') +
    chip(false, g("fPhotos")) + '</span></div></div>';
  return html;
}

/* ---- Guide: two questions as cards; the answer selects the option below ---- */
var GI = {
  lock: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
  globe: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 0 1 0 20M12 2a15 15 0 0 0 0 20"/></svg>',
  server: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><rect x="2" y="3" width="20" height="7" rx="2"/><rect x="2" y="14" width="20" height="7" rx="2"/><path d="M6 6.5h.01M6 17.5h.01"/></svg>',
  none: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="M9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3M12 17h.01"/></svg>'
};
var GL = {"pt-BR":{
  step:"Pergunta %n de 2", back:"Voltar",
  q1help:"Pense em processos, dados pessoais e documentos internos. Na dúvida, considere que não podem.",
  q1noD:"Processos, dados pessoais, documentos internos", q1yesD:"Textos públicos, rascunhos, pesquisas",
  q2s:"Sua organização tem um servidor de IA dentro da rede interna?",
  q2sHelp:"Pergunte à equipe de TI. É um endereço como https://ia.orgao.gov.br.",
  q2sYesD:"A TI fornece o endereço e a chave", q2sNoD:"Usar só este computador",
  q2help:"Por exemplo, uma chave da OpenAI ou do Azure contratada pelo órgão, ou um servidor próprio.",
  q2yesD:"Tenho o endereço e a chave", q2noD:"Quero começar sem configurar nada",
  pick:"Indicamos para você", gains:"O que você tem com esta opção",
  recOwnNet:"Use o servidor da sua organização: os textos ficam dentro da rede, e a TI cuida da capacidade.",
  recLocalTight:"Use Neste computador: nada sai daqui. A placa de vídeo dá conta dos modelos menores; feche outros programas pesados antes de usar.",
  open:"Abrir esta opção"},
en:{
  step:"Question %n of 2", back:"Back",
  q1help:"Think of case files, personal data and internal documents. If in doubt, assume they cannot.",
  q1noD:"Case files, personal data, internal documents", q1yesD:"Public text, drafts, research",
  q2s:"Does your organization have an AI server inside its internal network?",
  q2sHelp:"Ask your IT team. It is an address such as https://ai.agency.gov.",
  q2sYesD:"IT provides the address and the key", q2sNoD:"Use this computer only",
  q2help:"For example an OpenAI or Azure key your organization pays for, or its own server.",
  q2yesD:"I have the address and the key", q2noD:"I want to start without setting anything up",
  pick:"Our suggestion for you", gains:"What you get with this option",
  recOwnNet:"Use your organization’s server: your text stays inside the network, and IT looks after the capacity.",
  recLocalTight:"Use On this computer: nothing leaves it. The graphics card handles the smaller models; close other heavy programs first.",
  open:"Open this option"}};
function gl(k) { var lang = A.locale && A.locale.startsWith("pt") ? "pt-BR" : "en"; return (GL[lang] && GL[lang][k]) || GL.en[k] || g(k); }

function guide() {
  return '<div class="gb-set-card gb-gar-guide" id="gbGarGuide"><div class="gb-gar-guide-head"><strong>' + esc(g("guide")) + '</strong> ' +
    '<span class="gb-set-hint">' + esc(g("guideSub")) + '</span></div><div class="gb-gar-guide-body" aria-live="polite"></div></div>';
}
function bindGuide(panel, modes) {
  var box = panel.querySelector("#gbGarGuide .gb-gar-guide-body");
  if (!box) return;
  var hw = localHardware(modes);
  function choice(i, icon, title, desc) {
    return '<button type="button" class="gb-gar-choice" data-a="' + i + '"><span class="gb-gar-choice-icon">' + icon + '</span>' +
      '<span class="gb-gar-choice-text"><strong>' + esc(title) + '</strong><small>' + esc(desc) + '</small></span></button>';
  }
  /* opts: [{icon, title, desc, go}] */
  function ask(step, q, help, opts, back) {
    box.innerHTML = '<div class="gb-gar-step"><span class="gb-gar-dots"><i class="on"></i><i class="' + (step > 1 ? "on" : "") + '"></i></span>' +
      esc(gl("step").replace("%n", String(step))) + (back ? '<button type="button" class="gb-gar-back" data-back="1">' + esc(gl("back")) + '</button>' : '') + '</div>' +
      '<p class="gb-gar-q">' + esc(q) + '</p><p class="gb-set-hint gb-gar-help">' + esc(help) + '</p>' +
      '<div class="gb-gar-choices">' + opts.map(function (o, i) { return choice(i, o.icon, o.title, o.desc); }).join("") + '</div>';
    opts.forEach(function (o, i) { box.querySelector('[data-a="' + i + '"]').addEventListener("click", o.go); });
    if (back) { box.querySelector('[data-back="1"]').addEventListener("click", back); }
    var first = box.querySelector(".gb-gar-choice");
    if (first && step > 1) { first.focus(); }
  }
  function scrollTo(el) {
    var still = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (el && el.scrollIntoView) { el.scrollIntoView({ behavior: still ? "auto" : "smooth", block: "start" }); }
  }
  function result(mode, text) {
    var grid = panel.querySelector("#gbGarGrid");
    if (grid) grid.setAttribute("data-rec", mode);
    /* The suggested option is selected in the table and its details open below. */
    var head = panel.querySelector('.gb-set-mode-card[data-mode="' + mode + '"]');
    if (head && !head.disabled) { head.click(); }
    var f = facts(mode, A._modelShared && A._modelShared.cfg, hw);
    var gains = ["privacy", "offline", "cost", "needs"].map(function (k) {
      return '<li><span class="gb-gar-mark ' + f[k].k + '" aria-hidden="true"></span><span><b>' + esc(g(ROWS.filter(function (r) { return r[0] === k; })[0][1])) +
        ':</b> ' + esc(f[k].t) + '</span></li>';
    }).join("");
    box.innerHTML = '<div class="gb-gar-result"><span class="gb-gar-kicker">' + esc(gl("pick")) + '</span>' +
      '<div class="gb-gar-result-name">' + (ICONS[mode] || "") + esc(g(mode === "own_key" ? "own" : mode)) + '</div>' +
      '<p class="gb-gar-rec">' + esc(text) + '</p>' +
      '<p class="gb-gar-gains-title">' + esc(gl("gains")) + '</p><ul class="gb-gar-gains">' + gains + '</ul>' +
      '<div class="gb-gar-answers"><button class="gb-set-btn-sm gb-set-btn-accent" data-go="1">' + esc(gl("open")) + '</button>' +
      '<button class="gb-set-btn-sm" data-again="1">' + esc(g("restart")) + '</button></div></div>';
    box.querySelector('[data-go="1"]').addEventListener("click", function () {
      var h = panel.querySelector('.gb-set-mode-card[data-mode="' + mode + '"]');
      if (h && !h.classList.contains("gb-set-mode-selected")) { h.click(); }
      scrollTo(document.getElementById("gbModeDetail"));
    });
    box.querySelector('[data-again="1"]').addEventListener("click", start);
  }
  function secret() {
    ask(2, gl("q2s"), gl("q2sHelp"), [
      { icon: GI.server, title: g("q2yes"), desc: gl("q2sYesD"), go: function () { result("own_key", gl("recOwnNet")); } },
      { icon: ICONS.local, title: g("q2no"), desc: gl("q2sNoD"), go: function () {
        if (!machineFits(hw)) { result("own_key", g("recLocalSlow")); return; }
        result("local", hw.vram_gb >= 10 ? g("recLocal") : gl("recLocalTight"));
      } }
    ], start);
  }
  function open() {
    ask(2, g("q2"), gl("q2help"), [
      { icon: GI.server, title: g("q2yes"), desc: gl("q2yesD"), go: function () { result("own_key", g("recOwn")); } },
      { icon: GI.none, title: g("q2no"), desc: gl("q2noD"), go: function () { result("free", g("recFree")); } }
    ], start);
  }
  function start() {
    var grid = panel.querySelector("#gbGarGrid");
    if (grid) grid.removeAttribute("data-rec");
    ask(1, g("q1"), gl("q1help"), [
      { icon: GI.lock, title: g("q1no"), desc: gl("q1noD"), go: secret },
      { icon: GI.globe, title: g("q1yes"), desc: gl("q1yesD"), go: open }
    ], null);
  }
  start();
}

/* ---- Comparison ---- */
/* What each way of running the assistant does in this version. Every cell is
   yes, part (depends) or no, with a short sentence. */
function cell(kind, text) { return { k: kind, t: text }; }
function facts(mode, cfg, hw) {
  var scope = cfg && cfg.mode === "own_key" ? cfg.endpoint_scope : "";
  var ocr = isWindows() ? cell("yes", g("reads")) : cell("no", g("noRead"));
  var web = webAvailable() ? cell("part", g("optional")) : cell("no", g("unavailable"));
  if (mode === "local") {
    var machine = machineFits(hw)
      ? g("machineOk").replace("%d", hw.device || "").replace("%v", String(hw.vram_gb || 0))
      : g("machineSlow");
    var V = A._modelMachine ? A._modelMachine.verdict(hw, S().localModels) : null;
    if (V && V.tone !== "none") { machine += " · " + V.short; }
    var kind = !V ? (machineFits(hw) ? "yes" : "no") : V.tone === "good" ? "yes" : V.tone === "mid" ? "part" : "no";
    return { privacy: cell("yes", g("stayHere")), offline: cell("yes", g("works")), cost: cell("yes", g("costFree")),
      docs: cell("yes", g("reads")), ocr: ocr, web: web, photos: cell("no", g("notYet")),
      needs: cell(kind, g("needLocal")), note: machine };
  }
  if (mode === "own_key") {
    return { privacy: cell(scope === "network" ? "yes" : "part", scope === "network" ? g("goNet") : g("goServer")),
      offline: cell("part", g("onlyNet")), cost: cell("part", g("costContract")),
      docs: cell("yes", g("reads")), ocr: ocr, web: web, photos: cell("no", g("notYet")), needs: cell("part", g("needOwn")) };
  }
  return { privacy: cell("no", g("goExt")), offline: cell("no", g("noWork")), cost: cell("yes", g("costFreeLimit")),
    docs: cell("yes", g("reads")), ocr: ocr, web: web, photos: cell("no", g("notYet")), needs: cell("yes", g("needFree")) };
}
var ROWS = [["privacy", "rPrivacy"], ["offline", "rOffline"], ["cost", "rCost"], ["docs", "rDocs"],
  ["ocr", "rOcr"], ["web", "rWeb"], ["photos", "rPhotos"], ["needs", "rNeeds"]];

function compare(currentMode, modes, cfg) {
  var hw = localHardware(modes);
  var all = {}; MODES.forEach(function (m) { all[m] = facts(m, cfg, hw); });
  var html = '<h3 class="gb-set-section-title">' + esc(g("where")) + '</h3><p class="gb-set-hint gb-gar-sub">' + esc(g("whereSub")) + '</p>' +
    '<div class="gb-gar-compare" id="gbModelModes"><div class="gb-gar-grid" id="gbGarGrid" role="table">';
  html += '<div class="gb-gar-corner" role="columnheader"></div>';
  MODES.forEach(function (m) {
    var md = (modes || []).find(function (x) { return x.id === m; });
    var avail = !md || md.available !== false;
    var active = currentMode === m;
    html += '<button class="gb-set-mode-card gb-gar-head' + (active ? " gb-set-mode-active" : "") + '" data-mode="' + m + '" data-col="' + m + '"' + (avail ? "" : " disabled") + ' role="columnheader">' +
      ICONS[m] + '<span class="gb-set-mode-label">' + esc(g(m === "own_key" ? "own" : m)) + '</span>' +
      '<span class="gb-set-mode-sub">' + esc(g(m === "own_key" ? "subOwn" : (m === "local" ? "subLocal" : "subFree"))) + '</span>' +
      '<span class="gb-gar-head-badges">' + (active ? '<span class="gb-badge gb-badge-active">' + esc(g("inUse")) + '</span>' : '') +
      '<span class="gb-badge gb-gar-sugg">' + esc(g("suggested")) + '</span></span>' +
      '<span class="gb-gar-head-cta">' + esc(active ? g("ctaAdjust") : g("ctaOpen")) +
      '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true"><path d="M9 6l6 6-6 6"/></svg></span></button>';
  });
  ROWS.forEach(function (row) {
    html += '<div class="gb-gar-label" role="rowheader">' + esc(g(row[1])) + '</div>';
    MODES.forEach(function (m) {
      var c = all[m][row[0]];
      var note = row[0] === "needs" && all[m].note ? '<small class="gb-gar-note">' + esc(all[m].note) + '</small>' : '';
      html += '<div class="gb-gar-cell" data-col="' + m + '" role="cell"><span class="gb-gar-mark ' + c.k + '" aria-hidden="true"></span>' +
        '<span>' + esc(c.t) + note + '</span></div>';
    });
  });
  return html + '</div></div>';
}

A._modelGarage = {
  dashboard: function (cfg, privacyText) {
    return dashboard(cfg, S().shortName || function (m) { return m || "—"; }, privacyText);
  },
  guide: guide,
  bindGuide: bindGuide,
  compare: compare,
  /* The column of the card being viewed is lit across the table. */
  markView: function (mode) {
    var grid = document.getElementById("gbGarGrid");
    if (grid) { grid.setAttribute("data-view", mode); }
  }
};
})();
