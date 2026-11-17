"use strict";
/**
 * 10b_settings_model_local.js - Local model management for Settings > Modelo.
 * Split from 10_settings_model.js for the 450-line limit.
 * Uses A._modelShared (exposed by 10_settings_model.js) for shared state.
 *
 * Layout of the "Neste computador" detail: the engine first (state, install
 * with progress), then the installed models, then the ones that can be
 * installed. A download runs in its own row; nothing changes tab.
 */
(function () {
var A = window.GBAssistant;
function S() { return A._modelShared || {}; }
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function fmtBytes(b){if(!b)return"0 B";if(b<1024)return b+" B";if(b<1048576)return(b/1024).toFixed(1)+" KB";if(b<1073741824)return(b/1048576).toFixed(1)+" MB";return(b/1073741824).toFixed(1)+" GB";}
function fmtSpeed(bps){return bps?fmtBytes(bps)+"/s":"";}
function fmtEta(s){if(!s&&s!==0)return"";if(s<60)return s+"s";if(s<3600)return Math.floor(s/60)+"m "+(s%60)+"s";return Math.floor(s/3600)+"h "+Math.floor((s%3600)/60)+"m";}
function t(key) { return S().t ? S().t(key) : key; }
function tr(ptT, enT) { return A.locale && A.locale.startsWith("pt") ? ptT : enT; }
function shortName(m) { return typeof A.shortModelName === "function" ? A.shortModelName(m) : m; }

var jobPollers = {};
var CAPS = {
  chat: ["conversa e redação", "chat and writing"],
  reasoning: ["raciocínio passo a passo", "step-by-step reasoning"],
  multilingual: ["vários idiomas", "many languages"]
};

function renderLocalDetail(el) {
  el.innerHTML = '<div id="gbLocalContent"><p class="gb-set-busy">' +
    esc(A.t("assistant.connecting","Loading...")) + '</p></div>';
  loadLocalModels();
}

/* ---- Engine section ---- */
function engineSection(lr) {
  lr = lr || {};
  var job = lr.install_job;
  var installing = lr.installing && job;
  var state, badge = "", body = "";
  if (installing) {
    state = tr("Instalando", "Installing");
    body = '<div class="gb-dl-progress" id="gbEngineJob" data-job="' + esc(job.job_id) + '">' +
      '<div class="gb-dl-bar gb-dl-bar-wait"><i></i></div>' +
      '<span class="gb-dl-text">' + esc(tr("Baixando e instalando o motor...", "Downloading and installing the engine...")) + '</span></div>';
  } else if (!lr.installed) {
    state = tr("Não instalado", "Not installed");
    var info = lr.install_info || {};
    body = '<p class="gb-set-model-meta">' + esc(tr(
        "Necessário para usar modelos neste computador. Download de cerca de %s de %h.",
        "Needed to run models on this computer. A download of about %s from %h.")
        .replace("%s", fmtBytes(info.download_size_bytes)).replace("%h", info.download_host || "github.com")) + '</p>' +
      '<div class="gb-set-model-actions"><button class="gb-set-btn-sm gb-set-btn-accent" id="gbInstallRuntime">' +
      esc(tr("Instalar o motor local", "Install the local engine")) + '</button></div>';
    if (job && job.state === "failed") {
      body += '<p class="gb-set-warn">' + esc(tr("A instalação do motor falhou.", "The engine install failed.")) + '</p>';
    }
  } else if (lr.running && lr.loaded_model) {
    state = tr("Em execução", "Running"); badge = " gb-badge-active";
    body = '<p class="gb-set-model-meta">' + esc(tr("Modelo carregado: %m", "Model loaded: %m").replace("%m", shortName(lr.loaded_model))) + '</p>';
  } else {
    state = tr("Instalado, parado", "Installed, stopped"); badge = " gb-badge-ok";
    body = '<p class="gb-set-model-meta">' + esc(tr(
      "Inicia quando você escolhe um modelo deste computador.",
      "Starts when you choose a model on this computer.")) + '</p>';
  }
  return '<div class="gb-set-engine"><div class="gb-set-model-info">' +
    '<span class="gb-set-model-row-name">' + esc(tr("Motor local", "Local engine")) + '</span>' +
    '<span class="gb-badge' + badge + '">' + esc(state) + '</span></div>' + body +
    '<div id="gbInstallMsg"></div></div>';
}

function bindEngine(container) {
  var btn = container.querySelector("#gbInstallRuntime");
  if (btn) {
    btn.addEventListener("click", function () {
      S().setBusy(true, document.getElementById("gbInstallMsg"), tr("Iniciando a instalação...", "Starting the install..."));
      A.apiFetch("/api/llm/runtime/install", { method: "POST" }).then(function (r) {
        return r.json().catch(function () { return { error: "http_" + r.status }; });
      }).catch(function () { return { error: "unreachable" }; }).then(function (d) {
        S().setBusy(false);
        if (d && d.error) {
          var msg = document.getElementById("gbInstallMsg");
          if (msg) { msg.innerHTML = '<p class="gb-set-warn">' + esc(tr("A instalação não pôde começar.", "The install could not start.")) + ' (' + esc(d.error) + ')</p>'; }
          return;
        }
        S().refreshKeepingView();
      });
    });
  }
  var jobEl = container.querySelector("#gbEngineJob");
  if (jobEl) { pollEngine(jobEl.getAttribute("data-job")); }
}

/* The engine install reports a state and the time it has run, not bytes. */
function pollEngine(jobId) {
  var key = "engine:" + jobId;
  if (jobPollers[key]) return;
  jobPollers[key] = setInterval(async function () {
    var el = document.getElementById("gbEngineJob");
    if (!el) { stopJobPoller(key); return; }
    try {
      var r = await A.apiFetch("/api/llm/runtime/jobs/" + encodeURIComponent(jobId));
      var d = r.ok ? await r.json() : { state: "failed" };
      var txt = el.querySelector(".gb-dl-text");
      if (txt && d.state === "running") {
        txt.textContent = tr("Baixando e instalando o motor...", "Downloading and installing the engine...") +
          " " + fmtEta(Math.round((d.elapsed_ms || 0) / 1000));
      }
      if (d.state !== "running") { stopJobPoller(key); S().refreshKeepingView(); }
    } catch (e) { stopJobPoller(key); }
  }, 2000);
}

/* ---- Model lists ---- */
async function loadLocalModels() {
  var container = document.getElementById("gbLocalContent");
  if (!container) return;
  var localModels;
  try {
    var r = await A.apiFetch("/api/llm/local/models");
    if (!r.ok) { container.innerHTML = '<p class="gb-set-hint">' + esc(t("noModels")) + '</p>'; return; }
    localModels = await r.json();
  } catch (e) { container.innerHTML = '<p class="gb-set-hint">' + esc(t("noModels")) + '</p>'; return; }
  S().localModels = localModels;
  var cfg = S().cfg;
  var lr = cfg && cfg.local_runtime;
  var hw = localModels.hardware || {};
  var hwLine = hw.cuda_usable
    ? t("hw").replace("%d",hw.device||"?").replace("%v",String(hw.vram_gb||0)).replace("%f",String(hw.vram_free_gb||0)).replace("%r",String(hw.ram_gb||0))
    : t("hwCpu").replace("%r",String(hw.ram_gb||0));
  var html = engineSection(lr);
  html += '<p class="gb-set-model-meta gb-set-hw">' + esc(hwLine);
  if (hw.driver) html += ' &middot; ' + esc(t("hwDriver").replace("%drv",hw.driver));
  if (hw.cuda_usable) html += ' &middot; ' + esc(t("hwCuda"));
  html += '<br>' + esc(t("privLocal")) + ' ' + esc(t("noImageReading")) + '</p>';
  var installed = [], available = [];
  (localModels.tiers || []).forEach(function(tier) {
    (tier.models || []).forEach(function(m) {
      if (m.active || m.downloaded || m.status === "downloading") installed.push(m);
      else available.push(m);
    });
  });
  if (installed.length) {
    html += '<p class="gb-set-list-title">' + esc(t("installed")) + '</p>';
    installed.forEach(function(m) { html += modelRow(m, lr); });
  }
  if (available.length) {
    html += '<p class="gb-set-list-title">' + esc(t("installable")) + '</p>';
    available.forEach(function(m) { html += modelRow(m, lr); });
  }
  html += '<div id="gbLocalMsg"></div>';
  container.innerHTML = html;
  bindEngine(container);
  bindModelActions(container);
  container.querySelectorAll(".gb-dl-progress[data-model-job]").forEach(function (el) {
    startJobPoller(el.getAttribute("data-model-job"));
  });
}

function fitLabel(m) {
  if (m.server_only) return t("serverOnly");
  var v = m.fit_verdict;
  if (v === "comfortable") return t("comfortable");
  if (v === "tight") return t("tight");
  if (v === "wont_run") return t("wontRun").replace("%v", String(m.min_vram_gb));
  return m.fits ? t("fits") : t("nofit").replace("%v", String(m.min_vram_gb));
}

function capsLine(m) {
  var idx = A.locale && A.locale.startsWith("pt") ? 0 : 1;
  var words = (m.capabilities || []).map(function (c) { return CAPS[c] ? CAPS[c][idx] : ""; }).filter(Boolean);
  if (!words.length) return "";
  var line = words.join(", ");
  return line.charAt(0).toUpperCase() + line.slice(1) + ". " + tr("Só texto.", "Text only.");
}

function progressText(p) {
  if (!p || !p.bytes_total) return t("downloading");
  var pct = Math.round(p.bytes_done / p.bytes_total * 100);
  return pct + "% · " + fmtBytes(p.bytes_done) + " / " + fmtBytes(p.bytes_total) +
    (p.speed_bps ? " · " + fmtSpeed(p.speed_bps) : "") + (p.eta_secs ? " · " + fmtEta(p.eta_secs) : "");
}

function modelRow(m, lr) {
  var downloading = m.status === "downloading" && m.job_id;
  var failedHere = lr && lr.last_failure && lr.last_failure.model === m.file;
  var statusLabel = t(m.status || "available");
  var badge = m.active ? ' gb-badge-active' : (m.downloaded ? ' gb-badge-ok' : '');
  var size = fmtBytes(m.size_bytes);
  var actions = '';
  if (downloading) {
    var pct = m.progress && m.progress.bytes_total ? Math.round(m.progress.bytes_done / m.progress.bytes_total * 100) : 0;
    actions = '<div class="gb-dl-progress" id="gbJob_' + esc(m.job_id) + '" data-model-job="' + esc(m.job_id) + '">' +
      '<div class="gb-dl-bar"><i style="width:' + pct + '%"></i></div>' +
      '<span class="gb-dl-text">' + esc(progressText(m.progress)) + '</span>' +
      '<button class="gb-set-btn-sm gb-set-btn-danger" data-cancel="' + esc(m.job_id) + '">' + esc(t("cancel")) + '</button></div>';
  } else if (m.downloaded && !m.active) {
    actions = '<button class="gb-set-btn-sm gb-set-btn-accent" data-activate="' + esc(m.id) + '">' + esc(t("use")) + '</button>' +
      '<button class="gb-set-btn-sm gb-set-btn-danger" data-delete="' + esc(m.id) + '">' + esc(t("remove")) + '</button>';
  } else if (!m.downloaded && !m.active && m.repo) {
    actions = '<button class="gb-set-btn-sm" data-download="' + esc(m.id) + '" data-size="' + size + '" data-host="' + esc(m.source_host || "huggingface.co") + '">' + esc(t("install")) + '</button>';
  }
  var meta = esc(size) + ' &middot; ' + esc(fitLabel(m));
  if (m.licence) meta += ' &middot; ' + esc(t("licence")) + ': ' + esc(m.licence);
  if (m.source_host) meta += ' &middot; ' + esc(t("source")) + ': ' + esc(m.source_host);
  var caps = capsLine(m);
  return '<div class="gb-set-model-row' + (downloading ? ' gb-set-model-row-busy' : '') + '">' +
    '<div class="gb-set-model-info"><span class="gb-set-model-row-name">' + esc(m.name) + ' <small>' + esc(m.quant) + '</small></span>' +
    '<span class="gb-badge' + badge + '">' + esc(statusLabel) + '</span></div>' +
    (caps ? '<div class="gb-set-model-caps">' + esc(caps) + '</div>' : '') +
    '<div class="gb-set-model-meta">' + meta + '</div>' +
    (failedHere ? '<div class="gb-set-warn">' + esc(S().errText(lr.last_failure.code)) + '</div>' : '') +
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
      S().inlineConfirm(t("confirmDl").replace("%s", btn.getAttribute("data-size")).replace("%h", btn.getAttribute("data-host")),
        function() { doDownload(id, btn); });
    });
  });
  container.querySelectorAll("[data-cancel]").forEach(function(btn) {
    btn.addEventListener("click", function() { doCancelJob(btn.getAttribute("data-cancel")); });
  });
}

async function doActivate(id) {
  var msg = document.getElementById("gbLocalMsg");
  var btn = document.querySelector('[data-activate="' + id + '"]');
  if (btn) { btn.disabled = true; }
  S().setBusy(true, msg, t("loadingLocal"));
  try {
    var r = await A.apiFetch("/api/llm/local/activate", { method: "POST", body: JSON.stringify({ model_id: id }) });
    var d = await r.json().catch(function () { return { error: "http_" + r.status }; });
    S().setBusy(false);
    if (!r.ok || d.error) {
      if (msg) msg.innerHTML = '<p class="gb-set-warn">' + esc(S().errText(d.error)) + '</p>';
      if (btn) { btn.disabled = false; }
      return;
    }
    if (d.restart_required && msg) {
      msg.innerHTML = '<p class="gb-set-hint">' + esc(t("restartNeeded")) + '</p>';
    }
    S().refreshAfterChange();
  } catch (e) {
    S().setBusy(false);
    if (msg) msg.innerHTML = '<p class="gb-set-warn">' + esc(S().errText("")) + '</p>';
    if (btn) { btn.disabled = false; }
  }
}

async function doDelete(id) {
  try { await A.apiFetch("/api/llm/local/models/" + encodeURIComponent(id), { method: "DELETE" }); } catch (e) {}
  loadLocalModels();
}

async function doDownload(id, btn) {
  if (btn) { btn.disabled = true; }
  var failure = "";
  try {
    var r = await A.apiFetch("/api/llm/local/models/" + encodeURIComponent(id) + "/download", { method: "POST" });
    var d = await r.json().catch(function () { return { error: "http_" + r.status }; });
    if (!r.ok || d.error) { failure = d.error || "http_" + r.status; }
  } catch (e) { failure = "unreachable"; }
  await loadLocalModels();
  if (failure) {
    var msg = document.getElementById("gbLocalMsg");
    if (msg) { msg.innerHTML = '<p class="gb-set-warn">' + esc(tr("O download não pôde começar.", "The download could not start.")) + ' (' + esc(failure) + ')</p>'; }
  }
}

async function doCancelJob(jobId) {
  try { await A.apiFetch("/api/llm/local/jobs/" + encodeURIComponent(jobId) + "/cancel", { method: "POST" }); } catch (e) {}
  stopJobPoller(jobId);
  loadLocalModels();
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
      if (txt) { txt.textContent = progressText(d); }
      var bar = el.querySelector(".gb-dl-bar > i");
      if (bar && d.bytes_total) { bar.style.width = Math.round(d.bytes_done / d.bytes_total * 100) + "%"; }
      if (d.state === "done" || d.state === "failed" || d.state === "cancelled") {
        stopJobPoller(jobId);
        loadLocalModels();
      }
    } catch (e) { stopJobPoller(jobId); }
  }, 2000);
}

function stopJobPoller(jobId) {
  if (jobPollers[jobId]) { clearInterval(jobPollers[jobId]); delete jobPollers[jobId]; }
}

/* ---- Web search card (below the modes) ---- */
function renderWebCard(panel) {
  if (!A.webSearchAvailable || !A.webSearchAvailable()) return;
  var card = document.createElement("div");
  card.className = "gb-set-card";
  card.innerHTML = '<h2 class="gb-set-title">' + esc(tr("Pesquisa na web", "Web search")) + '</h2>' +
    '<div class="gb-set-toggle-row"><label class="gb-set-label" for="gbWebSearch">' +
    esc(tr("Pesquisar na web antes de responder", "Search the web before answering")) + '</label>' +
    '<input type="checkbox" id="gbWebSearch" class="gb-set-toggle"' + (A.webSearchOn() ? ' checked' : '') + '></div>' +
    '<p class="gb-set-privacy">' + esc(tr(
      "Desligada, nada é pesquisado. Ligada, o começo de cada pergunta (até 160 caracteres) é enviado ao DuckDuckGo (duckduckgo.com) para buscar páginas; os trechos encontrados vão para o modelo em uso e a resposta mostra as fontes.",
      "Off, nothing is searched. On, the start of each question (up to 160 characters) is sent to DuckDuckGo (duckduckgo.com) to find pages; the excerpts found go to the model in use and the answer shows its sources.")) + '</p>';
  panel.appendChild(card);
  card.querySelector("#gbWebSearch").addEventListener("change", function (e) { A.setWebSearch(e.target.checked); });
}

/* Expose to parent module */
A._modelLocal = {
  renderLocalDetail: renderLocalDetail,
  renderWebCard: renderWebCard,
  startJobPoller: startJobPoller,
  stopJobPoller: stopJobPoller
};
})();
