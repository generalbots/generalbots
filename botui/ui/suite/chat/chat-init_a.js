/* Chat initialisation: deep link, event handlers, model picker, boot. */
"use strict";

// #OFFLINE-LOGIN — the packaged desktop is an offline-first, single-user
// shell. Its pages are already mounted when the local sidecar is still
// starting (or its directory is unreachable), and /api/auth then reports
// "anonymous" even for a valid session (session rehydration fails closed on
// a cold DB). Treating that transient answer as a revoked token and calling
// window.location.href = GB_LOGIN_URL makes the shell "switch to the login
// page" after it had loaded. Keep the session and let the shell/sidebar own
// re-authentication instead; the multi-tenant web flow is unchanged.
function isLocalDesktopShell() {
  return (
    !!(window.__TAURI__ || window.__TAURI_INTERNALS__) ||
    !!(
      window.productConfig &&
      (window.productConfig.shell_mode === true ||
        window.productConfig.edition === "os")
    )
  );
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
  } catch (e) { window.gbLogError ? gbLogError(e, "proceedWithChatInit") : console.error("proceedWithChatInit", e); }

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
  // Switching bots must not inherit a missing-bot latch or access probe for
  // another bot; self-healing fallback is also scoped per attempted bot.
  if (ChatState.botUnavailableFor && ChatState.botUnavailableFor !== botName) {
    ChatState.botUnavailable = false;
    ChatState.botUnavailableFor = null;
  }
  if (window.__gbBotFallbackFor && window.__gbBotFallbackFor !== botName) {
    window.__gbBotFallbackDone = false;
    window.__gbBotFallbackFor = null;
  }
  ChatState.botAccess = null;
  var storageKey = "gb_chat_" + botName;
  var requestedSession = readRequestedSession();
  // #2222 — the sidebar's "New conversation" fallback (chat not mounted) parks
  // a `gb:new-conversation` flag and reloads. Consume it here: without a
  // one-shot drop of the remembered session the reload just reopened the
  // previous conversation, so the button appeared to do nothing. An explicit
  // deep link (?session_id=) always wins over the flag.
  if (!requestedSession) {
    try {
      if (sessionStorage.getItem("gb:new-conversation") === "1") {
        sessionStorage.removeItem("gb:new-conversation");
        localStorage.removeItem("gb.chat.session." + botName);
      }
    } catch (e) { /* storage unavailable — keep the stored session */ }
  }
  // #1668 — reload returns to the same conversation so the run cards and
  // surfaces can be restored from the event log instead of starting over.
  if (!requestedSession) {
    try {
      requestedSession = localStorage.getItem("gb.chat.session." + botName) || "";
    } catch (e) { /* storage unavailable */ }
  }

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

  function clearStaleChatAuthTokens() {
    var lsKeys = ["gb-access-token", "management_token", "token", "id_token", "gb_token",
      "management_email", "management_name", "management_is_admin", "gb-refresh-token"];
    var ssKeys = ["gb-access-token", "token", "id_token", "gb_token", "gb-refresh-token"];
    lsKeys.forEach(function (k) { try { localStorage.removeItem(k); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); } });
    ssKeys.forEach(function (k) { try { sessionStorage.removeItem(k); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); } });
    if (window.GBSecurity && window.GBSecurity.clearTokens) {
      try { window.GBSecurity.clearTokens(); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
    }
  }

  // Capture auth token passed back from the login server (?token=...).
  // localStorage is origin-scoped, so the login domain's storage is not
  // visible here. Consume the param without long-lived localStorage persistence
  // (#2347): promote it into memory/session auth state, purge legacy localStorage
  // bearer copies, and strip the token from the URL immediately.
  try {
    var u = new URL(window.location.href);
    var urlTok = u.searchParams.get("token");
    if (urlTok) {
      u.searchParams.delete("token");
      window.history.replaceState({}, "", u.pathname + u.search + u.hash);
      try { localStorage.removeItem("gb-access-token"); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
      try { localStorage.removeItem("management_token"); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
      try { localStorage.removeItem("token"); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
      try { localStorage.removeItem("id_token"); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
      try { localStorage.removeItem("gb_token"); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
      if (window.GBSecurity && window.GBSecurity.setTokens) {
        window.GBSecurity.setTokens(urlTok, sessionStorage.getItem("gb-refresh-token"), null, true);
      } else {
        try { sessionStorage.setItem("gb-access-token", urlTok); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
      }
    } else {
      // Migrate any legacy persisted bearer token into transient auth state once,
      // then drop the localStorage copies so they are not exfiltrable across reloads.
      var legacyPersisted =
        (function () { try { return localStorage.getItem("gb-access-token"); } catch (e) { return ""; } })() ||
        (function () { try { return localStorage.getItem("management_token"); } catch (e) { return ""; } })() ||
        (function () { try { return localStorage.getItem("token"); } catch (e) { return ""; } })() ||
        (function () { try { return localStorage.getItem("id_token"); } catch (e) { return ""; } })() ||
        (function () { try { return localStorage.getItem("gb_token"); } catch (e) { return ""; } })();
      if (legacyPersisted) {
        var hasTransient = !!((window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
          (function () { try { return sessionStorage.getItem("gb-access-token"); } catch (e) { return ""; } })());
        if (!hasTransient) {
          if (window.GBSecurity && window.GBSecurity.setTokens) {
            window.GBSecurity.setTokens(legacyPersisted, sessionStorage.getItem("gb-refresh-token"), null, true);
          } else {
            try { sessionStorage.setItem("gb-access-token", legacyPersisted); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
          }
        }
        ["gb-access-token", "management_token", "token", "id_token", "gb_token"].forEach(function (k) {
          try { localStorage.removeItem(k); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
        });
      }
    }
  } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }

  var authUrl = "/api/auth?bot_name=" + encodeURIComponent(botName);
  if (requestedSession) {
    authUrl += "&session_id=" + encodeURIComponent(requestedSession);
  }

  var authHeaders = {};
  var gbToken = null;
  if (window.getGBAccessToken) {
    gbToken = window.getGBAccessToken();
  } else if (window.GBSecurity && window.GBSecurity.getToken) {
    gbToken = window.GBSecurity.getToken();
  } else {
    try { gbToken = sessionStorage.getItem("gb-access-token"); } catch (e) { gbToken = null; }
  }
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
        if (isLocalDesktopShell()) {
          // Degraded/offline sidecar: do NOT delete a possibly valid token and
          // do NOT navigate. Stay on the mounted desktop and continue with an
          // anonymous chat session until local services answer again.
          console.warn("[chat] session could not be verified; staying in the local desktop shell");
        } else {
          clearStaleChatAuthTokens();
          var loginUrl = window.GB_LOGIN_URL || "/login";
          window.location.href = loginUrl + "?redirect=" + encodeURIComponent(window.location.href);
          return;
        }
      }
      window.__gbChatInitRetries = 0;
      ChatState.currentUserId = auth.user_id;
      ChatState.currentSessionId = auth.session_id;
      ChatState.currentBotId = auth.bot_id || "default";
      ChatState.currentBotName = botName;
      // #2153 — /api/auth resolves an unknown bot to the nil UUID. Dialling
      // the socket for it left the window "Connecting…" forever, so say that
      // the bot is missing. A stale selection heals itself once by dropping
      // the poisoned value and reopening the server-default bot.
      if (auth.bot_id === "00000000-0000-0000-0000-000000000000") {
        var fallbackBot = window.__INITIAL_BOT_NAME__ || "default";
        try { localStorage.removeItem("gb_selected_bot"); } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }
        var canFallback = (!window.__gbBotFallbackDone || window.__gbBotFallbackFor !== botName) &&
          botName !== fallbackBot;
        if (canFallback) {
          window.__gbBotFallbackDone = true;
          window.__gbBotFallbackFor = botName;
          notify('Bot "' + botName + '" was not found — opening ' + fallbackBot, "error");
          ChatState.currentBotName = fallbackBot;
          setTimeout(proceedWithChatInit, 0);
          return;
        }
        showBotUnavailable(botName);
        return;
      }
      // A real bot resolved: clear any stale missing-bot latch so later boots
      // and other bots can connect without a page reload.
      ChatState.botUnavailable = false;
      ChatState.botUnavailableFor = null;
      window.__gbBotFallbackDone = false;
      window.__gbBotFallbackFor = null;
      try {
        localStorage.setItem(storageKey, JSON.stringify({ user_id: auth.user_id }));
        localStorage.setItem("gb.chat.session." + botName, auth.session_id);
      } catch (e) { window.gbLogError ? gbLogError(e, "clearStaleChatAuthTokens") : console.error("clearStaleChatAuthTokens", e); }

      window.dispatchEvent(new CustomEvent("gb-chat-session-changed", {
        detail: { session_id: auth.session_id },
      }));

      var readyToConnect = requestedSession
        ? loadSessionHistory(requestedSession, true)
        : Promise.resolve();
      readyToConnect.then(function () {

      // Check bot visibility — redirect private bots to login if not authenticated
      fetch("/api/bot/public?bot_name=" + encodeURIComponent(botName))
        .then(function (r) { return r.json(); })
        .then(function (cfg) {
          var isPub = cfg.is_public === "true" || cfg.is_public === true;
          var isAuth = window.GBSecurity && window.GBSecurity.isAuthenticated && window.GBSecurity.isAuthenticated();
          if (!isPub && !isAuth) {
            // Mirror the shell gate's 'default' exemption (04_auth_fullscreen_a.js):
            // in the packaged desktop "default" is a static segment, not a tenant
            // bot, and an offline sidecar can report is_public=false with no auth
            // context. Never bounce a mounted local shell to the login page.
            if (isLocalDesktopShell() || botName === "default") {
              console.warn("[chat] bot access could not be verified; staying in the local desktop shell");
            } else {
              window.location.href = (window.GB_LOGIN_URL || "/login") + "?redirect=" + encodeURIComponent(window.location.href);
              return;
            }
          }
          if (isPub) window.__BOT_IS_PUBLIC__ = true;
          connectWebSocket();
        })
        .catch(function () { connectWebSocket(); });

      });
    })
    .catch(function () {
      notify("Failed to connect to chat server", "error");
      // Bound the offline retry loop: without a cap a cold sidecar re-runs the
      // whole self-heal chain every 3 s forever, eventually landing on a
      // transient "anonymous" answer that used to bounce to login.
      var retries = Number(window.__gbChatInitRetries || 0);
      if (retries >= 20) return; // ~60 s of retries, then let the user retry manually
      window.__gbChatInitRetries = retries + 1;
      setTimeout(proceedWithChatInit, 3000);
    });
}

// #2933 — focus() forces layout, so it never runs inside a boot timer. Every
// path (autofocus, deep link, idle refocus) queues this one idle pass instead.
var gbFocusQueued = false;
function gbScheduleComposerFocus() {
  if (gbFocusQueued) return;
  gbFocusQueued = true;
  if (window.gbIdle) window.gbIdle(gbApplyComposerFocus);
  else setTimeout(gbApplyComposerFocus, 1);
}
function gbApplyComposerFocus() {
  gbFocusQueued = false;
  var input = document.getElementById("messageInput");
  if (!input || !input.isConnected) return;
  var active = document.activeElement;
  var typingElsewhere = active && active !== document.body && active !== input &&
    (active.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(active.tagName));
  if (typingElsewhere) return;
  input.focus({ preventScroll: true });
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
      gbScheduleComposerFocus();
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
    gbScheduleComposerFocus();
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
  // #2163 — focus is coalesced into an idle pass; no layout read in this timer.
  setTimeout(function() {
    gbScheduleComposerFocus();
    applyDeepLinkMessage();
  }, 500);
}

// #2171 — when chat opens a document ("Open as document", snapshot, plan
// `open` step) the layout goes side by side and the newly opened window used
// to take the caret, so the next keystroke went nowhere. Refocus the composer
// briefly after a chat-originated layout change, and never while the user is
// typing inside the document.
function refocusComposerAfterLayoutChange() {
  setTimeout(gbScheduleComposerFocus, 120);
}

