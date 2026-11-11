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

  // The <label> wraps #messageFile, so the browser already forwards its click
  // to the input. Calling click() again here re-entered this same handler from
  // the input's bubbled event; Chrome suppressed that second request and the
  // picker never opened, which made the attach button look dead (#1691).
  fileBtn.addEventListener("click", function (e) {
    if (e.target === fileInput) return;
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
    if (
      e.target &&
      e.target.id !== "messageInput" &&
      (e.target.tagName === "TEXTAREA" ||
        (e.target.tagName === "INPUT" && e.target.type === "text"))
    ) return;
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
      addMessage(m.sender, m.content, null, null, m.ts);
    });
    ChatState.suppressPersist = false;
    scrollToBottom(false);
    if (typeof scheduleScrollbarMarkers === "function") scheduleScrollbarMarkers();
    if (typeof stickToBottom === "function") stickToBottom(2500);
  }

  var HISTORY_PAGE_TURNS = 500;

  function fetchSessionHistory(sessionId, offsetTurns) {
    var headers = {};
    var tok = window.getGBAccessToken ? window.getGBAccessToken()
      : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || localStorage.getItem("management_token"));
    if (tok) headers["Authorization"] = "Bearer " + tok;
    var url = "/api/chat/history/sessions/" + encodeURIComponent(sessionId) +
      "/messages?limit=" + HISTORY_PAGE_TURNS + "&offset=" + (offsetTurns || 0);
    var grant = null;
    try { grant = sessionStorage.getItem("gb-unlock:" + sessionId); } catch (e) {}
    if (grant) url += "&unlock_token=" + encodeURIComponent(grant);
    return fetch(url, { headers: headers });
  }

  function renderHistoryMessages(messages, pane) {
    var previousRole = "";
    var previousContent = "";
    (messages || []).forEach(function (m) {
      var role = m.role === "user" ? "user" : "bot";
      // The pipeline can persist the same assistant turn twice; do not
      // replay both bubbles into the conversation.
      if (role === "bot" && previousRole === "bot" && previousContent === m.content) {
        return;
      }
      previousRole = role;
      previousContent = m.content;
      // Server history does not expose timestamps yet; pass one through when
      // a future payload includes it, otherwise the bubble omits the label.
      addMessage(role, m.content, null, null, m.ts || m.timestamp || m.created_at || m.time);
    });
  }

  function removeShowMoreControl() {
    var existing = document.getElementById("gbShowMoreMessages");
    if (existing && existing.parentNode) existing.parentNode.removeChild(existing);
  }

  function insertShowMoreControl(sessionId) {
    removeShowMoreControl();
    if (!ChatState.historyHasMore) return;
    var pane = document.getElementById("messages");
    if (!pane) return;
    var btn = document.createElement("button");
    btn.id = "gbShowMoreMessages";
    btn.type = "button";
    btn.className = "gb-show-more-messages";
    btn.textContent = "show more…";
    btn.style.cssText = "display:block;margin:8px auto;padding:6px 14px;border:1px solid #cbd5e1;border-radius:8px;background:#f8fafc;color:#334155;cursor:pointer;font-size:13px;";
    btn.addEventListener("click", function () {
      loadMoreHistory(sessionId);
    });
    pane.insertBefore(btn, pane.firstChild);
  }

  function loadMoreHistory(sessionId) {
    if (ChatState.historyLoadingMore) return;
    ChatState.historyLoadingMore = true;
    var btn = document.getElementById("gbShowMoreMessages");
    if (btn) { btn.disabled = true; btn.textContent = "Loading…"; }
    var pane = document.getElementById("messages");
    var prevHeight = pane ? pane.scrollHeight : 0;
    var prevTop = pane ? pane.scrollTop : 0;
    fetchSessionHistory(sessionId, ChatState.historyOffsetTurns || 0)
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        ChatState.historyLoadingMore = false;
        if (!data || !Array.isArray(data.messages) || data.messages.length === 0) {
          ChatState.historyHasMore = false;
          removeShowMoreControl();
          return;
        }
        // Render older messages into a temporary container, then move them
        // to the top of the real pane so scroll position can be restored.
        var realPane = document.getElementById("messages");
        if (realPane) realPane.id = "gbMessagesLive";
        var tempPane = document.createElement("div");
        tempPane.id = "messages";
        tempPane.style.display = "none";
        document.body.appendChild(tempPane);
        ChatState.suppressPersist = true;
        renderHistoryMessages(data.messages, tempPane);
        ChatState.suppressPersist = false;
        if (realPane) {
          while (tempPane.firstChild) {
            realPane.insertBefore(tempPane.firstChild, realPane.firstChild);
          }
          realPane.id = "messages";
        }
        tempPane.remove();
        if (pane) {
          pane.scrollTop = pane.scrollHeight - prevHeight + prevTop;
        }
        ChatState.historyOffsetTurns = (ChatState.historyOffsetTurns || 0) + HISTORY_PAGE_TURNS;
        ChatState.historyHasMore = (data.has_more === true) ||
          (data.has_more === undefined && data.messages.length >= HISTORY_PAGE_TURNS * 2);
        removeShowMoreControl();
        insertShowMoreControl(sessionId);
        if (typeof hydrateVmAttachments === "function") {
          try { hydrateVmAttachments(document.getElementById("messages")); } catch (e) {}
        }
      })
      .catch(function () {
        ChatState.historyLoadingMore = false;
        if (btn) { btn.disabled = false; btn.textContent = "show more…"; }
      });
  }
  window.loadMoreHistory = loadMoreHistory;

  function loadSessionHistory(sessionId, replace) {
    ChatState.historySessionId = sessionId;
    ChatState.historyOffsetTurns = 0;
    ChatState.historyHasMore = false;
    ChatState.historyLoadingMore = false;
    var headers = {};
    var tok = window.getGBAccessToken ? window.getGBAccessToken()
      : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || localStorage.getItem("management_token"));
    if (tok) headers["Authorization"] = "Bearer " + tok;
    return fetchSessionHistory(sessionId, 0)
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
        renderHistoryMessages(data.messages, pane);
        ChatState.suppressPersist = false;
        ChatState.historyOffsetTurns = HISTORY_PAGE_TURNS;
        ChatState.historyHasMore = (data.has_more === true) ||
          (data.has_more === undefined && data.messages.length >= HISTORY_PAGE_TURNS * 2);
        scrollToBottom(false);
        setTimeout(snapToBottom, 250);
        if (typeof scheduleScrollbarMarkers === "function") scheduleScrollbarMarkers();
        if (typeof followLiveEdge === "function") followLiveEdge(3000);
        else if (typeof stickToBottom === "function") stickToBottom(2500);
        insertShowMoreControl(sessionId);
      })
      .catch(function () { restoreLocalHistory(); });
  }
