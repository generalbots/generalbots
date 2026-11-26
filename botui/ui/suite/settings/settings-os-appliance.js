(function (window) {
  "use strict";

  var PAGES = [
    { id: "os-network", endpoints: ["/api/os/network"], body: "os-net-status-body" },
    { id: "os-power", endpoints: ["/api/system/power/status", "/api/system/os/power/status", "/api/os/power"], body: "os-power-status-body" },
    { id: "os-display", endpoints: ["/api/os/display"], body: "os-display-status-body" },
    { id: "os-sound", endpoints: ["/api/os/sound"], body: "os-sound-status-body" },
    { id: "os-keyboard", endpoints: ["/api/os/input"], body: "os-keyboard-status-body" },
    { id: "os-peripherals", endpoints: ["/api/os/peripherals"], body: "os-peripherals-section" },
    { id: "os-storage-reset", endpoints: ["/api/os/storage"], body: "os-storage-reset-status-body" },
    { id: "os-users", endpoints: ["/api/os/users"], body: "os-users-status-body" },
    { id: "os-diagnostics", endpoints: ["/api/os/diagnostics"], body: "os-diagnostics-status-body" }
  ];

  // #2470 — the data-protection surface lives inside the Storage & Reset card
  // (no new navigation entry is needed). It is loaded whenever that page opens.
  var DATA_PAGE = {
    id: "os-data-protection",
    endpoints: ["/api/os/data-protection"],
    body: "os-data-status-body"
  };
  var resetToken = "";
  var manifestScope = "";
  var protectionReady = false;

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

  // #2470 — every region this module writes is a live status region. The
  // shipped partial already carries these attributes, but set them from JS as
  // well so the contract holds even when the HTML is swapped by another owner.
  function setBody(id, text) {
    var el = byId(id);
    if (!el) return;
    if (el.getAttribute("role") !== "status") el.setAttribute("role", "status");
    if (el.getAttribute("aria-live") !== "polite") el.setAttribute("aria-live", "polite");
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
    lines.push('<li>' + esc(t("os-power-policy-note", "Lid policy is managed by the OS image; the critical-battery action is configurable above.")) + '</li>');
    lines.push('</ul>');
    return lines.join("");
  }

  function row(label, value) {
    return '<div class="os-power-row"><span class="os-power-label">' + esc(label) + '</span><span class="os-power-value">' + esc(value) + '</span></div>';
  }

  function syncResetButton() {
    var button = byId("osDataReset");
    var input = byId("osDataResetToken");
    if (!button) return;
    // Fail closed: without a token parsed from a live status response the
    // reset control stays disabled, regardless of what is typed in the box.
    button.disabled = !resetToken || !input || input.value !== resetToken;
  }

  function captureDataProtection(data) {
    if (!data || typeof data !== "object") return;
    if (!data.reset && !data.export) return;
    // A fresh status payload replaces the previous one: a rotated token or a
    // removed export scope must never leave a stale credential/scope usable.
    resetToken = data.reset && data.reset.confirmation_token
      ? String(data.reset.confirmation_token) : "";
    manifestScope = data.export && data.export.contract && data.export.contract.request
      ? String(data.export.contract.request.scope || "") : "";
    protectionReady = true;
    syncResetButton();
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
        captureDataProtection(data);
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
    if (id === "os-storage-reset") loadPage(DATA_PAGE);
  }

  function onHash() {
    var hash = (window.location.hash || "").replace(/^#/, "");
    loadIfActive(hash);
  }

  // #2445 — configurable critical-battery action, persisted per browser and
  // consumed by the tray threshold notifier. "none" means notify only.
  var CRITICAL_KEY = "gb-os-critical-action";
  function initCriticalAction() {
    var select = byId("osCriticalAction");
    if (!select) return;
    try {
      var saved = localStorage.getItem(CRITICAL_KEY);
      if (saved && Array.prototype.some.call(select.options, function (o) { return o.value === saved; })) {
        select.value = saved;
      }
    } catch (e) { /* storage may be disabled */ }
    select.addEventListener("change", function () {
      try { localStorage.setItem(CRITICAL_KEY, select.value); } catch (e) { /* ignore */ }
      document.dispatchEvent(new CustomEvent("gb:critical-action-change", { detail: { action: select.value } }));
    });
  }
  // #2470 — bounded, non-destructive data-protection actions. The server
  // returns the human-readable descriptor/failure text; this file adds no
  // UI copy of its own beyond existing i18n keys.
  function unavailableMessage() {
    return t("os-backend-unavailable", "OS backend endpoint unavailable in this build. No change was made.");
  }

  // Server text is untrusted: never echo back a typed confirmation token or
  // snapshot reason that a response happened to include verbatim.
  function sanitizeStatusText(text) {
    var value = String(text == null ? "" : text);
    var reason = byId("osDataReason");
    var secrets = [resetToken, reason ? reason.value : ""];
    secrets.forEach(function (secret) {
      if (secret && String(secret).length > 3) {
        value = value.split(String(secret)).join("");
      }
    });
    return value;
  }

  function postJson(path, payload) {
    return fetch(path, {
      method: "POST",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify(payload)
    }).then(function (r) {
      return r.json().catch(function () { return {}; }).then(function (data) {
        return { ok: r.ok, data: data || {} };
      });
    }).catch(function () { return { ok: false, data: {} }; });
  }

  function showStatus(id, result) {
    if (!result || result.ok === false) {
      // A failed request gets a localized generic message only: no raw error
      // body and no secret can reach the status region on this path.
      setBody(id, sanitizeStatusText(unavailableMessage()));
      return;
    }
    var data = result.data || {};
    var text = data.message || data.error;
    setBody(id, sanitizeStatusText(text ? String(text) : t("os-data-not-implemented", "Not implemented")));
  }

  function requestManifest() {
    if (!protectionReady || !manifestScope) {
      setBody("osDataExportStatus", unavailableMessage());
      return Promise.resolve();
    }
    return postJson("/api/os/data-protection/export", { scope: manifestScope })
      .then(function (result) { showStatus("osDataExportStatus", result); });
  }

  function requestSnapshot() {
    var input = byId("osDataReason");
    var reason = input ? input.value : "";
    if (!protectionReady || !reason) {
      setBody("osDataSnapshotStatus", unavailableMessage());
      return Promise.resolve();
    }
    return postJson("/api/os/data-protection/snapshots", { reason: reason })
      .then(function (result) { showStatus("osDataSnapshotStatus", result); });
  }

  function requestReset() {
    var input = byId("osDataResetToken");
    // Fail closed: no POST unless the exact server-issued token was typed.
    if (!resetToken || !input || input.value !== resetToken) {
      return Promise.resolve();
    }
    return postJson("/api/os/data-protection/reset", { confirmation: input.value })
      .then(function (result) { showStatus("osDataResetStatus", result); });
  }

  function initDataProtection() {
    var input = byId("osDataResetToken");
    if (input) input.addEventListener("input", syncResetButton);
    syncResetButton();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initCriticalAction);
    document.addEventListener("DOMContentLoaded", initDataProtection);
  } else {
    initCriticalAction();
    initDataProtection();
  }

  window.GBDataProtection = {
    manifest: requestManifest,
    snapshot: requestSnapshot,
    reset: requestReset
  };
  window.loadOsAppliancePage = loadIfActive;
  document.addEventListener("DOMContentLoaded", onHash);
  window.addEventListener("hashchange", onHash);
  document.addEventListener("htmx:load", onHash);
  document.addEventListener("htmx:load", initDataProtection);
})(window);
