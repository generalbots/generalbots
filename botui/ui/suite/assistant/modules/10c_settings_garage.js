"use strict";
/**
 * 10c_settings_garage.js - Settings > Modelo laid out as the assistant's
 * workshop: the engine in use on top, the four engines as cards with a
 * privacy meter, the details of the one being viewed in lettered panels, the
 * full comparison in a side panel, a two-question guide and the tooltips.
 * It also words where the local model in use runs (graphics card or not).
 * The cards are the mode buttons (data-mode) that 10_settings_model.js binds.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function S() { return A._modelShared || {}; }
function pt() { return !!(A.locale && A.locale.startsWith("pt")); }
function tr(p, e) { return pt() ? p : e; }

var MODES = ["local", "own_key", "free", "acp"];
/* name, sub, destination, tone, privacy level, icon path */
var INFO = {
  local: [["Neste computador", "On this computer"], ["Privado e sem internet", "Private and offline"], ["Fica neste computador", "Stays on this computer"], "ok", 5, "M3 4h18v12H3zM8 20h8M12 16v4"],
  own_key: [["Servidor ou chave própria", "Own server or key"], ["O da sua organização ou de um provedor", "Your organization’s or a provider’s"], ["Vai ao servidor indicado", "Goes to the server you set"], "mid", 3, "M3 4h18v6H3zM3 14h18v6H3zM7 7h.01M7 17h.01"],
  free: [["Modelo gratuito", "Free model"], ["Pronto para usar, sem chave", "Ready to use, no key"], ["Vai a um provedor externo", "Goes to an external provider"], "out", 1, "M13 2 3 14h9l-1 8 10-12h-9z"],
  acp: [["Agente por assinatura", "Subscription agent"], ["Sua assinatura, num espaço isolado neste computador", "Your subscription, in an isolated space on this computer"], ["Vai ao provedor do agente", "Goes to the agent’s provider"], "out", 2, "M4 4h16v16H4zM8 9l3 3-3 3M13 15h3"]
};
function L2(pair) { return pair[pt() ? 0 : 1]; }
function modeLabel(mode) { return A.engineName(mode, "detail") || mode; }
function svg(d, size, w) { return '<svg width="' + size + '" height="' + size + '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="' + (w || 1.7) + '" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="' + d + '"/></svg>'; }
var I = { info: "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM12 11v5M12 8h.01", help: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .9-1 1.6v.6M12 17h.01",
  table: "M3 4h18v16H3zM3 10h18M9 4v16", check: "M20 6 9 17l-5-5", arrow: "M5 12h14M13 6l6 6-6 6", next: "m9 18 6-6-6-6", close: "M18 6 6 18M6 6l12 12",
  warn: "M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0zM12 9v4M12 17h.01",
  lock: "M5 11h14v10H5zM8 11V7a4 4 0 0 1 8 0v4", globe: "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM2 12h20" };
function sl() { return '<span class="gb-of-sl">//</span>'; }
function help(text, label) { return '<span class="gb-of-help" tabindex="0" data-tip="' + esc(text) + '">' + svg(I.info, 13, 1.8) + (label ? esc(label) : '') + '</span>'; }
function segDesc(n) {
  if (n >= 5) { return tr("Nada sai deste computador", "Nothing leaves this computer"); }
  if (n >= 4) { return tr("O texto fica na sua rede", "Your text stays on your network"); }
  if (n >= 3) { return tr("O texto vai a um servidor externo que você escolheu", "Your text goes to an external server you chose"); }
  return tr("O texto vai a um provedor externo", "Your text goes to an external provider");
}
function segs(n, tone, cls) {
  var label = tr("Privacidade", "Privacy") + " " + n + "/5: " + segDesc(n);
  var h = ""; for (var i = 0; i < 5; i++) { h += '<i class="' + (i < n ? "on" : "") + '"></i>'; }
  return '<span class="gb-of-segs' + (cls ? " " + cls : "") + '" data-tone="' + tone + '" role="img" aria-label="' + esc(label) + '" title="' + esc(label) + '">' + h + '</span>';
}
function cap(kind, text, tip) { return '<span class="gb-of-cap' + (kind === "yes" ? "" : kind === "part" ? " part" : " no") + '"' + (tip ? ' tabindex="0" data-tip="' + esc(tip) + '"' : '') + '><b>' + (kind === "yes" ? "✓" : kind === "part" ? "~" : "–") + '</b>' + esc(text) + '</span>'; }
function isWindows() { return /Windows/i.test(navigator.userAgent || ""); }
function localHardware(modes) { var m = (modes || []).find(function (x) { return x.id === "local"; }); return (m && m.hardware) || {}; }
function machineFits(hw) { return !!(hw.cuda_usable && hw.vram_gb >= 6); }
function tone(level) { return level >= 4 ? "ok" : level === 3 ? "mid" : "out"; }

/* ---- Header and the engine in use ---- */
function dashboard(cfg, showTools) {
  var html = '<div class="gb-of-head"><div><span class="gb-of-k">' + sl() + tr("OFICINA DO ASSISTENTE", "ASSISTANT WORKSHOP") + '</span>' +
    '<h1>' + tr("Onde o seu assistente pensa.", "Where your assistant thinks.") + '</h1></div>';
  if (showTools) {
    html += '<div class="gb-of-tools"><button type="button" class="gb-of-btn" id="gbOpenGuide">' + svg(I.help, 14, 1.8) + tr("Me ajude a escolher", "Help me choose") +
      '</button><button type="button" class="gb-of-btn" id="gbOpenCompare">' + svg(I.table, 14, 1.8) + tr("Comparar os 4 motores", "Compare the 4 engines") + '</button></div>';
  }
  return html + '</div>' + runCard(cfg);
}
/* The engine the chat uses now (10j_engine_now.js), not only the stored runtime. */
function now(cfg) { return A._engineNow ? A._engineNow.describe(cfg) : { kind: cfg && cfg.mode, name: (cfg && cfg.model) || "—", dest: "", level: 1 }; }
function inUseKind(cfg) { return now(cfg).kind || ""; }
function runCard(cfg) {
  var E = A._engineNow, info = now(cfg), mode = info.kind;
  var rt = cfg && cfg.local_runtime;
  var state = !mode ? "" : mode !== "local" ? ['gb-of-ok', tr("EM USO", "IN USE")]
    : rt && rt.running ? ['gb-of-ok', tr("EM EXECUÇÃO", "RUNNING")] : ['gb-of-muc', tr("PARADO", "STOPPED")];
  var meta = mode === "local" && rt && rt.build ? tr("motor ", "engine ") + rt.build : info.dest;
  var priv = E ? E.privacy(info) : "";
  var list = E ? E.caps(info) : [];
  return '<section class="gb-of-card gb-of-run" data-engine="' + esc(info.value || "") + '"><div class="gb-of-run-h"><div>' + sl() + '<span>' + tr("MOTOR EM USO", "ENGINE IN USE") + '</span></div>' +
    (state ? '<div class="gb-of-state ' + state[0] + '"><span class="gb-of-sq"></span>' + state[1] + '</div>' : '') + '</div>' +
    '<div class="gb-of-run-cells"><div><span class="gb-of-engine">' + esc(info.name) + '</span><div class="gb-of-row">' +
    (mode ? '<span class="gb-of-chip">' + esc(modeLabel(mode)) + '</span>' : '') + (meta ? '<span class="gb-of-meta">' + esc(meta) + '</span>' : '') + '</div></div>' +
    '<div><div class="gb-of-row sb"><span class="gb-of-lab">' + tr("PRIVACIDADE", "PRIVACY") + '</span>' + (mode ? segs(info.level, tone(info.level)) : '') + '</div>' +
    (priv ? '<p class="gb-of-priv">' + priv + '</p>' : '') +
    '<div class="gb-of-caps"><span class="gb-of-lab">' + tr("RECURSOS", "FEATURES") + '</span>' +
    list.map(function (c) { return cap(c[0], c[1], c[2]); }).join("") + '</div></div></div></section>';
}
/* After a new pick: the "in use" chip moves to the card of the engine's kind. */
function markInUse(panel, kind, cfg) {
  panel.querySelectorAll(".gb-of-mode .gb-of-inuse").forEach(function (c) { c.remove(); });
  var card = kind && panel.querySelector('.gb-of-mode[data-mode="' + kind + '"] .gb-of-sp');
  if (card) { card.insertAdjacentHTML("afterend", '<span class="gb-of-chip sm gb-of-inuse">' + tr("EM USO", "IN USE") + '</span>'); }
  var viewed = panel.querySelector('#gbOfModes .gb-of-mode[aria-pressed="true"]'), act = S().activate;
  if (viewed && act) { setDetail(panel, viewed.getAttribute("data-mode"), cfg, act); }
}

/* ---- What each engine does in this version (the comparison) ---- */
function cell(kind, text, note) { return { k: kind, t: text, n: note || "" }; }
function facts(mode, cfg, hw) {
  var scope = cfg && cfg.mode === "own_key" ? cfg.endpoint_scope : "";
  var ocr = isWindows() ? cell("yes", tr("Lê", "Reads")) : cell("no", tr("Não lê", "Does not read"));
  var reads = cell("yes", tr("Lê", "Reads")), notYet = cell("no", tr("Ainda não", "Not yet"));
  if (mode === "local") {
    var V = A._modelMachine ? A._modelMachine.verdict(hw, S().localModels) : null;
    var note = hw.device ? tr("Este computador: ", "This computer: ") + hw.device.replace(/^NVIDIA (GeForce )?/, "").replace(/ GPU$/, "") + ", " + (hw.vram_gb || 0) + " GB" : "";
    if (V && V.tone !== "none") { note += (note ? " · " : "") + V.short.toLowerCase(); }
    var kind = !V ? (machineFits(hw) ? "yes" : "no") : V.tone === "good" ? "yes" : V.tone === "mid" ? "part" : "no";
    return { privacy: cell("yes", tr("Ficam neste computador", "Stay on this computer")), offline: cell("yes", tr("Funciona", "Works")), cost: cell("yes", tr("Grátis", "Free")),
      docs: reads, ocr: ocr, photos: notYet, needs: cell(kind, tr("Placa de vídeo com 6 GB ou mais e um download de 4 a 6 GB", "A graphics card with 6 GB or more and a 4 to 6 GB download"), note) };
  }
  if (mode === "own_key") {
    return { privacy: cell(scope === "network" ? "yes" : "part", scope === "network" ? tr("Ficam na rede da sua organização", "Stay in your organization’s network") : tr("Vão ao servidor que você indicar", "Go to the server you set")),
      offline: cell("part", tr("Só com servidor na sua rede", "Only with a server on your network")), cost: cell("part", tr("Conforme o contrato", "Per your contract")),
      docs: reads, ocr: ocr, photos: cell("part", tr("Depende do servidor", "Depends on the server")), needs: cell("part", tr("Endereço do servidor e uma chave", "The server address and a key")) };
  }
  if (mode === "acp") {
    var does = cell("yes", tr("O agente faz", "The agent does it"));
    return { privacy: cell("part", tr("Vão ao provedor do agente", "Go to the agent’s provider")), offline: cell("no", tr("Não funciona", "Does not work")),
      cost: cell("part", tr("Sua assinatura (Claude, ChatGPT, Gemini…)", "Your subscription (Claude, ChatGPT, Gemini…)")), docs: does, ocr: does, photos: does,
      needs: cell("part", tr("Instalar o agente e entrar na sua conta", "Install the agent and sign in to your account")) };
  }
  return { privacy: cell("no", tr("Vão a um provedor externo", "Go to an external provider")), offline: cell("no", tr("Não funciona", "Does not work")),
    cost: cell("yes", tr("Grátis, com limite de uso", "Free, with usage limits")), docs: reads, ocr: ocr, photos: notYet, needs: cell("yes", tr("Nada", "Nothing")) };
}
var ROWS = [
  ["privacy", ["Seus textos", "Your text"], ["Para onde vai o que você escreve e anexa.", "Where what you write and attach goes."]],
  ["offline", ["Sem internet", "Without internet"], ["Se o assistente continua funcionando com a rede desligada.", "Whether the assistant keeps working with the network off."]],
  ["cost", ["Custo", "Cost"], null], ["docs", ["PDF, Word e texto", "PDF, Word and text"], null],
  ["ocr", ["Texto em imagens e digitalizações", "Text in images and scans"], ["O texto de fotos e documentos digitalizados é lido neste computador antes de ir ao modelo.", "The text of photos and scanned documents is read on this computer before it goes to the model."]],
  ["photos", ["Entender fotos e gráficos", "Understand photos and charts"], ["Interpretar o conteúdo de uma imagem, não só o texto escrito nela.", "Interpreting what an image shows, not only the text written in it."]],
  ["needs", ["Você precisa de", "You need"], null]];
var MK = { yes: "✓", part: "~", no: "–" };

/* ---- The four engines, the detail frame ---- */
var rec = null;
function modeButtons(currentMode, modes) {
  return '<section class="gb-of-pick" id="gbOfModes"><div class="gb-of-pick-h"><span class="gb-of-k">' + sl() + tr("ESCOLHA O MOTOR", "CHOOSE THE ENGINE") + '</span>' +
    '<span class="gb-of-hint">' + tr("Clique para ver os detalhes. Nada muda até você ativar.", "Click to see the details. Nothing changes until you turn it on.") + '</span></div>' +
    '<div class="gb-of-card"><div class="gb-of-grid" id="gbGarModeGrid"' + (rec ? ' data-rec="' + rec + '"' : '') + '>' + MODES.map(function (mode, i) {
      var info = (modes || []).find(function (e) { return e.id === mode; });
      var off = !info || info.available === false;
      // The server's mode list has no entry for subscription agents: they are
      // available whenever the agent workspace is (product config says so).
      if (mode === "acp" && !info) {
        var pc = A.productConfig || {};
        off = pc.agent_backend_unavailable !== null && pc.agent_backend_unavailable !== undefined;
      }
      var f = INFO[mode];
      return '<button type="button" class="gb-of-mode" data-mode="' + mode + '" aria-controls="gbModeDetail" aria-pressed="false"' + (off ? " disabled" : "") + '>' +
        '<span class="gb-of-mode-top"><span class="gb-of-ico">' + svg(f[5], 15) + '</span><span class="gb-of-num">0' + (i + 1) + '</span><span class="gb-of-sp"></span>' +
        (currentMode === mode ? '<span class="gb-of-chip sm gb-of-inuse">' + tr("EM USO", "IN USE") + '</span>' : '') +
        (off ? '<span class="gb-of-chip sm mute">' + tr("INDISPONÍVEL", "UNAVAILABLE") + '</span>' : '') +
        '<span class="gb-of-chip sm line gb-of-rec">' + tr("RECOMENDADO", "RECOMMENDED") + '</span></span>' +
        '<span class="gb-of-mode-txt"><strong>' + esc(L2(f[0])) + '</strong><span>' + esc(L2(f[1])) + '</span></span>' +
        '<span class="gb-of-mode-foot"><span class="gb-of-dest" data-tone="' + f[3] + '"><span class="gb-of-sq"></span>' + esc(L2(f[2])) + '</span>' + segs(f[4], f[3], "sm") + '</span></button>';
    }).join("") + '</div><div class="gb-of-dhead"><div><span class="gb-of-k">' + sl() + tr("DETALHES", "DETAILS") + '</span><span class="gb-of-dname" id="gbOfDName"></span>' +
    '<span class="gb-of-sp"></span><span id="gbOfDState"></span></div></div><div class="gb-of-dbody"><div class="gb-of-gut"></div><div class="gb-of-tray">' +
    '<div id="gbOfConfirm"></div><div id="gbOfActMsg" class="gb-of-say"></div><div id="gbModeDetail"></div></div></div></div></section>';
}
var ACT = { local: ["Usar neste computador", "Use on this computer"], own_key: ["Usar este servidor", "Use this server"],
  free: ["Usar modelo gratuito", "Use the free model"], acp: ["Usar agente por assinatura", "Use the subscription agent"] };
/* The head of the detail frame: the engine viewed and either "in use now" or the button that turns it on. */
function setDetail(panel, mode, cfg, onActivate) {
  var name = panel.querySelector("#gbOfDName"), st = panel.querySelector("#gbOfDState");
  ["#gbOfConfirm", "#gbOfActMsg"].forEach(function (s) { var n = panel.querySelector(s); if (n) { n.innerHTML = ""; } });
  if (name) { name.textContent = modeLabel(mode); }
  if (!st) { return; }
  if (inUseKind(cfg) === mode) {
    st.innerHTML = '<span class="gb-of-now">' + svg(I.check, 13, 2.2) + tr("EM USO AGORA", "IN USE NOW") + '</span>';
  } else {
    st.innerHTML = '<button type="button" class="gb-of-btn acc" id="gbOfActivate">' + esc(L2(ACT[mode])) + svg(I.arrow, 14, 3) + '</button>';
    st.querySelector("#gbOfActivate").addEventListener("click", function () { onActivate(mode, st.querySelector("#gbOfActivate")); });
  }
}

function compare(modes, cfg, viewed) {
  var hw = localHardware(modes), all = {};
  MODES.forEach(function (m) { all[m] = facts(m, cfg, hw); });
  return '<div class="gb-of-cmp"><div><div class="gb-of-cr hd"><div></div>' + MODES.map(function (m, i) {
    return '<div class="gb-of-ch' + (m === viewed ? " sel" : "") + '"><div><span class="gb-of-num">0' + (i + 1) + '</span>' +
      (inUseKind(cfg) === m ? '<span class="gb-of-chip xs">' + tr("EM USO", "IN USE") + '</span>' : '') + '</div><strong>' + esc(modeLabel(m)) + '</strong>' +
      '<button type="button" data-cmp-go="' + m + '">' + tr("Ver detalhes", "See details") + svg(I.next, 12, 3) + '</button></div>';
  }).join("") + '</div>' + ROWS.map(function (r) {
    return '<div class="gb-of-cr"><div class="gb-of-cl"><span>' + esc(L2(r[1])) + '</span>' + (r[2] ? help(L2(r[2])) : '') + '</div>' + MODES.map(function (m) {
      var c = all[m][r[0]];
      return '<div class="gb-of-cc' + (m === viewed ? " sel" : "") + '"><span class="gb-of-mk ' + c.k + '">' + MK[c.k] + '</span><div><span>' + esc(c.t) + '</span>' +
        (c.n ? '<small>' + esc(c.n) + '</small>' : '') + '</div></div>';
    }).join("") + '</div>';
  }).join("") + '<div class="gb-of-leg"><span><b class="gb-of-ok">✓</b>' + tr("atende", "meets it") + '</span><span><b class="gb-of-warnc">~</b>' +
    tr("em parte ou depende", "partly or it depends") + '</span><span><b>–</b>' + tr("não atende", "does not meet it") + '</span></div></div></div>';
}

function dialogs() {
  var x = '<button type="button" class="gb-of-x" data-close-dialog aria-label="' + tr("Fechar", "Close") + '">' + svg(I.close, 15, 1.8) + '</button>';
  return '<dialog id="gbGarageGuideDialog" class="gb-of-dlg gb-of-guide" aria-labelledby="gbGarageGuideTitle"><div class="gb-of-dlg-h"><span id="gbGarageGuideTitle">' + sl() +
    tr("ME AJUDE A ESCOLHER", "HELP ME CHOOSE") + '</span><span class="gb-of-sp"></span><span class="gb-of-dots"><i></i><i></i><i></i></span>' + x + '</div>' +
    '<div class="gb-of-gbody" aria-live="polite"></div></dialog><dialog id="gbGarageCompareDialog" class="gb-of-dlg gb-of-side" aria-labelledby="gbGarageCompareTitle">' +
    '<div class="gb-of-dlg-h"><span id="gbGarageCompareTitle">' + sl() + tr("COMPARAR OS 4 MOTORES", "COMPARE THE 4 ENGINES") + '</span><span class="gb-of-sp"></span>' + x + '</div>' +
    '<div class="gb-of-cmpslot"></div></dialog>';
}

/* ---- Guide: two questions, then the engine it suggests ---- */
function bindGuide(panel, modes, dialog) {
  var box = dialog.querySelector(".gb-of-gbody"), hw = localHardware(modes);
  function dots(n) { dialog.querySelectorAll(".gb-of-dots i").forEach(function (d, i) { d.classList.toggle("on", i < n); }); }
  function ask(step, q, hint, opts) {
    dots(step);
    box.innerHTML = '<div class="gb-of-gb"><span class="gb-of-lab">' + esc(tr("PERGUNTA %n DE 2", "QUESTION %n OF 2").replace("%n", step)) + '</span><div><span class="gb-of-gq">' +
      esc(q) + '</span><span class="gb-of-gh">' + esc(hint) + '</span></div><div class="gb-of-opts">' + opts.map(function (o, i) {
        return '<button type="button" class="gb-of-opt" data-a="' + i + '"><span>' + svg(o[2], 15) + '</span><span><b>' + esc(o[0]) + '</b><small>' + esc(o[1]) + '</small></span>' + svg(I.next, 14, 3) + '</button>';
      }).join("") + '</div></div>';
    opts.forEach(function (o, i) { box.querySelector('[data-a="' + i + '"]').addEventListener("click", o[3]); });
  }
  function result(mode) {
    dots(3);
    rec = mode;
    var grid = panel.querySelector("#gbGarModeGrid"); if (grid) { grid.setAttribute("data-rec", mode); }
    var V = A._modelMachine ? A._modelMachine.verdict(hw, S().localModels) : null;
    var why = { local: tr("Nada sai deste computador e funciona sem internet.", "Nothing leaves this computer and it works offline.") +
        (V && V.tone !== "none" ? " " + tr("A sua placa de vídeo: ", "Your graphics card: ") + V.short.toLowerCase() + "." : ""),
      own_key: tr("Os textos vão só ao servidor que a sua organização indicar.", "Your text goes only to the server your organization sets."),
      acp: tr("Usa a assinatura que você já paga, num espaço isolado neste computador.", "Uses the subscription you already pay for, in an isolated space on this computer."),
      free: tr("Não precisa de chave nem de placa de vídeo. Os textos vão a um provedor externo.", "Needs no key and no graphics card. Your text goes to an external provider.") }[mode];
    var card = panel.querySelector('.gb-of-mode[data-mode="' + mode + '"]');
    if (card && card.disabled) {
      why += " " + tr("Ainda não está disponível neste computador.", "It is not available on this computer yet.");
    }
    box.innerHTML = '<div class="gb-of-gb"><span class="gb-of-lab gb-of-ok">' + tr("RECOMENDADO PARA VOCÊ", "RECOMMENDED FOR YOU") + '</span><div><span class="gb-of-gr">' +
      esc(modeLabel(mode)) + '</span><span class="gb-of-gh">' + esc(why) + '</span></div><div class="gb-of-gend"><button type="button" class="gb-of-btn" data-again>' +
      tr("Recomeçar", "Start over") + '</button><button type="button" class="gb-of-btn acc" data-go>' + tr("Ver este motor", "See this engine") + svg(I.arrow, 14, 3) + '</button></div></div>';
    box.querySelector("[data-again]").addEventListener("click", start);
    box.querySelector("[data-go]").addEventListener("click", function () {
      dialog.close();
      var h = panel.querySelector('.gb-of-mode[data-mode="' + mode + '"]');
      if (h && !h.disabled) { h.click(); h.focus({ preventScroll: true }); }
      var d = panel.querySelector(".gb-of-dhead");
      if (d && d.scrollIntoView) { d.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" }); }
    });
  }
  function start() {
    ask(1, tr("O que você escreve pode sair deste computador?", "Can what you write leave this computer?"),
      tr("Pense em processos, dados pessoais e documentos internos. Na dúvida, considere que não podem.", "Think of case files, personal data and internal documents. If in doubt, assume they cannot."), [
      [tr("Não, precisa ficar aqui", "No, it must stay here"), tr("Processos, dados pessoais, documentos internos", "Case files, personal data, internal documents"), I.lock, function () {
        ask(2, tr("Este computador tem placa de vídeo com 6 GB ou mais?", "Does this computer have a graphics card with 6 GB or more?"),
          hw.device ? tr("Já examinamos: %d, com %v GB.", "We already checked: %d, with %v GB.").replace("%d", hw.device).replace("%v", String(hw.vram_gb || 0)) : tr("Não conseguimos examinar a placa deste computador.", "We could not check this computer’s card."), [
          [tr("Sim, quero usar este computador", "Yes, I want to use this computer"), tr("Nada sai daqui", "Nothing leaves it"), INFO.local[5], function () { result("local"); }],
          [tr("Não, ou prefiro o servidor da organização", "No, or I prefer the organization’s server"), tr("Os textos vão à rede da organização", "Your text goes to the organization’s network"), INFO.own_key[5], function () { result("own_key"); }]]);
      }],
      [tr("Sim, pode sair", "Yes, it can"), tr("Textos públicos, rascunhos, pesquisas", "Public text, drafts, research"), I.globe, function () {
        ask(2, tr("Você já paga Claude, ChatGPT ou Gemini?", "Do you already pay for Claude, ChatGPT or Gemini?"),
          tr("Uma assinatura que você já tem pode virar o motor do assistente.", "A subscription you already have can become the assistant’s engine."), [
          [tr("Sim, tenho assinatura", "Yes, I have a subscription"), tr("Usa a sua conta, num espaço isolado", "Uses your account, in an isolated space"), INFO.acp[5], function () { result("acp"); }],
          [tr("Não", "No"), tr("Use o modelo gratuito, sem chave", "Use the free model, no key"), INFO.free[5], function () { result("free"); }]]);
      }]]);
  }
  return start;
}

/* ---- Tooltips: any [data-tip] in the page or its dialogs ---- */
var tipEl = null;
function tipShow(t) {
  if (!tipEl) { tipEl = document.createElement("div"); tipEl.className = "gb-of-tip"; tipEl.setAttribute("role", "tooltip"); }
  (t.closest("dialog[open]") || document.body).appendChild(tipEl);
  tipEl.textContent = t.getAttribute("data-tip");
  var z = window.GBZoom ? window.GBZoom.factor() : 1;
  var r = t.getBoundingClientRect(), below = r.top / z < 90;
  tipEl.style.left = Math.max(160, Math.min(innerWidth / z - 160, (r.left + r.width / 2) / z)) + "px";
  tipEl.style.top = (below ? r.bottom / z + 8 : r.top / z - 8) + "px";
  tipEl.style.transform = below ? "translateX(-50%)" : "translate(-50%,-100%)";
}
function tipHide() { if (tipEl && tipEl.parentNode) { tipEl.remove(); } }
function bindTips(root) {
  ["mouseover", "focusin"].forEach(function (ev) { root.addEventListener(ev, function (e) { var t = e.target.closest && e.target.closest("[data-tip]"); if (t) { tipShow(t); } }); });
  ["mouseout", "focusout"].forEach(function (ev) { root.addEventListener(ev, function (e) { var t = e.target.closest && e.target.closest("[data-tip]"); if (t && !t.contains(e.relatedTarget)) { tipHide(); } }); });
}

function bindDialog(panel, buttonId, dialog, onOpen) {
  var button = panel.querySelector("#" + buttonId);
  if (!button || !dialog) { return; }
  dialog.tabIndex = -1;
  button.onclick = function () { if (onOpen) { onOpen(); } dialog.showModal(); dialog.focus(); };
  dialog.querySelector("[data-close-dialog]").onclick = function () { dialog.close(); };
  dialog.addEventListener("close", tipHide);
  dialog.addEventListener("keydown", function (e) { if (e.key === "Escape") { e.stopPropagation(); } });
  dialog.addEventListener("click", function (e) {
    if (e.target !== dialog) { return; }
    var r = dialog.getBoundingClientRect();
    if (e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom) { dialog.close(); }
  });
  bindTips(dialog);
}

/* ---- Where the model in use runs: "last_launch" of /api/llm/local/models ----
   gpu: a quiet line in its row; split or cpu: a notice above the list. Nothing
   when the field is absent, the placement is unknown, the engine is stopped
   or the record is about another model. */
function launchOf(rec, active, lr) {
  if (!rec || !active || !active.file || !rec.model_file || (lr && lr.running === false)) { return null; }
  var a = String(active.file), b = String(rec.model_file);
  if (a.slice(-b.length) !== b && b.slice(-a.length) !== a) { return null; }
  return ["gpu", "split", "cpu"].indexOf(rec.placement) >= 0 ? rec : null;
}
function gbOf(mib) {
  var n = Number(mib) / 1024, s = n >= 10 ? String(Math.round(n)) : n.toFixed(1);
  return (pt() ? s.replace(".", ",") : s) + " GB";
}
function placeLine(rec) {
  if (!rec || rec.placement !== "gpu") { return ""; }
  return '<span class="gb-of-place">' + svg(I.check, 12, 2.4) + esc(tr("Rodando inteiramente na placa de vídeo.", "Running entirely on the graphics card.")) + '</span>';
}
function placeNotice(rec) {
  if (!rec || (rec.placement !== "split" && rec.placement !== "cpu")) { return ""; }
  var before = rec.vram_free_before_mib, after = rec.vram_free_after_mib, has = typeof before === "number";
  var head, body, step1 = "";
  if (rec.placement === "split") {
    head = tr("Este modelo não cabe inteiro na placa de vídeo agora", "This model does not fully fit on the graphics card right now");
    body = (has ? tr("A placa de vídeo tinha só %f livres quando o modelo abriu.", "The graphics card had only %f free when the model opened.").replace("%f", gbOf(before))
      : tr("A placa de vídeo não tinha espaço livre suficiente quando o modelo abriu.", "The graphics card did not have enough free space when the model opened.")) + " " +
      tr("Por isso, uma parte roda no processador e as respostas demoram bem mais para começar.", "So part of it runs on the processor, and answers take much longer to start.");
    step1 = esc(tr("Feche os programas que usam a placa de vídeo (editores de imagem, navegadores com muitas abas, jogos) e reinicie o modelo.",
      "Close the programs that use the graphics card (image editors, browsers with many tabs, games) and restart the model."));
  } else {
    head = tr("Este modelo roda só no processador", "This model runs on the processor only");
    body = tr("A placa de vídeo não está sendo usada, então as respostas vão ser lentas. Um modelo menor responde mais rápido.",
      "The graphics card is not being used, so answers will be slow. A smaller model answers faster.");
  }
  var steps = (step1 ? '<li>' + step1 + '</li>' : '') +
    '<li>' + esc(tr("Ou escolha um modelo menor, que caiba na placa.", "Or choose a smaller model that fits on the card.")) + '</li>';
  var used = has && typeof after === "number" && before - after >= 103 ? gbOf(before - after) : "";
  var detail = has ? tr("Memória da placa de vídeo: %f livres antes de abrir", "Graphics card memory: %f free before opening").replace("%f", gbOf(before)) +
    (used ? tr(" · %u usados pelo modelo", " · %u used by the model").replace("%u", used) : "") : "";
  var restart = '<button type="button" class="gb-of-btn sm pri" data-restart-model>' + esc(tr("Reiniciar o modelo", "Restart the model")) + '</button>';
  var smaller = '<button type="button" class="gb-of-btn sm" data-pick-smaller>' + esc(tr("Escolher um modelo menor", "Choose a smaller model")) + '</button>';
  return '<div class="gb-of-placen" role="status"><span class="gb-of-placen-i">' + svg(I.info, 16, 1.8) + '</span><div><b>' + esc(head) + '</b><p>' + esc(body) + '</p>' +
    '<ul>' + steps + '</ul>' + (detail ? '<small>' + esc(detail) + '</small>' : '') +
    '<div class="gb-of-placen-msg" id="gbPlaceMsg"></div>' +
    '<div class="gb-of-row">' + restart + smaller + '</div></div></div>';
}
/* Called by 10b when the server returns 404 (endpoint not available). */
function placeFallbackAdvice(box) {
  var btn = box && box.querySelector("[data-restart-model]");
  if (btn) { btn.remove(); }
  var li = box && box.querySelector("ul li");
  if (li && li.textContent.indexOf("reinicie") >= 0) {
    li.textContent = tr("Feche os programas que usam a placa de vídeo (editores de imagem, navegadores com muitas abas, jogos). Depois feche e abra o aplicativo para reiniciar o modelo.",
      "Close the programs that use the graphics card (image editors, browsers with many tabs, games). Then close and reopen the app to restart the model.");
  }
}

A._modelGarage = {
  launchOf: launchOf, placeLine: placeLine, placeNotice: placeNotice, placeFallbackAdvice: placeFallbackAdvice,
  dashboard: dashboard, runCard: runCard, markInUse: markInUse, inUseKind: inUseKind,
  modeButtons: modeButtons, dialogs: dialogs, modeLabel: modeLabel, setDetail: setDetail, help: help, svg: svg, sl: sl,
  bind: function (panel, modes, viewed) {
    var cmp = panel.querySelector("#gbGarageCompareDialog"), guide = panel.querySelector("#gbGarageGuideDialog");
    bindDialog(panel, "gbOpenCompare", cmp, function () {
      cmp.querySelector(".gb-of-cmpslot").outerHTML = compare(modes, S().cfg, viewed());
      cmp.querySelectorAll("[data-cmp-go]").forEach(function (b) {
        b.addEventListener("click", function () { cmp.close(); var h = panel.querySelector('.gb-of-mode[data-mode="' + b.getAttribute("data-cmp-go") + '"]'); if (h && !h.disabled) { h.click(); } });
      });
      cmp.querySelector(".gb-of-cmp").classList.add("gb-of-cmpslot");
    });
    if (guide) { bindDialog(panel, "gbOpenGuide", guide, bindGuide(panel, modes, guide)); }
    bindTips(panel);
  }
};
})();
