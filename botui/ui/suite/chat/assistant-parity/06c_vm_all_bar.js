/* Assistant parity — VM-only "allow all" grant bar for classic chat.
 *
 * The assistant stores a broad grant as a /api/consent/permissions row with
 * action_class "vm_all" and app_id "vm:<agent>:<thread>". This module shows
 * those rows above chat's composer and revokes them with DELETE; it never
 * grants a permission itself. The bar is a polite status region.
 */
"use strict";
(function () {
  var P = (window.GBChatAssistantParity = window.GBChatAssistantParity || {});
  var mounted = false;
  var shownFor = null;
  var grants = [];

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value).replace(/&/g, "&amp;")
      .replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function headers(extra) {
    var h = extra || {};
    try {
      if (window.GBShared && window.GBShared.auth && typeof window.GBShared.auth.headers === "function") {
        var all = window.GBShared.auth.headers();
        Object.keys(all).forEach(function (key) { h[key] = all[key]; });
      }
    } catch (error) { /* fall through */ }
    try {
      var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") || "";
      if (token) h.Authorization = "Bearer " + token;
    } catch (error) { /* anonymous chat */ }
    return h;
  }
  function thread() {
    return String((window.ChatState && ChatState.currentSessionId) || "");
  }
  function liveFor(rows, id) {
    var now = Date.now();
    return (Array.isArray(rows) ? rows : []).filter(function (row) {
      if (!row || row.action_class !== "vm_all" || row.granted === false) return false;
      var app = String(row.app_id || "");
      return app.indexOf("vm:") === 0 && app.slice(-id.length - 1) === ":" + id &&
        (!row.expires_at || Date.parse(row.expires_at) > now);
    });
  }
  function bar() {
    var el = document.getElementById("gbParityVmAllBar");
    var form = document.getElementById("chatForm");
    if (el || !form || !form.parentNode) return el;
    el = document.createElement("div");
    el.id = "gbParityVmAllBar";
    el.className = "gb-parity-vm-allbar";
    el.setAttribute("role", "status");
    el.setAttribute("aria-live", "polite");
    el.hidden = true;
    form.parentNode.insertBefore(el, form);
    el.addEventListener("click", function (event) {
      var btn = event.target && event.target.closest
        ? event.target.closest(".gb-parity-vm-allbar-revoke") : null;
      if (btn) revoke(btn.getAttribute("data-id"), btn);
    });
    return el;
  }
  function agentName(row) {
    var parts = String(row && row.app_id || "").split(":");
    var id = parts[1] || "chat";
    return P.consentVm && typeof P.consentVm.agentName === "function"
      ? P.consentVm.agentName(id) : id;
  }
  function render() {
    var el = bar();
    if (!el) return;
    var id = thread();
    var rows = shownFor === id ? grants : [];
    el.hidden = rows.length === 0;
    el.innerHTML = rows.map(function (row) {
      return '<span class="gb-parity-vm-allbar-item"><span aria-hidden="true">\u{1F6E1}</span>' +
        "<strong>" + esc(t("chat-vm-all-chip", "Full permission") + ": " + agentName(row)) + "</strong>" +
        '<span class="gb-parity-vm-allbar-scope">' +
        esc(t("chat-vm-all-scope", "only inside the Virtual Machine")) + "</span>" +
        '<span aria-hidden="true">\u00B7</span>' +
        '<button type="button" class="gb-parity-vm-allbar-revoke" data-id="' + esc(row.id) + '">' +
        esc(t("chat-vm-all-revoke", "Revoke")) + "</button></span>";
    }).join("");
  }
  function refresh(force) {
    var id = thread();
    try {
      var cfg = window.productConfig;
      if (cfg && Array.isArray(cfg.compiled_features) &&
          cfg.compiled_features.indexOf("consent") === -1) {
        shownFor = id;
        grants = [];
        render();
        return;
      }
    } catch (error) { /* no manifest: try the endpoint */ }
    if (!force && shownFor === id) { render(); return; }
    fetch("/api/consent/permissions", { cache: "no-store", headers: headers() })
      .then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        if (thread() !== id) return;
        var rows = data ? (data.permissions || data.items || data.grants || data) : [];
        shownFor = id;
        grants = liveFor(rows, id);
        render();
      })
      .catch(function () { /* the bar keeps its previous state */ });
  }
  function revoke(id, button) {
    if (!id || !button || button.disabled) return;
    button.disabled = true;
    fetch("/api/consent/permissions/" + encodeURIComponent(id), {
      method: "DELETE", headers: headers()
    }).then(function (response) {
      if (!response.ok) throw new Error("HTTP " + response.status);
      grants = grants.filter(function (row) { return String(row.id) !== String(id); });
      render();
      var input = document.getElementById("messageInput");
      if (input) input.focus();
      if (P.consentVm && P.consentVm.announce) {
        P.consentVm.announce(t("chat-vm-all-revoked", "Full permission revoked"));
      }
    }).catch(function () {
      button.disabled = false;
      if (P.consentVm && P.consentVm.announce) {
        P.consentVm.announce(t("chat-vm-consent-outcome-failed", "Could not record your decision"));
      }
    });
  }
  function onDecision(event) {
    var detail = (event && event.detail) || {};
    if (detail.decision === "allow_all" && detail.ok !== false) refresh(true);
  }

  P.vmAllBar = {
    mount: function () {
      if (mounted) return P;
      mounted = true;
      window.addEventListener("gb-vm-consent-decision", onDecision);
      window.addEventListener("gb:auth:login", function () { refresh(true); });
      window.addEventListener("gb-tab-focused", function () { refresh(false); });
      if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", function () { refresh(true); }, { once: true });
      } else {
        refresh(true);
      }
      return P;
    },
    refresh: refresh,
    liveFor: liveFor
  };
})();
