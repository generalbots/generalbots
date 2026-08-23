"use strict";

// GB Sidebar Conversations (#1188): rendering and loading of the chat
// history list in the left bar. Extracted from sidebar.js to respect the
// file size budget; sidebar.js keeps thin delegators.

window.GBSidebarConvos = window.GBSidebarConvos || {};

(function (mod) {
  var historyLoading = false;

  function authToken() {
    if (window.getGBAccessToken) {
      var t = window.getGBAccessToken();
      if (t) return t;
    }
    try {
      return (
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") ||
        ""
      );
    } catch (e) {
      return "";
    }
  }

  function authHeaders() {
    var h = {};
    var token = authToken();
    if (token) h["Authorization"] = "Bearer " + token;
    return h;
  }

  function resolveBot() {
    if (window.GBResolveActiveBot) return window.GBResolveActiveBot();
    return "";
  }

  mod.timeAgo = function (iso) {
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
  };

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
      el.classList.toggle(
        "active",
        el.getAttribute("data-session-id") === sessionId
      );
    });
  }

  mod.highlightActive = highlightActive;
  mod.currentActiveSession = currentActiveSession;

  function historyItem(session) {
    var item = document.createElement("div");
    item.className = "chat-sidebar-conv-item";
    item.setAttribute("data-session-id", session.session_id);
    item.innerHTML =
      '<div class="chat-sidebar-conv-info">' +
      '<div class="chat-sidebar-conv-name"></div>' +
      "</div>" +
      '<div class="chat-sidebar-conv-time"></div>';
    item.querySelector(".chat-sidebar-conv-name").textContent =
      session.title || "Conversation";
    item.querySelector(".chat-sidebar-conv-time").textContent = mod.timeAgo(
      session.updated_at
    );
    item.addEventListener("click", function () {
      mod.openConversation(session.session_id);
    });
    return item;
  }

  mod.renderHistory = function (sessions) {
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
  };

  function renderSignInHint() {
    var list = document.getElementById("chatConversations");
    if (!list) return;
    list.innerHTML = "";
    var hint = document.createElement("div");
    hint.className = "chat-sidebar-signin-hint";
    hint.textContent = "Sign in to see your conversations";
    hint.addEventListener("click", function () {
      window.location.href =
        (window.GB_LOGIN_URL || "/login") +
        "?redirect=" +
        encodeURIComponent(window.location.href);
    });
    list.appendChild(hint);
  }

  mod.loadHistory = function () {
    if (historyLoading) return;
    var list = document.getElementById("chatConversations");
    if (!list) return;
    historyLoading = true;
    fetch(
      "/api/chat/history/sessions?limit=30&bot_name=" +
        encodeURIComponent(resolveBot()),
      { headers: authHeaders() }
    )
      .then(function (r) {
        if (r.status === 401) {
          renderSignInHint();
          return null;
        }
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        if (data) mod.renderHistory(data.sessions || []);
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
  };

  mod.openConversation = function (sessionId) {
    if (!sessionId) return;
    if (window.openDeepLink) {
      window.openDeepLink("chat", { session: sessionId });
    }
    highlightActive(sessionId);
  };
})(window.GBSidebarConvos);
