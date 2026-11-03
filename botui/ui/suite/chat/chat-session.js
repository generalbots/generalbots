/* Chat session helpers: file attachment chip, session info, bot switching.
 * Split out of chat-init.js so every chat script stays under 450 lines. */
"use strict";


(function initFileButton() {
  var fileBtn = document.getElementById("fileBtn");
  var fileInput = document.getElementById("messageFile");
  var chip = document.getElementById("fileChip");
  if (!fileBtn || !fileInput || !chip) return;

  var MAX_FILE_SIZE = 25 * 1024 * 1024;

  function showFileChip(file) {
    if (file.size > MAX_FILE_SIZE) {
      notify("File too large (max 25 MB)", "warning");
      fileInput.value = "";
      return;
    }
    chip.textContent = "📎 " + file.name + " (" + formatAttachSize(file.size) + ")";
    chip.style.display = "inline-flex";
    chip.title = file.name;
  }

  fileBtn.addEventListener("click", function (e) {
    e.preventDefault();
    fileInput.click();
  });
  fileInput.addEventListener("change", function () {
    if (fileInput.files && fileInput.files[0]) showFileChip(fileInput.files[0]);
  });

  chip.addEventListener("click", function () {
    fileInput.value = "";
    chip.style.display = "none";
    chip.textContent = "";
  });

  var chatArea = document.getElementById("chatContentWrapper") || document.getElementById("chat-app");
  if (chatArea) {
    ["dragenter", "dragover", "dragleave", "drop"].forEach(function (ev) {
      chatArea.addEventListener(ev, function (e) { e.preventDefault(); e.stopPropagation(); });
    });
    chatArea.addEventListener("dragover", function () { chatArea.classList.add("gb-chat-drag"); });
    chatArea.addEventListener("dragleave", function (e) {
      if (!chatArea.contains(e.relatedTarget)) chatArea.classList.remove("gb-chat-drag");
    });
    chatArea.addEventListener("drop", function (e) {
      chatArea.classList.remove("gb-chat-drag");
      var files = e.dataTransfer && e.dataTransfer.files;
      if (files && files[0]) {
        var dt = new DataTransfer();
        dt.items.add(files[0]);
        fileInput.files = dt.files;
        showFileChip(files[0]);
      }
    });
  }

  document.addEventListener("paste", function (e) {
    if (e.target && (e.target.tagName === "TEXTAREA" || (e.target.tagName === "INPUT" && e.target.type === "text" && e.target.id !== "messageInput"))) return;
    var items = e.clipboardData && e.clipboardData.items;
    if (!items) return;
    for (var i = 0; i < items.length; i++) {
      if (items[i].kind === "file") {
        var f = items[i].getAsFile();
        if (!f) continue;
        e.preventDefault();
        var dt = new DataTransfer();
        dt.items.add(f);
        fileInput.files = dt.files;
        showFileChip(f);
        break;
      }
    }
  });
})();

window.getChatSessionInfo = function () {
  return {
    ws: ChatState.ws,
    currentBotId: ChatState.currentBotId,
    currentUserId: ChatState.currentUserId,
    currentSessionId: ChatState.currentSessionId,
    currentBotName: ChatState.currentBotName,
  };
};

// Seamless bot switch (sidebar combo): re-authenticates for the new bot and
// swaps the live session context in place — no page/app reload, the window
// stays exactly where it is.
window.ChatSwitchBot = function (botName) {
  if (!botName) return Promise.resolve();
  var headers = {};
  var tok = window.getGBAccessToken ? window.getGBAccessToken()
    : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || localStorage.getItem("management_token"));
  if (tok) headers["Authorization"] = "Bearer " + tok;

  return fetch("/api/auth?bot_name=" + encodeURIComponent(botName), { headers: headers })
    .then(function (r) { return r.json(); })
    .then(function (auth) {
      // Drop the previous socket silently — connectWebSocket() opens a fresh
      // one bound to the new bot/session pair.
      try {
        if (ChatState.ws) {
          ChatState.ws.onclose = null;
          ChatState.ws.onerror = null;
          ChatState.ws.onmessage = null;
          ChatState.ws.close();
        }
      } catch (e) {}

      ChatState.reconnectAttempts = 0;
      ChatState.currentUserId = auth.user_id;
      ChatState.currentSessionId = auth.session_id;
      ChatState.currentBotId = auth.bot_id || "default";
      ChatState.currentBotName = botName;
      try {
        localStorage.setItem("gb_chat_" + botName, JSON.stringify({ user_id: auth.user_id }));
      } catch (e) {}

      // Swap the conversation view in place (not a reload).
      hideThinkingIndicator();
      var pane = document.getElementById("messages");
      if (pane) pane.innerHTML = "";
      var sugg = document.getElementById("suggestions");
      if (sugg) sugg.innerHTML = "";

      // Re-apply the new bot's theme/colors without touching the shell.
      if (typeof loadBotConfig === "function") loadBotConfig();

      connectWebSocket();

      window.dispatchEvent(new CustomEvent("gb-chat-session-changed", {
        detail: { session_id: auth.session_id, bot_name: botName },
      }));
    })
    .catch(function () {
      notify("Failed to switch to bot " + botName, "warning");
    });
};

  // Sidebar deep-link payload (?session=… or __gbAppParams__.session) —
  // reopening an existing conversation from the desktop sidebar.
  function readRequestedSession() {
    try {
      if (window.__gbAppParams__ && window.__gbAppParams__.session) {
        var sid = String(window.__gbAppParams__.session);
        delete window.__gbAppParams__.session;
        return sid;
      }
      return new URLSearchParams(window.location.search).get("session") || "";
    } catch (e) {
      return "";
    }
  }

  // #1289 — launcher bot tiles deep-link the Chat window to a specific bot
  // (catalog entries carry deep_link_params { bot: slug }). The param wins
  // over the stored/path-derived bot ONLY inside this chat window instance,
  // letting the desktop host several bot chats side by side.
  function readRequestedBot() {
    try {
      if (window.__gbAppParams__ && window.__gbAppParams__.bot) {
        var bot = String(window.__gbAppParams__.bot);
        delete window.__gbAppParams__.bot;
        return bot;
      }
      return new URLSearchParams(window.location.search).get("bot") || "";
    } catch (e) {
      return "";
    }
  }

  // Replays a stored conversation into #messages before the WebSocket opens,
  // so the user sees prior turns when opening a conversation from history.
  function restoreLocalHistory() {
    if (typeof loadLocalHistory !== "function") return;
    var local = loadLocalHistory();
    if (!local || local.length === 0) return;
    var pane = document.getElementById("messages");
    if (pane) pane.innerHTML = "";
    local.forEach(function (m) {
      addMessage(m.sender, m.content);
    });
    scrollToBottom(false);
  }

  function loadSessionHistory(sessionId) {
    var headers = {};
    var tok = window.getGBAccessToken ? window.getGBAccessToken()
      : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || localStorage.getItem("management_token"));
    if (tok) headers["Authorization"] = "Bearer " + tok;
    return fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId) + "/messages", { headers: headers })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data || !Array.isArray(data.messages) || data.messages.length === 0) {
          restoreLocalHistory();
          return;
        }
        var pane = document.getElementById("messages");
        if (pane) pane.innerHTML = "";
        data.messages.forEach(function (m) {
          addMessage(m.role === "user" ? "user" : "bot", m.content);
        });
        scrollToBottom(false);
      })
      .catch(function () { restoreLocalHistory(); });
  }
