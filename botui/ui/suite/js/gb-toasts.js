"use strict";
/* GB Toasts (#1158): transient toast notifications + a notification center
   tray. Any module can call window.GBToasts.show(title, message, kind). */

// #2568 — idempotent load: desktop.html already loads this globally, and some
// apps (e.g. mail.html) also include it. Re-declaring the top-level const throws
// `Identifier 'GBToasts' has already been declared` and breaks the app's JS.
if (typeof window !== "undefined" && window.GBToasts) {
  // Already loaded — skip re-definition entirely.
} else {
const GBToasts = (() => {
  const STORAGE_KEY = "gb-notifications";

  function tr(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }
  let notifications = [];
  let initialized = false;

  function init() {
    if (initialized) return;
    initialized = true;
    ensureAnnouncers();
    try {
      notifications = gbStoreGet(STORAGE_KEY, []);
    } catch (e) {
      notifications = [];
    }
  }

  function ensureAnnouncers() {
    if (!document || !document.body) return null;
    var polite = document.getElementById("gb-toast-live-polite");
    if (!polite) {
      polite = document.createElement("div");
      polite.id = "gb-toast-live-polite";
      polite.className = "sr-only";
      polite.setAttribute("role", "status");
      polite.setAttribute("aria-live", "polite");
      polite.setAttribute("aria-atomic", "true");
      document.body.appendChild(polite);
    }
    var assertive = document.getElementById("gb-toast-live-assertive");
    if (!assertive) {
      assertive = document.createElement("div");
      assertive.id = "gb-toast-live-assertive";
      assertive.className = "sr-only";
      assertive.setAttribute("role", "alert");
      assertive.setAttribute("aria-live", "assertive");
      assertive.setAttribute("aria-atomic", "true");
      document.body.appendChild(assertive);
    }
    return { polite: polite, assertive: assertive };
  }

  function announceToast(title, message, kind) {
    var regions = ensureAnnouncers();
    if (!regions) return;
    var text = [title, message].filter(Boolean).join(": ").trim();
    if (!text) return;
    var target = kind === "error" ? regions.assertive : regions.polite;
    var other = kind === "error" ? regions.polite : regions.assertive;
    try { other.textContent = ""; } catch (e) { window.gbLogError ? gbLogError(e, "announceToast") : console.error("announceToast", e); }
    try {
      target.textContent = "";
      setTimeout(function () {
        try { target.textContent = text; } catch (e) { window.gbLogError ? gbLogError(e, "setTimeout") : console.error("setTimeout", e); }
        setTimeout(function () {
          try { if (target.textContent === text) target.textContent = ""; } catch (e) { window.gbLogError ? gbLogError(e, "setTimeout") : console.error("setTimeout", e); }
        }, 1500);
      }, 20);
    } catch (e) { window.gbLogError ? gbLogError(e, "setTimeout") : console.error("setTimeout", e); }
  }

  function persist() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(notifications.slice(0, 50)));
    } catch (e) { window.gbLogError ? gbLogError(e, "persist") : console.error("persist", e); }
  }

  function show(title, message, kind, options) {
    if (!initialized) init();
    kind = kind || "info";
    options = (options && typeof options === "object") ? options : {};
    const appId = options.appId || options.app || null;
    const params = options.params || {};
    const label = options.label || tr("toast-open", "Open");
    const id = "gb-toast-" + Date.now() + "-" + Math.floor(Math.random() * 1000);
    const toast = document.createElement("div");
    toast.id = id;
    toast.className = "gb-toast gb-toast-" + kind;
    if (kind === "error") {
      toast.setAttribute("role", "alert");
    } else {
      toast.setAttribute("role", "status");
    }
    var actionHtml = "";
    if (appId) {
      actionHtml = '<button class="gb-toast-action" data-app="' + esc(appId) + '">' + esc(label) + "</button>";
    }
    toast.innerHTML =
      '<div class="gb-toast-icon">' + iconFor(kind) + "</div>" +
      '<div class="gb-toast-body"><div class="gb-toast-title">' + esc(title) + '</div><div class="gb-toast-msg">' + esc(message) + "</div></div>" +
      actionHtml +
      '<button class="gb-toast-close" title="' + esc(tr("toast-dismiss", "Dismiss")) + '">✕</button>';
    document.body.appendChild(toast);
    announceToast(title, message, kind);
    const closeBtn = toast.querySelector(".gb-toast-close");
    if (closeBtn) closeBtn.addEventListener("click", () => dismiss(toast));
    const actionBtn = toast.querySelector(".gb-toast-action");
    if (actionBtn && appId) {
      actionBtn.addEventListener("click", function () {
        if (window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
          window.WindowManager.openDeepLink(appId, params);
        } else if (typeof window.openDeepLink === "function") {
          window.openDeepLink(appId, params);
        }
        dismiss(toast);
      });
    }
    setTimeout(() => dismiss(toast), 6000);

    notifications.unshift({ id: id, title: title, message: message, kind: kind, ts: Date.now(), appId: appId, params: params, label: label });
    persist();
    updateBadge();
  }

  function dismiss(toast) {
    if (toast && toast.parentNode) {
      toast.classList.add("gb-toast-out");
      setTimeout(() => toast.remove(), 250);
    }
  }

  function iconFor(kind) {
    switch (kind) {
      case "success": return "✓";
      case "warning": return "⚠";
      case "error": return "✕";
      default: return "ℹ";
    }
  }

  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  function updateBadge() {
    const badge = document.getElementById("trayNotificationsBadge");
    if (!badge) return;
    const unread = notifications.filter((n) => !n.read).length;
    if (unread > 0) {
      badge.textContent = unread > 9 ? "9+" : String(unread);
      badge.style.display = "inline-flex";
    } else {
      badge.style.display = "none";
    }
  }

  function toggleCenter() {
    const existing = document.getElementById("gb-notification-center");
    if (existing) { existing.remove(); return; }
    const center = document.createElement("div");
    center.id = "gb-notification-center";
    center.className = "gb-notification-center";
    center.innerHTML =
      '<div class="gb-nc-header"><h3>' + esc(tr("notifications-title", "Notifications")) + '</h3><button id="gbNcClear">' + esc(tr("notifications-clear", "Clear all")) + '</button></div>' +
      '<div class="gb-nc-list">' +
      (notifications.length
        ? notifications
            .slice(0, 30)
            .map((n) =>
              '<div class="gb-nc-item gb-nc-' + n.kind + '">' +
              '<div class="gb-nc-title">' + esc(n.title) + "</div>" +
              '<div class="gb-nc-msg">' + esc(n.message) + "</div>" +
              (n.appId ? '<button class="gb-nc-open" data-app="' + esc(n.appId) + '" data-params="' + esc(JSON.stringify(n.params || {})) + '">' + esc(n.label || tr("toast-open", "Open")) + "</button>" : "") +
              '<div class="gb-nc-time">' + gbFormatTime(new Date(n.ts)) + "</div>" +
              "</div>"
            )
            .join("")
         : '<div class="gb-nc-empty">' + esc(tr("notifications-empty", "No notifications")) + '</div>') +
      "</div>";
    document.body.appendChild(center);
    const clearBtn = document.getElementById("gbNcClear");
    if (clearBtn) {
      clearBtn.addEventListener("click", () => {
        notifications = [];
        persist();
        center.querySelector(".gb-nc-list").innerHTML = '<div class="gb-nc-empty">' + esc(tr("notifications-empty", "No notifications")) + '</div>';
        updateBadge();
      });
    }
    center.addEventListener("click", function (e) {
      var btn = e.target && e.target.closest ? e.target.closest(".gb-nc-open") : null;
      if (!btn) return;
      e.stopPropagation();
      var appId = btn.getAttribute("data-app");
      var params = {};
      try { params = JSON.parse(btn.getAttribute("data-params") || "{}"); } catch (_) { window.gbLogError ? gbLogError(_, "toggleCenter") : console.error("toggleCenter", _); }
      if (window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
        window.WindowManager.openDeepLink(appId, params);
      }
    });
    const dismiss = (e) => {
      if (!center.contains(e.target)) center.remove();
    };
    setTimeout(() => document.addEventListener("click", dismiss, { once: true }), 0);
    // Mark all as read.
    notifications.forEach((n) => { n.read = true; });
    persist();
    updateBadge();
  }

  window.addEventListener("localeChanged", function () {
    const center = document.getElementById("gb-notification-center");
    if (!center) return;
    const heading = center.querySelector("h3");
    const clear = document.getElementById("gbNcClear");
    if (heading) heading.textContent = tr("notifications-title", "Notifications");
    if (clear) clear.textContent = tr("notifications-clear", "Clear all");
    const empty = center.querySelector(".gb-nc-empty");
    if (empty) empty.textContent = tr("notifications-empty", "No notifications");
  });

  return { init, show, toggleCenter, list: () => notifications.slice() };
})();

window.GBToasts = GBToasts;
}