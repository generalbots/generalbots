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
var ENGINE_FAIL = {
  engine_in_use: ["O motor em uso não pôde ser parado para a troca. Feche outros programas que usem o modelo local e tente de novo.",
    "The engine in use could not be stopped for the swap. Close other programs that use the local model and try again."],
  engine_download_failed: ["O download do motor novo falhou. O motor anterior continua instalado.",
    "The download of the new engine failed. The previous engine is still installed."],
  engine_new_failed: ["O motor novo não conseguiu iniciar o modelo que estava em uso. O motor anterior foi restaurado.",
    "The new engine could not start the model that was in use. The previous engine was restored."]
};
function engineStageText(job) {
  if (!job || !job.update) return tr("Baixando e instalando o motor...", "Downloading and installing the engine...");
  if (job.stage === "stopping") return tr("Parando o motor em uso...", "Stopping the engine in use...");
  if (job.stage === "starting") return tr("Iniciando o modelo com o motor novo...", "Starting the model with the new engine...");
  return tr("Baixando o motor novo...", "Downloading the new engine...");
}
function engineFailText(job) {
  var pair = ENGINE_FAIL[job.code] || ["A instalação do motor falhou.", "The engine install failed."];
  return tr(pair[0], pair[1]);
}

function engineSection(lr) {
  lr = lr || {};
  var job = lr.install_job;
  var installing = lr.installing && job;
  var info = lr.install_info || {};
  var size = fmtBytes(info.download_size_bytes), host = info.download_host || "github.com";
  var state, badge = "", body = "";
  if (installing) {
    state = job.update ? tr("Atualizando", "Updating") : tr("Instalando", "Installing");
    body = '<div class="gb-dl-progress" id="gbEngineJob" data-job="' + esc(job.job_id) + '">' +
      '<div class="gb-dl-bar gb-dl-bar-wait"><i></i></div>' +
      '<span class="gb-dl-text">' + esc(engineStageText(job)) + '</span></div>';
  } else if (!lr.installed) {
    state = tr("Não instalado", "Not installed");
    body = '<p class="gb-set-model-meta">' + esc(tr(
        "Necessário para usar modelos neste computador. Download de cerca de %s de %h.",
        "Needed to run models on this computer. A download of about %s from %h.")
        .replace("%s", size).replace("%h", host)) + '</p>' +
      '<div class="gb-set-model-actions"><button class="gb-set-btn-sm gb-set-btn-accent" id="gbInstallRuntime">' +
      esc(tr("Instalar o motor local", "Install the local engine")) + '</button></div>';
  } else {
    if (lr.running && lr.loaded_model) {
      state = tr("Em execução", "Running"); badge = " gb-badge-active";
      body = '<p class="gb-set-model-meta">' + esc(tr("Modelo carregado: %m", "Model loaded: %m").replace("%m", shortName(lr.loaded_model))) + '</p>';
    } else {
      state = tr("Instalado, parado", "Installed, stopped"); badge = " gb-badge-ok";
      body = '<p class="gb-set-model-meta">' + esc(tr(
        "Inicia quando você escolhe um modelo deste computador.",
        "Starts when you choose a model on this computer.")) + '</p>';
    }
    if (lr.update_available) {
      body += '<p class="gb-set-model-meta">' + esc(tr(
          "Há uma versão mais nova do motor (%n). Modelos recentes, como o Qwen3.5, só abrem com ela. Download de cerca de %s de %h.",
          "A newer version of the engine is available (%n). Recent models, such as Qwen3.5, only open with it. A download of about %s from %h.")
          .replace("%n", lr.available_build || "").replace("%s", size).replace("%h", host)) + '</p>' +
        '<div class="gb-set-model-actions"><button class="gb-set-btn-sm gb-set-btn-accent" id="gbInstallRuntime" data-update="1"' +
        ' data-size="' + esc(size) + '" data-host="' + esc(host) + '">' +
        esc(tr("Atualizar o motor local", "Update the local engine")) + '</button></div>';
    } else if (lr.build) {
      body += '<p class="gb-set-model-meta">' + esc(tr("Versão do motor: %n", "Engine version: %n").replace("%n", lr.build)) + '</p>';
    }
  }
  if (!installing && job && job.state === "failed") {
    body += '<p class="gb-set-warn">' + esc(engineFailText(job)) + '</p>';
  }
  return '<div class="gb-set-engine"><div class="gb-set-model-info">' +
    '<span class="gb-set-model-row-name">' + esc(tr("Motor local", "Local engine")) + '</span>' +
    '<span class="gb-badge' + badge + '">' + esc(state) + '</span></div>' + body +
    '<div id="gbInstallMsg"></div></div>';
}

function startEngineInstall() {
  S().setBusy(true, document.getElementById("gbInstallMsg"), tr("Iniciando...", "Starting..."));
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
}

function bindEngine(container) {
  var btn = container.querySelector("#gbInstallRuntime");
  if (btn) {
    btn.addEventListener("click", function () {
      if (!btn.getAttribute("data-update")) { startEngineInstall(); return; }
      S().inlineConfirm(tr(
          "Baixar cerca de %s de %h e trocar o motor local? O modelo em uso fica parado durante a troca. Se o motor novo não funcionar, o anterior volta sozinho.",
          "Download about %s from %h and replace the local engine? The model in use stops during the swap. If the new engine does not work, the previous one comes back by itself.")
          .replace("%s", btn.getAttribute("data-size")).replace("%h", btn.getAttribute("data-host")), startEngineInstall);
    });
  }
  var jobEl = container.querySelector("#gbEngineJob");
  if (jobEl) { pollEngine(jobEl.getAttribute("data-job")); }
}

/* The engine job reports its stage and the time it has run, not bytes. */
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
        txt.textContent = engineStageText(d) + " " + fmtEta(Math.round((d.elapsed_ms || 0) / 1000));
      }
      if (d.state !== "running") { stopJobPoller(key); S().refreshAfterChange(); }
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
  var html = A._modelMachine ? A._modelMachine.card(hw, localModels, !!(lr && lr.running && lr.loaded_model)) : '';
  html += engineSection(lr);
  html += '<p class="gb-set-model-meta gb-set-hw">' + esc(t("privLocal")) + ' ' + esc(t("noImageReading")) + '</p>';
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

/* What the model does, as chips; the last one says what it cannot do. */
function capsChips(m) {
  var idx = A.locale && A.locale.startsWith("pt") ? 0 : 1;
  var words = (m.capabilities || []).map(function (c) { return CAPS[c] ? CAPS[c][idx] : ""; }).filter(Boolean);
  if (!words.length) return "";
  return '<div class="gb-gar-chips">' + words.map(function (w) {
    return '<span class="gb-gar-chip on"><span class="gb-gar-mark yes" aria-hidden="true"></span>' + esc(w) + '</span>';
  }).join("") + '<span class="gb-gar-chip"><span class="gb-gar-mark no" aria-hidden="true"></span>' +
    esc(tr("fotos e gráficos", "photos and charts")) + '</span></div>';
}

/* Two gauges per model: its size (bigger models answer better and need more
   memory) and how it fits the memory of this computer. */
function meter(label, level, word, neutral) {
  var segs = "";
  for (var i = 1; i <= 5; i++) { segs += '<i class="' + (i <= level ? "on" : "") + '"></i>'; }
  return '<div class="gb-gar-meter"><span class="gb-gar-meter-label">' + esc(label) + '</span>' +
    '<span class="gb-gar-gauge gb-gar-level-' + level + (neutral ? ' gb-gar-neutral' : '') + '" aria-hidden="true">' + segs + '</span>' +
    '<span class="gb-gar-meter-word">' + esc(word) + '</span></div>';
}
function sizeMeter(m) {
  var b = parseFloat(String(m.params || "")) || 0;
  var level = m.server_only ? 5 : b <= 4 ? 1 : b <= 7 ? 2 : b <= 9 ? 3 : b <= 14 ? 4 : 5;
  var words = [["", ""], ["pequeno", "small"], ["compacto", "compact"], ["médio", "medium"], ["grande", "large"], ["muito grande", "very large"]];
  return meter(tr("Porte", "Size"), level, tr(words[level][0], words[level][1]) + " · " + (m.params || ""), true);
}
function fitMeter(m) {
  if (m.server_only) return meter(tr("Nesta máquina", "On this machine"), 0, t("serverOnly"));
  var v = m.fit_verdict;
  var level = v === "comfortable" ? 5 : v === "tight" ? 3 : 1;
  return meter(tr("Nesta máquina", "On this machine"), level, fitLabel(m));
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
  var meta = esc(tr("Download", "Download")) + ': ' + esc(size);
  if (m.licence) meta += ' &middot; ' + esc(t("licence")) + ': ' + esc(m.licence);
  if (m.source_host) meta += ' &middot; ' + esc(t("source")) + ': ' + esc(m.source_host);
  var models = S().localModels || {};
  var recommended = !m.server_only && models.recommended === m.id;
  return '<div class="gb-set-model-row gb-gar-part' + (downloading ? ' gb-set-model-row-busy' : '') + (recommended ? ' gb-gar-part-rec' : '') + '">' +
    '<div class="gb-set-model-info"><span class="gb-set-model-row-name">' + esc(m.name) + ' <small>' + esc(m.quant) + '</small></span>' +
    '<span class="gb-badge' + badge + '">' + esc(statusLabel) + '</span>' +
    (recommended ? '<span class="gb-badge gb-gar-rec-badge">' + esc(tr("Recomendado para este computador", "Recommended for this computer")) + '</span>' : '') + '</div>' +
    '<div class="gb-gar-meters">' + sizeMeter(m) + (A._modelMachine ? '' : fitMeter(m)) + '</div>' +
    (A._modelMachine ? A._modelMachine.affinityBar(m, models.hardware) : '') +
    capsChips(m) +
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
  // Best-effort: a failed DELETE is reflected by the refreshed list below,
  // which re-reads authoritative server state; no toast needed here.
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
  // Best-effort: the poller and list refresh below converge on server state,
  // so a failed cancel request surfaces as the job still appearing in the list.
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
