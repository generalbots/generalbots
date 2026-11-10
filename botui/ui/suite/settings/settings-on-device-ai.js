"use strict";

// Settings → On-device AI (#2109). Local models only: the table is grouped by
// VRAM tier and every row is driven by the /api/llm/local/* routes. The active
// choice is persisted server-side in Vault; no provider API keys are involved.
(function () {
  var state = {
    tiers: [],
    recommended: null,
    recommendedFits: true,
    profiles: [],
    activeJob: null,
    pollTimer: null,
    pendingDelete: {}
  };

  function token() {
    if (typeof window.getGBAccessToken === "function") {
      var value = window.getGBAccessToken();
      if (value) return value;
    }
    return (
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  }

  function json(path, options) {
    var headers = { "Content-Type": "application/json" };
    var value = token();
    if (value) headers.Authorization = "Bearer " + value;
    return fetch(path, Object.assign({ headers: headers }, options || {})).then(function (response) {
      return response.json().catch(function () {
        return {};
      }).then(function (body) {
        if (!response.ok) {
          throw new Error(body.error || body.detail || "HTTP " + response.status);
        }
        return body;
      });
    });
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function fmtBytes(bytes) {
    var value = Number(bytes) || 0;
    if (value >= 1024 * 1024 * 1024) return (value / (1024 * 1024 * 1024)).toFixed(1) + " GB";
    if (value >= 1024 * 1024) return (value / (1024 * 1024)).toFixed(0) + " MB";
    return value + " B";
  }

  function say(message) {
    var node = document.getElementById("oda-status");
    if (node) node.textContent = message || "";
  }

  function allModels() {
    var rows = [];
    state.tiers.forEach(function (tier) {
      (tier.models || []).forEach(function (model) {
        rows.push(model);
      });
    });
    return rows;
  }

  function findModel(id) {
    return allModels().filter(function (model) {
      return model.id === id;
    })[0] || null;
  }

  function statusCell(model) {
    if (model.status === "active") {
      return '<span class="oda-status-active">Active</span>';
    }
    if (model.status === "downloaded") {
      return "Downloaded";
    }
    if (model.status === "downloading") {
      var percent = model.progress == null ? 0 : model.progress;
      return (
        "Downloading " + percent + "%" +
        '<div class="oda-progress"><span style="width:' + percent + '%"></span></div>'
      );
    }
    if (model.partial_bytes > 0) {
      return 'Available <span class="oda-muted">(resume at ' + fmtBytes(model.partial_bytes) + ")</span>";
    }
    return "Available";
  }

  function actionsCell(model) {
    var id = escapeHtml(model.id);
    var parts = [];
    if (model.status === "active") {
      var options = state.profiles.map(function (profile) {
        return '<option value="' + escapeHtml(profile.id) + '">' + escapeHtml(profile.label) + "</option>";
      }).join("");
      parts.push(
        '<select class="oda-profile-select" data-profile-for="' + id + '" title="Context and speed preset">' +
          options +
        "</select>"
      );
      parts.push('<button type="button" class="btn-secondary btn-sm" data-oda="profile" data-id="' + id + '">Apply</button>');
      return '<div class="oda-actions">' + parts.join("") + "</div>";
    }
    if (model.status === "downloaded") {
      parts.push('<button type="button" class="btn-primary btn-sm" data-oda="activate" data-id="' + id + '">Activate</button>');
      parts.push(
        '<button type="button" class="btn-secondary btn-sm" data-oda="delete" data-id="' + id + '">' +
          (state.pendingDelete[id] ? "Confirm delete" : "Delete") +
        "</button>"
      );
      return '<div class="oda-actions">' + parts.join("") + "</div>";
    }
    if (model.status === "downloading") {
      parts.push('<button type="button" class="btn-secondary btn-sm" data-oda="cancel" data-id="' + id + '">Cancel</button>');
      return '<div class="oda-actions">' + parts.join("") + "</div>";
    }
    if (model.server_only || !model.repo) {
      return '<span class="oda-muted">Bundled</span>';
    }
    parts.push('<button type="button" class="btn-primary btn-sm" data-oda="download" data-id="' + id + '">Download</button>');
    return '<div class="oda-actions">' + parts.join("") + "</div>";
  }

  function modelRow(model) {
    var vramClass = model.fits ? "oda-muted" : "oda-warn";
    var vramTitle = model.fits ? "" : ' title="Needs more memory than this device has"';
    var name =
      '<span class="oda-model">' + escapeHtml(model.name) +
      "<small>" + escapeHtml(model.quant) + "</small></span>";
    return (
      '<tr data-model="' + escapeHtml(model.id) + '">' +
        "<td>" + name + "</td>" +
        "<td>" + escapeHtml(model.params) + "</td>" +
        "<td>" + escapeHtml(model.quant) + "</td>" +
        "<td>" + fmtBytes(model.size_bytes) + "</td>" +
        '<td class="' + vramClass + '"' + vramTitle + ">" + model.min_vram_gb + " GB</td>" +
        '<td class="oda-status-cell">' + statusCell(model) + "</td>" +
        "<td>" + actionsCell(model) + "</td>" +
      "</tr>"
    );
  }

  function renderTable() {
    var body = document.getElementById("oda-models");
    if (!body) return;
    var html = "";
    state.tiers.forEach(function (tier) {
      html +=
        '<tr class="oda-tier"><td colspan="7">' + escapeHtml(tier.label) + "</td></tr>";
      (tier.models || []).forEach(function (model) {
        html += modelRow(model);
      });
    });
    body.innerHTML = html || '<tr><td colspan="7" class="oda-empty">No models in the catalogue.</td></tr>';
  }

  function renderHardware(data) {
    var node = document.getElementById("oda-hardware");
    if (!node) return;
    var hardware = data.hardware || {};
    node.textContent =
      (hardware.device || "Unknown device") +
      " · " + (hardware.vram_gb ? hardware.vram_gb + " GB VRAM" : "no dedicated VRAM") +
      " · " + (hardware.ram_gb || 0) + " GB RAM";
    var disk = document.getElementById("oda-disk");
    if (disk) {
      disk.textContent = data.free_bytes == null
        ? ""
        : data.models_dir + ": " + fmtBytes(data.free_bytes) + " free";
    }
  }

  function renderRecommended(data) {
    var box = document.getElementById("oda-recommended");
    var label = document.getElementById("oda-recommended-model");
    var meta = document.getElementById("oda-recommended-meta");
    var button = document.getElementById("oda-recommended-action");
    if (!box || !label || !meta || !button) return;
    var model = findModel(data.recommended);
    state.recommended = data.recommended;
    state.recommendedFits = data.recommended_fits !== false;
    if (!model) {
      box.hidden = true;
      return;
    }
    box.hidden = false;
    label.textContent = model.name;
    meta.textContent =
      " · " + model.quant + " · " + fmtBytes(model.size_bytes) +
      (state.recommendedFits ? "" : " · needs more memory than this device has");
    button.hidden = false;
    button.setAttribute("data-id", model.id);
    if (model.status === "active") {
      button.textContent = "Active";
      button.disabled = true;
      button.setAttribute("data-oda", "none");
    } else if (model.status === "downloading") {
      button.textContent = "Cancel download";
      button.disabled = false;
      button.setAttribute("data-oda", "cancel");
    } else if (model.status === "downloaded") {
      button.textContent = "Activate";
      button.disabled = false;
      button.setAttribute("data-oda", "activate");
    } else {
      button.textContent = "Download " + fmtBytes(model.size_bytes);
      button.disabled = false;
      button.setAttribute("data-oda", "download");
    }
  }

  function stopPolling() {
    if (state.pollTimer) {
      clearInterval(state.pollTimer);
      state.pollTimer = null;
    }
  }

  function updateProgress(modelId, job) {
    var model = findModel(modelId);
    if (!model) return;
    var percent = job.bytes_total > 0
      ? Math.min(100, Math.round((job.bytes_done / job.bytes_total) * 100))
      : 0;
    model.status = job.state === "downloading" ? "downloading" : model.status;
    model.progress = percent;
    var row = document.querySelector('#oda-models tr[data-model="' + modelId + '"] .oda-status-cell');
    if (row) {
      row.innerHTML =
        "Downloading " + percent + "%" +
        '<div class="oda-progress"><span style="width:' + percent + '%"></span></div>';
    }
  }

  function poll(jobId, modelId) {
    json("/api/llm/local/jobs/" + encodeURIComponent(jobId)).then(function (job) {
      if (job.state === "downloading") {
        updateProgress(modelId, job);
        return;
      }
      stopPolling();
      state.activeJob = null;
      if (job.state === "failed") {
        say("Download failed: " + (job.error || "unknown error") + ". The partial file is kept for a retry.");
      } else if (job.state === "cancelled") {
        say("Download cancelled. The partial file is kept and resumes on the next download.");
      }
      load();
    }).catch(function (error) {
      stopPolling();
      say("Lost track of the download: " + error.message);
    });
  }

  function startPolling(jobId, modelId) {
    stopPolling();
    state.activeJob = { jobId: jobId, modelId: modelId };
    state.pollTimer = setInterval(function () {
      poll(jobId, modelId);
    }, 2000);
    poll(jobId, modelId);
  }

  function download(id) {
    say("Starting download…");
    json("/api/llm/local/models/" + encodeURIComponent(id) + "/download", { method: "POST" })
      .then(function (body) {
        say("Download started. Progress resumes after a page reload.");
        startPolling(body.job_id, id);
      })
      .catch(function (error) {
        say("Cannot download: " + error.message);
      });
  }

  function cancel(id) {
    var job = state.activeJob;
    if (!job || job.modelId !== id) {
      say("No running download for this model.");
      return;
    }
    json("/api/llm/local/jobs/" + encodeURIComponent(job.jobId) + "/cancel", { method: "POST" })
      .then(function () {
        say("Cancelling…");
      })
      .catch(function (error) {
        say("Cannot cancel: " + error.message);
      });
  }

  function activate(id, profile) {
    var model = findModel(id);
    if (!model) return;
    if (!model.fits) {
      if (!window.confirm("This model needs more memory than this device has. Activate it anyway?")) {
        return;
      }
    }
    if (!window.confirm("Switch the on-device model? It is used the next time the local model starts.")) {
      return;
    }
    var payload = { model_id: id };
    if (profile) payload.profile = profile;
    json("/api/llm/local/activate", { method: "POST", body: JSON.stringify(payload) })
      .then(function (body) {
        say(
          "Saved. " + body.active + " is the on-device model; restart the local model to load it."
        );
        load();
      })
      .catch(function (error) {
        say("Cannot activate: " + error.message);
      });
  }

  function remove(id) {
    var model = findModel(id);
    if (!model) return;
    var big = Number(model.size_bytes) > 10 * 1024 * 1024 * 1024;
    if (big && !state.pendingDelete[id]) {
      state.pendingDelete[id] = true;
      renderTable();
      say("Click Confirm delete to remove " + model.name + " (" + fmtBytes(model.size_bytes) + ").");
      return;
    }
    delete state.pendingDelete[id];
    json("/api/llm/local/models/" + encodeURIComponent(id), { method: "DELETE" })
      .then(function () {
        say("Deleted " + model.name + ".");
        load();
      })
      .catch(function (error) {
        say("Cannot delete: " + error.message);
      });
  }

  function onAction(event) {
    var target = event.target;
    if (!target || !target.closest) return;
    if (!target.closest("#on-device-ai-section")) return;
    if (target.closest("#oda-refresh")) {
      event.preventDefault();
      return load();
    }
    var button = target.closest("[data-oda]");
    if (!button) return;
    var action = button.getAttribute("data-oda");
    var id = button.getAttribute("data-id");
    if (!action || action === "none" || !id) return;
    event.preventDefault();
    if (action === "download") return download(id);
    if (action === "cancel") return cancel(id);
    if (action === "activate") return activate(id, null);
    if (action === "delete") return remove(id);
    if (action === "profile") {
      var select = document.querySelector('[data-profile-for="' + id + '"]');
      return activate(id, select ? select.value : null);
    }
  }

  function load() {
    var body = document.getElementById("oda-models");
    if (!body) return;
    say("Loading…");
    Promise.all([
      json("/api/llm/local/models"),
      json("/api/llm/local/profiles").catch(function () { return []; })
    ]).then(function (results) {
      var data = results[0] || {};
      state.tiers = data.tiers || [];
      state.profiles = results[1] || [];
      renderHardware(data);
      renderTable();
      renderRecommended(data);
      say("");
    }).catch(function (error) {
      say("Cannot load the on-device models: " + error.message);
    });
  }

  window.loadOnDeviceAi = load;

  // The section arrives through an HTMX swap, so listeners are delegated on
  // the document instead of bound to elements that may not exist yet.
  document.addEventListener("click", onAction);
  document.addEventListener("DOMContentLoaded", function () {
    if (window.location.hash === "#on-device-ai") {
      window.setTimeout(load, 800);
    }
  });
})();
