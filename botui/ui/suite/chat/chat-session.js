/* Chat session helpers: file attachment chip, session info, bot switching.
 * Split out of chat-init.js so every chat script stays under 450 lines. */
"use strict";


(function initFileButton() {
  var fileBtn = document.getElementById("fileBtn");
  var fileInput = document.getElementById("messageFile");
  var chip = document.getElementById("fileChip");
  if (!fileBtn || !fileInput || !chip) return;

  var MAX_FILE_SIZE = 25 * 1024 * 1024;
  // One turn can carry PDF + PNG (+ more): the server stages every entry of
  // the `files` array. The chip lists them; the limit is per file.
  fileInput.multiple = true;

  function showFileChip(files) {
    var list = Array.prototype.slice.call(files || []);
    if (!list.length) {
      chip.style.display = "none";
      chip.textContent = "";
      chip.title = "";
      return;
    }
    var tooBig = list.find(function (file) { return file.size > MAX_FILE_SIZE; });
    if (tooBig) {
      notify("File too large (max 25 MB): " + tooBig.name, "warning");
      fileInput.value = "";
      return;
    }
    var total = list.reduce(function (sum, file) { return sum + file.size; }, 0);
    chip.textContent = "📎 " + list.map(function (f) { return f.name; }).join(", ") +
      " (" + list.length + (list.length === 1 ? " file, " : " files, ") + formatAttachSize(total) + ")";
    chip.style.display = "inline-flex";
    chip.title = list.map(function (f) { return f.name; }).join("\n");
  }

  fileBtn.addEventListener("click", function (e) {
    e.preventDefault();
    fileInput.click();
  });
  fileInput.addEventListener("change", function () {
    showFileChip(fileInput.files);
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
      var dropped = e.dataTransfer && e.dataTransfer.files;
      if (dropped && dropped.length) {
        var dt = new DataTransfer();
        for (var d = 0; d < dropped.length; d++) dt.items.add(dropped[d]);
        fileInput.files = dt.files;
        showFileChip(dropped);
      }
    });
  }

  document.addEventListener("paste", function (e) {
    if (e.target && (e.target.tagName === "TEXTAREA" || (e.target.tagName === "INPUT" && e.target.type === "text" && e.target.id !== "messageInput"))) return;
    var items = e.clipboardData && e.clipboardData.items;
    if (!items) return;
    var pasted = [];
    for (var i = 0; i < items.length; i++) {
      if (items[i].kind === "file") {
        var f = items[i].getAsFile();
        if (f) pasted.push(f);
      }
    }
    if (pasted.length) {
      e.preventDefault();
      var dt = new DataTransfer();
      for (var j = 0; j < pasted.length; j++) dt.items.add(pasted[j]);
      fileInput.files = dt.files;
      showFileChip(pasted);
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
    ChatState.suppressPersist = true;
    local.forEach(function (m) {
      addMessage(m.sender, m.content);
    });
    ChatState.suppressPersist = false;
    scrollToBottom(false);
    if (typeof stickToBottom === "function") stickToBottom(2500);
  }

  function loadSessionHistory(sessionId, replace) {
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
        // A reply can land while the history request is in flight; never wipe
        // turns the reader already sees unless this is an explicit open.
        if (!replace && pane && pane.childElementCount > 0) return;
        if (pane) pane.innerHTML = "";
        ChatState.suppressPersist = true;
        var previousRole = "";
        var previousContent = "";
        data.messages.forEach(function (m) {
          var role = m.role === "user" ? "user" : "bot";
          // The pipeline can persist the same assistant turn twice; do not
          // replay both bubbles into the conversation.
          if (role === "bot" && previousRole === "bot" && previousContent === m.content) {
            return;
          }
          previousRole = role;
          previousContent = m.content;
          addMessage(role, m.content);
        });
        ChatState.suppressPersist = false;
        scrollToBottom(false);
        setTimeout(snapToBottom, 250);
        if (typeof followLiveEdge === "function") followLiveEdge(12000);
        else if (typeof stickToBottom === "function") stickToBottom(2500);
      })
      .catch(function () { restoreLocalHistory(); });
  }
