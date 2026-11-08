/* Drive #1955 — local sync manager: rsync jobs owned by the botserver.
 *
 * Lists jobs with their last status, creates/edits/deletes them, runs one now,
 * and resolves conflicts (keep local / keep Drive / keep both). Opens from the
 * toolbar sync button, the tray "Drive Sync…" entry, or /drive?panel=sync.
 * The browser-side File System Access mirror (12_offline_sync*) stays as it is:
 * that one needs no server, this one runs on a schedule. */
"use strict";

(function () {
  var API = "/api/drive/sync";
  var jobs = [];
  var dialog = null;
  var poll = null;

  function token() {
    return sessionStorage.getItem("gb_access_token") || localStorage.getItem("gb_access_token") || "";
  }

  function esc(value) {
    var d = document.createElement("div");
    d.textContent = value == null ? "" : String(value);
    return d.innerHTML;
  }

  function el(id) {
    return document.getElementById(id);
  }

  function api(path, options) {
    options = options || {};
    var headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
    var t = token();
    if (t) headers.Authorization = "Bearer " + t;
    options.headers = headers;
    return fetch(API + path, options).then(function (response) {
      return response
        .json()
        .catch(function () { return { success: false, error: "HTTP " + response.status }; })
        .then(function (body) {
          if (!response.ok || body.success === false) {
            throw new Error((body && body.error) || "HTTP " + response.status);
          }
          return body;
        });
    });
  }

  function build() {
    if (dialog) return dialog;
    dialog = document.createElement("div");
    dialog.id = "drive-sync-modal";
    dialog.className = "drive-sync-modal hidden";
    dialog.innerHTML =
      '<div class="drive-sync-overlay" data-close="1"></div>' +
      '<div class="drive-sync-dialog" role="dialog" aria-modal="true" aria-labelledby="drive-sync-title">' +
      '<header><h2 id="drive-sync-title">Local sync</h2>' +
      '<span class="drive-sync-state" id="drive-sync-state">idle</span>' +
      '<button type="button" class="drive-sync-x" data-close="1" aria-label="Close">&times;</button></header>' +
      '<div class="drive-sync-body">' +
      '<p class="drive-sync-hint">Mirror a local folder with a Drive folder using rsync. The server runs the schedule; every job can also run on demand.</p>' +
      '<div class="drive-sync-jobs" id="drive-sync-jobs"></div>' +
      '<form class="drive-sync-form" id="drive-sync-form">' +
      '<div class="drive-sync-grid">' +
      '<label>Name<input type="text" id="drive-sync-name" placeholder="Photos" required></label>' +
      '<label>Local folder<input type="text" id="drive-sync-local" placeholder="/home/me/Pictures" required></label>' +
      '<label>Drive folder<input type="text" id="drive-sync-drive" placeholder="/home/me/GeneralBots/Drive/Photos" required></label>' +
      '<label>Direction<select id="drive-sync-direction"><option value="push">Local to Drive</option><option value="pull">Drive to local</option><option value="two-way">Two-way</option></select></label>' +
      '<label>Every (minutes)<input type="number" id="drive-sync-interval" min="0" step="5" value="0"></label>' +
      '</div>' +
      '<div class="drive-sync-actions"><button type="submit" class="drive-sync-primary">Create job</button>' +
      '<span class="drive-sync-err" id="drive-sync-err" role="alert"></span></div>' +
      '</form>' +
      '</div></div>';
    document.body.appendChild(dialog);
    dialog.addEventListener("click", function (event) {
      if (event.target.getAttribute("data-close")) close();
    });
    el("drive-sync-form").addEventListener("submit", onCreate);
    return dialog;
  }

  function stateLabel(job) {
    if (!job.enabled) return "paused";
    if (job.last_status === "syncing") return "syncing";
    if (job.last_status === "error") return "error";
    if (job.last_status === "conflict") return "conflict";
    if (job.last_status === "ok") return "ok";
    return "idle";
  }

  function renderJobs() {
    var host = el("drive-sync-jobs");
    if (!host) return;
    if (!jobs.length) {
      host.innerHTML = '<p class="drive-sync-empty">No sync jobs yet. Add one below to mirror a folder.</p>';
      return;
    }
    host.innerHTML = jobs
      .map(function (job) {
        var state = stateLabel(job);
        var schedule = job.interval_minutes > 0 ? "every " + job.interval_minutes + " min" : "manual";
        var conflicts = (job.conflicts || [])
          .map(function (conflict) {
            return (
              '<li class="drive-sync-conflict"><span title="' + esc(conflict.detail) + '">' + esc(conflict.path) + "</span>" +
              '<button type="button" data-conflict="' + esc(job.id) + '" data-path="' + esc(conflict.path) + '" data-resolution="local">keep local</button>' +
              '<button type="button" data-conflict="' + esc(job.id) + '" data-path="' + esc(conflict.path) + '" data-resolution="remote">keep Drive</button>' +
              '<button type="button" data-conflict="' + esc(job.id) + '" data-path="' + esc(conflict.path) + '" data-resolution="both">keep both</button></li>'
            );
          })
          .join("");
        return (
          '<article class="drive-sync-job" data-id="' + esc(job.id) + '">' +
          '<div class="drive-sync-job-head"><strong>' + esc(job.name) + "</strong>" +
          '<span class="drive-sync-badge is-' + state + '">' + state + "</span>" +
          '<span class="drive-sync-schedule">' + esc(schedule) + "</span></div>" +
          '<div class="drive-sync-paths"><code>' + esc(job.local_path) + "</code> → <code>" + esc(job.drive_path) + "</code></div>" +
          '<div class="drive-sync-meta">' +
          (job.last_run ? "last run " + esc(new Date(job.last_run).toLocaleString()) : "never ran") +
          (job.last_duration_ms != null ? " · " + job.last_duration_ms + " ms" : "") +
          (job.last_summary ? " · " + esc(job.last_summary) : "") +
          "</div>" +
          (job.last_error ? '<div class="drive-sync-error" role="alert">' + esc(job.last_error) + "</div>" : "") +
          (conflicts ? '<ul class="drive-sync-conflicts">' + conflicts + "</ul>" : "") +
          '<div class="drive-sync-job-actions">' +
          '<button type="button" data-run="' + esc(job.id) + '">Sync now</button>' +
          '<button type="button" data-toggle="' + esc(job.id) + '" data-enabled="' + (job.enabled ? "1" : "0") + '">' + (job.enabled ? "Pause" : "Resume") + "</button>" +
          '<button type="button" data-delete="' + esc(job.id) + '">Delete</button>' +
          "</div></article>"
        );
      })
      .join("");
    host.querySelectorAll("[data-run]").forEach(function (button) {
      button.addEventListener("click", function () { runJob(button.getAttribute("data-run")); });
    });
    host.querySelectorAll("[data-toggle]").forEach(function (button) {
      button.addEventListener("click", function () {
        toggleJob(button.getAttribute("data-toggle"), button.getAttribute("data-enabled") !== "1");
      });
    });
    host.querySelectorAll("[data-delete]").forEach(function (button) {
      button.addEventListener("click", function () { deleteJob(button.getAttribute("data-delete")); });
    });
    host.querySelectorAll("[data-conflict]").forEach(function (button) {
      button.addEventListener("click", function () {
        resolveConflict(
          button.getAttribute("data-conflict"),
          button.getAttribute("data-path"),
          button.getAttribute("data-resolution")
        );
      });
    });
  }

  function setError(message) {
    var box = el("drive-sync-err");
    if (box) box.textContent = message || "";
  }

  function setState(state) {
    var chip = el("drive-sync-status");
    if (chip) {
      chip.hidden = !state || state === "idle";
      chip.textContent = state === "conflict" ? "sync conflicts" : state === "error" ? "sync error" : state === "syncing" ? "syncing" : "";
      chip.className = "drive-sync-chip is-" + (state || "idle");
      chip.title = state === "conflict" ? "Some files changed on both sides — open Local sync" : "Local sync status";
    }
  }

  function refresh() {
    return api("/jobs")
      .then(function (body) {
        jobs = body.jobs || [];
        renderJobs();
        return api("/status");
      })
      .then(function (status) {
        setState(status.state);
        var label = el("drive-sync-state");
        if (label) label.textContent = status.state || "idle";
      })
      .catch(function (error) {
        setError(error.message);
        setState("error");
      });
  }

  function onCreate(event) {
    event.preventDefault();
    setError("");
    var payload = {
      name: el("drive-sync-name").value.trim(),
      local_path: el("drive-sync-local").value.trim(),
      drive_path: el("drive-sync-drive").value.trim(),
      direction: el("drive-sync-direction").value,
      interval_minutes: parseInt(el("drive-sync-interval").value, 10) || 0,
      enabled: true
    };
    api("/jobs", { method: "POST", body: JSON.stringify(payload) })
      .then(function () {
        el("drive-sync-name").value = "";
        return refresh();
      })
      .catch(function (error) { setError(error.message); });
  }

  function runJob(id) {
    setError("");
    setState("syncing");
    api("/jobs/" + encodeURIComponent(id) + "/run", { method: "POST", body: "{}" })
      .then(function () { return refresh(); })
      .catch(function (error) { setError(error.message); return refresh(); });
  }

  function toggleJob(id, enabled) {
    var job = jobs.filter(function (item) { return item.id === id; })[0];
    if (!job) return;
    api("/jobs/" + encodeURIComponent(id), {
      method: "POST",
      body: JSON.stringify({
        name: job.name,
        local_path: job.local_path,
        drive_path: job.drive_path,
        direction: job.direction,
        interval_minutes: job.interval_minutes,
        enabled: enabled
      })
    })
      .then(function () { return refresh(); })
      .catch(function (error) { setError(error.message); });
  }

  function deleteJob(id) {
    api("/jobs/" + encodeURIComponent(id), { method: "DELETE" })
      .then(function () { return refresh(); })
      .catch(function (error) { setError(error.message); });
  }

  function resolveConflict(id, path, resolution) {
    api("/jobs/" + encodeURIComponent(id) + "/resolve", {
      method: "POST",
      body: JSON.stringify({ path: path, resolution: resolution })
    })
      .then(function () { return refresh(); })
      .catch(function (error) { setError(error.message); });
  }

  function open() {
    build();
    dialog.classList.remove("hidden");
    refresh();
    if (!poll) poll = setInterval(function () { if (dialog && !dialog.classList.contains("hidden")) refresh(); }, 4000);
  }

  function close() {
    if (dialog) dialog.classList.add("hidden");
    if (poll) { clearInterval(poll); poll = null; }
  }

  function init() {
    var button = el("drive-sync-btn");
    if (button && button.getAttribute("data-wired") !== "1") {
      button.setAttribute("data-wired", "1");
      button.addEventListener("click", open);
    }
    if (!window.GBDriveSync) window.GBDriveSync = {};
    window.GBDriveSync.open = open;
    window.GBDriveSync.close = close;
    window.GBDriveSync.refresh = refresh;
    var panel = (location.search.match(/[?&]panel=([^&]+)/) || [])[1];
    if (panel === "sync") open();
    api("/status").then(function (status) { setState(status.state); }).catch(function () {});
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
