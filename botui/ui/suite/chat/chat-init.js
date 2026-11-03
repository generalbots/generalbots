/* Chat initialisation: deep link, event handlers, model picker, boot. */
"use strict";

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
  // #1289 — a launcher deep-link (bot param) retargets THIS chat window to
  // the requested bot; the desktop-wide selected bot is left untouched.
  var requestedBot = readRequestedBot();
  if (requestedBot && requestedBot !== botName) {
    botName = requestedBot;
    ChatState.currentBotName = botName;
  }
  var storageKey = "gb_chat_" + botName;
  var requestedSession = readRequestedSession();

  // #753 — cross-surface deeplinks: ?vibe= / ?run_id= open the Vibe app
  // (desktop shell) or mark the chat tab for the Vibe surface.
  try {
    var qs = new URLSearchParams(window.location.search);
    if (qs.get("vibe") !== null || qs.get("run_id") !== null) {
      // Respect a remembered close (gb.vibe.closed=1) for the bare ?vibe
      // case: a reload must not pop the bar back open once the user closed
      // it. Explicit project/run deep links carry intent and still open.
      var vibeBare = !qs.get("vibe") && !qs.get("run_id");
      var vibeClosed = false;
      try {
        vibeClosed = vibeBare &&
          localStorage.getItem("gb.vibe.closed") === "1";
      } catch (e2) { /* storage unavailable — default to opening */ }
      if (!vibeClosed) {
        if (window.VibeB) {
          window.VibeB.open({
            project: qs.get("vibe") || "",
            run_id: qs.get("run_id") || ""
          });
        } else {
          sessionStorage.setItem("gb_vibe_deeplink", window.location.search);
        }
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
      if (!auth.is_authenticated && auth.status !== "authenticated" && gbToken && !window.__BOT_IS_PUBLIC__) {
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
// BBSIA: grow the composer with its content, never past 180px.
var autoGrow = function () {
  if (input.tagName !== "TEXTAREA") return;
  input.style.height = "auto";
  input.style.height = Math.min(input.scrollHeight, 180) + "px";
};
input.addEventListener("input", autoGrow);

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

// #1549: remember the selected model tier across reloads.
var modelPicker = document.getElementById("modelPicker");
if (modelPicker) {
var MODEL_KEY = "gb-model-preference";
try {
var savedModel = localStorage.getItem(MODEL_KEY);
if (savedModel) modelPicker.value = savedModel;
} catch (e) {}
modelPicker.addEventListener("change", function () {
try { localStorage.setItem(MODEL_KEY, modelPicker.value); } catch (e) {}
});
// #1637 — label each tier with the model it actually selects. A tier the bot
// has not configured is disabled instead of pretending to switch models.
(function hydrateModelTiers() {
  var botName = window.__INITIAL_BOT_NAME__ || "";
  var url = "/api/chat/models" + (botName ? "?bot=" + encodeURIComponent(botName) : "");
  fetch(url)
    .then(function (res) { return res.ok ? res.json() : null; })
    .then(function (data) {
      if (!data || !Array.isArray(data.tiers)) return;
      data.tiers.forEach(function (tier) {
        var option = modelPicker.querySelector('option[value="' + tier.id + '"]');
        if (!option) return;
        var model = tier.model ? " · " + tier.model : "";
        option.textContent = tier.label + model;
        option.title = tier.tradeoff || "";
        option.disabled = tier.id !== "auto" && tier.configured === false;
        if (option.disabled && option.selected) {
          modelPicker.value = "auto";
        }
      });
    })
    .catch(function () {
      /* keep the static honest labels when the endpoint is unavailable */
    });
})();
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
