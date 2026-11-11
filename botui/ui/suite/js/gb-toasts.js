"use strict";
/* GB Toasts (#1158): transient toast notifications + a notification center
   tray. Any module can call window.GBToasts.show(title, message, kind). */

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
    try {
      notifications = JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
    } catch (e) {
      notifications = [];
    }
  }

  function persist() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(notifications.slice(0, 50)));
    } catch (e) {}
  }

  function show(title, message, kind, options) {
    kind = kind || "info";
    options = (options && typeof options === "object") ? options : {};
    const appId = options.appId || options.app || null;
    const params = options.params || {};
    const label = options.label || tr("toast-open", "Open");
    const id = "gb-toast-" + Date.now() + "-" + Math.floor(Math.random() * 1000);
    const toast = document.createElement("div");
    toast.id = id;
    toast.className = "gb-toast gb-toast-" + kind;
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
              '<div class="gb-nc-time">' + new Date(n.ts).toLocaleTimeString() + "</div>" +
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
      try { params = JSON.parse(btn.getAttribute("data-params") || "{}"); } catch (_) {}
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