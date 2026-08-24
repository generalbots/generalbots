"use strict";
/* GB Toasts (#1158): transient toast notifications + a notification center
   tray. Any module can call window.GBToasts.show(title, message, kind). */

const GBToasts = (() => {
  const STORAGE_KEY = "gb-notifications";
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

  function show(title, message, kind) {
    kind = kind || "info";
    const id = "gb-toast-" + Date.now() + "-" + Math.floor(Math.random() * 1000);
    const toast = document.createElement("div");
    toast.id = id;
    toast.className = "gb-toast gb-toast-" + kind;
    toast.innerHTML =
      '<div class="gb-toast-icon">' + iconFor(kind) + "</div>" +
      '<div class="gb-toast-body"><div class="gb-toast-title">' + esc(title) + '</div><div class="gb-toast-msg">' + esc(message) + "</div></div>" +
      '<button class="gb-toast-close" title="Dismiss">✕</button>';
    document.body.appendChild(toast);
    const closeBtn = toast.querySelector(".gb-toast-close");
    if (closeBtn) closeBtn.addEventListener("click", () => dismiss(toast));
    setTimeout(() => dismiss(toast), 6000);

    notifications.unshift({ id: id, title: title, message: message, kind: kind, ts: Date.now() });
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
      '<div class="gb-nc-header"><h3>Notifications</h3><button id="gbNcClear">Clear all</button></div>' +
      '<div class="gb-nc-list">' +
      (notifications.length
        ? notifications
            .slice(0, 30)
            .map((n) =>
              '<div class="gb-nc-item gb-nc-' + n.kind + '">' +
              '<div class="gb-nc-title">' + esc(n.title) + "</div>" +
              '<div class="gb-nc-msg">' + esc(n.message) + "</div>" +
              '<div class="gb-nc-time">' + new Date(n.ts).toLocaleTimeString() + "</div>" +
              "</div>"
            )
            .join("")
        : '<div class="gb-nc-empty">No notifications</div>') +
      "</div>";
    document.body.appendChild(center);
    const clearBtn = document.getElementById("gbNcClear");
    if (clearBtn) {
      clearBtn.addEventListener("click", () => {
        notifications = [];
        persist();
        center.querySelector(".gb-nc-list").innerHTML = '<div class="gb-nc-empty">No notifications</div>';
        updateBadge();
      });
    }
    const dismiss = (e) => {
      if (!center.contains(e.target)) center.remove();
    };
    setTimeout(() => document.addEventListener("click", dismiss, { once: true }), 0);
    // Mark all as read.
    notifications.forEach((n) => { n.read = true; });
    persist();
    updateBadge();
  }

  return { init, show, toggleCenter, list: () => notifications.slice() };
})();

window.GBToasts = GBToasts;