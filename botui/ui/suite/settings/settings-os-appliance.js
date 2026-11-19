(function (window) {
  "use strict";

  var PAGES = [
    { id: "os-network", endpoints: ["/api/os/network"], body: "os-net-status-body" },
    { id: "os-power", endpoints: ["/api/system/power/status", "/api/system/os/power/status", "/api/os/power"], body: "os-power-status-body" },
    { id: "os-display", endpoints: ["/api/os/display"], body: "os-display-status-body" },
    { id: "os-sound", endpoints: ["/api/os/sound"], body: "os-sound-status-body" },
    { id: "os-keyboard", endpoints: ["/api/os/input"], body: "os-keyboard-status-body" },
    { id: "os-storage-reset", endpoints: ["/api/os/storage"], body: "os-storage-reset-status-body" },
    { id: "os-users", endpoints: ["/api/os/users"], body: "os-users-status-body" },
    { id: "os-diagnostics", endpoints: ["/api/os/diagnostics"], body: "os-diagnostics-status-body" }
  ];

  function t(key, fallback, params) {
    var v = fallback;
    if (typeof window._t === "function") {
      var r = window._t(key, fallback);
      if (r && r !== key) v = r;
    }
    if (params && v) {
      Object.keys(params).forEach(function (k) {
        v = String(v).split("{" + k + "}").join(String(params[k]));
      });
    }
    return v;
  }

  function byId(id) { return document.getElementById(id); }

  function setBody(id, text) {
    var el = byId(id);
    if (!el) return;
    el.textContent = text;
  }

  function setBodyHtml(id, html) {
    var el = byId(id);
    if (!el) return;
    el.innerHTML = html;
  }

  function esc(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function renderPower(data) {
    var lines = [];
    var present = !!(data && data.present);
    var pct = data && typeof data.percent === "number" ? data.percent : null;
    var state = data && data.state ? String(data.state) : (present ? "Unknown" : "NoBattery");
    var ac = !!(data && data.ac_online);
    lines.push('<div class="os-power-grid">');
    lines.push(row(t("os-power-present", "Battery present"), present ? t("os-yes", "Yes") : t("os-no", "No")));
    if (pct !== null) lines.push(row(t("os-power-percent", "Charge"), pct + "%"));
    lines.push(row(t("os-power-state", "State"), esc(state)));
    lines.push(row(t("os-power-ac", "AC power"), ac ? t("os-power-ac-online", "Online") : t("os-power-ac-offline", "Offline")));
    if (data && typeof data.time_remaining_minutes === "number" && data.time_remaining_minutes > 0) {
      lines.push(row(t("os-power-time-remaining", "Estimated time remaining"), t("os-power-minutes", "{minutes} min", { minutes: data.time_remaining_minutes })));
    }
    var sources = (data && Array.isArray(data.sources)) ? data.sources.length : 0;
    lines.push(row(t("os-power-sources", "Power sources detected"), String(sources)));
    lines.push('</div>');
    lines.push('<ul class="os-power-notes">');
    lines.push('<li>' + esc(t("os-power-thresholds-note", "Low-battery notifications are shown at 20%, 10%, and 5% when host telemetry is available.")) + '</li>');
    lines.push('<li>' + esc(t("os-power-policy-note", "Lid policy and configurable critical actions (suspend/hibernate/shutdown) require backend support and remain unavailable in this build.")) + '</li>');
    lines.push('</ul>');
    return lines.join("");
  }

  function row(label, value) {
    return '<div class="os-power-row"><span class="os-power-label">' + esc(label) + '</span><span class="os-power-value">' + esc(value) + '</span></div>';
  }

  function tryEndpoints(endpoints, i, page) {
    var msgUnavailable = t("os-backend-unavailable", "OS backend endpoint unavailable in this build. This page is read-only.");
    if (i >= endpoints.length) {
      setBody(page.body, msgUnavailable);
      return Promise.resolve();
    }
    return fetch(endpoints[i], { headers: { Accept: "application/json" }, credentials: "same-origin" })
      .then(function (r) {
        if (!r.ok) throw new Error(String(r.status));
        return r.json().catch(function () { return {}; });
      })
      .then(function (data) {
        if (page.id === "os-power" && data && (typeof data.present === "boolean" || data.state || data.sources)) {
          setBodyHtml(page.body, renderPower(data));
          return;
        }
        if (data && (data.message || data.status)) {
          setBody(page.body, String(data.message || data.status));
        } else {
          setBody(page.body, t("os-backend-connected", "Connected to OS backend."));
        }
      })
      .catch(function () {
        return tryEndpoints(endpoints, i + 1, page);
      });
  }

  function loadPage(page) {
    var msgPending = t("os-backend-pending", "Waiting for OS backend…");
    setBody(page.body, msgPending);
    tryEndpoints(page.endpoints || [page.endpoint], 0, page);
  }

  function loadIfActive(id) {
    var page = null;
    for (var i = 0; i < PAGES.length; i++) {
      if (PAGES[i].id === id) { page = PAGES[i]; break; }
    }
    if (!page) return;
    loadPage(page);
  }

  function onHash() {
    var hash = (window.location.hash || "").replace(/^#/, "");
    loadIfActive(hash);
  }

  window.loadOsAppliancePage = loadIfActive;
  document.addEventListener("DOMContentLoaded", onHash);
  window.addEventListener("hashchange", onHash);
  document.addEventListener("htmx:load", onHash);
})(window);
