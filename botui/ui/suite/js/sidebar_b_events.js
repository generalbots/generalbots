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

  // Delegated: sidebar-sections replaces the rail button on boot/swap.
  document.addEventListener("click", function (e) {
    if (e.target && e.target.closest && e.target.closest("#sidebarNewChatBtn")) {
      newConversation();
    }
  });

  // Programmatic API.
  window.GBSidebar = {
    loadHistory: loadHistory,
    openConversation: openConversation,
    newConversation: newConversation,
    refreshUser: refreshUser,
  };

  // #1299: Home app retired.

  // ── Init ──
  // #2933 — render the rail on idle, off the DOMContentLoaded path.
  function deferIdle(fn) {
    if (typeof window.requestIdleCallback === "function") return window.requestIdleCallback(fn, { timeout: 800 });
    return window.setTimeout(fn, 1);
  }

  function init() {
    deferIdle(fns.renderApps || window.renderApps || function () {});
    refreshUser();
    loadHistory();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
