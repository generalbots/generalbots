/* #1852 — Operating System panel controller (shell mode only).
 *
 * Everything here is lazy: no request leaves the page until a shell-mode
 * session opens the panel. The backend repeats the gate (shell_mode + admin +
 * loopback), so hiding the UI is not the only protection.
 */
(function (window) {
  "use strict";

  var loaded = false;
  var loading = false;

  function byId(id) {
    return document.getElementById(id);
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function token() {
    return localStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      "";
  }

  function headers(extra) {
    var result = {};
    var value = token();
    if (value) { result.Authorization = "Bearer " + value; }
    if (extra) {
      Object.keys(extra).forEach(function (key) { result[key] = extra[key]; });
    }
    return result;
  }

  function notify(message, type) {
    if (typeof window.showToast === "function") {
      window.showToast(message, type || "success");
    }
  }

  function fetchJson(url, options) {
    return fetch(url, options).then(function (response) {
      return response.json().catch(function () { return {}; }).then(function (body) {
        if (!response.ok) {
          throw new Error(body && body.error ? body.error : "HTTP " + response.status);
        }
        return body;
      });
    });
  }

  function formatBytes(bytes) {
    var value = Number(bytes) || 0;
    var units = ["B", "KB", "MB", "GB", "TB"];
    var index = 0;
    while (value >= 1024 && index < units.length - 1) {
      value /= 1024;
      index += 1;
    }
    return value.toFixed(index === 0 ? 0 : 1) + " " + units[index];
  }

  function formatUptime(seconds) {
    var total = Number(seconds) || 0;
    var days = Math.floor(total / 86400);
    var hours = Math.floor((total % 86400) / 3600);
    var minutes = Math.floor((total % 3600) / 60);
    if (days > 0) { return days + "d " + hours + "h " + minutes + "m"; }
    if (hours > 0) { return hours + "h " + minutes + "m"; }
    return minutes + "m";
  }

  function setText(id, value) {
    var node = byId(id);
    if (node) { node.textContent = value == null || value === "" ? "—" : String(value); }
  }

  function renderSystem(data) {
    setText("os-hostname", data.hostname);
    setText("os-timezone", data.timezone);
    setText("os-os", data.os);
    setText("os-kernel", data.kernel);
    setText("os-arch", data.arch);
    setText("os-uptime", formatUptime(data.uptime_secs));
  }

  function renderServices(services) {
    var container = byId("os-services");
    if (!container) { return; }
    if (!Array.isArray(services) || services.length === 0) {
      container.innerHTML = '<p class="os-empty">No services reported.</p>';
      return;
    }
    container.innerHTML = services.map(function (service) {
      var running = service.running === true;
      return '<div class="os-service-row">' +
        '<div class="os-service-name">' +
        '<span class="os-status-dot ' + (running ? "running" : "stopped") + '"></span>' +
        '<span>' + escapeHtml(service.name) + '</span>' +
        '</div>' +
        '<span class="os-status-text">' + (running ? "running" : "stopped") + '</span>' +
        '<button type="button" class="btn-secondary btn-sm" data-os-restart="' +
        escapeHtml(service.name) + '">Restart</button>' +
        '</div>';
    }).join("");
  }

  function renderDisks(disks) {
    var container = byId("os-disks");
    if (!container) { return; }
    if (!Array.isArray(disks) || disks.length === 0) {
      container.innerHTML = '<p class="os-empty">No mounted filesystems reported.</p>';
      return;
    }
    container.innerHTML = disks.map(function (disk) {
      var percent = Math.max(0, Math.min(100, Number(disk.percent) || 0));
      var tone = percent > 90 ? "danger" : percent > 70 ? "warn" : "";
      return '<div class="os-disk-row">' +
        '<div class="os-disk-main">' +
        '<span class="os-disk-mount">' + escapeHtml(disk.mount) + '</span>' +
        '<span class="os-disk-meta">' + escapeHtml(disk.filesystem) + ' · ' +
        formatBytes(disk.used_bytes) + ' / ' + formatBytes(disk.total_bytes) + '</span>' +
        '</div>' +
        '<div class="os-disk-bar"><div class="os-disk-fill ' + tone +
        '" style="width:' + percent + '%"></div></div>' +
        '<span class="os-status-text">' + percent + '%</span>' +
        '</div>';
    }).join("");
  }

  function renderLogs(text) {
    var output = byId("os-logs");
    if (output) { output.textContent = text || "(log is empty)"; }
  }

  function loadLogs(service) {
    var output = byId("os-logs");
    if (output) { output.textContent = "Loading logs…"; }
    return fetch("/api/system/os/logs?service=" + encodeURIComponent(service || "botserver") +
      "&lines=200", { headers: headers(), cache: "no-store" })
      .then(function (response) {
        return response.text().then(function (body) {
          renderLogs(response.ok ? body : "Could not load logs (HTTP " + response.status + ")\n" + body);
        });
      })
      .catch(function (error) {
        renderLogs("Could not load logs: " + error.message);
      });
  }

  window.changeOsLogService = function (service) {
    return loadLogs(service);
  };

  window.loadOsPanel = function (force) {
    var section = byId("os-section");
    if (!section || section.hidden || section.style.display === "none") { return; }
    if (loading || (loaded && !force)) { return; }
    loading = true;

    fetchJson("/api/system/os", { headers: headers(), cache: "no-store" })
      .then(function (data) {
        renderSystem(data);
        renderServices(data.services);
        renderDisks(data.disks);
        renderLogFiles(data.log_files);
        loaded = true;
        return loadLogs("botserver");
      })
      .catch(function (error) {
        var services = byId("os-services");
        if (services) {
          services.innerHTML = '<p class="os-empty os-error">' +
            escapeHtml(error.message) + '</p>';
        }
        notify("Could not load Operating System data", "error");
      })
      .then(function () { loading = false; });
  };

  window.refreshOsPanel = function () {
    loaded = false;
    window.loadOsPanel(true);
  };

  // #1874 — log files: location, size and a download of the whole file.
  function renderLogFiles(files) {
    var container = byId("os-log-files");
    if (!container) { return; }
    if (!Array.isArray(files) || files.length === 0) {
      container.innerHTML = '<p class="os-empty">No log files reported.</p>';
      return;
    }
    container.innerHTML = files.map(function (file) {
      var service = escapeHtml(file.service);
      var size = file.exists ? formatBytes(file.size_bytes) : "missing";
      var actions = file.exists
        ? '<button type="button" class="btn-secondary btn-sm" data-os-log-view="' + service + '">View</button>' +
          '<button type="button" class="btn-secondary btn-sm" data-os-log-download="' + service + '">Download</button>'
        : "";
      return '<div class="os-log-file-row">' +
        '<div class="os-log-file-main">' +
        '<span class="os-log-file-service">' + service + '</span>' +
        '<code class="os-log-file-path">' + escapeHtml(file.path || "not found") + '</code>' +
        '</div>' +
        '<span class="os-status-text">' + escapeHtml(size) + '</span>' +
        '<div class="os-log-file-actions">' + actions + '</div>' +
        '</div>';
    }).join("");
  }

  function downloadLog(service) {
    fetch("/api/system/os/logs/download?service=" + encodeURIComponent(service),
      { headers: headers(), cache: "no-store" })
      .then(function (response) {
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return response.blob();
      })
      .then(function (blob) {
        var url = URL.createObjectURL(blob);
        var link = document.createElement("a");
        link.href = url;
        link.download = service + ".log";
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
        notify("Downloaded " + service + ".log");
      })
      .catch(function (error) {
        notify("Download failed: " + error.message, "error");
      });
  }

  function bindLogFileButtons() {
    var container = byId("os-log-files");
    if (!container || container.getAttribute("data-wired") === "1") { return; }
    container.setAttribute("data-wired", "1");
    container.addEventListener("click", function (event) {
      var target = event.target && event.target.closest ? event.target : null;
      if (!target) { return; }
      var view = target.closest("[data-os-log-view]");
      if (view) {
        var service = view.getAttribute("data-os-log-view");
        var select = byId("os-log-service");
        if (select) { select.value = service; }
        loadLogs(service);
        return;
      }
      var download = target.closest("[data-os-log-download]");
      if (download) {
        downloadLog(download.getAttribute("data-os-log-download"));
      }
    });
  }

  // #1879 — restart confirmation uses the app dialog, never window.prompt/
  // confirm/alert. The typed service name stays the server-side proof.
  function performRestart(service, confirmation) {
    fetchJson("/api/system/os/services/restart", {
      method: "POST",
      headers: headers({ "Content-Type": "application/json" }),
      body: JSON.stringify({ service: service, confirmation: confirmation })
    }).then(function () {
      notify("Restart requested for " + service);
      window.loadOsPanel(true);
    }).catch(function (error) {
      notify("Restart failed: " + error.message, "error");
    });
  }

  window.restartOsService = function (service) {
    var dialog = byId("os-restart-modal");
    var input = byId("os-restart-confirm");
    var nameNode = byId("os-restart-service-name");
    if (!dialog || typeof dialog.showModal !== "function") {
      notify("Restart confirmation is unavailable in this environment", "error");
      return;
    }
    if (nameNode) { nameNode.textContent = service; }
    if (input) { input.value = ""; }
    dialog.dataset.service = service;
    dialog.showModal();
    if (input) { input.focus(); }
  };

  function bindRestartDialog() {
    var dialog = byId("os-restart-modal");
    if (!dialog || dialog.getAttribute("data-wired") === "1") { return; }
    dialog.setAttribute("data-wired", "1");
    var form = byId("os-restart-form");
    var cancel = byId("os-restart-cancel");
    var cancelButton = byId("os-restart-cancel-button");
    function close() { dialog.close(); }
    if (cancel) { cancel.addEventListener("click", close); }
    if (cancelButton) { cancelButton.addEventListener("click", close); }
    if (form) {
      form.addEventListener("submit", function (event) {
        event.preventDefault();
        var service = dialog.dataset.service || "";
        var input = byId("os-restart-confirm");
        var typed = input ? input.value.trim() : "";
        dialog.close();
        if (!service) { return; }
        if (typed !== service) {
          notify("Confirmation did not match", "error");
          return;
        }
        performRestart(service, typed);
      });
    }
  }

  function bindRestartButtons() {
    var container = byId("os-services");
    if (!container || container.getAttribute("data-wired") === "1") { return; }
    container.setAttribute("data-wired", "1");
    container.addEventListener("click", function (event) {
      var button = event.target && event.target.closest
        ? event.target.closest("[data-os-restart]")
        : null;
      if (!button) { return; }
      window.restartOsService(button.getAttribute("data-os-restart"));
    });
  }

  function bindPanel() {
    bindRestartButtons();
    bindRestartDialog();
    bindLogFileButtons();
  }

  function maybeLoad() {
    if (window.location.hash === "#os") { window.loadOsPanel(); }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      bindPanel();
      maybeLoad();
    });
  } else {
    bindPanel();
    maybeLoad();
  }
  // The partial arrives through HTMX after this module is evaluated; rebind
  // whenever a swap brings the panel markup in.
  document.addEventListener("htmx:load", bindPanel);
  document.addEventListener("htmx:afterSwap", bindPanel);
  window.addEventListener("hashchange", maybeLoad);
})(window);
