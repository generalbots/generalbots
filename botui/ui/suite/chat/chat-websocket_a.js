var CHAT_FRIENDLY_ERRORS = {
  "Disconnected from chat server": "Chat disconnected. Reconnecting...",
  "Failed to connect to chat server": "Could not reach the chat server. Retrying...",
  "WebSocket connection failed": "Connection lost. Trying to reconnect...",
};

function chatFriendlyError(message) {
  if (typeof gbT !== "function") return CHAT_FRIENDLY_ERRORS[message] || message;
  if (message === "Disconnected from chat server") return gbT("chat-disconnected-reconnecting", "Chat disconnected. Reconnecting...");
  if (message === "Failed to connect to chat server") return gbT("chat-connect-retrying", "Could not reach the chat server. Retrying...");
  if (message === "WebSocket connection failed") return gbT("chat-connection-lost", "Connection lost. Trying to reconnect...");
  return CHAT_FRIENDLY_ERRORS[message] || message;
}

function notify(message, type) {
  type = type || "info";
  var friendly = chatFriendlyError(message);
  if (window.GBAlerts) {
    if (type === "error") {
      window.GBAlerts.warning("Chat", friendly);
    } else {
      window.GBAlerts.info("Chat", friendly);
    }
    return;
  }
  // The suite/desktop shell does not load GBAlerts; without this fallback
  // every disconnect / delivery / upload message was silently dropped.
  if (window.GBToasts && typeof window.GBToasts.show === "function") {
    window.GBToasts.show("Chat", friendly, type === "error" ? "error" : "info");
    return;
  }
  if (typeof window.showToast === "function") {
    window.showToast(friendly, type);
    return;
  }
  console.info(friendly);
}

// #1701 — steering feedback. The chat route has no global showToast and not
// every shell build exposes GBAlerts; GBToasts is the shared component.
function notifySteering(message) {
  if (typeof window.showToast === "function") {
    window.showToast(message, "info");
    return;
  }
  if (window.GBToasts && typeof window.GBToasts.show === "function") {
    window.GBToasts.show("Steering", message, "info");
    return;
  }
  console.info(message);
}

function updateConnectionStatus(status) {
  var statusEl = document.getElementById("connectionStatus");
  if (statusEl) {
    statusEl.className = "connection-status " + status;
  }
  var statusText = statusEl ? statusEl.querySelector(".connection-text") : null;
  var label = "";
  switch (status) {
    case "connected":
      label = tr("chat-online", "Connected");
      if (statusText) statusText.textContent = label;
      if (statusEl) statusEl.style.display = "none";
      // #2155 UI-REG-01c: announce the transition so gb-sr-live does not
      // keep "Connecting…" after the socket is online.
      if (window.GBAppLifecycle) {
        GBAppLifecycle.setState(null, label);
      } else {
        announceChatStatus(label);
      }
      break;
    case "disconnected":
      label = tr("chat-offline", "Disconnected");
      if (statusText) statusText.textContent = label;
      if (statusEl) statusEl.style.display = "flex";
      if (window.GBAppLifecycle) {
        GBAppLifecycle.setState("error", label);
      } else {
        announceChatStatus(label);
      }
      break;
    case "connecting":
      label = tr("chat-connecting", "Connecting...");
      if (statusText) statusText.textContent = label;
      if (statusEl) statusEl.style.display = "flex";
      if (window.GBAppLifecycle) {
        GBAppLifecycle.setState("loading", label);
      } else {
        announceChatStatus(label);
      }
      break;
  }
}

function announceChatStatus(message) {
  if (!message) return;
  var el = document.getElementById("gb-sr-live");
  if (!el) {
    el = document.createElement("div");
    el.id = "gb-sr-live";
    el.className = "gb-sr-status";
    el.setAttribute("role", "status");
    document.body.appendChild(el);
  }
  el.textContent = "";
  el.textContent = message;
}

function tr(key, fallback) {
  if (window.i18n && typeof window.i18n.t === "function") {
    var value = window.i18n.t(key);
    return value === key ? fallback : value;
  }
  return fallback;
}

// A private bot refuses the WebSocket upgrade with 401, and the browser
// WebSocket API hides that response — the window only saw an endless
// "Disconnected / connecting" flap. Probe the bot first so a private bot with
// an anonymous visitor explains the real reason and offers a way in.
function probeBotAccess(done) {
  if (window.__BOT_IS_PUBLIC__) { ChatState.botAccess = "allowed"; done("allowed"); return; }
  // Cache only resolved states. Probe errors stay retryable.
  if (ChatState.botAccess === "allowed" || ChatState.botAccess === "auth_required") {
    done(ChatState.botAccess);
    return;
  }
  ChatState.botAccess = "probing";
  var bot = ChatState.currentBotName || window.__INITIAL_BOT_NAME__ || "default";
  fetch("/api/bot/public?bot_name=" + encodeURIComponent(bot), { credentials: "same-origin" })
    .then(function (r) {
      if (!r || !r.ok) {
        throw new Error("probe_failed");
      }
      return r.json();
    })
    .then(function (info) {
      if (!info || typeof info !== "object") {
        throw new Error("probe_failed");
      }
      var isPublic = info.is_public === true || String(info.is_public) === "true";
      var token = (window.GBSecurity && typeof GBSecurity.getToken === "function")
        ? GBSecurity.getToken() : null;
      ChatState.botAccess = (isPublic || token) ? "allowed" : "auth_required";
      done(ChatState.botAccess);
    })
    .catch(function (err) {
      // Fail closed: an unreachable or invalid probe must not be treated as public.
      ChatState.botAccess = "probe_error";
      console.warn("[WS] bot access probe failed for '" + bot + "':", err);
      done("probe_error");
    });
}

function showAuthRequired(botName) {
  var statusEl = document.getElementById("connectionStatus");
  if (statusEl) {
    statusEl.className = "connection-status disconnected";
    statusEl.style.cursor = "";
    statusEl.onclick = null;
    var statusText = statusEl.querySelector(".connection-text");
    if (statusText) {
      statusText.textContent = (window.i18n && window.i18n.t("chat-private-signin")) || "Private bot — sign in required";
    }
    statusEl.style.display = "flex";
  }
  var login = window.GB_LOGIN_URL || "/login";
  if (statusEl) {
    statusEl.title = gbT("chat-str-sign-in-at", "Sign in at ") + login;
  }
  var privateMsg = (typeof gbT === "function") ? gbT("chat-bot-private-signin", "This bot is private — sign in to continue") : "This bot is private — sign in to continue";
  if (window.GBAppLifecycle) {
    GBAppLifecycle.setState("error", privateMsg);
  }
  notify(privateMsg, "error");
  console.warn("[WS] bot '" + botName + "' is private; WebSocket not attempted");
}

function showProbeError(botName) {
  var message = "Could not verify bot access";
  var detail = "The server did not respond or returned an invalid response. Bot access is unknown, so chat will not connect automatically.";
  var retryProbe = function () {
    ChatState.botAccess = null;
    updateConnectionStatus("connecting");
    connectWebSocket();
  };
  var statusEl = document.getElementById("connectionStatus");
  if (statusEl) {
    statusEl.className = "connection-status disconnected";
    statusEl.style.display = "flex";
    statusEl.style.cursor = "pointer";
    statusEl.title = (typeof window.gbT === "function" ? window.gbT("chat-retry-connection", "Retry connection") : "Retry connection");
    statusEl.onclick = retryProbe;
    var statusText = statusEl.querySelector(".connection-text");
    if (statusText) statusText.textContent = message + " — click to retry";
  }
  var messages = document.getElementById("messages");
  var loading = messages && messages.querySelector(".gb-state-loading");
  if (loading) {
    loading.className = "gb-state gb-state-error";
    var stateMsg = loading.querySelector(".gb-state-msg");
    if (stateMsg) {
      stateMsg.textContent = detail + " ";
      var link = document.createElement("button");
      link.type = "button";
      link.className = "gb-state-retry";
      link.textContent = (typeof gbT === "function") ? gbT("chat-retry", "Retry") : "Retry";
      link.style.marginLeft = "8px";
      link.onclick = retryProbe;
      stateMsg.appendChild(link);
    }
  } else if (messages && !messages.querySelector(".gb-probe-error")) {
    var box = document.createElement("div");
    box.className = "gb-state gb-state-error gb-probe-error";
    var msg = document.createElement("div");
    msg.className = "gb-state-msg";
    msg.textContent = message + ". " + detail + " ";
    var retry = document.createElement("button");
    retry.type = "button";
    retry.className = "gb-state-retry";
    retry.textContent = (typeof gbT === "function") ? gbT("chat-retry", "Retry") : "Retry";
    retry.onclick = retryProbe;
    msg.appendChild(retry);
    box.appendChild(msg);
    messages.appendChild(box);
  }
  if (window.GBAppLifecycle) {
    GBAppLifecycle.setState("error", message);
  }
  notify(message + " — chat will not connect until access can be verified", "error");
  console.warn("[WS] bot access probe failed for '" + botName + "'; WebSocket not attempted");
}

// #2153 — a route segment or a stale selection can name a bot that does not
// exist. The server refuses the upgrade and the page used to sit on
// "Connecting…" forever while a generic "Reconnecting…" toast blamed the
// network. Say what is wrong and offer an in-app recovery that keeps the
// desktop shell intact.
function recoverFromUnavailableBot() {
  var fallbackBot = window.__INITIAL_BOT_NAME__ || "default";
  try { localStorage.removeItem("gb_selected_bot"); } catch (e) { window.gbLogError ? gbLogError(e, "recoverFromUnavailableBot") : console.error("recoverFromUnavailableBot", e); }
  try { localStorage.removeItem("gb.chat.session." + (ChatState.botUnavailableFor || ChatState.currentBotName || "")); } catch (e) { window.gbLogError ? gbLogError(e, "recoverFromUnavailableBot") : console.error("recoverFromUnavailableBot", e); }
  ChatState.botUnavailable = false;
  ChatState.botUnavailableFor = null;
  ChatState.botAccess = null;
  ChatState.currentBotName = fallbackBot;
  window.__gbBotFallbackDone = false;
  window.__gbBotFallbackFor = null;
  if (typeof proceedWithChatInit === "function") {
    proceedWithChatInit();
  }
}

function showBotUnavailable(botName) {
  var missingBot = botName || ChatState.botUnavailableFor || ChatState.currentBotName || "default";
  // Scope the latch to the bot it refers to so a different bot is not affected.
  ChatState.botUnavailable = true;
  ChatState.botUnavailableFor = missingBot;
  var message = 'Bot "' + missingBot + '" was not found';
  var statusEl = document.getElementById("connectionStatus");
  if (statusEl) {
    statusEl.className = "connection-status disconnected";
    statusEl.style.display = "flex";
    statusEl.style.cursor = "pointer";
    statusEl.title = (typeof window.gbT === "function" ? window.gbT("chat-open-default-bot", "Open the default bot") : "Open the default bot");
    statusEl.onclick = function () { recoverFromUnavailableBot(); };
    var statusText = statusEl.querySelector(".connection-text");
    if (statusText) statusText.textContent = message + " — open the default bot";
  }
  var messages = document.getElementById("messages");
  var loading = messages && messages.querySelector(".gb-state-loading");
  if (loading) {
    loading.className = "gb-state gb-state-error";
    var stateMsg = loading.querySelector(".gb-state-msg");
    if (stateMsg) {
      stateMsg.textContent = message + ". ";
      var link = document.createElement("a");
      link.href = "#";
      link.textContent = (typeof window.gbT === "function" ? window.gbT("chat-open-default-bot-arrow", "Open the default bot →") : "Open the default bot →");
      link.onclick = function (e) { if (e && e.preventDefault) e.preventDefault(); recoverFromUnavailableBot(); };
      stateMsg.appendChild(link);
    }
  }
  if (window.GBAppLifecycle) GBAppLifecycle.setState("error", message);
  notify(message + " — open the default bot", "error");
  console.warn("[WS] bot '" + missingBot + "' does not exist; no socket dialled");
}

// One dial at a time: several boot modules call connectWebSocket() while the
// first probe is still in flight, and each late callback used to dial another
// socket. The loser stayed open as an orphan while ChatState.ws pointed at a
// dead one, so the composer kept writing into a closed socket.
var chatDialSeq = 0;
var CHAT_CONNECT_TIMEOUT_MS = 12000;

function clearChatConnectTimeout() {
  if (ChatState.wsConnectTimeout) { clearTimeout(ChatState.wsConnectTimeout); ChatState.wsConnectTimeout = null; }
}

