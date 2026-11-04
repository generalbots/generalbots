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
    // Carry the real ISO timestamp so the history grouping/search module
    // can bucket by updated_at even after this node is re-rendered.
    if (session.updated_at) item.setAttribute("data-ts", session.updated_at);
    item.innerHTML =
      '<div class="chat-sidebar-conv-info">' +
      '<div class="chat-sidebar-conv-name"></div>' +
      "</div>" +
      '<div class="chat-sidebar-conv-time"></div>' +
       '<button type="button" class="chat-sidebar-conv-lock" ' +
       'aria-label="Lock conversation" title="Lock conversation">' +
       '\u{1F512}</button>' +
       '<button type="button" class="chat-sidebar-conv-rename" ' +
       'aria-label="Rename conversation" title="Rename conversation">' +
       '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" ' +
       'stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
       '<path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>' +
       '</svg></button>' +
      '<button type="button" class="chat-sidebar-conv-delete" ' +
      'aria-label="Delete conversation" title="Delete conversation">' +
       '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" ' +
       'stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
       '<polyline points="3 6 5 6 21 6"></polyline>' +
       '<path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"></path>' +
       '<path d="M10 11v6"></path><path d="M14 11v6"></path>' +
       '<path d="M9 6V4a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"></path>' +
       '</svg></button>';
    item.querySelector(".chat-sidebar-conv-name").textContent =
      session.title || "Conversation";
    item.querySelector(".chat-sidebar-conv-time").textContent = mod.timeAgo(
      session.updated_at
    );
    item.addEventListener("click", function () {
      mod.openConversation(session.session_id);
    });
    var lockBtn = item.querySelector(".chat-sidebar-conv-lock");
    if (lockBtn) {
      lockBtn.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        mod.lockConversation(session.session_id, item);
      });
    }
    var renameBtn = item.querySelector(".chat-sidebar-conv-rename");
    if (renameBtn) {
      renameBtn.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        mod.renameConversation(session.session_id, item);
      });
    }
    var remove = item.querySelector(".chat-sidebar-conv-delete");
    if (remove) {
      remove.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        mod.deleteConversation(session.session_id, item);
      });
    }
    return item;
  }

  /// Deletes the conversation, its messages and its container folder, then
  /// refreshes the rail. Used by the hover button and the context menu.
  mod.deleteConversation = function (sessionId, item) {
    if (!sessionId) return;
    if (!window.confirm("Delete this conversation and its workspace files?")) {
      return;
    }
    fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId), {
      method: "DELETE",
      headers: authHeaders(),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        if (item && item.parentNode) item.parentNode.removeChild(item);
        mod.loadHistory();
        window.dispatchEvent(
          new CustomEvent("gb-conversation-deleted", {
            detail: { sessionId: sessionId },
          })
        );
      })
      .catch(function (err) {
        window.alert("Could not delete conversation: " + err.message);
      });
  };

  mod.renameConversation = function (sessionId, item) {
    if (!sessionId) return;
    var currentName = "";
    if (item) {
      var nameEl = item.querySelector(".chat-sidebar-conv-name");
      if (nameEl) currentName = nameEl.textContent || "";
    }
    var next = window.prompt("Rename conversation:", currentName);
    if (next === null) return;
    next = String(next).trim();
    if (!next) return;
    fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId), {
      method: "PUT",
      headers: Object.assign({ "Content-Type": "application/json" }, authHeaders()),
      body: JSON.stringify({ title: next }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        if (item) {
          var nameEl = item.querySelector(".chat-sidebar-conv-name");
          if (nameEl) nameEl.textContent = next;
        }
        mod.loadHistory();
      })
      .catch(function (err) {
        window.alert("Could not rename conversation: " + err.message);
      });
  };

  /// Locks the conversation behind a 4-8 digit PIN (#1682). The item
  /// disappears from history; it is only reachable from "Locked".
  mod.lockConversation = function (sessionId) {
    if (!sessionId) return;
    var pin = window.prompt("Set a 4-8 digit PIN for this conversation:");
    if (pin === null) return;
    pin = String(pin).trim();
    if (!/^[0-9]{4,8}$/.test(pin)) {
      window.alert("PIN must be 4 to 8 digits.");
      return;
    }
    fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId) + "/lock", {
      method: "POST",
      headers: Object.assign({ "Content-Type": "application/json" }, authHeaders()),
      body: JSON.stringify({ pin: pin }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        mod.loadHistory();
      })
      .catch(function (err) {
        window.alert("Could not lock conversation: " + err.message);
      });
  };

  /// Lists locked conversations (ids and dates only — no titles) and opens one
  /// after the PIN is verified. The grant is short-lived; the conversation
  /// stays locked in storage.
  mod.showLocked = function () {
    var list = document.getElementById("chatConversations");
    if (!list) return;
    fetch(
      "/api/chat/history/sessions?locked=1&limit=50&bot_name=" +
        encodeURIComponent(resolveBot()),
      { headers: authHeaders() }
    )
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        var sessions = (data && data.sessions) || [];
        list.innerHTML = "";
        var back = document.createElement("button");
        back.type = "button";
        back.className = "chat-sidebar-conv-back";
        back.textContent = "\u2190 Back to conversations";
        back.addEventListener("click", function () {
          mod.loadHistory();
        });
        list.appendChild(back);
        if (!sessions.length) {
          var empty = document.createElement("div");
          empty.className = "chat-sidebar-history-empty";
          empty.textContent = "No locked conversations";
          list.appendChild(empty);
          return;
        }
        sessions.forEach(function (session) {
          var item = document.createElement("div");
          item.className = "chat-sidebar-conv-item locked";
          item.setAttribute("data-session-id", session.session_id);
          item.innerHTML =
            '<div class="chat-sidebar-conv-info">' +
            '<div class="chat-sidebar-conv-name">\u{1F512} Locked conversation</div>' +
            "</div>" +
            '<div class="chat-sidebar-conv-time"></div>';
          item.addEventListener("click", function () {
            mod.unlockConversation(session.session_id);
          });
          list.appendChild(item);
        });
      })
      .catch(function (err) {
        window.alert("Could not list locked conversations: " + err.message);
      });
  };

  /// Verifies the PIN, stores the short-lived grant and opens the chat tab.
  mod.unlockConversation = function (sessionId) {
    if (!sessionId) return;
    var pin = window.prompt("Enter the conversation PIN:");
    if (pin === null) return;
    fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId) + "/unlock", {
      method: "POST",
      headers: Object.assign({ "Content-Type": "application/json" }, authHeaders()),
      body: JSON.stringify({ pin: String(pin).trim() }),
    })
      .then(function (r) {
        if (r.status === 403) throw new Error("Incorrect PIN");
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        if (data && data.unlock_token) {
          sessionStorage.setItem("gb-unlock:" + sessionId, data.unlock_token);
        }
        mod.openConversation(sessionId);
      })
      .catch(function (err) {
        window.alert("Could not unlock conversation: " + err.message);
      });
  };

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
    var lockedToggle = document.createElement("button");
    lockedToggle.type = "button";
    lockedToggle.className = "chat-sidebar-conv-locked-toggle";
    lockedToggle.textContent = "\u{1F512} Locked conversations";
    lockedToggle.addEventListener("click", function (e) {
      e.preventDefault();
      mod.showLocked();
    });
    list.appendChild(lockedToggle);
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
    // Reuse the open chat window: this opens the conversation as a TAB in the
    // existing window instead of spawning a second window (#1439 regression).
    var hasChatWindow = !!(
      window.WindowManager &&
      typeof WindowManager.getWindow === "function" &&
      WindowManager.getWindow("chat")
    );
    if (hasChatWindow) {
      window.dispatchEvent(
        new CustomEvent("gb-open-history-tab", {
          detail: { sessionId: sessionId, title: "Conversation" },
        })
      );
    } else if (window.openDeepLink) {
      window.openDeepLink("chat", { session: sessionId });
    }
    highlightActive(sessionId);
  };
})(window.GBSidebarConvos);
