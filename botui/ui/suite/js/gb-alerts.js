"use strict";
/* GBAlerts compatibility layer (#2308).
 *
 * Live modules call window.GBAlerts as a notification fallback. The suite shell
 * does not ship the old omnibox/suite-app UI, so this module provides a safe,
 * always-loadable contract that bridges to GBToasts (and to the bell/notification
 * panel elements when they exist) without depending on orphaned DOM.
 */
(function () {
  if (window.GBAlerts && window.GBAlerts.__gbCompat) return;

  var notifications = [];

  function nowTime() {
    try {
      if (typeof gbFormatTime === "function") return gbFormatTime(new Date(), { hour: "2-digit", minute: "2-digit" });
    } catch (e) {}
    try {
      return new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" }).format(new Date());
    } catch (e2) {
      var d = new Date();
      return ("0" + d.getHours()).slice(-2) + ":" + ("0" + d.getMinutes()).slice(-2);
    }
  }

  function badgeEl() { return document.getElementById("notificationsBadge"); }
  function listEl() { return document.getElementById("notificationsList"); }
  function panelEl() { return document.getElementById("notificationsPanel"); }
  function btnEl() { return document.getElementById("notificationsBtn"); }

  function updateBadge() {
    var badge = badgeEl();
    if (!badge) return;
    if (notifications.length > 0) {
      badge.textContent = notifications.length > 99 ? "99+" : String(notifications.length);
      badge.style.display = "flex";
    } else {
      badge.style.display = "none";
    }
  }

  function renderList() {
    var list = listEl();
    if (!list) return;
    if (notifications.length === 0) {
      list.innerHTML = '<div class="notifications-empty"><span>🔔</span><p>No notifications</p></div>';
      return;
    }
    list.innerHTML = notifications.map(function (n, i) {
      var action = n.action ? '<button class="notification-action" onclick="GBAlerts.handleAction(' + i + ')">' + (n.actionText || "Open") + '</button>' : "";
      return '<div class="notification-item notification-' + (n.type || "info") + '" data-index="' + i + '">' +
        '<div class="notification-icon">' + (n.icon || "📢") + '</div>' +
        '<div class="notification-content">' +
          '<div class="notification-title">' + (n.title || "") + '</div>' +
          '<div class="notification-message">' + (n.message || "") + '</div>' +
          '<div class="notification-time">' + (n.time || "") + '</div>' +
        '</div>' + action +
        '<button class="notification-dismiss" onclick="GBAlerts.dismiss(' + i + ')">×</button>' +
      '</div>';
    }).join("");
  }

  function openPanel() {
    var panel = panelEl();
    var btn = btnEl();
    if (panel) panel.classList.add("show");
    if (btn) btn.setAttribute("aria-expanded", "true");
  }

  function toastKind(type) {
    if (type === "error") return "error";
    if (type === "warning") return "warning";
    if (type === "success") return "success";
    return "info";
  }

  function add(notification) {
    notification = notification || {};
    var item = {
      type: notification.type || "info",
      icon: notification.icon || "📢",
      title: notification.title || "",
      message: notification.message || "",
      action: notification.action,
      actionText: notification.actionText,
      time: nowTime(),
    };
    notifications.unshift(item);
    updateBadge();
    renderList();
    if (panelEl() && btnEl()) openPanel();
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      try { window.GBToasts.show(item.title, item.message, toastKind(item.type)); } catch (e) {}
    }
    return item;
  }

  function dismiss(index) {
    if (index < 0 || index >= notifications.length) return;
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
    var n = notifications[index];
    if (n && n.action) {
      try {
        if (typeof n.action === "function") n.action();
        else if (typeof n.action === "string") { var w = window.open(n.action, "_blank", "noopener,noreferrer"); if (w) { try { w.opener = null; } catch (e) {} } }
      } catch (e) {}
    }
    dismiss(index);
  }

  function bindPanel() {
    var btn = btnEl();
    var panel = panelEl();
    var clearBtn = document.getElementById("clearNotificationsBtn");
    if (btn && panel && !btn.__gbAlertsBound) {
      btn.__gbAlertsBound = true;
      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        var isOpen = panel.classList.toggle("show");
        btn.setAttribute("aria-expanded", isOpen ? "true" : "false");
      });
      document.addEventListener("click", function (e) {
        if (!panel.contains(e.target) && !btn.contains(e.target)) {
          panel.classList.remove("show");
          btn.setAttribute("aria-expanded", "false");
        }
      });
    }
    if (clearBtn && !clearBtn.__gbAlertsBound) {
      clearBtn.__gbAlertsBound = true;
      clearBtn.addEventListener("click", clearAll);
    }
    renderList();
    updateBadge();
  }

  window.GBAlerts = {
    __gbCompat: true,
    add: add,
    dismiss: dismiss,
    clearAll: clearAll,
    handleAction: handleAction,
    taskCompleted: function (title, url) {
      add({ type: "success", icon: "✅", title: "Task Completed", message: title || "", action: url, actionText: "Open App" });
    },
    taskFailed: function (title, error) {
      add({ type: "error", icon: "❌", title: "Task Failed", message: (title || "") + (error ? ": " + error : "") });
    },
    info: function (title, message) {
      add({ type: "info", icon: "ℹ️", title: title || "", message: message || "" });
    },
    warning: function (title, message) {
      add({ type: "warning", icon: "⚠️", title: title || "", message: message || "" });
    },
    error: function (title, message) {
      add({ type: "error", icon: "❌", title: title || "", message: message || "" });
    },
    success: function (title, message) {
      add({ type: "success", icon: "✅", title: title || "", message: message || "" });
    },
    connectionStatus: function (status, message) {
      var connected = status === "connected";
      var disconnected = status === "disconnected";
      add({
        type: connected ? "success" : disconnected ? "error" : "warning",
        icon: connected ? "🟢" : disconnected ? "🔴" : "🟡",
        title: "Connection " + (status ? status.charAt(0).toUpperCase() + status.slice(1) : ""),
        message: message || "",
      });
    },
  };

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bindPanel);
  else bindPanel();
})();
