"use strict";
/**
 * 10b_settings_model_local.js - Local model management for Settings > Modelo.
 * Split from 10_settings_model.js for the 450-line limit.
 * Uses A._modelShared (exposed by 10_settings_model.js) for shared state.
 */
(function () {
var A = window.GBAssistant;
function S() { return A._modelShared || {}; }
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function fmtBytes(b){if(!b)return"0 B";if(b<1024)return b+" B";if(b<1048576)return(b/1024).toFixed(1)+" KB";if(b<1073741824)return(b/1048576).toFixed(1)+" MB";return(b/1073741824).toFixed(1)+" GB";}
function fmtSpeed(bps){return bps?fmtBytes(bps)+"/s":"";}
function fmtEta(s){if(!s&&s!==0)return"";if(s<60)return s+"s";if(s<3600)return Math.floor(s/60)+"m "+(s%60)+"s";return Math.floor(s/3600)+"h "+Math.floor((s%3600)/60)+"m";}
function t(key) { return S().t ? S().t(key) : key; }

var jobPollers = {};

function renderLocalDetail(el) {
  el.innerHTML = '<p class="gb-set-desc">' + esc(t("local")) + '</p>' +
    '<div id="gbLocalContent"><p class="gb-set-hint">' +
    esc(A.t("assistant.connecting","Loading...")) + '</p></div>';
  loadLocalModels(el);
}

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
  var localModels;
  try {
    var r = await A.apiFetch("/api/llm/local/models");
    if (!r.ok) { container.innerHTML = '<p class="gb-set-hint">' + esc(t("noModels")) + '</p>'; return; }
    localModels = await r.json();
  } catch (e) { container.innerHTML = '<p class="gb-set-hint">' + esc(t("noModels")) + '</p>'; return; }
  S().localModels = localModels;
  var hw = localModels.hardware || {};
  var hwLine = hw.cuda_usable
    ? t("hw").replace("%d",hw.device||"?").replace("%v",String(hw.vram_gb||0)).replace("%f",String(hw.vram_free_gb||0)).replace("%r",String(hw.ram_gb||0))
    : t("hwCpu").replace("%r",String(hw.ram_gb||0));
  var html = '<p class="gb-set-desc" style="font-size:12px">' + esc(hwLine);
  if (hw.driver) html += ' &middot; ' + esc(t("hwDriver").replace("%drv",hw.driver));
  if (hw.cuda_usable) html += ' &middot; ' + esc(t("hwCuda"));
  html += '</p>';
  html += '<p class="gb-set-desc gb-set-privacy" style="font-size:12px;margin-top:4px">' + esc(t("privLocal")) + '</p>';
  html += '<p class="gb-set-hint" style="font-size:11px;margin-top:2px">' + esc(t("noImageReading")) + '</p>';
  var cfg = S().cfg;
  var lr = cfg && cfg.local_runtime;
  if (lr && !lr.installed) {
    html += '<p class="gb-set-warn">' + esc(t("runtimeNotInstalled")) + '</p>';
  } else if (lr && !lr.runtime_reachable) {
    html += '<p class="gb-set-warn">' + esc(t("runtimeNotReachable")) + '</p>';
  }
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
  if (lr && !lr.installed) { renderInstallButton(container); }
  bindModelActions(container);
}

function fitLabel(m) {
  if (m.server_only) return t("serverOnly");
  var v = m.fit_verdict;
  if (v === "comfortable") return t("comfortable");
  if (v === "tight") return t("tight");
  if (v === "wont_run") return t("wontRun").replace("%v", String(m.min_vram_gb));
  return m.fits ? t("fits") : t("nofit").replace("%v", String(m.min_vram_gb));
}

function modelRow(m) {
  var statusLabel = t(m.status || "available");
  var badge = m.active ? ' gb-badge-active' : (m.downloaded ? ' gb-badge-ok' : '');
  var size = fmtBytes(m.size_bytes);
  var fitsText = fitLabel(m);
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
  var meta = esc(size) + ' &middot; ' + esc(fitsText);
  if (m.licence) meta += ' &middot; ' + esc(t("licence")) + ': ' + esc(m.licence);
  if (m.source_host) meta += ' &middot; ' + esc(t("source")) + ': ' + esc(m.source_host);
  return '<div class="gb-set-model-row">' +
    '<div class="gb-set-model-info"><span class="gb-set-model-row-name">' + esc(m.name) + ' <small>' + esc(m.quant) + '</small></span>' +
    '<span class="gb-badge' + badge + '">' + esc(statusLabel) + '</span></div>' +
    '<div class="gb-set-model-meta">' + meta + '</div>' +
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
      var host = "huggingface.co";
      try { host = new URL("https://"+repo.split("/")[0]).hostname || repo; } catch(e) { host = repo; }
      S().inlineConfirm(t("confirmDl").replace("%s",sz).replace("%h",host), function() { doDownload(id); });
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
    if (d.restart_required) {
      if (msg) msg.innerHTML = '<p class="gb-set-hint">' + esc(t("restartNeeded")) + '</p>';
    } else if (d.provider_swapped) {
      if (msg) msg.innerHTML = '<p class="gb-set-hint gb-set-ok">' + esc(t("swapped")) + '</p>';
    } else {
      if (msg) msg.innerHTML = '<p class="gb-set-hint gb-set-ok">' + esc(t("saved")) + '</p>';
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

function stopJobPoller(jobId) {
  if (jobPollers[jobId]) { clearInterval(jobPollers[jobId]); delete jobPollers[jobId]; }
}

/* Expose to parent module */
A._modelLocal = {
  renderLocalDetail: renderLocalDetail,
  startJobPoller: startJobPoller,
  stopJobPoller: stopJobPoller
};
})();
