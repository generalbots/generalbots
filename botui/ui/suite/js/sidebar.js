"use strict";

// GB Sidebar (v16) — Claude/zo.computer style navigation rail.
// Renders the principal app links (Browser, Terminal, Drive) and the user's
// chat history below them. Clicking a history item opens the Chat app loaded
// with that conversation (deep-link via ?session= / __gbAppParams__).

(function () {
  var APP_LINK_IDS = ["chat", "browser", "terminal", "drive"];

  var FALLBACK_APPS = {
    chat: {
      id: "chat", title: "Chat", hxGet: "/suite/partials/chat.html",
      icon: '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>',
    },
    browser: {
      id: "browser", title: "Browser", hxGet: "/suite/browser/browser.html",
      icon: '<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
    },
    terminal: {
      id: "terminal", title: "Terminal", hxGet: "/suite/terminal/terminal.html",
      icon: '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>',
    },
    drive: {
      id: "drive", title: "Drive", hxGet: "/suite/drive/drive.html",
      icon: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
    },
  };

  function findApp(id) {
    var registry = window.APPS_REGISTRY || [];
    for (var i = 0; i < registry.length; i++) {
      if (registry[i].id === id && registry[i].hxGet) return registry[i];
    }
    return FALLBACK_APPS[id];
  }

  function appIcon(app) {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      (app.icon || "") + "</svg>";
  }

  // ── Auth helpers (same slots as chat-init.js / desktop taskbar) ──
  function authToken() {
    if (window.getGBAccessToken) {
      var t = window.getGBAccessToken();
      if (t) return t;
    }
    try {
      return localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") || "";
    } catch (e) {
      return "";
    }
  }

  // ── Principal app links ──
  function renderApps() {
    var nav = document.getElementById("sidebarAppsNav");
    if (!nav) return;
    nav.innerHTML = "";
    APP_LINK_IDS.forEach(function (id) {
      var app = findApp(id);
      if (!app) return;
      var link = document.createElement("div");
      link.className = "sidebar-app-link";
      link.setAttribute("data-app-id", app.id);
      link.setAttribute("title", app.title);
      link.innerHTML =
        '<span class="sidebar-app-icon">' + appIcon(app) + "</span>" +
        '<span class="sidebar-app-label"></span>';
      link.querySelector(".sidebar-app-label").textContent = app.title;
      link.addEventListener("click", function () {
        if (window.openDeepLink) window.openDeepLink(app.id, {});
      });
      nav.appendChild(link);
    });
  }

  // ── Time formatting ──
  function timeAgo(iso) {
    var then = new Date(iso).getTime();
    if (!then) return "";
    var diff = Math.max(0, Date.now() - then);
    var min = Math.floor(diff / 60000);
    if (min < 1) return "now";
    if (min < 60) return min + "m";
    var hours = Math.floor(min / 60);
    if (hours < 24) return hours + "h";
    var days = Math.floor(hours / 24);
    if (days < 7) return days + "d";
    return new Date(iso).toLocaleDateString();
  }

  // ── Chat history ──
  function historyItem(session) {
    var item = document.createElement("div");
    item.className = "chat-sidebar-conv-item";
    item.setAttribute("data-session-id", session.session_id);
    item.innerHTML =
      '<div class="chat-sidebar-conv-info">' +
      '<div class="chat-sidebar-conv-name"></div>' +
      "</div>" +
      '<div class="chat-sidebar-conv-time"></div>';
    item.querySelector(".chat-sidebar-conv-name").textContent = session.title || "Conversation";
    item.querySelector(".chat-sidebar-conv-time").textContent = timeAgo(session.updated_at);
    item.addEventListener("click", function () {
      openConversation(session.session_id);
    });
    return item;
  }

  function renderHistory(sessions) {
    var list = document.getElementById("chatConversations");
    if (!list) return;
    list.innerHTML = "";
    if (!sessions || !sessions.length) {
      var empty = document.createElement("div");
      empty.className = "chat-sidebar-history-empty";
      empty.textContent = "No conversations yet";
      list.appendChild(empty);
      return;
    }
    sessions.forEach(function (s) {
      list.appendChild(historyItem(s));
    });
    highlightActive(currentActiveSession());
  }

  function renderSignInHint() {
    var list = document.getElementById("chatConversations");
    if (!list) return;
    list.innerHTML = "";
    var hint = document.createElement("div");
    hint.className = "chat-sidebar-signin-hint";
    hint.textContent = "Sign in to see your conversations";
    hint.addEventListener("click", function () {
      window.location.href = (window.GB_LOGIN_URL || "/login") +
        "?redirect=" + encodeURIComponent(window.location.href);
    });
    list.appendChild(hint);
  }

  var historyLoading = false;

  function loadHistory() {
    if (historyLoading) return;
    var list = document.getElementById("chatConversations");
    if (!list) return;
    historyLoading = true;
    var headers = {};
    var token = authToken();
    if (token) headers["Authorization"] = "Bearer " + token;
    fetch("/api/chat/history/sessions?limit=30", { headers: headers })
      .then(function (r) {
        if (r.status === 401) { renderSignInHint(); return null; }
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        if (data) renderHistory(data.sessions || []);
      })
      .catch(function () {
        if (!list.children.length) {
          var empty = document.createElement("div");
          empty.className = "chat-sidebar-history-empty";
          empty.textContent = "History unavailable";
          list.appendChild(empty);
        }
      })
      .finally(function () {
        historyLoading = false;
      });
  }

  // ── Conversation activation ──
  function currentActiveSession() {
    if (window.ChatState && window.ChatState.currentSessionId) {
      return String(window.ChatState.currentSessionId);
    }
    return "";
  }

  function highlightActive(sessionId) {
    var list = document.getElementById("chatConversations");
    if (!list) return;
    list.querySelectorAll(".chat-sidebar-conv-item").forEach(function (el) {
      el.classList.toggle("active", el.getAttribute("data-session-id") === sessionId);
    });
  }

  function openConversation(sessionId) {
    if (!sessionId) return;
    if (window.openDeepLink) {
      window.openDeepLink("chat", { session: sessionId });
    }
    highlightActive(sessionId);
  }

  // ── New conversation ──
  function newConversation() {
    if (window.openDeepLink) {
      window.openDeepLink("chat", {});
    }
    highlightActive("");
  }

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

  var newBtn = document.getElementById("sidebarNewChatBtn");
  if (newBtn) {
    newBtn.addEventListener("click", newConversation);
  }

  // Expose for programmatic use (taskbar, collapsed icons, other apps).
  window.GBSidebar = {
    loadHistory: loadHistory,
    openConversation: openConversation,
    newConversation: newConversation,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      renderApps();
      loadHistory();
    });
  } else {
    renderApps();
    loadHistory();
  }
})();
