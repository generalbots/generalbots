"use strict";
/* #2468 — OS diagnostics export, time-boxed remote support and the branded
 * black-screen recovery surface. All paths are DOM-event driven so the web
 * shell can be UAT'd without the native host. */
(function (window, document) {
  function t(key, fallback, params) {
    if (!window.gbT) return String(fallback).replace(/\{(\w+)\}/g, function (_, k) {
      return params && params[k] != null ? String(params[k]) : "{" + k + "}";
    });
    var value = window.gbT(key, fallback, params);
    if (!value || value.charAt(0) === "[") value = String(fallback);
    return value.replace(/\{(\w+)\}/g, function (_, k) {
      return params && params[k] != null ? String(params[k]) : "{" + k + "}";
    });
  }
  function id(x) { return document.getElementById(x); }

  function download() {
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    var headers = { Accept: "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/api/os/diagnostics/bundle", { headers: headers, credentials: "same-origin" })
      .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.json(); })
      .then(function (bundle) {
        var blob = new Blob([JSON.stringify(bundle, null, 2)], { type: "application/json" });
        var url = URL.createObjectURL(blob);
        var a = document.createElement("a");
        a.href = url;
        a.download = "gb-diagnostics-" + new Date().toISOString().slice(0, 10) + ".json";
        document.body.appendChild(a);
        a.click();
        a.remove();
        setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
        var status = id("os-diagnostics-status-body");
        if (status) status.textContent = t("os-diagnostics-exported", "Diagnostics bundle downloaded (secrets redacted).");
      })
      .catch(function (e) {
        var status = id("os-diagnostics-status-body");
        if (status) status.textContent = t("os-diagnostics-export-failed", "Could not export diagnostics: ") + e.message;
      });
  }

  var remoteTimer = null;
  function setRemoteSupport(minutes) {
    if (remoteTimer) { clearTimeout(remoteTimer); remoteTimer = null; }
    var indicator = id("osRemoteSupportIndicator");
    var n = Number(minutes) || 0;
    if (!indicator) return;
    if (n <= 0) {
      indicator.hidden = true;
      indicator.textContent = "";
      document.body.classList.remove("gb-remote-support-active");
      return;
    }
    indicator.hidden = false;
    indicator.textContent = t("os-remote-support-active", "Remote support active for {n} min", { n: n });
    document.body.classList.add("gb-remote-support-active");
    remoteTimer = setTimeout(function () {
      var sel = id("osRemoteSupport");
      if (sel) sel.value = "off";
      setRemoteSupport("off");
    }, n * 60000);
  }

  function action(labelKey, fallback, onClick) {
    var b = document.createElement("button");
    b.type = "button";
    b.className = "gb-recovery-btn";
    b.textContent = t(labelKey, fallback);
    b.addEventListener("click", onClick);
    return b;
  }

  function recovery() {
    var node = id("gb-recovery-screen");
    if (!node) {
      node = document.createElement("div");
      node.id = "gb-recovery-screen";
      node.className = "gb-recovery-screen";
      node.setAttribute("role", "dialog");
      node.setAttribute("aria-modal", "true");
      node.innerHTML = '<div class="gb-recovery-card">' +
        '<div class="gb-recovery-logo" aria-hidden="true">GB</div>' +
        '<h1 data-i18n="os-recovery-title"></h1>' +
        '<p data-i18n="os-recovery-hint"></p>' +
        '<div class="gb-recovery-actions"></div></div>';
      document.body.appendChild(node);
    }
    var title = node.querySelector("h1");
    var hint = node.querySelector("p");
    if (title) title.textContent = t("os-recovery-title", "The shell needs attention");
    if (hint) hint.textContent = t("os-recovery-hint", "Choose a recovery action below.");
    var actions = node.querySelector(".gb-recovery-actions");
    actions.textContent = "";
    actions.appendChild(action("os-recovery-retry", "Retry", function () { closeRecovery(); }));
    actions.appendChild(action("os-recovery-safe", "Safe mode", function () { closeRecovery(); safeMode(); }));
    actions.appendChild(action("os-recovery-export", "Export diagnostics to USB", download));
    actions.appendChild(action("os-recovery-reset", "Factory reset", function () {
      window.location.href = "/settings#os-factory-reset";
    }));
    node.hidden = false;
  }
  function closeRecovery() {
    var node = id("gb-recovery-screen");
    if (node) node.hidden = true;
  }
  function safeMode() {
    document.body.classList.add("gb-safe-mode");
    // #2468 — in safe mode the shell offers Diagnostics + Network only: no
    // provisioning, bot/model or other app windows.
    if (window.WindowManager && Array.isArray(window.WindowManager.openWindows)) {
      window.WindowManager.openWindows.slice().forEach(function (w) {
        if (["os-diagnostics", "os-network"].indexOf(w.id) < 0) {
          window.WindowManager.close(w.id);
        }
      });
    }
    window.location.hash = "#os-diagnostics";
    document.dispatchEvent(new CustomEvent("gb:safe-mode", { detail: { enabled: true } }));
  }

  document.addEventListener("gb:shell-recovery", function () { recovery(); });
  function initTauri() {
    var tauri = window.GBTAURI;
    if (tauri && tauri.event && typeof tauri.event.listen === "function") {
      tauri.event.listen("gb-shell-recovery", function () { recovery(); });
    }
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initTauri);
  } else {
    initTauri();
  }

  window.GBDiagnostics = {
    download: download,
    setRemoteSupport: setRemoteSupport,
    recovery: recovery,
    safeMode: safeMode
  };
})(window, document);
