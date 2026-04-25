function sendMessage(messageContent) {
  var input = document.getElementById("messageInput");
  if (!input) return;

  var content = messageContent || input.value.trim();
  if (!content) return;

  if (ChatState.isStreaming && ChatState.streamingMessageId) {
    finalizeStreaming();
    ChatState.isStreaming = false;
  }

  if (!messageContent) {
    hideMentionDropdown();
    input.value = "";
    input.focus();
  }

  addMessage("user", content);

  if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    ChatState.ws.send(JSON.stringify({
      bot_id: ChatState.currentBotId,
      user_id: ChatState.currentUserId,
      session_id: ChatState.currentSessionId,
      channel: "web",
      content: content,
      message_type: MessageType.USER,
      active_switchers: Array.from(ChatState.activeSwitchers),
      timestamp: new Date().toISOString(),
    }));
  } else {
    notify("Not connected to server. Message not sent.", "warning");
  }
}

window.sendMessage = sendMessage;

window.getChatSessionInfo = function () {
  return {
    ws: ChatState.ws,
    currentBotId: ChatState.currentBotId,
    currentUserId: ChatState.currentUserId,
    currentSessionId: ChatState.currentSessionId,
    currentBotName: ChatState.currentBotName,
  };
};

function proceedWithChatInit() {
  var botName = window.__INITIAL_BOT_NAME__ || "default";
  var storageKey = "gb_chat_" + botName;
  var stored = {};
  try { stored = JSON.parse(localStorage.getItem(storageKey) || "{}"); } catch (e) {}

  var authUrl = "/api/auth?bot_name=" + encodeURIComponent(botName);
  if (stored.user_id) authUrl += "&user_id=" + encodeURIComponent(stored.user_id);
  if (stored.session_id) authUrl += "&session_id=" + encodeURIComponent(stored.session_id);

  fetch(authUrl)
    .then(function (response) { return response.json(); })
    .then(function (auth) {
      ChatState.currentUserId = auth.user_id;
      ChatState.currentSessionId = auth.session_id;
      ChatState.currentBotId = auth.bot_id || "default";
      ChatState.currentBotName = botName;
      try {
        localStorage.setItem(storageKey, JSON.stringify({ user_id: auth.user_id, session_id: auth.session_id }));
      } catch (e) {}
      connectWebSocket();
    })
    .catch(function () {
      notify("Failed to connect to chat server", "error");
      setTimeout(proceedWithChatInit, 3000);
    });
}

function setupEventHandlers() {
  var form = document.getElementById("chatForm");
  var input = document.getElementById("messageInput");
  var sendBtn = document.getElementById("sendBtn");

  if (form) {
    form.onsubmit = function (e) { e.preventDefault(); sendMessage(); return false; };
  }

  if (input) {
    input.addEventListener("input", handleMentionInput);
    input.onkeydown = function (e) {
      if (handleMentionKeydown(e)) return;
      if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(); }
    };
  }

  if (sendBtn) {
    sendBtn.onclick = function (e) { e.preventDefault(); sendMessage(); };
  }

  var scrollBtn = document.getElementById("scrollToBottom");
  if (scrollBtn) {
    scrollBtn.addEventListener("click", function () { scrollToBottom(true); ChatState.isUserScrolling = false; });
  }

  var messagesEl = document.getElementById("messages");
  if (messagesEl) {
    messagesEl.addEventListener("scroll", function () {
      ChatState.isUserScrolling = true;
      updateScrollButton();
      clearTimeout(messagesEl.scrollTimeout);
      messagesEl.scrollTimeout = setTimeout(function () { ChatState.isUserScrolling = false; }, 1000);
    });
  }

  document.addEventListener("click", function (e) {
    if (!e.target.closest("#mentionDropdown") && !e.target.closest("#messageInput")) {
      hideMentionDropdown();
    }
  });
}

function initChat() {
  loadBotConfig();
  proceedWithChatInit();
}

setupEventHandlers();
initChat();
