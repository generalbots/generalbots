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
  now:"Agora",where:"Onde o assistente pensa",local:"Neste computador",
  own:"Servidor da sua organização ou chave",free:"Modelo gratuito hospedado",
  hw:"Placa: %d, memória: %v GB VRAM, %r GB RAM",
  active:"Em uso",downloaded:"Baixado",downloading:"Baixando...",available:"Disponível",
  use:"Usar este",install:"Instalar",remove:"Remover",cancel:"Cancelar",
  installable:"Modelos que você pode instalar",installed:"Modelos instalados",
  confirmDl:"Este modelo tem %s e será baixado de %h pela internet. Continuar?",
  confirmExt:"O que você escrever e anexar será enviado a %h, fora deste computador. Continuar?",
  endpoint:"URL do endpoint",model:"Modelo",key:"Chave de API",
  keyStored:"chave armazenada",test:"Testar",save:"Salvar",
  testOk:"Alcançável, latência %ms ms",testFail:"Falha: %e",
  saved:"Salvo. Vale a partir da próxima mensagem.",
  errAdmin:"Somente o administrador pode alterar o modelo.",
  runtimeNotInstalled:"O motor de inferência local não está instalado.",
  runtimeNotReachable:"O motor de inferência local não está respondendo.",
  restartNeeded:"Ativado. Reinicie o aplicativo para usar o novo modelo.",
  freeNote:"Tudo que você escrever e anexar é enviado a %h.",
  noModels:"Nenhum modelo local disponível.",
  fits:"cabe nesta máquina",nofit:"não cabe (precisa %v GB)",serverOnly:"apenas servidor"},
en:{
  now:"Now",where:"Where the assistant thinks",local:"On this computer",
  own:"Your organization’s server or provider key",free:"Free hosted model",
  hw:"Device: %d, memory: %v GB VRAM, %r GB RAM",
  active:"Active",downloaded:"Downloaded",downloading:"Downloading...",available:"Available",
  use:"Use this",install:"Install",remove:"Remove",cancel:"Cancel",
  installable:"Models you can install",installed:"Installed models",
  confirmDl:"This model is %s and will be downloaded from %h over the internet. Continue?",
  confirmExt:"What you write and attach will be sent to %h, outside this computer. Continue?",
  endpoint:"Endpoint URL",model:"Model",key:"API key",
  keyStored:"key stored",test:"Test",save:"Save",
  testOk:"Reachable, latency %ms ms",testFail:"Failed: %e",
  saved:"Saved. Takes effect from the next message.",
  errAdmin:"Only the administrator can change the model.",
  runtimeNotInstalled:"The local inference engine is not installed.",
  runtimeNotReachable:"The local inference engine is not responding.",
  restartNeeded:"Activated. Restart the app to use the new model.",
  freeNote:"Everything you write and attach is sent to %h.",
  noModels:"No local models available.",
  fits:"fits this machine",nofit:"does not fit (needs %v GB)",serverOnly:"server only"}};
function t(key) {
  var lang = A.locale && A.locale.startsWith("pt") ? "pt-BR" : "en";
  return (T[lang] && T[lang][key]) || (T.en[key]) || key;
}

/* ---- In-page confirm (replaces window.confirm) ---- */
function inlineConfirm(text, onYes) {
  var overlay = document.createElement("div");
  overlay.className = "gb-inline-dialog";
  var A2 = window.GBAssistant;
  var yesLabel = A2 && A2.t ? A2.t("assistant.confirm-continue", "Continue?") : "OK";
  var noLabel = A2 && A2.t ? A2.t("assistant.cancel", "Cancel") : "Cancel";
  overlay.innerHTML =
    '<div class="gb-inline-dialog-backdrop"></div>' +
    '<div class="gb-inline-dialog-box">' +
      '<div class="gb-inline-dialog-text">' + esc(text) + '</div>' +
      '<div class="gb-inline-dialog-actions">' +
        '<button class="gb-set-btn-sm" data-act="no">' + esc(noLabel) + '</button>' +
        '<button class="gb-set-btn-sm gb-set-btn-accent" data-act="yes">' + esc(yesLabel) + '</button>' +
      '</div></div>';
  document.body.appendChild(overlay);
  var noBtn = overlay.querySelector('[data-act="no"]');
  if (noBtn) noBtn.focus();
  function close() { overlay.remove(); document.removeEventListener("keydown", escH); }
  function escH(e) { if (e.key === "Escape") { e.preventDefault(); close(); } }
  document.addEventListener("keydown", escH);
  overlay.querySelector(".gb-inline-dialog-backdrop").addEventListener("click", close);
  overlay.querySelector('[data-act="no"]').addEventListener("click", close);
  overlay.querySelector('[data-act="yes"]').addEventListener("click", function () { close(); onYes(); });
}

/* state */
var cfg = null, modes = null, localModels = null, readOnly = false;
var cfgError = null;

A.renderModelFull = function (panel) {
  readOnly = false;
  cfgError = null;
  cfg = null;
  modes = [];

  /* Chain with .then() — avoids async/await edge cases in WebView2. */
  return A.apiFetch("/api/llm/config").then(function (r) {
    if (r.status === 401 || r.status === 403) { readOnly = true; return null; }
    if (r.ok) { return r.json(); }
    cfgError = "HTTP " + r.status; return null;
  }).catch(function (e) { cfgError = e.message || String(e); return null;
  }).then(function (cfgData) {
    if (cfgData) { cfg = cfgData; }
    if (readOnly) { return null; }
    return A.apiFetch("/api/llm/config/modes").then(function (r2) {
      if (r2.ok) { return r2.json(); }
      return null;
    }).catch(function () { return null; });
  }).then(function (modesData) {
    if (modesData) { modes = modesData.modes || modesData || []; }
    return A.apiFetch("/api/llm/runtime/status").then(function (r3) {
      if (r3.ok) { return r3.json(); }
      return null;
    }).catch(function () { return null; });
  }).then(function (runtimeData) {
    if (runtimeData && cfg && !cfg.local_runtime) { cfg.local_runtime = runtimeData; }
    renderFull(panel);
  }).catch(function (e) {
    cfgError = cfgError || (e.message || String(e));
    renderFull(panel);
  });
};

function renderFull(panel) {
  /* Current model summary card */
  var modelName = cfg ? cfg.model : (A.defaultModel || A.selectedModel || "");
  var scope = cfg ? cfg.endpoint_scope : (A.destination && A.destination.scope || "");
  var host = cfg ? cfg.endpoint_host : (A.destination && A.destination.host || "");
  var currentMode = cfg ? (cfg.mode || "") : "";

  var html = '<div class="gb-set-card"><h2 class="gb-set-title">' + esc(t("now")) + '</h2>';
  html += '<div class="gb-set-field"><span class="gb-set-label">' +
    esc(A.t("settings.current-model","")) + '</span>' +
    '<span class="gb-set-value gb-set-model-name">' + esc(shortName(modelName)) + '</span></div>';
  html += destLine(scope, host);
  if (readOnly) {
    html += '<p class="gb-set-hint gb-set-warn">' + esc(t("errAdmin")) + '</p>';
  }
  if (cfgError) {
    html += '<p class="gb-set-hint gb-set-warn">' + esc(cfgError) + '</p>';
  }
  html += '</div>';

  /* Mode cards — always shown unless read-only (admin restriction) */
  if (!readOnly) {
    html += '<h3 class="gb-set-section-title">' + esc(t("where")) + '</h3>';
    html += '<div class="gb-set-modes" id="gbModelModes">';
    html += modeCard("local", currentMode);
    html += modeCard("own_key", currentMode);
    html += modeCard("free", currentMode);
    html += '</div>';
  }

  panel.innerHTML = html;
  if (!readOnly) { bindModeCards(panel); }
}

function shortName(m) { if (!m) return "—"; var s=m; var i=s.lastIndexOf("/"); if(i>=0)s=s.substring(i+1); var c=s.indexOf(":"); if(c>0)s=s.substring(0,c); return s||m; }

function destLine(scope, host) {
  if (scope === "machine") return '<p class="gb-set-privacy">' + esc(A.t("settings.model-machine","")) + '</p>';
  if (scope === "network") return '<p class="gb-set-privacy">' + esc(A.t("settings.model-network","")) + (host ? " ("+esc(host)+")." : ".") + '</p>';
  if (scope === "external") return '<p class="gb-set-privacy"><span class="gb-set-external-warn">' + esc(A.t("settings.model-external","")) + " " + esc(host||"") + '.</span></p>';
  return '';
}

function modeCard(id, currentMode) {
  var sel = currentMode === id;
  var cls = "gb-set-mode-card" + (sel ? " gb-set-mode-active" : "");
  var label = t(id === "own_key" ? "own" : id);
  var modeData = modes && modes.find(function(m){return m.id===id});
  var avail = !modeData || modeData.available !== false;
  /* Icon per mode */
  var icons = {
    local: '<svg class="gb-set-mode-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>',
    own_key: '<svg class="gb-set-mode-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
    free: '<svg class="gb-set-mode-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>'
  };
  var icon = icons[id] || "";
  return '<button class="' + cls + '" data-mode="' + id + '"' + (avail ? '' : ' disabled') + '>' +
    icon +
    '<span class="gb-set-mode-label">' + esc(label) + '</span>' +
    (sel ? '<span class="gb-badge gb-badge-active">' + esc(t("active")) + '</span>' : '') +
    '</button>';
}

function bindModeCards(panel) {
  panel.querySelectorAll(".gb-set-mode-card").forEach(function(btn) {
    btn.addEventListener("click", function() {
      var mode = btn.getAttribute("data-mode");
      panel.querySelectorAll(".gb-set-mode-card").forEach(function(b) { b.classList.remove("gb-set-mode-active"); });
      btn.classList.add("gb-set-mode-active");
      renderModeDetail(panel, mode);
    });
  });
  if (cfg) renderModeDetail(panel, cfg.mode || "local");
}

function renderModeDetail(panel, mode) {
  var existing = document.getElementById("gbModeDetail");
  if (existing) existing.remove();
  var detail = document.createElement("div");
  detail.id = "gbModeDetail";
  detail.className = "gb-set-card gb-set-mode-detail";
  var modesDiv = document.getElementById("gbModelModes");
  if (modesDiv) modesDiv.after(detail);
  else panel.appendChild(detail);
  if (mode === "local") renderLocalDetail(detail);
  else if (mode === "own_key") renderOwnKeyDetail(detail);
  else if (mode === "free") renderFreeDetail(detail);
}

function renderLocalDetail(el) {
  el.innerHTML = '<p class="gb-set-desc">' + esc(t("local")) + '</p>' +
    '<div id="gbLocalContent"><p class="gb-set-hint">' +
    esc(A.t("assistant.connecting","Loading...")) + '</p></div>';
  loadLocalModels(el);
}

/* Install button for the local runtime engine. */
function renderInstallButton(container) {
  var btnId = "gbInstallRuntime";
  container.innerHTML += '<div style="margin-top:12px">' +
    '<button class="gb-set-btn-sm gb-set-btn-accent" id="' + btnId + '">' +
    esc(A.locale && A.locale.startsWith("pt")
      ? "Instalar o motor local"
      : "Install the local engine") +
    '</button><div id="gbInstallMsg"></div></div>';
  var btn = document.getElementById(btnId);
  if (btn) {
    btn.addEventListener("click", function () {
      btn.disabled = true;
      btn.textContent = A.locale && A.locale.startsWith("pt") ? "Instalando..." : "Installing...";
      A.apiFetch("/api/llm/runtime/install", { method: "POST" }).then(function (r) {
        return r.json();
      }).then(function (d) {
        var msg = document.getElementById("gbInstallMsg");
        if (d && d.job_id) {
          if (msg) { msg.innerHTML = '<p class="gb-set-hint">Job: ' + esc(d.job_id) + '</p>'; }
          startJobPoller(d.job_id);
        } else if (d && d.error) {
          if (msg) { msg.innerHTML = '<p class="gb-set-warn">' + esc(d.error) + '</p>'; }
        }
      }).catch(function (e) {
        var msg = document.getElementById("gbInstallMsg");
        if (msg) { msg.innerHTML = '<p class="gb-set-warn">' + esc(String(e)) + '</p>'; }
      });
    });
  }
}

async function loadLocalModels(el) {
  var container = document.getElementById("gbLocalContent");
  if (!container) return;
  try {
    var r = await A.apiFetch("/api/llm/local/models");
    if (!r.ok) { container.innerHTML = '<p class="gb-set-hint">' + esc(t("noModels")) + '</p>'; return; }
    localModels = await r.json();
  } catch (e) { container.innerHTML = '<p class="gb-set-hint">' + esc(t("noModels")) + '</p>'; return; }
  var hw = localModels.hardware || {};
  var html = '<p class="gb-set-desc" style="font-size:12px">' +
    esc(t("hw").replace("%d",hw.device||"?").replace("%v",String(hw.vram_gb||0)).replace("%r",String(hw.ram_gb||0))) + '</p>';
  var lr = cfg && cfg.local_runtime;
  if (lr && !lr.installed) {
    html += '<p class="gb-set-warn">' + esc(t("runtimeNotInstalled")) + '</p>';
  } else if (lr && !lr.runtime_reachable) {
    html += '<p class="gb-set-warn">' + esc(t("runtimeNotReachable")) + '</p>';
  }
  /* Installed models */
  var installed = [], available = [];
  (localModels.tiers || []).forEach(function(tier) {
    (tier.models || []).forEach(function(m) {
      if (m.active || m.downloaded || m.status === "downloading") installed.push(m);
      else available.push(m);
    });
  });
  if (installed.length) {
    html += '<p class="gb-set-desc" style="margin-top:12px;font-weight:600">' + esc(t("installed")) + '</p>';
    installed.forEach(function(m) { html += modelRow(m); });
  }
  if (available.length) {
    html += '<p class="gb-set-desc" style="margin-top:12px;font-weight:600">' + esc(t("installable")) + '</p>';
    available.forEach(function(m) { html += modelRow(m); });
  }
  html += '<div id="gbLocalMsg"></div>';
  container.innerHTML = html;
  /* Show install button when runtime is not installed */
  if (lr && !lr.installed) { renderInstallButton(container); }
  bindModelActions(container);
}

function modelRow(m) {
  var statusLabel = t(m.status || "available");
  var badge = m.active ? ' gb-badge-active' : (m.downloaded ? ' gb-badge-ok' : '');
  var size = fmtBytes(m.size_bytes);
  var fitsLabel = m.fits ? t("fits") : t("nofit").replace("%v", String(m.min_vram_gb));
  if (m.server_only) fitsLabel = t("serverOnly");
  var actions = '';
  if (m.status === "downloading" && m.job_id) {
    actions = '<div class="gb-dl-progress" id="gbJob_' + esc(m.job_id) + '"><span class="gb-dl-text">' + esc(t("downloading")) + '</span>' +
      '<button class="gb-set-btn-sm gb-set-btn-danger" data-cancel="' + esc(m.job_id) + '">' + esc(t("cancel")) + '</button></div>';
    startJobPoller(m.job_id);
  } else if (m.downloaded && !m.active) {
    actions = '<button class="gb-set-btn-sm gb-set-btn-accent" data-activate="' + esc(m.id) + '">' + esc(t("use")) + '</button>' +
      '<button class="gb-set-btn-sm gb-set-btn-danger" data-delete="' + esc(m.id) + '">' + esc(t("remove")) + '</button>';
  } else if (!m.downloaded && !m.active && m.status !== "downloading" && m.repo) {
    actions = '<button class="gb-set-btn-sm" data-download="' + esc(m.id) + '" data-size="' + size + '" data-repo="' + esc(m.repo) + '">' + esc(t("install")) + '</button>';
  }
  return '<div class="gb-set-model-row">' +
    '<div class="gb-set-model-info"><span class="gb-set-model-row-name">' + esc(m.name) + ' <small>' + esc(m.quant) + '</small></span>' +
    '<span class="gb-badge' + badge + '">' + esc(statusLabel) + '</span></div>' +
    '<div class="gb-set-model-meta">' + esc(size) + ' &middot; ' + esc(fitsLabel) + '</div>' +
    '<div class="gb-set-model-actions">' + actions + '</div></div>';
}

function bindModelActions(container) {
  container.querySelectorAll("[data-activate]").forEach(function(btn) {
    btn.addEventListener("click", function() { doActivate(btn.getAttribute("data-activate")); });
  });
  container.querySelectorAll("[data-delete]").forEach(function(btn) {
    btn.addEventListener("click", function() { doDelete(btn.getAttribute("data-delete")); });
  });
  container.querySelectorAll("[data-download]").forEach(function(btn) {
    btn.addEventListener("click", function() {
      var id = btn.getAttribute("data-download");
      var sz = btn.getAttribute("data-size");
      var repo = btn.getAttribute("data-repo");
      var host = "huggingface.co"; try { host = new URL("https://"+repo.split("/")[0]).hostname || repo; } catch(e) { host = repo; }
      inlineConfirm(t("confirmDl").replace("%s",sz).replace("%h",host), function() { doDownload(id); });
    });
  });
  container.querySelectorAll("[data-cancel]").forEach(function(btn) {
    btn.addEventListener("click", function() { doCancelJob(btn.getAttribute("data-cancel")); });
  });
}

async function doActivate(id) {
  var msg = document.getElementById("gbLocalMsg");
  try {
    var r = await A.apiFetch("/api/llm/local/activate", { method: "POST", body: JSON.stringify({ model_id: id }) });
    var d = await r.json();
    if (d.restart_required) { if (msg) msg.innerHTML = '<p class="gb-set-hint">' + esc(t("restartNeeded")) + '</p>'; }
    else { if (msg) msg.innerHTML = '<p class="gb-set-hint">' + esc(t("saved")) + '</p>'; }
    refreshAfterChange();
  } catch (e) { if (msg) msg.innerHTML = '<p class="gb-set-warn">' + esc(String(e)) + '</p>'; }
}

async function doDelete(id) {
  try { await A.apiFetch("/api/llm/local/models/" + encodeURIComponent(id), { method: "DELETE" }); } catch (e) {}
  var panel = document.getElementById("gbSetPanel"); if (panel) A.renderModelFull(panel);
}

async function doDownload(id) {
  try {
    var r = await A.apiFetch("/api/llm/local/models/" + encodeURIComponent(id) + "/download", { method: "POST" });
    var d = await r.json();
    if (d.job_id) startJobPoller(d.job_id);
  } catch (e) {}
  var panel = document.getElementById("gbSetPanel"); if (panel) A.renderModelFull(panel);
}

async function doCancelJob(jobId) {
  try { await A.apiFetch("/api/llm/local/jobs/" + encodeURIComponent(jobId) + "/cancel", { method: "POST" }); } catch (e) {}
  stopJobPoller(jobId);
  var panel = document.getElementById("gbSetPanel"); if (panel) A.renderModelFull(panel);
}

function startJobPoller(jobId) {
  if (jobPollers[jobId]) return;
  jobPollers[jobId] = setInterval(async function() {
    var el = document.getElementById("gbJob_" + jobId);
    if (!el) { stopJobPoller(jobId); return; }
    try {
      var r = await A.apiFetch("/api/llm/local/jobs/" + encodeURIComponent(jobId));
      if (!r.ok) { stopJobPoller(jobId); return; }
      var d = await r.json();
      var txt = el.querySelector(".gb-dl-text");
      if (txt) {
        var pct = d.bytes_total ? Math.round(d.bytes_done / d.bytes_total * 100) : 0;
        txt.textContent = pct + "% · " + fmtBytes(d.bytes_done) + " / " + fmtBytes(d.bytes_total) +
          (d.speed_bps ? " · " + fmtSpeed(d.speed_bps) : "") +
          (d.eta_secs ? " · " + fmtEta(d.eta_secs) : "");
      }
      if (d.state === "done" || d.state === "failed" || d.state === "cancelled") {
        stopJobPoller(jobId);
        var panel = document.getElementById("gbSetPanel"); if (panel) A.renderModelFull(panel);
      }
    } catch (e) { stopJobPoller(jobId); }
  }, 2000);
}
function stopJobPoller(jobId) { if (jobPollers[jobId]) { clearInterval(jobPollers[jobId]); delete jobPollers[jobId]; } }

/* ---- Own Key detail ---- */
function renderOwnKeyDetail(el) {
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
      '<button class="gb-set-btn-sm gb-set-btn-accent" id="gbOwnSave">' + esc(t("save")) + '</button>' +
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
  var host = ""; try { host = new URL(ep).hostname; } catch(e) { host = ep; }
  inlineConfirm(t("confirmExt").replace("%h", host), async function () {
    var body = { mode: "own_key", endpoint: ep, model: md }; if (key) body.api_key = key;
    try {
      var r = await A.apiFetch("/api/llm/config", { method: "PUT", body: JSON.stringify(body) });
      var d = await r.json();
      if (d.ok) { msg.innerHTML = '<p class="gb-set-hint gb-set-ok">' + esc(t("saved")) + '</p>'; refreshAfterChange(); }
      else { msg.innerHTML = '<p class="gb-set-warn">' + esc(d.error || "Error") + '</p>'; }
    } catch (e) { msg.innerHTML = '<p class="gb-set-warn">' + esc(String(e)) + '</p>'; }
    var keyEl = document.getElementById("gbOwnKey"); if (keyEl) keyEl.value = "";
  });
}

/* ---- Free detail ---- */
function renderFreeDetail(el) {
  var modeData = modes && modes.find(function(m){return m.id==="free"});
  var note = modeData && modeData.note ? modeData.note : "";
  var host = cfg && cfg.endpoint_host && cfg.mode === "free" ? cfg.endpoint_host : "cloud provider";
  el.innerHTML = '<p class="gb-set-desc">' + esc(t("freeNote").replace("%h", host)) + '</p>' +
    (note ? '<p class="gb-set-hint">' + esc(note) + '</p>' : '') +
    (cfg && cfg.mode !== "free" ? '<button class="gb-set-btn-sm gb-set-btn-accent" id="gbFreeSave">' + esc(t("save")) + '</button>' : '') +
    '<div id="gbFreeMsg"></div>';
  var saveBtn = document.getElementById("gbFreeSave");
  if (saveBtn) {
    saveBtn.addEventListener("click", function() {
      inlineConfirm(t("confirmExt").replace("%h", host), async function () {
        var msg = document.getElementById("gbFreeMsg");
        try {
          var r = await A.apiFetch("/api/llm/config", { method: "PUT", body: JSON.stringify({ mode: "free" }) });
          var d = await r.json();
          if (d.ok) { if (msg) msg.innerHTML = '<p class="gb-set-hint gb-set-ok">' + esc(t("saved")) + '</p>'; refreshAfterChange(); }
          else { if (msg) msg.innerHTML = '<p class="gb-set-warn">' + esc(d.error||"Error") + '</p>'; }
        } catch (e) { if (msg) msg.innerHTML = '<p class="gb-set-warn">' + esc(String(e)) + '</p>'; }
      });
    });
  }
}

async function refreshAfterChange() {
  await A.loadModels(); A.renderModelPickers(); A.renderStatus();
}
})();
