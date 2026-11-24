/* #2604 — Protection tab. Every state comes from the real Protection API;
 * nothing is derived from a hardcoded "protection enabled" flag.
 * One action per request; HTMX-style partial refresh, no CDN assets. */
(function (window) {
  "use strict";

  var API = "/api/security/protection";

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function byId(id) { return document.getElementById(id); }

  function text(id, value) {
    var node = byId(id);
    if (node) node.textContent = value;
  }

  async function request(path, options) {
    var response = await window.fetch(path, Object.assign({ credentials: "same-origin" }, options || {}));
    if (!response.ok) throw new Error("HTTP " + response.status);
    var payload = await response.json();
    return payload && payload.data !== undefined ? payload.data : payload;
  }

  function dataText(value, fallback) {
    return value === null || value === undefined || value === "" ? fallback : String(value);
  }

  // The API serialises the tool enum in lowercase; show a localized label while
  // keeping the raw value for the action endpoints.
  function toolName(tool) {
    var name = String(tool || "").toLowerCase();
    if (name === "clamav") return t("protection-clamav", "ClamAV");
    if (name === "lmd") return t("protection-lmd", "LMD");
    return dataText(tool, t("protection-unknown-tool", "unknown tool"));
  }

  function quarantined(metrics) {
    var count = metrics && metrics.quarantined_count;
    var value = typeof count === "number" ? count : 0;
    return t("protection-quarantined-count", value + " quarantined").replace("{count}", String(value));
  }

  function signatureAge(lastUpdate) {
    if (!lastUpdate) return t("protection-unknown", "unknown");
    var elapsed = Date.now() - Date.parse(lastUpdate);
    if (!isFinite(elapsed) || elapsed < 0) return t("protection-unknown", "unknown");
    var days = Math.floor(elapsed / 86400000);
    return t("protection-days-old", days + " days old").replace("{days}", String(days));
  }

  function card(tool) {
    var wrapper = document.createElement("article");
    wrapper.className = "protection-card";
    var running = tool.service_running === true && tool.installed === true;
    var title = document.createElement("h3");
    title.textContent = toolName(tool.tool);
    var state = document.createElement("p");
    state.className = "protection-state " + (running ? "protection-state--on" : "protection-state--off");
    state.textContent = running
      ? t("protection-active", "Active")
      : t("protection-not-scanning", "NOT SCANNING — no scanner is active");
    var details = document.createElement("p");
    details.textContent = [
      t("protection-version", "Version") + ": " + dataText(tool.version, t("protection-unknown", "unknown")),
      t("protection-signature-age", "Signature age") + ": " + signatureAge(tool.last_update),
      t("protection-last-scan", "Last scan") + ": " + dataText(tool.last_scan, t("protection-never", "never")),
      quarantined(tool.metrics)
    ].join(" · ");
    var actions = document.createElement("div");
    var scan = document.createElement("button");
    scan.type = "button";
    scan.textContent = t("protection-run-scan", "Run scan");
    scan.setAttribute("data-i18n", "protection-run-scan");
    scan.addEventListener("click", function () { window.GBProtection.scan(tool.tool); });
    actions.appendChild(scan);
    wrapper.appendChild(title);
    wrapper.appendChild(state);
    wrapper.appendChild(details);
    wrapper.appendChild(actions);
    return wrapper;
  }

  function renderTools(payload) {
    var container = byId("protection-tools");
    var warning = byId("protection-warning");
    if (!container) return;
    var tools = (payload && payload.tools) || [];
    var active = tools.filter(function (tool) { return tool.installed && tool.service_running === true; });
    container.textContent = "";
    tools.forEach(function (tool) { container.appendChild(card(tool)); });
    if (!container.childNodes.length) {
      text("protection-tools", t("protection-no-scanner", "No scanner is registered."));
    }
    var optOut = payload && (payload.opt_out || payload.allow_unscanned_import) === true;
    if (warning) {
      warning.hidden = !(optOut || active.length === 0);
      warning.textContent = optOut
        ? t("protection-opt-out", "Uploads are NOT scanned: unscanned imports are allowed.")
        : t("protection-not-scanning", "NOT SCANNING — no scanner is active.");
    }
  }

  async function refresh() {
    try {
      renderTools(await request(API + "/status"));
    } catch (error) {
      text("protection-tools", t("protection-load-error", "Scanner status is unavailable."));
      var warning = byId("protection-warning");
      if (warning) {
        warning.hidden = false;
        warning.textContent = t("protection-not-scanning", "NOT SCANNING — no scanner is active.");
      }
    }
  }

  async function scan(tool) {
    if (!tool) return;
    try {
      await request(API + "/" + encodeURIComponent(String(tool)) + "/run", { method: "POST" });
      await refresh();
    } catch (error) {
      text("protection-quarantine-list", t("protection-action-failed", "The action failed."));
    }
  }

  async function loadQuarantine() {
    var select = byId("protection-tool");
    var tool = select ? select.value : "clamav";
    var route = tool === "lmd" ? "lmd" : "clamav";
    try {
      var rows = await request(API + "/" + route + "/quarantine");
      var list = Array.isArray(rows) ? rows : (rows && rows.files) || [];
      var container = byId("protection-quarantine-list");
      if (!container) return;
      container.textContent = "";
      if (!list.length) {
        container.textContent = t("protection-quarantine-empty", "No quarantined files.");
        return;
      }
      list.forEach(function (item) {
        var line = document.createElement("p");
        line.textContent = dataText(item.original_path, dataText(item.id, "")) +
          " — " + dataText(item.threat_name, t("protection-unknown", "unknown"));
        var restore = document.createElement("button");
        restore.type = "button";
        restore.textContent = t("protection-restore", "Restore");
        restore.setAttribute("data-i18n", "protection-restore");
        restore.addEventListener("click", function () { window.GBProtection.restore(route, item.id); });
        line.appendChild(restore);
        container.appendChild(line);
      });
    } catch (error) {
      text("protection-quarantine-list", t("protection-load-error", "Scanner status is unavailable."));
    }
  }

  async function restore(tool, id) {
    if (!id) return;
    try {
      await request(API + "/" + encodeURIComponent(String(tool)) + "/quarantine/" + encodeURIComponent(String(id)), { method: "POST" });
      await loadQuarantine();
    } catch (error) {
      text("protection-quarantine-list", t("protection-action-failed", "The action failed."));
    }
  }

  window.GBProtection = { refresh: refresh, scan: scan, loadQuarantine: loadQuarantine, restore: restore };

  // The panel is injected by HTMX, so DOMContentLoaded can fire before its
  // container exists. Refresh on section activation (sidebar/deep link/UI plan)
  // and again once the partial is actually swapped in.
  var previousHook = window.GBSettingsOnSection;
  window.GBSettingsOnSection = function (section) {
    if (typeof previousHook === "function") {
      try { previousHook(section); } catch (e) { /* an earlier hook is optional */ }
    }
    if (section === "protection") refresh();
  };
  document.addEventListener("htmx:afterSwap", function (event) {
    var target = event && (event.target || (event.detail && event.detail.target));
    if (target && target.id === "protection-section") refresh();
  });
  document.addEventListener("DOMContentLoaded", refresh);
})(window);
