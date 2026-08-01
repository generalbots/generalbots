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

  var authUrl = "/api/auth?bot_name=" + encodeURIComponent(botName);

  var authHeaders = {};
  var gbToken = window.getGBAccessToken ? window.getGBAccessToken() : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token"));
  if (gbToken) authHeaders["Authorization"] = "Bearer " + gbToken;

  fetch(authUrl, { headers: authHeaders })
    .then(function (response) { return response.json(); })
    .then(function (auth) {
      ChatState.currentUserId = auth.user_id;
      ChatState.currentSessionId = auth.session_id;
      ChatState.currentBotId = auth.bot_id || "default";
      ChatState.currentBotName = botName;
      try {
        localStorage.setItem(storageKey, JSON.stringify({ user_id: auth.user_id }));
      } catch (e) {}
      
      // Check bot visibility — redirect private bots to login if not authenticated
      fetch("/api/bot/config?bot_name=" + encodeURIComponent(botName))
        .then(function (r) { return r.json(); })
        .then(function (cfg) {
          var isPub = cfg.is_public === "true" || cfg.is_public === true;
          var isAuth = window.GBSecurity && window.GBSecurity.isAuthenticated && window.GBSecurity.isAuthenticated();
          if (!isPub && !isAuth) {
            window.location.href = (window.GB_LOGIN_URL || "/login") + "?redirect=" + encodeURIComponent(window.location.href);
            return;
          }
          if (isPub) window.__BOT_IS_PUBLIC__ = true;
          connectWebSocket();
        })
        .catch(function () { connectWebSocket(); });
    })
    .catch(function () {
      notify("Failed to connect to chat server", "error");
      setTimeout(proceedWithChatInit, 3000);
    });
}

function autoFocusInput() {
  setTimeout(function() {
    var input = document.getElementById("messageInput");
    if (input) input.focus();
  }, 500);
}

function setupEventHandlers() {
var form = document.getElementById("chatForm");
var input = document.getElementById("messageInput");
var sendBtn = document.getElementById("sendBtn");

if (form) {
form.onsubmit = function (e) { e.preventDefault(); sendMessage(); return false; };
}

if (input) {
// Only attach mention handlers if they exist
var mentionInputHandler = window.handleMentionInput;
var mentionKeydownHandler = window.handleMentionKeydown;

if (mentionInputHandler) {
input.addEventListener("input", mentionInputHandler);
}
if (mentionKeydownHandler) {
input.onkeydown = function (e) {
if (mentionKeydownHandler(e)) return;
if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(); }
if (e.key === "PageUp" || e.key === "PageDown") {
e.preventDefault();
var messages = document.getElementById("messages");
if (messages) {
var dir = e.key === "PageUp" ? -1 : 1;
messages.scrollBy({ top: dir * messages.clientHeight, behavior: "instant" });
}
}
};
} else {
input.onkeydown = function (e) {
if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(); }
if (e.key === "PageUp" || e.key === "PageDown") {
e.preventDefault();
var messages = document.getElementById("messages");
if (messages) {
var dir = e.key === "PageUp" ? -1 : 1;
messages.scrollBy({ top: dir * messages.clientHeight, behavior: "instant" });
}
}
};
}
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
var hideMention = window.hideMentionDropdown;
if (hideMention) {
hideMention();
}
}
});
}

function initChat() {
if (typeof loadBotConfig === 'function') {
loadBotConfig();
}
proceedWithChatInit();
autoFocusInput();

// Show signup CTA for public bots when not authenticated
var cta = document.getElementById('publicSignupCta');
if (cta && window.__BOT_IS_PUBLIC__ === true) {
  setTimeout(function() {
    var isAuth = window.GBSecurity && window.GBSecurity.isAuthenticated();
    if (!isAuth) {
      cta.style.display = 'flex';
    }
  }, 2000);
}
}

function showChatApp() {
  var chatApp = document.getElementById("chat-app");
  if (chatApp) {
    chatApp.style.opacity = "1";
    chatApp.style.visibility = "visible";
  }
}

window.showChatApp = showChatApp;

// Wait for DOM to be ready before initializing
if (typeof document !== 'undefined') {
if (document.readyState === 'loading') {
document.addEventListener('DOMContentLoaded', function() {
setupEventHandlers();
initChat();
});
} else {
setupEventHandlers();
initChat();
}
}
