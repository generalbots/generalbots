/* Suite app — notifications panel and keyboard shortcuts (6/7). */
(function () {
  "use strict";
  var __cb = function () {
// ==========================================================================
// GBAlerts - Global Notification System
// ==========================================================================
window.GBAlerts = (function () {
  const notifications = [];
  const badge = document.getElementById("notificationsBadge");
  const list = document.getElementById("notificationsList");
  const btn = document.getElementById("notificationsBtn");
  const panel = document.getElementById("notificationsPanel");
  const clearBtn = document.getElementById("clearNotificationsBtn");

  function updateBadge() {
    if (badge) {
      if (notifications.length > 0) {
        badge.textContent =
          notifications.length > 99 ? "99+" : notifications.length;
        badge.style.display = "flex";
      } else {
        badge.style.display = "none";
      }
    }
  }

  function renderList() {
    if (!list) return;
    if (notifications.length === 0) {
      list.innerHTML =
        '<div class="notifications-empty"><span>🔔</span><p>No notifications</p></div>';
    } else {
      list.innerHTML = notifications
        .map(
          (n, i) => `
                              <div class="notification-item notification-${n.type}" data-index="${i}">
                                  <div class="notification-icon">${n.icon || "📢"}</div>
                                  <div class="notification-content">
                                      <div class="notification-title">${n.title}</div>
                                      <div class="notification-message">${n.message || ""}</div>
                                      <div class="notification-time">${n.time}</div>
                                  </div>
                                  ${n.action ? `<button class="notification-action" onclick="GBAlerts.handleAction(${i})">${n.actionText || "Open"}</button>` : ""}
                                  <button class="notification-dismiss" onclick="GBAlerts.dismiss(${i})">×</button>
                              </div>
                          `,
        )
        .join("");
    }
  }

  function add(notification) {
    notifications.unshift({
      ...notification,
      time: new Date().toLocaleTimeString(),
    });
    updateBadge();
    renderList();
    // Auto-open panel when new notification arrives
    if (panel) {
      panel.classList.add("show");
      if (btn) btn.setAttribute("aria-expanded", "true");
    }
  }

  function dismiss(index) {
    notifications.splice(index, 1);
    updateBadge();
    renderList();
  }

  function clearAll() {
    notifications.length = 0;
    updateBadge();
    renderList();
  }

  function handleAction(index) {
    const n = notifications[index];
    if (n && n.action) {
      if (typeof n.action === "function") {
        n.action();
      } else if (typeof n.action === "string") {
        window.open(n.action, "_blank");
      }
    }
    dismiss(index);
  }

  // Toggle panel
  if (btn && panel) {
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      const isOpen = panel.classList.toggle("show");
      btn.setAttribute("aria-expanded", isOpen);
    });

    document.addEventListener("click", (e) => {
      if (!panel.contains(e.target) && !btn.contains(e.target)) {
        panel.classList.remove("show");
        btn.setAttribute("aria-expanded", "false");
      }
    });
  }

  if (clearBtn) {
    clearBtn.addEventListener("click", clearAll);
  }

  // Convenience methods for common notifications
  return {
    add,
    dismiss,
    clearAll,
    handleAction,
    taskCompleted: function (title, url) {
      add({
        type: "success",
        icon: "✅",
        title: "Task Completed",
        message: title,
        action: url,
        actionText: "Open App",
      });
    },
    taskFailed: function (title, error) {
      add({
        type: "error",
        icon: "❌",
        title: "Task Failed",
        message: title + (error ? ": " + error : ""),
      });
    },
    info: function (title, message) {
      add({
        type: "info",
        icon: "ℹ️",
        title: title,
        message: message,
      });
    },
    warning: function (title, message) {
      add({
        type: "warning",
        icon: "⚠️",
        title: title,
        message: message,
      });
    },
    connectionStatus: function (status, message) {
      add({
        type:
          status === "connected"
            ? "success"
            : status === "disconnected"
              ? "error"
              : "warning",
        icon:
          status === "connected"
            ? "🟢"
            : status === "disconnected"
              ? "🔴"
              : "🟡",
        title:
          "Connection " + status.charAt(0).toUpperCase() + status.slice(1),
        message: message || "",
      });
    },
  };
})();

    var appsBtn = document.getElementById("appsButton");
// Keyboard shortcuts
document.addEventListener("keydown", (e) => {
  // Alt + number for quick app switching
  if (e.altKey && !e.ctrlKey && !e.shiftKey) {
    const num = parseInt(e.key);
    if (num >= 1 && num <= 9) {
      const items = document.querySelectorAll(".app-item");
      if (items[num - 1]) {
        items[num - 1].click();
        e.preventDefault();
      }
    }
  }

  // Alt + A to open apps menu
  if (e.altKey && e.key.toLowerCase() === "a") {
    appsBtn.click();
    e.preventDefault();
  }
});
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", __cb);
  } else {
    __cb();
  }
})();
