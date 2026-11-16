"use strict";
// Split from sidebar.js for #2305 (450-line limit): events/init/API exposure.
(function () {
  var fns = window.__GBSidebarFns || {};
  var loadHistory = fns.loadHistory || function () {};
  var highlightActive = fns.highlightActive || function () {};
  var openConversation = fns.openConversation || function () {};
  var newConversation = fns.newConversation || function () {};
  var refreshUser = fns.refreshUser || function () {};
  window.updateLoginButton = refreshUser;

  // ── Events from the chat app ──
  window.addEventListener("gb-chat-session-changed", function (e) {
    var sid = e && e.detail && e.detail.session_id ? String(e.detail.session_id) : "";
    if (sid) highlightActive(sid);
    loadHistory();
  });

  window.addEventListener("gb-chat-message-sent", function () {
    clearTimeout(window.__gbSidebarRefreshTimer);
    window.__gbSidebarRefreshTimer = setTimeout(loadHistory, 1500);
  });

  var toggleBtn = document.querySelector(".chat-sidebar-toggle");
  if (toggleBtn) {
    toggleBtn.addEventListener("click", loadHistory);
  }

  // Delegated: the rail is re-rendered by sidebar-sections on boot and after
  // workspace changes, which replaces the button node and drops listeners
  // bound to the original instance ("New Conversation" did nothing).
  document.addEventListener("click", function (e) {
    if (e.target && e.target.closest && e.target.closest("#sidebarNewChatBtn")) {
      newConversation();
    }
  });

  // Expose for programmatic use (taskbar, collapsed icons, other apps).
  window.GBSidebar = {
    loadHistory: loadHistory,
    openConversation: openConversation,
    newConversation: newConversation,
    refreshUser: refreshUser,
  };

  // #1299 — Home app removed: the welcome-only window added no capability
  // beyond the command palette it advertised.

  // ── Init ────────────────────────────────────────────────────────
  // Sidebar visibility is toggled by the existing .chat-sidebar-toggle
  // button; no extra gripper is installed on the bar.

  function init() {
    (fns.renderApps || window.renderApps || function(){})();
    refreshUser();
    loadHistory();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
