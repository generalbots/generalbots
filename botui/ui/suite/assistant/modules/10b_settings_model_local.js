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

var jobPollers = {}, jobBusy = {};
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
  var state, body = "";
  if (installing) {
    state = job.update ? tr("Atualizando", "Updating") : tr("Instalando", "Installing");
    body = '<div class="gb-of-dlp gb-dl-progress" id="gbEngineJob" data-job="' + esc(job.job_id) + '">' +
      '<div class="gb-of-bar gb-of-wait"><i></i></div>' +
      '<span class="gb-dl-text">' + esc(engineStageText(job)) + '</span></div>';
  } else if (!lr.installed) {
    state = tr("Não instalado", "Not installed");
    body = '<p class="gb-of-hint">' + esc(tr(
        "Necessário para usar modelos neste computador. Download de cerca de %s de %h.",
        "Needed to run models on this computer. A download of about %s from %h.")
        .replace("%s", size).replace("%h", host)) + '</p>' +
      '<div class="gb-of-row"><button class="gb-of-btn pri" id="gbInstallRuntime">' +
      esc(tr("Instalar o motor local", "Install the local engine")) + '</button></div>';
  } else {
    if (lr.running && lr.loaded_model) {
      state = tr("Em execução", "Running");
      body = '<p class="gb-of-hint">' + esc(tr("Modelo carregado: %m", "Model loaded: %m").replace("%m", shortName(lr.loaded_model))) + '</p>';
    } else {
      state = tr("Instalado, parado", "Installed, stopped");
      body = '<p class="gb-of-hint">' + esc(tr(
        "Inicia quando você escolhe um modelo deste computador.",
        "Starts when you choose a model on this computer.")) + '</p>';
    }
    if (lr.update_available) {
      body += '<p class="gb-of-hint">' + esc(tr(
          "Há uma versão mais nova do motor (%n). Modelos recentes, como o Qwen3.5, só abrem com ela. Download de cerca de %s de %h.",
          "A newer version of the engine is available (%n). Recent models, such as Qwen3.5, only open with it. A download of about %s from %h.")
          .replace("%n", lr.available_build || "").replace("%s", size).replace("%h", host)) + '</p>' +
        '<div class="gb-of-row"><button class="gb-of-btn pri" id="gbInstallRuntime" data-update="1"' +
        ' data-size="' + esc(size) + '" data-host="' + esc(host) + '">' +
        esc(tr("Atualizar o motor local", "Update the local engine")) + '</button></div>';
    } else if (lr.build) {
      body += '<p class="gb-of-hint">' + esc(tr("Versão do motor: %n", "Engine version: %n").replace("%n", lr.build)) + '</p>';
    }
  }
  if (!installing && job && job.state === "failed") {
    body += '<p class="gb-set-warn">' + esc(engineFailText(job)) + '</p>';
  }
  return { state: state, body: body, quiet: !installing && lr.installed && !lr.update_available && !(job && job.state === "failed") };
}
/* The engine is only shown while it needs the person: not installed, an
   update, an install running or one that failed. Its build is in the head. */
function engineNotice(lr) {
  var e = engineSection(lr);
  if (e.quiet) { return ""; }
  return '<div class="gb-of-sec"><div class="gb-of-sec-h"><span class="gb-of-letter">' + "\u2699" + '</span><span class="gb-of-sec-t">' +
    esc(tr("Motor local", "Local engine")) + '</span><span class="gb-of-sp"></span><span class="gb-of-meta">' + esc(e.state.toUpperCase()) + '</span></div>' +
    '<div class="gb-of-sec-b gb-of-engine-b">' + e.body + '<div id="gbInstallMsg" class="gb-of-say"></div></div></div>';
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
    if (jobBusy[key]) return;
    jobBusy[key] = true;
    try {
      var res = await A.fetchJSON("/api/llm/runtime/jobs/" + encodeURIComponent(jobId), null, 8000);
      var d = res.ok && res.data ? res.data : { state: "failed" };
      var txt = el.querySelector(".gb-dl-text");
      if (txt && d.state === "running") {
        txt.textContent = engineStageText(d) + " " + fmtEta(Math.round((d.elapsed_ms || 0) / 1000));
      }
      if (d.state !== "running") { stopJobPoller(key); S().refreshAfterChange(); }
    } catch (e) { if (!e.timeout) stopJobPoller(key); }
    jobBusy[key] = false;
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
  var all = [];
  (localModels.tiers || []).forEach(function (tier) { (tier.models || []).forEach(function (m) { all.push(m); }); });
  /* The model in use (or the first installed one) starts open, as in the reference. */
  if (openId === null) { openId = (all.filter(function (m) { return m.active; })[0] || all.filter(isInstalled)[0] || {}).id || ""; }
  var fit = all.filter(function (m) { return isInstalled(m) || runsHere(m, hw); });
  var shown = filter === "all" ? all : fit;
  var html = (A._modelMachine ? A._modelMachine.card(hw, localModels) : '') + engineNotice(lr);
  html += '<div class="gb-of-sec"><div class="gb-of-sec-h"><span class="gb-of-letter">B</span><span class="gb-of-sec-t">' + esc(tr("Modelos locais", "Local models")) + '</span>' +
    '<span class="gb-of-sp"></span><div class="gb-of-seg" role="group">' + [["fit", tr("Cabem aqui", "Fit here"), fit.length], ["all", tr("Todos", "All"), all.length]].map(function (f) {
      return '<button type="button" data-filter="' + f[0] + '" aria-pressed="' + (filter === f[0]) + '">' + esc(f[1]) + '<span>' + f[2] + '</span></button>';
    }).join("") + '</div></div><div class="gb-of-tbl"><div><div class="gb-of-tr gb-of-th"><span></span><span>' + esc(tr("MODELO", "MODEL")) + '</span><div class="gb-of-3"><span>' +
    esc(tr("PORTE", "SIZE")) + '</span><span>' + esc(tr("AFINIDADE", "FIT")) + (A._modelGarage ? A._modelGarage.help(tr(
      "Quanto este computador aguenta o modelo. 100% roda com folga; abaixo de 60% não roda bem aqui. Passe o mouse no número para ver o motivo.",
      "How well this computer handles the model. 100% runs comfortably; under 60% it does not run well here. Hover the number to see why.")) : '') + '</span><span>' +
    esc(tr("TAMANHO", "DOWNLOAD")) + '</span></div><span></span><span></span></div>';
  [[tr("INSTALADOS", "INSTALLED"), shown.filter(isInstalled)], [tr("PARA INSTALAR", "TO INSTALL"), shown.filter(function (m) { return !isInstalled(m); })]].forEach(function (grp) {
    if (!grp[1].length) { return; }
    html += '<div class="gb-of-grp">' + esc(grp[0]) + ' · ' + grp[1].length + '</div>' + grp[1].map(function (m) { return modelRow(m, lr, hw); }).join("");
  });
  html += '</div></div></div><div id="gbLocalMsg" class="gb-of-say"></div>';
  container.innerHTML = html;
  bindEngine(container);
  bindModelActions(container);
  container.querySelectorAll("[data-filter]").forEach(function (b) {
    b.addEventListener("click", function () { filter = b.getAttribute("data-filter"); loadLocalModels(); });
  });
  container.querySelectorAll("[data-open]").forEach(function (b) {
    b.addEventListener("click", function () {
      var id = b.getAttribute("data-open"), row = b.closest(".gb-of-m");
      openId = openId === id ? "" : id;
      container.querySelectorAll(".gb-of-m").forEach(function (r) {
        var on = r === row && openId === id;
        r.classList.toggle("open", on);
        r.querySelector("[data-open]").setAttribute("aria-expanded", String(on));
        r.querySelector(".gb-of-tr.x").hidden = !on;
      });
    });
  });
  container.querySelectorAll(".gb-dl-progress[data-model-job]").forEach(function (el) {
    startJobPoller(el.getAttribute("data-model-job"));
  });
}

var filter = "fit", openId = null;
function isInstalled(m) { return !!(m.active || m.downloaded || m.status === "downloading"); }
function aff(m, hw) { return A._modelMachine ? A._modelMachine.affinity(m, hw) : { pct: 0, tone: "none", label: "", why: "" }; }
function runsHere(m, hw) { var a = aff(m, hw); return a.tone === "good" || a.tone === "mid"; }
function gbs(n) { return A._modelMachine ? A._modelMachine.gb(n) : n + " GB"; }
var ICON = { dl: "M12 4v11M7 10l5 5 5-5M5 20h14", ok: "M20 6 9 17l-5-5", play: "M7 4l12 8-12 8z" };
function ic(d) { return A._modelGarage ? A._modelGarage.svg(d, 13, 2.2) : ""; }

/* What the model does, as chips; the last one says what it cannot do. */
function capsChips(m) {
  var idx = A.locale && A.locale.startsWith("pt") ? 0 : 1;
  var words = (m.capabilities || []).map(function (c) { return CAPS[c] ? CAPS[c][idx] : ""; }).filter(Boolean);
  return words.map(function (w) { return '<span class="gb-of-cap"><b>✓</b>' + esc(w) + '</span>'; }).join("") +
    '<span class="gb-of-cap no"><b>–</b>' + esc(tr("fotos e gráficos", "photos and charts")) + '</span>';
}
function sizeLevel(m) { var b = parseFloat(String(m.params || "")) || 0; return m.server_only ? 5 : b <= 4 ? 1 : b <= 7 ? 2 : b <= 9 ? 3 : b <= 14 ? 4 : 5; }
var WORDS = [["", ""], ["pequeno", "small"], ["compacto", "compact"], ["médio", "medium"], ["grande", "large"], ["muito grande", "very large"]];

function progressText(p) {
  if (!p || !p.bytes_total) return t("downloading");
  var pct = Math.round(p.bytes_done / p.bytes_total * 100);
  return pct + "% · " + fmtBytes(p.bytes_done) + " / " + fmtBytes(p.bytes_total) +
    (p.speed_bps ? " · " + fmtSpeed(p.speed_bps) : "") + (p.eta_secs ? " · " + fmtEta(p.eta_secs) : "");
}

/* Recommended, this card and what is missing (or the comfortable size). */
function reqCells(m, hw) {
  var need = Number(m.min_vram_gb) || 0, cuda = !!(hw.cuda_usable && hw.vram_gb > 0);
  var have = cuda ? Number(hw.vram_gb) : Number(hw.ram_gb) || 0;
  var here = cuda ? tr("ESTA PLACA", "THIS CARD") : tr("ESTE COMPUTADOR", "THIS COMPUTER");
  function c(k, v, cls) { return '<div class="' + (cls || "") + '"><span>' + esc(k) + '</span><b>' + esc(v) + '</b></div>'; }
  if (m.server_only || !need) { return c(tr("ONDE RODA", "WHERE IT RUNS"), tr("Só em servidor", "Servers only"), "mu") + c(here, gbs(have)) + c("—", "—", "mu"); }
  var gap = need - have;
  return c(tr("RECOMENDADO", "RECOMMENDED"), gbs(need)) + c(here, gbs(have)) +
    (gap > 0 ? c(tr("FALTAM", "SHORT BY"), gbs(gap), "gap") : c(tr("IDEAL", "IDEAL"), gbs(Math.ceil(need * 1.25)), "ideal"));
}

function modelRow(m, lr, hw) {
  var downloading = m.status === "downloading" && m.job_id;
  var failedHere = lr && lr.last_failure && lr.last_failure.model === m.file;
  var size = fmtBytes(m.size_bytes);
  var a = aff(m, hw), open = openId === m.id;
  var models = S().localModels || {};
  var recommended = !m.server_only && models.recommended === m.id;
  var act;
  if (m.active) { act = '<span class="gb-of-btn sm inuse">' + ic(ICON.ok) + esc(tr("Em uso", "In use")) + '</span>'; }
  else if (downloading) { act = '<button type="button" class="gb-of-btn sm dng" data-cancel="' + esc(m.job_id) + '">' + esc(t("cancel")) + '</button>'; }
  else if (m.downloaded) { act = '<button type="button" class="gb-of-btn sm" data-activate="' + esc(m.id) + '">' + ic(ICON.play) + esc(tr("Usar", "Use")) + '</button>'; }
  else if (m.server_only || !m.repo) { act = '<span class="gb-of-btn sm srv" tabindex="0" data-tip="' + esc(a.why) + '">' + esc(tr("Só servidor", "Server only")) + '</span>'; }
  else {
    var weak = !runsHere(m, hw);
    act = '<button type="button" class="gb-of-btn sm' + (weak ? ' weak' : '') + '" data-download="' + esc(m.id) + '" data-size="' + size + '" data-host="' + esc(m.source_host || "huggingface.co") + '"' +
      (weak ? ' data-tip="' + esc(tr("Não roda bem neste computador. ", "Does not run well on this computer. ") + a.why) + '"' : '') + '>' + ic(ICON.dl) + esc(t("install")) + '</button>';
  }
  var pct = downloading && m.progress && m.progress.bytes_total ? Math.round(m.progress.bytes_done / m.progress.bytes_total * 100) : 0;
  var lvl = sizeLevel(m), segs = "";
  for (var i = 0; i < 5; i++) { segs += '<i class="' + (i < lvl ? "on" : "") + '"></i>'; }
  var meta = [m.licence ? t("licence") + " " + m.licence : "", m.source_host ? t("source") + " " + m.source_host : ""].filter(Boolean).join(" · ");
  return '<div class="gb-of-m' + (open ? ' open' : '') + '"><div class="gb-of-tr"><div class="gb-of-gt"></div><div class="gb-of-mn"><div><b>' + esc(m.name) + '</b><span>' + esc(m.quant || "") + '</span></div>' +
    (m.active || recommended ? '<div class="gb-of-tags">' + (m.active ? '<span class="gb-of-chip xs">' + esc(tr("EM USO", "IN USE")) + '</span>' : '') +
      (recommended ? '<span class="gb-of-chip xs line">' + esc(tr("RECOMENDADO AQUI", "RECOMMENDED HERE")) + '</span>' : '') + '</div>' : '') +
    (downloading ? '<div class="gb-of-dlp gb-dl-progress" id="gbJob_' + esc(m.job_id) + '" data-model-job="' + esc(m.job_id) + '"><div class="gb-of-bar gb-dl-bar"><i style="width:' + pct + '%"></i></div>' +
      '<span class="gb-dl-text">' + esc(progressText(m.progress)) + '</span></div>' : '') + '</div>' +
    '<div class="gb-of-box gb-of-3"><div><span class="gb-of-segs xs" data-tone="ink">' + segs + '</span><span class="t">' + esc(tr(WORDS[lvl][0], WORDS[lvl][1]) + (m.params ? " · " + m.params : "")) + '</span></div>' +
    '<div class="gb-of-aff" data-tone="' + a.tone + '" tabindex="0" data-tip="' + esc(a.why) + '"><div><b>' + (a.tone === "none" ? "—" : a.pct + "%") + '</b><span class="t">' + esc(a.label) + '</span></div>' +
    '<div class="gb-of-bar"><i style="width:' + (a.tone === "none" ? 0 : a.pct) + '%"></i></div></div><span class="dl">' + esc(size) + '</span></div>' +
    '<div class="gb-of-act">' + act + '</div><button type="button" class="gb-of-chev" data-open="' + esc(m.id) + '" aria-expanded="' + open + '" title="' + esc(tr("Detalhes", "Details")) + '">' +
    (A._modelGarage ? A._modelGarage.svg("m6 9 6 6 6-6", 14, 3) : "") + '</button></div>' +
    '<div class="gb-of-tr x"' + (open ? '' : ' hidden') + '><div class="gb-of-gt"></div><div class="gb-of-mx"><div class="gb-of-row">' + capsChips(m) + '</div>' +
    (meta ? '<span class="m2">' + esc(meta) + '</span>' : '') +
    (failedHere ? '<p class="gb-set-warn">' + esc(S().errText(lr.last_failure.code)) + '</p>' : '') +
    (m.downloaded && !m.active ? '<div><button type="button" class="gb-of-btn sm dng" data-delete="' + esc(m.id) + '">' + esc(t("remove")) + '</button></div>' : '') +
    '</div><div class="gb-of-req gb-of-3">' + reqCells(m, hw) + '</div></div></div>';
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
    S().refreshAfterChange(true);
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
    if (jobBusy[jobId]) return;
    jobBusy[jobId] = true;
    try {
      var res = await A.fetchJSON("/api/llm/local/jobs/" + encodeURIComponent(jobId), null, 8000);
      if (!res.ok || !res.data) { stopJobPoller(jobId); return; }
      var d = res.data;
      var txt = el.querySelector(".gb-dl-text");
      if (txt) { txt.textContent = progressText(d); }
      var bar = el.querySelector(".gb-dl-bar > i");
      if (bar && d.bytes_total) { bar.style.width = Math.round(d.bytes_done / d.bytes_total * 100) + "%"; }
      if (d.state === "done" || d.state === "failed" || d.state === "cancelled") {
        stopJobPoller(jobId);
        loadLocalModels();
      }
    } catch (e) { if (!e.timeout) stopJobPoller(jobId); }
    jobBusy[jobId] = false;
  }, 2000);
}

function stopJobPoller(jobId) {
  if (jobPollers[jobId]) { clearInterval(jobPollers[jobId]); delete jobPollers[jobId]; }
}

/* Expose to parent module */
A._modelLocal = {
  renderLocalDetail: renderLocalDetail,
  activate: function (id) { return doActivate(id); },
  startJobPoller: startJobPoller,
  stopJobPoller: stopJobPoller
};
})();
