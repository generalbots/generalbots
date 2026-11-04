/* #1682 — locked conversations: sidebar actions split from sidebar-convos.js
   to keep both modules inside the repository size limit. Loaded right after
   the main module; shares its namespace and helpers. */
"use strict";

(function (mod) {
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
      headers: Object.assign({ "Content-Type": "application/json" }, mod.authHeaders()),
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
        encodeURIComponent(mod.resolveBot()),
      { headers: mod.authHeaders() }
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
      headers: Object.assign({ "Content-Type": "application/json" }, mod.authHeaders()),
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
})(window.GBSidebarConvos = window.GBSidebarConvos || {});
