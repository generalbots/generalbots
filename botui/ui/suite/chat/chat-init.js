function sendMessage(messageContent) {
  var input = document.getElementById("messageInput");
  if (!input) return;

  var content = messageContent || input.value.trim();

  var fileInput = document.getElementById("messageFile");
  var file = fileInput && fileInput.files && fileInput.files[0];

  if (!content && !file) return;

  if (ChatState.isStreaming && ChatState.streamingMessageId) {
    finalizeStreaming();
    ChatState.isStreaming = false;
  }

  if (!messageContent) {
    hideMentionDropdown();
    input.value = "";
    input.focus();
  }

  addMessage("user", content || ("📎 " + file.name));

  var payload = {
    bot_id: ChatState.currentBotId,
    user_id: ChatState.currentUserId,
    session_id: ChatState.currentSessionId,
    channel: "web",
    content: content,
    message_type: MessageType.USER,
    active_switchers: Array.from(ChatState.activeSwitchers),
    timestamp: new Date().toISOString(),
  };

  var finalSend = function () {
    ChatState.ws.send(JSON.stringify(payload));
    if (fileInput) { fileInput.value = ""; }
    var chip = document.getElementById("fileChip");
    if (chip) { chip.style.display = "none"; chip.textContent = ""; }
  };

  if (file && ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    var reader = new FileReader();
    reader.onload = function () {
      payload.file = {
        name: file.name,
        content_base64: String(reader.result).split(",")[1] || reader.result,
      };
      finalSend();
    };
    reader.readAsDataURL(file);
  } else if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    finalSend();
  } else {
    notify("Not connected to server. Message not sent.", "warning");
  }
}

window.sendMessage = sendMessage;

(function initFileButton() {
  var fileBtn = document.getElementById("fileBtn");
  var fileInput = document.getElementById("messageFile");
  var chip = document.getElementById("fileChip");
  if (!fileBtn || !fileInput || !chip) return;
  fileBtn.addEventListener("click", function (e) {
    e.preventDefault();
    fileInput.click();
  });
  fileInput.addEventListener("change", function () {
    if (fileInput.files && fileInput.files[0]) {
      chip.textContent = "📎 " + fileInput.files[0].name;
      chip.style.display = "inline-flex";
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

function proceedWithChatInit() {
  var botName = window.__INITIAL_BOT_NAME__ || "default";
  var storageKey = "gb_chat_" + botName;

  // #753 — cross-surface deeplinks: ?vibe= / ?run_id= open the Vibe app
  // (desktop shell) or mark the chat tab for the Vibe surface.
  try {
    var qs = new URLSearchParams(window.location.search);
    if (qs.get("vibe") !== null || qs.get("run_id") !== null) {
      if (window.VibeB) {
        window.VibeB.open({
          project: qs.get("vibe") || "",
          run_id: qs.get("run_id") || ""
        });
      } else {
        sessionStorage.setItem("gb_vibe_deeplink", window.location.search);
      }
    }
  } catch (e) { /* non-fatal */ }

  // Capture auth token passed back from the login server (?token=...) —
  // localStorage is origin-scoped, so the login domain's storage is not
  // visible here. Consume the param and persist for this origin.
  try {
    var urlTok = new URLSearchParams(window.location.search).get("token");
    if (urlTok) {
      localStorage.setItem("management_token", urlTok);
      var u = new URL(window.location.href);
      u.search = "";
      window.history.replaceState({}, "", u);
    }
  } catch (e) {}

  var authUrl = "/api/auth?bot_name=" + encodeURIComponent(botName);

  var authHeaders = {};
  var gbToken = window.getGBAccessToken ? window.getGBAccessToken() : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || localStorage.getItem("management_token"));
  if (gbToken) authHeaders["Authorization"] = "Bearer " + gbToken;

  fetch(authUrl, { headers: authHeaders })
    .then(function (response) { return response.json(); })
    .then(function (auth) {
      // Stale or revoked suite token: the server resolved us as anonymous even
      // though we sent a Bearer token. Self-heal by dropping the dead token and
      // going through the login server again (redirect round-trip re-mints a
      // valid suite session). Without this, all cookie-carrying apps (CRM,
      // Drive, ...) silently scope to an empty anonymous branch.
      if (!auth.is_authenticated && auth.status !== "authenticated" && gbToken) {
        try {
          localStorage.removeItem("gb-access-token");
          sessionStorage.removeItem("gb-access-token");
        } catch (e) {}
        var loginUrl = window.GB_LOGIN_URL || "/login";
        window.location.href = loginUrl + "?redirect=" + encodeURIComponent(window.location.href);
        return;
      }
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
(function(){ var __cb = function() {
setupEventHandlers();
initChat();
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
} else {
setupEventHandlers();
initChat();
}
}
