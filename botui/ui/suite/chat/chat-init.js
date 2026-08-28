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

  var selectedMentions = Array.isArray(ChatState.selectedMentions) ? ChatState.selectedMentions : [];
  ChatState.selectedMentions = [];

  var payload = {
    bot_id: ChatState.currentBotId,
    user_id: ChatState.currentUserId,
    session_id: ChatState.currentSessionId,
    channel: "web",
    content: content,
    message_type: MessageType.USER,
    active_switchers: Array.from(ChatState.activeSwitchers),
    mentions: selectedMentions.map(function (mention) {
      return {
        kind: mention.kind,
        id: mention.id,
        label: mention.name,
        project_id: mention.project_id || null,
      };
    }),
    timestamp: new Date().toISOString(),
  };

  var projectMention = selectedMentions.find(function (mention) {
    return mention.kind === "project" && mention.id;
  });
  // Also resolve a manually typed `@calculator` when its project list was
  // already loaded by the mention picker.
  if (!projectMention && Array.isArray(ChatState.projectCatalog)) {
    var directMatch = content.match(/(?:^|\s)@([A-Za-z0-9][A-Za-z0-9_.-]*)/);
    if (directMatch) {
      var directName = directMatch[1].toLowerCase();
      var directProject = ChatState.projectCatalog.find(function (project) {
        return String(project.name || "").toLowerCase() === directName;
      });
      if (directProject) {
        projectMention = {
          id: directProject.id || directProject.project_id,
          name: directProject.name,
        };
      }
    }
  }
  if (projectMention) {
    payload.project_context = {
      project_id: String(projectMention.id),
      project_name: String(projectMention.name || ""),
    };
    // Keep flat aliases for older server versions; the nested object is the
    // canonical field used by the current chat pipeline.
    payload.project_id = payload.project_context.project_id;
    payload.project_name = payload.project_context.project_name;
  }

  var finalSend = function () {
    ChatState.ws.send(JSON.stringify(payload));
    if (fileInput) { fileInput.value = ""; }
    var chip = document.getElementById("fileChip");
    if (chip) { chip.style.display = "none"; chip.textContent = ""; }
    window.dispatchEvent(new CustomEvent("gb-chat-message-sent", {
      detail: { session_id: ChatState.currentSessionId },
    }));
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

  // Replays a stored conversation into #messages before the WebSocket opens,
  // so the user sees prior turns when opening a conversation from history.
  function loadSessionHistory(sessionId) {
    var headers = {};
    var tok = window.getGBAccessToken ? window.getGBAccessToken()
      : (localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || localStorage.getItem("management_token"));
    if (tok) headers["Authorization"] = "Bearer " + tok;
    return fetch("/api/chat/history/sessions/" + encodeURIComponent(sessionId) + "/messages", { headers: headers })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data || !Array.isArray(data.messages)) return;
        var pane = document.getElementById("messages");
        if (pane) pane.innerHTML = "";
        data.messages.forEach(function (m) {
          addMessage(m.role === "user" ? "user" : "bot", m.content);
        });
        scrollToBottom(false);
      })
      .catch(function () { /* history replay is best-effort */ });
  }

function proceedWithChatInit() {
  // Re-entry guard (deep-link relaunch): drop any previous socket so two
  // WebSockets never fight over the same chat window.
  try {
    if (ChatState.ws) {
      ChatState.ws.onclose = null;
      ChatState.ws.onerror = null;
      ChatState.ws.onmessage = null;
      ChatState.ws.close();
    }
  } catch (e) {}

  var botName = (typeof window.GBResolveActiveBot === "function")
    ? window.GBResolveActiveBot()
    : (window.__INITIAL_BOT_NAME__ || "default");
  var storageKey = "gb_chat_" + botName;
  var requestedSession = readRequestedSession();

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
  // visible here. Consume the param and persist for this origin, promoting
  // it into every auth slot so the freshly acquired credential wins over
  // any stale session token from a previous login.
  try {
    var urlTok = new URLSearchParams(window.location.search).get("token");
    if (urlTok) {
      localStorage.setItem("management_token", urlTok);
      localStorage.setItem("gb-access-token", urlTok);
      sessionStorage.setItem("gb-access-token", urlTok);
      if (window.GBSecurity && window.GBSecurity.setTokens) {
        window.GBSecurity.setTokens(urlTok, sessionStorage.getItem("gb-refresh-token"), null, true);
      }
      var u = new URL(window.location.href);
      u.search = "";
      window.history.replaceState({}, "", u);
    }
  } catch (e) {}

  var authUrl = "/api/auth?bot_name=" + encodeURIComponent(botName);
  if (requestedSession) {
    authUrl += "&session_id=" + encodeURIComponent(requestedSession);
  }

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

      window.dispatchEvent(new CustomEvent("gb-chat-session-changed", {
        detail: { session_id: auth.session_id },
      }));

      var readyToConnect = requestedSession
        ? loadSessionHistory(requestedSession)
        : Promise.resolve();
      readyToConnect.then(function () {

      // Check bot visibility — redirect private bots to login if not authenticated
      fetch("/api/bot/public?bot_name=" + encodeURIComponent(botName))
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

      });
    })
    .catch(function () {
      notify("Failed to connect to chat server", "error");
      setTimeout(proceedWithChatInit, 3000);
    });
}

// #1271 — a deep link can carry a pre-filled chat message (e.g. the vibe
// Chat button sends a botbook directive to route the fresh conversation to
// the app running in vibe). Apply it once, then consume the params so a
// later reopen does not re-inject it.
function applyDeepLinkMessage() {
  try {
    var msg = "";
    if (window.__gbAppParams__ && window.__gbAppParams__.message) {
      msg = String(window.__gbAppParams__.message);
      delete window.__gbAppParams__.message;
    } else {
      msg = new URLSearchParams(window.location.search).get("message") || "";
    }
    if (!msg) return;
    var input = document.getElementById("messageInput");
    if (input) {
      input.value = msg;
      input.focus();
    }
  } catch (e) { /* non-fatal */ }
}

// Apply a deep-link message ROBUSTLY. initChat()'s autoFocusInput timer
// does not fire in every injection context (the window body can be
// re-injected after the timer is scheduled), and a re-targeted chat window
// (gb:deep-link on an already-open window) never applied the message at all.
// So: try immediately, retry briefly until the input carries the value, and
// re-apply whenever the shell re-targets this chat window with new params.
(function ensureDeepLinkApplies() {
  var applied = false;
  function tryApply() {
    if (applied) return;
    var input = document.getElementById("messageInput");
    if (!input) return;
    var msg = "";
    try {
      if (window.__gbAppParams__ && window.__gbAppParams__.message) {
        msg = String(window.__gbAppParams__.message);
      } else {
        msg = new URLSearchParams(window.location.search).get("message") || "";
      }
    } catch (e) { msg = ""; }
    if (!msg) { applied = true; return; }
    input.value = msg;
    input.focus();
    if (window.__gbAppParams__) delete window.__gbAppParams__.message;
    applied = true;
  }
  var tries = 0;
  var iv = setInterval(function () {
    tryApply();
    if (applied || ++tries > 20) clearInterval(iv);
  }, 150);
  document.addEventListener("gb:deep-link", function (e) {
    var detail = e.detail || {};
    if (String(detail.appId) === "chat") {
      applied = false;
      tryApply();
    }
  });
  tryApply();
})();

function autoFocusInput() {
  setTimeout(function() {
    var input = document.getElementById("messageInput");
    if (input) input.focus();
    applyDeepLinkMessage();
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
  if (window.GBAppLifecycle) GBAppLifecycle.begin("chat");
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
