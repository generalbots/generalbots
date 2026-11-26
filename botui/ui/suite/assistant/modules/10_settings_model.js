"use strict";
/**
 * 10_settings_model.js - Settings > Modelo: full model configuration.
 * Called from 09_settings.js renderModel(panel).
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function fmtBytes(b){if(!b)return"0 B";if(b<1024)return b+" B";if(b<1048576)return(b/1024).toFixed(1)+" KB";if(b<1073741824)return(b/1048576).toFixed(1)+" MB";return(b/1073741824).toFixed(1)+" GB";}
function fmtSpeed(bps){return bps?fmtBytes(bps)+"/s":"";}
function fmtEta(s){if(!s&&s!==0)return"";if(s<60)return s+"s";if(s<3600)return Math.floor(s/60)+"m "+(s%60)+"s";return Math.floor(s/3600)+"h "+Math.floor((s%3600)/60)+"m";}
var jobPollers = {};
var T = {"pt-BR":{
  now:"Agora",where:"Onde o assistente pensa",
  subLocal:"Nada sai deste computador",subOwn:"Servidor da sua organização ou chave de um provedor",
  subFree:"Hospedado fora deste computador",
  useFree:"Usar o modelo gratuito",useOwn:"Usar este servidor",inUse:"Em uso agora.",
  switching:"Trocando o modelo...",
  loadingLocal:"Carregando o modelo neste computador. Pode levar até dois minutos.",
  errEngineOld:"O motor local instalado é mais antigo que este modelo e não consegue abri-lo. Atualize o motor local ou escolha outro modelo.",
  errMemory:"Não há memória livre suficiente para este modelo. Feche outros programas ou escolha um modelo menor.",
  errStart:"O modelo não pôde ser iniciado neste computador.",
  errVault:"A escolha não pôde ser gravada: o cofre de segredos não está disponível.",
  errNotDownloaded:"Este modelo ainda não foi baixado.",
  errGeneric:"Não foi possível trocar o modelo.",
  localFailed:"O modelo local %m não pôde ser iniciado.",
  hw:"Placa: %d, memória: %v GB VRAM (%f GB livre), %r GB RAM",
  hwCpu:"Processador (sem GPU dedicada), %r GB RAM",
  hwDriver:"Driver: %drv",hwCuda:"Aceleração CUDA disponível",
  active:"Em uso",downloaded:"Baixado",downloading:"Baixando...",available:"Disponível",
  use:"Usar este modelo",install:"Instalar",remove:"Remover",cancel:"Cancelar",
  installable:"Modelos que você pode instalar",installed:"Modelos instalados",
  confirmDl:"Este modelo tem %s e será baixado de %h pela internet. Continuar?",
  confirmExt:"O que você escrever e anexar será enviado a %h, fora deste computador. Continuar?",
  endpoint:"URL do endpoint",model:"Modelo",key:"Chave de API",
  keyStored:"chave armazenada",test:"Testar",save:"Salvar",
  testOk:"Alcançável, latência %ms ms",testFail:"Falha: %e",
  saved:"Salvo. Vale a partir da próxima mensagem.",
  swapped:"Pronto. O novo modelo já está ativo.",
  errAdmin:"Somente o administrador pode alterar o modelo.",
  runtimeNotInstalled:"O motor de inferência local não está instalado.",
  runtimeNotReachable:"O motor de inferência local não está respondendo.",
  restartNeeded:"Ativado. Reinicie o aplicativo para usar o novo modelo.",
  inconsistent:"A configuração do modelo está inconsistente.",
  restoreFree:"Restaurar modelo gratuito",
  freeNote:"Tudo que você escrever e anexar é enviado a %h.",
  freeHost:"um provedor externo",freeLimits:"Não precisa de chave. Os limites de uso são menores.",
  noModels:"Nenhum modelo local disponível.",
  comfortable:"confortável nesta máquina",tight:"apertado nesta máquina",
  wontRun:"não roda (precisa de %v GB)",
  fits:"cabe nesta máquina",nofit:"não cabe (precisa %v GB)",serverOnly:"apenas servidor",
  licence:"Licença",source:"Fonte",
  privLocal:"Suas conversas ficam neste computador. Nada é enviado para fora.",
  privNet:"Suas conversas são enviadas ao servidor da sua organização (%h).",
  privExt:"Suas conversas são enviadas a um servidor externo (%h).",
  noImageReading:"O texto de imagens e digitalizações é lido neste computador; fotos e gráficos ainda não são interpretados.",
  serverDown:"O serviço do assistente não está respondendo. Tente de novo em alguns segundos; se continuar, feche o aplicativo e abra outra vez.",
  retry:"Tentar de novo"},
en:{
  now:"Now",where:"Where the assistant thinks",
  subLocal:"Nothing leaves this computer",subOwn:"Your organization’s server or a provider key",
  subFree:"Hosted outside this computer",
  useFree:"Use the free model",useOwn:"Use this server",inUse:"In use now.",
  switching:"Switching the model...",
  loadingLocal:"Loading the model on this computer. This can take up to two minutes.",
  errEngineOld:"The installed local engine is older than this model and cannot open it. Update the local engine or choose another model.",
  errMemory:"There is not enough free memory for this model. Close other programs or choose a smaller model.",
  errStart:"The model could not be started on this computer.",
  errVault:"The choice could not be saved: the secret store is not available.",
  errNotDownloaded:"This model has not been downloaded yet.",
  errGeneric:"The model could not be switched.",
  localFailed:"The local model %m could not be started.",
  hw:"Device: %d, memory: %v GB VRAM (%f GB free), %r GB RAM",
  hwCpu:"Processor (no dedicated GPU), %r GB RAM",
  hwDriver:"Driver: %drv",hwCuda:"CUDA acceleration available",
  active:"Active",downloaded:"Downloaded",downloading:"Downloading...",available:"Available",
  use:"Use this model",install:"Install",remove:"Remove",cancel:"Cancel",
  installable:"Models you can install",installed:"Installed models",
  confirmDl:"This model is %s and will be downloaded from %h over the internet. Continue?",
  confirmExt:"What you write and attach will be sent to %h, outside this computer. Continue?",
  endpoint:"Endpoint URL",model:"Model",key:"API key",
  keyStored:"key stored",test:"Test",save:"Save",
  testOk:"Reachable, latency %ms ms",testFail:"Failed: %e",
  saved:"Saved. Takes effect from the next message.",
  swapped:"Ready. The new model is now active.",
  errAdmin:"Only the administrator can change the model.",
  runtimeNotInstalled:"The local inference engine is not installed.",
  runtimeNotReachable:"The local inference engine is not responding.",
  restartNeeded:"Activated. Restart the app to use the new model.",
  inconsistent:"The model configuration is inconsistent.",
  restoreFree:"Restore free model",
  freeNote:"Everything you write and attach is sent to %h.",
  freeHost:"an external provider",freeLimits:"No key needed. Usage limits are lower.",
  noModels:"No local models available.",
  comfortable:"comfortable for this machine",tight:"tight for this machine",
  wontRun:"won’t run (needs %v GB)",
  fits:"fits this machine",nofit:"does not fit (needs %v GB)",serverOnly:"server only",
  licence:"Licence",source:"Source",
  privLocal:"Your conversations stay on this computer. Nothing is sent outside.",
  privNet:"Your conversations are sent to your organization’s server (%h).",
  privExt:"Your conversations are sent to an external host (%h).",
  noImageReading:"Text in images and scans is read on this computer; photos and charts are not interpreted yet.",
  serverDown:"The assistant service is not answering. Try again in a few seconds; if it persists, close the app and open it again.",
  retry:"Try again"}};
function t(key) {
  var lang = A.locale && A.locale.startsWith("pt") ? "pt-BR" : "en";
  return (T[lang] && T[lang][key]) || (T.en[key]) || key;
}

/* ---- In-page confirm + error text (split to 10l_settings_confirm.js) ---- */
function inlineConfirm(text, onYes) {
  if (typeof A._inlineConfirm === "function") { A._inlineConfirm(text, onYes); }
}
function errText(code) {
  return typeof A._errText === "function" ? A._errText(code) : t("errGeneric");
}
/* One switch at a time: every mode and action button waits while it runs. */
function setBusy(on, msgEl, text) {
  var panel = document.querySelector('#gbSetPanel[data-section="model"]');
  if (panel) { if (on) panel.setAttribute("aria-busy", "true"); else panel.removeAttribute("aria-busy"); }
  if (on && msgEl) { msgEl.innerHTML = '<p class="gb-set-busy" role="status">' + esc(text || t("switching")) + '</p>'; }
}

/* state — exposed through A._modelShared for the split local-models module */
var cfg = null, modes = null, localModels = null, readOnly = false;
var cfgError = null;
var pendingStatus = null; /* {html, isOk} shown after the next re-render */
var modelRender = 0;
var viewMode = null;      /* the mode card being viewed; kept when a list refreshes */
A._modelShared = {
  get cfg() { return cfg; },
  get localModels() { return localModels; },
  set localModels(v) { localModels = v; },
  t: t,
  shortName: shortName,
  errText: errText,
  setBusy: setBusy,
  inlineConfirm: inlineConfirm,
  doSwitchMode: function (body, msgEl, btn) { return doSwitchMode(body, msgEl, btn); },
  activate: function (mode, btn) { return activate(mode, btn); },
  refreshAfterChange: function (picked) { return refreshAfterChange(picked); },
  /* Lists changed (download, removal, engine install) but the model in use did not. */
  refreshKeepingView: function () {
    var panel = document.querySelector('#gbSetPanel[data-section="model"]');
    return panel ? A.renderModelFull(panel, true) : null;
  },
  /* A switch asked from the chat's model menu failed: Modelo opens with the reason. */
  showFailure: function (code) {
    pendingStatus = { html: '<p class="gb-set-warn">' + esc(errText(code)) + '</p>' };
    if (typeof A.openSettings === "function") { A.openSettings("model"); }
  }
};

A.renderModelFull = function (panel, keepView) {
  var revision = ++modelRender;
  if (!keepView) { viewMode = null; }
  readOnly = false;
  cfgError = null;
  cfg = null;
  modes = [];

  /* The three reads run together: each waits on the local engine probe, so
     one after the other they took about seven seconds when it is stopped. */
  function getJson(path) {
    return A.apiFetch(path).then(function (r) {
      return r.ok ? r.json().then(function (d) { return { data: d }; }) : { status: r.status };
    }).catch(function (e) { return { error: e.message || String(e) }; });
  }
  return Promise.all([
    getJson("/api/llm/config"), getJson("/api/llm/config/modes"), getJson("/api/llm/runtime/status")
  ]).then(function (res) {
    if (!panel.isConnected || revision !== modelRender) { return; }
    var c = res[0], m = res[1], rt = res[2];
    if (c.status === 401 || c.status === 403) { readOnly = true; }
    else if (c.data) { cfg = c.data; }
    else { cfgError = c.status ? "HTTP " + c.status : (c.error || ""); }
    if (m.data) { modes = m.data.modes || m.data || []; }
    if (rt.data && cfg && !cfg.local_runtime) { cfg.local_runtime = rt.data; }
    renderFull(panel);
  });
};

function renderFull(panel) {
  /* Current model summary card */
  var modelName = cfg ? cfg.model : (A.defaultModel || A.selectedModel || "");
  var scope = cfg ? cfg.endpoint_scope : (A.destination && A.destination.scope || "");
  var host = cfg ? cfg.endpoint_host : (A.destination && A.destination.host || "");
  var currentMode = cfg ? (cfg.mode || "") : "";
  var G = A._modelGarage;

  var html;
  var editable = !readOnly && !cfgError;
  if (G) {
    html = '<div class="gb-of">' + G.dashboard(cfg, editable) + '<div class="gb-gar-dash-warn">';
  } else {
    html = '<div class="gb-set-card"><h2 class="gb-set-title">' + esc(t("now")) + '</h2>' +
      '<div class="gb-set-field"><span class="gb-set-label">' + esc(A.t("settings.current-model","")) + '</span>' +
      '<span class="gb-set-value gb-set-model-name">' + esc(shortName(modelName)) + '</span></div>' + destLine(scope, host);
  }
  if (readOnly) {
    html += '<p class="gb-set-hint gb-set-warn">' + esc(t("errAdmin")) + '</p>';
  }
  if (cfgError) {
    /* A network error (the server is not answering) says so in words; an HTTP
       status keeps its code for support. */
    var down = !/^HTTP /.test(cfgError);
    html += '<div class="gb-gar-down" role="alert"><p class="gb-set-warn">' +
      esc(down ? t("serverDown") : t("errGeneric") + " (" + cfgError + ")") + '</p>' +
      '<button class="gb-set-btn-sm" id="gbModelRetry">' + esc(t("retry")) + '</button></div>';
  }
  html += '</div>';

  /* Inconsistent state banner */
  if (cfg && cfg.inconsistent) {
    html += '<div class="gb-set-card gb-set-warn-card">' +
      '<p class="gb-set-warn">' + esc(t("inconsistent")) + '</p>' +
      '<button class="gb-set-btn-sm gb-set-btn-accent" id="gbRestoreFree">' +
      esc(t("restoreFree")) + '</button>' +
      '<div id="gbRestoreMsg"></div></div>';
  }

  /* The stored local model did not start: say why, offer the way out */
  var lf = cfg && cfg.local_runtime && cfg.local_runtime.last_failure;
  if (lf && !cfg.inconsistent) {
    html += '<div class="gb-set-card gb-set-warn-card">' +
      '<p class="gb-set-warn">' + esc(t("localFailed").replace("%m", shortName(lf.model))) + ' ' + esc(errText(lf.code)) + '</p>' +
      (currentMode === "local" ? '<button class="gb-set-btn-sm gb-set-btn-accent" id="gbRestoreFree">' +
      esc(t("useFree")) + '</button><div id="gbRestoreMsg"></div>' : '') + '</div>';
  }

  /* Guide and the three ways compared — always shown unless read-only (admin restriction) */
  if (!readOnly && G && !cfgError) {
    html += G.modeButtons(G.inUseKind(cfg), modes, cfg) + G.dialogs(modes, cfg);
  }
  if (G) { html += '</div>'; }

  if (pendingStatus) {
    html += '<div class="gb-set-card" id="gbModelStatus">' + pendingStatus.html + '</div>';
    pendingStatus = null;
  }
  panel.innerHTML = html;
  bindRestoreButton();
  var retry = document.getElementById("gbModelRetry");
  if (retry) { retry.addEventListener("click", function () { retry.disabled = true; A.renderModelFull(panel, true); }); }
  if (!readOnly) { bindModeCards(panel); if (G) { G.bind(panel, modes, function () { return viewMode; }); } }
}

function shortName(m) { if (!m) return "—"; var s=m; var i=s.lastIndexOf("/"); if(i>=0)s=s.substring(i+1); var c=s.indexOf(":"); if(c>0)s=s.substring(0,c); return s||m; }

function destLine(scope, host) {
  if (scope === "machine") return '<p class="gb-set-privacy">' + esc(t("privLocal")) + '</p>';
  if (scope === "network") return '<p class="gb-set-privacy">' + esc(t("privNet").replace("%h", host || "")) + '</p>';
  if (scope === "external") return '<p class="gb-set-privacy"><span class="gb-set-external-warn">' + esc(t("privExt").replace("%h", host || "")) + '</span></p>';
  return '';
}

function bindRestoreButton() {
  var btn = document.getElementById("gbRestoreFree");
  if (!btn) { return; }
  btn.addEventListener("click", function () {
    btn.disabled = true;
    doSwitchMode({ mode: "free" }, document.getElementById("gbRestoreMsg"), btn);
  });
}

/** Shared PUT /api/llm/config handler for free, own_key and restore. */
function doSwitchMode(body, msgEl, disableBtn) {
  if (disableBtn) { disableBtn.disabled = true; }
  setBusy(true, msgEl, body.mode === "local" ? t("loadingLocal") : t("switching"));
  A.apiFetch("/api/llm/config", { method: "PUT", body: JSON.stringify(body) })
    .then(function (r) {
      return r.json().catch(function () { return { error: "http_" + r.status }; })
        .then(function (d) { return { ok: r.ok, data: d }; });
    })
    .then(function (res) {
      var d = res.data || {};
      setBusy(false);
      if (!res.ok || d.error) {
        if (msgEl) { msgEl.innerHTML = '<p class="gb-set-warn">' + esc(errText(d.error)) + '</p>'; }
        if (disableBtn) { disableBtn.disabled = false; }
        return;
      }
      if (d.restart_required) {
        pendingStatus = { html: '<p class="gb-set-hint">' + esc(t("restartNeeded")) + '</p>' };
      } else {
        var label = d.provider_swapped ? t("swapped") : t("saved");
        pendingStatus = { html: '<p class="gb-set-hint gb-set-ok">' + esc(label) + '</p>' };
      }
      refreshAfterChange(true);
    })
    .catch(function () {
      setBusy(false);
      if (msgEl) { msgEl.innerHTML = '<p class="gb-set-warn">' + esc(errText("")) + '</p>'; }
      if (disableBtn) { disableBtn.disabled = false; }
    });
}

function bindModeCards(panel) {
  panel.querySelectorAll(".gb-of-mode").forEach(function(btn) {
    btn.addEventListener("click", function() {
      var mode = btn.getAttribute("data-mode");
      renderModeDetail(panel, mode);
    });
  });
  if (cfg) renderModeDetail(panel, viewMode || (A._modelGarage && A._modelGarage.inUseKind(cfg)) || cfg.mode || "local");
}

function renderModeDetail(panel, mode) {
  panel.querySelectorAll("#gbOfModes .gb-of-mode").forEach(function(b) {
    b.setAttribute("aria-pressed", String(b.getAttribute("data-mode") === mode));
  });
  viewMode = mode;
  var content = document.getElementById("gbModeDetail");
  if (!content) { return; }
  content.innerHTML = "";
  if (A._modelGarage) { A._modelGarage.setDetail(panel, mode, cfg, activate); }
  if (mode === "local") renderLocalDetail(content);
  else if (mode === "own_key") renderOwnKeyDetail(content);
  else if (mode === "free") renderFreeDetail(content);
  else if (mode === "acp" && A._modelProviders && A._modelProviders.renderAcp) A._modelProviders.renderAcp(content);
}

/* The head button of the detail frame turns the viewed engine on with the
   action its own panel offers: a local model, the provider form, the free
   model or a connected agent. */
function activate(mode, btn) {
  var msg = document.getElementById("gbOfActMsg");
  function say(text) { if (msg) { msg.innerHTML = '<p class="gb-set-warn">' + esc(text) + '</p>'; } }
  if (mode === "free") { askFree(btn); return; }
  if (mode === "local") {
    var lm = localModels || {}, pick = null;
    (lm.tiers || []).forEach(function (tier) { (tier.models || []).forEach(function (m) {
      if (m.downloaded && (!pick || m.id === lm.recommended)) { pick = m; }
    }); });
    if (pick && A._modelLocal && A._modelLocal.activate) { A._modelLocal.activate(pick.id); return; }
    say(A.settingsText("Instale um modelo da lista abaixo primeiro.", "Install a model from the list below first."));
    return;
  }
  var target = document.querySelector(mode === "own_key" ? "#gbProvSave" : "[data-agent-use]");
  if (target) { target.click(); return; }
  say(A.settingsText("Conecte um agente abaixo primeiro.", "Connect an agent below first."));
}
function freeHost() { return cfg && cfg.endpoint_host && cfg.mode === "free" ? cfg.endpoint_host : t("freeHost"); }
function askFree(btn) {
  inlineConfirm(t("confirmExt").replace("%h", freeHost()), function () {
    doSwitchMode({ mode: "free" }, document.getElementById("gbOfActMsg"), btn);
  });
}

function renderLocalDetail(el) {
  if (A._modelLocal && A._modelLocal.renderLocalDetail) {
    A._modelLocal.renderLocalDetail(el);
  }
}

/* ---- Own Key detail: the provider catalog lives in 10e_settings_providers.js ---- */
function renderOwnKeyDetail(el) {
  if (A._modelProviders && A._modelProviders.render) { A._modelProviders.render(el); return; }
  /* Fallback for a build that does not ship the provider module. */
  var ep = (cfg && cfg.mode === "own_key" && cfg.endpoint_host) ? cfg.endpoint_host : "";
  var md = (cfg && cfg.mode === "own_key") ? cfg.model : "";
  var hasKey = cfg && cfg.has_key;
  el.innerHTML = '<div class="gb-set-form">' +
    '<label class="gb-set-label">' + esc(t("endpoint")) + '</label>' +
    '<input class="gb-set-input" id="gbOwnEndpoint" value="' + esc(ep) + '" placeholder="https://api.example.com/v1/chat/completions"/>' +
    '<label class="gb-set-label">' + esc(t("model")) + '</label>' +
    '<input class="gb-set-input" id="gbOwnModel" value="' + esc(md) + '" placeholder="gpt-4o-mini"/>' +
    '<label class="gb-set-label">' + esc(t("key")) + (hasKey ? ' <small>(' + esc(t("keyStored")) + ')</small>' : '') + '</label>' +
    '<input class="gb-set-input" id="gbOwnKey" type="password" placeholder="sk-..." autocomplete="off"/>' +
    '<div class="gb-set-form-actions">' +
      '<button class="gb-set-btn-sm" id="gbOwnTest">' + esc(t("test")) + '</button>' +
      '<button class="gb-set-btn-sm gb-set-btn-accent" id="gbOwnSave">' + esc(t("useOwn")) + '</button>' +
    '</div><div id="gbOwnMsg"></div></div>';
  document.getElementById("gbOwnTest").addEventListener("click", doTestEndpoint);
  document.getElementById("gbOwnSave").addEventListener("click", doSaveOwnKey);
}

async function doTestEndpoint() {
  var ep = document.getElementById("gbOwnEndpoint").value.trim();
  var md = document.getElementById("gbOwnModel").value.trim();
  var key = document.getElementById("gbOwnKey").value;
  var msg = document.getElementById("gbOwnMsg"); if (!msg) return;
  msg.innerHTML = '<p class="gb-set-hint">' + esc(A.t("assistant.connecting","Testing...")) + '</p>';
  try {
    var body = { endpoint: ep, model: md }; if (key) body.api_key = key;
    var r = await A.apiFetch("/api/llm/config/test", { method: "POST", body: JSON.stringify(body) });
    var d = await r.json();
    if (d.reachable && d.model_responded) {
      msg.innerHTML = '<p class="gb-set-hint gb-set-ok">' + esc(t("testOk").replace("%ms", String(d.latency_ms||"?"))) + '</p>';
    } else {
      var errText = d.error || "unreachable";
      msg.innerHTML = '<p class="gb-set-warn">' + esc(t("testFail").replace("%e", errText)) + '</p>';
    }
  } catch (e) { msg.innerHTML = '<p class="gb-set-warn">' + esc(String(e)) + '</p>'; }
}

function doSaveOwnKey() {
  var ep = document.getElementById("gbOwnEndpoint").value.trim();
  var md = document.getElementById("gbOwnModel").value.trim();
  var key = document.getElementById("gbOwnKey").value;
  var msg = document.getElementById("gbOwnMsg"); if (!msg) return;
  var saveBtn = document.getElementById("gbOwnSave");
  var host = ""; try { host = new URL(ep).hostname; } catch(e) { host = ep; }
  inlineConfirm(t("confirmExt").replace("%h", host), function () {
    var body = { mode: "own_key", endpoint: ep, model: md }; if (key) body.api_key = key;
    doSwitchMode(body, msg, saveBtn);
    var keyEl = document.getElementById("gbOwnKey"); if (keyEl) keyEl.value = "";
  });
}

/* ---- Free detail ---- */
function renderFreeDetail(el) {
  var G = A._modelGarage;
  el.innerHTML = '<div class="gb-of-sec"><div class="gb-of-sec-h"><span class="gb-of-letter">A</span><span class="gb-of-sec-t">' +
    esc(A.settingsText("Antes de usar", "Before you use it")) + '</span><span class="gb-of-sp"></span><span class="gb-of-meta">' +
    esc(A.settingsText("SEM CHAVE · LIMITES MENORES", "NO KEY · LOWER LIMITS")) + '</span></div>' +
    '<div class="gb-of-sec-b" style="max-width:720px"><div class="gb-of-alert">' + (G ? G.svg("M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 8v5M12 16h.01", 16, 1.8) : '') +
    '<div><b>' + esc(t("freeNote").replace("%h", freeHost())) + '</b><span>' +
    esc(A.settingsText("Use para textos públicos, rascunhos e pesquisas. Não precisa de chave; os limites de uso são menores.",
      "Use it for public text, drafts and research. No key needed; usage limits are lower.")) + '</span></div></div></div></div>';
}

/* `picked`: the person turned an engine on here, so the chat leaves the agent
   or key model it had chosen in the menu and uses this one. */
async function refreshAfterChange(picked) {
  if (A._engineNow) { A._engineNow.refreshHosts(); }
  if (picked && /^(sub|acct):/.test(A.selectedModel || "")) {
    A.selectedModel = "";
    try { localStorage.removeItem("gb-model-preference"); } catch (e) { /* private window */ }
  }
  if (typeof A.loadModels === "function") { await A.loadModels(); }
  if (picked && typeof A.syncAgentToggle === "function") { A.syncAgentToggle(); }
  if (typeof A.renderModelPickers === "function") { A.renderModelPickers(); }
  if (typeof A.renderStatus === "function") { A.renderStatus(); }
  var panel = document.querySelector('#gbSetPanel[data-section="model"]');
  if (panel) { A.renderModelFull(panel); }
}
})();
