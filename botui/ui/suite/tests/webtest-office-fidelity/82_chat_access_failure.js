/**
 * Anonymous chat send regression guard (offline, no browser/server/cargo).
 *
 * In an unauthenticated tab a private `default` bot is exempted from the
 * login redirect (chat-init_a.js logs "bot access could not be verified"),
 * the WS probe settles on `auth_required`, and showAuthRequired() refuses to
 * dial. Before the fix the composer still accepted a message: sendMessage()
 * painted "Sending…", queued the payload and called connectWebSocket(), which
 * re-ran the cached auth_required branch. No frame ever ended the turn and the
 * only trace was a console warning — a stuck "Sending…" with no visible cause.
 *
 * The fix fails fast while the access state is terminal: clear the turn and
 * pending timers, keep the text in the composer, surface the existing
 * `chat-bot-private-signin` catalogue key (shipped in all 8 locales) and
 * return before addMessage()/GBTurn.start()/queueOfflineMessage(). The same
 * reset runs when a late probe denies access, covering the race where the
 * message was composed before the probe resolved.
 *
 * Probes:
 *   (a) a shared reset helper clears GBTurn + pending state;
 *   (b) the block branch precedes every turn/queue call and never queues;
 *   (c) late auth_required / probe_error handlers also reset state;
 *   (d) only an existing i18n key is reused (all catalogs verified);
 *   (e) probe_failed is cleared at its source, even when connectWebSocket_b's
 *       seq guard drops the callback so no UI handler runs;
 *   (f) a terminal access state never queues or promises "will be sent";
 *   (g) runtime: auth_required -> reset + restored composer + one error toast,
 *       no turn, no send, no retry; probe_error -> reset, no turn; allowed ->
 *       the authenticated send path is untouched; stranded payload is dropped;
 *   (h) non-vacuity: small modules, pinned APIs, guard-75 .focus() count,
 *       runner require.
 *
 * GB_CHAT_ACCESS_PROBE_ROOT can point at a /tmp tree for mutation probes
 * without touching the repository. The driver owns UAT and issue closure.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var SEND_A = "botui/ui/suite/chat/chat-send_a.js";
var SEND_B = "botui/ui/suite/chat/chat-send_b.js";
var WS_A = "botui/ui/suite/chat/chat-websocket_a.js";
var INIT_A = "botui/ui/suite/chat/chat-init_a.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var KEY = "chat-bot-private-signin";
var FALLBACK = "This bot is private — sign in to continue";

function readSrc(rel) {
  var probe = process.env.GB_CHAT_ACCESS_PROBE_ROOT;
  if (probe) {
    var candidate = path.join(probe, path.basename(rel));
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function countIn(src, needle) { return src.split(needle).length - 1; }

h.staticTest("chat access failure has one reset helper that clears turn + pending state", function () {
  var src = readSrc(SEND_A);
  var helper = span(src, "function clearBlockedChatTurn()", "\n// #2244");
  h.assert(helper.length > 100, "non-vacuity: reset helper must be substantial, got " + helper.length);
  contains(helper, "GBTurn.forceHide", "the stuck progress element is removed");
  contains(helper, "clearChatPending", "pending timers and status are cleared");
  contains(src, "window.clearBlockedChatTurn = clearBlockedChatTurn;", "helper is shared with WS handlers");
});

h.staticTest("terminal bot access fails the send before any turn or queue is started", function () {
  var src = readSrc(SEND_A);
  var send = span(src, "function sendMessage(messageContent, extra) {",
    "// #1275 — messages composed while the socket is down");
  h.assert(send.length > 2000, "non-vacuity: sendMessage body must be substantial, got " + send.length);
  var branch = span(send, "var blocked = ChatState.botAccess", "  // #1947");
  contains(branch, 'ChatState.botAccess === "auth_required" ? "auth"', "private bot maps to auth");
  contains(branch, 'ChatState.botAccess === "probe_error" ? "probe"', "failed probe maps to probe");
  contains(branch, "if (blocked) {", "a terminal state gates the branch");
  contains(branch, "clearBlockedChatTurn();", "the failure path resets sending state");
  contains(branch, "blockedInput.value = content", "the composer keeps the text for a retry");
  contains(branch, 'notify(chatPrivateSigninMessage(), "error")', "existing localized auth message is surfaced");
  contains(branch, "notify(", "the failure is surfaced, not silent");
  contains(branch, "return;", "the send stops here");
  absent(branch, "queueOfflineMessage(", "no silent offline retry on terminal access");
  var atBranch = send.indexOf("var blocked = ChatState.botAccess");
  var atTurn = send.indexOf("GBTurn.start(");
  var atBubble = send.indexOf('addMessage("user", display)');
  var atQueue = send.indexOf("queueOfflineMessage(payload)");
  h.assert(atBranch !== -1 && atBubble > atBranch && atTurn > atBranch && atQueue > atBranch,
    "block branch must run before bubble/turn/queue (branch=" + atBranch + " bubble=" + atBubble +
    " turn=" + atTurn + " queue=" + atQueue + ")");
  h.assertEqual(countIn(branch, "gbT("), 0,
    "block branch adds no gbT call (the aria baseline pins the suite-wide count)");
});

h.staticTest("late auth denial and probe failure also clear a stuck turn", function () {
  var src = readSrc(WS_A);
  var auth = span(src, "function showAuthRequired(botName) {", "function showProbeError(botName) {");
  var probe = span(src, "function showProbeError(botName) {", "// #2153 — an unknown bot");
  ["the private-bot handler", "the probe-failure handler"].forEach(function (label, i) {
    var body = i === 0 ? auth : probe;
    contains(body, "if (typeof window.clearBlockedChatTurn === \"function\") window.clearBlockedChatTurn();",
      label + " resets the turn when access is denied");
  });
  contains(auth, "private; WebSocket not attempted", "pinned private-bot diagnostic survives");
  contains(src, "function probeBotAccess(done) {", "access probe API preserved");
  contains(src, '"allowed" : "auth_required"', "terminal auth state still recorded");
  var helper = span(src, "function chatPrivateSigninMessage()", "function showAuthRequired(botName) {");
  contains(helper, 'gbT("' + KEY + '", "' + FALLBACK + '")',
    "the one existing catalogue lookup for the auth message lives here");
  h.assertEqual(countIn(src, 'gbT("' + KEY + '"'), 1, "exactly one gbT call for " + KEY + " in the WS module");
});

h.staticTest("probe_failed is resolved at its source, before a stale callback can drop it", function () {
  var ws = readSrc(WS_A);
  var probe = span(ws, "function probeBotAccess(done) {", "function chatPrivateSigninMessage()");
  h.assert(probe.indexOf('throw new Error("probe_failed")') !== -1,
    "non-vacuity: the failed-probe signal is still raised here");
  var caught = span(probe, ".catch(function (err) {", "    });");
  contains(caught, 'ChatState.botAccess = "probe_error"', "failure is recorded even without a UI handler");
  contains(caught, "window.clearBlockedChatTurn();", "the stuck turn clears at the source");
  contains(caught, "window.gbRestoreBlockedComposer()", "the stranded composer text is restored");
  contains(caught, "Failed to connect to chat server", "an honest failure replaces the optimistic toast");
  var atClear = caught.indexOf("window.clearBlockedChatTurn();");
  var atDone = caught.indexOf("done(\"probe_error\")");
  h.assert(atClear !== -1 && atDone > atClear,
    "reset must run before handing the result to a possibly-stale callback");
  var send = readSrc(SEND_A);
  h.assert(send.indexOf("function gbRestoreBlockedComposer()") !== -1 &&
    send.indexOf("window.gbRestoreBlockedComposer = gbRestoreBlockedComposer;") !== -1,
    "composer recovery helper is shipped and shared");
  contains(send, "ChatState.offlineQueue.filter(", "a stranded payload is not left queued for a later send");
});

h.staticTest("offline queue never promises delivery in a terminal access state", function () {
  var src = readSrc(SEND_B);
  var queue = span(src, "function queueOfflineMessage(payload) {", "// #2155 UI-REG-01b");
  contains(queue, 'ChatState.botAccess === "auth_required" || ChatState.botAccess === "probe_error"',
    "terminal access gates the optimistic path");
  var atGate = queue.indexOf("if (ChatState.botAccess ===");
  var atReturn = queue.indexOf("return;", atGate);
  var atPromise = queue.indexOf("chat-reconnecting-queued", atGate);
  h.assert(atGate !== -1 && atReturn > atGate && atPromise > atReturn,
    "the delivery promise must be unreachable after the terminal gate");
  contains(queue, "window.gbRestoreBlockedComposer()", "queued text is put back in the composer");
  contains(queue, "window.clearBlockedChatTurn()", "the stuck turn is cleared without a socket");
  contains(queue, "chat-reconnecting-queued", "normal reconnect keeps its existing queued toast");
});

h.staticTest("the reused auth-failure key is catalogued in all eight locales", function () {
  LOCALES.forEach(function (locale) {
    var dir = path.join(h.REPO_DIR, "botlib", "locales", locale);
    var all = h.fs.readdirSync(dir).filter(function (name) { return /\.ftl$/.test(name); })
      .map(function (name) { return h.fs.readFileSync(path.join(dir, name), "utf8"); }).join("\n");
    h.assert(new RegExp("^" + KEY + "\\s*=", "m").test(all), locale + " is missing key " + KEY);
  });
});

/** Deterministic composer sandbox: real shipped send modules, stubbed globals. */
function makeChat(access, authed) {
  var calls = { turnStart: [], forceHide: 0, sent: [], notify: [], status: [], queued: 0 };
  function el(id) {
    return {
      id: id, value: "", tagName: "TEXTAREA", style: {},
      files: [], focus: function () {},
      querySelector: function () { return null; },
    };
  }
  var input = el("messageInput");
  var ws = { readyState: authed ? 1 : 0, send: function (raw) { calls.sent.push(raw); } };
  var state = {
    botAccess: access, botUnavailable: false, ws: ws,
    pendingResponseAt: null, pendingText: "", pendingSend: null,
    offlineQueue: [], isStreaming: false, selectedMentions: [],
    activeSwitchers: [], currentBotId: "b", currentUserId: "u",
    currentSessionId: "s", clientId: "c", reconnectAttempts: 0,
    maxReconnectAttempts: 5,
  };
  var sandbox = {
    console: console, Math: Math, Date: Date, JSON: JSON, Array: Array, Object: Object,
    String: String, Promise: Promise,
    setTimeout: function () { return 1; }, clearTimeout: function () {}, setInterval: function () { return 1; },
    localStorage: { getItem: function () { return null; }, setItem: function () {}, removeItem: function () {} },
    navigator: { language: "en" },
    MessageType: { USER: 1 }, WebSocket: { OPEN: 1, CONNECTING: 0 },
    ChatState: state,
    gbT: function (key, fallback) { return "LOC:" + key + ":" + fallback; },
    chatPrivateSigninMessage: function () { return "LOC:" + KEY + ":" + FALLBACK; },
    notify: function (message, type) { calls.notify.push({ message: message, type: type }); },
    addMessage: function () {}, hideMentionDropdown: function () {}, scrollToBottom: function () {},
    setChatStatus: function (v) { calls.status.push(v); },
    connectWebSocket: function () { calls.queued++; },
    queueOfflineMessage: function () { calls.queued++; state.offlineQueue.push("queued"); },
    activeSwitcherIds: null,
  };
  sandbox.window = sandbox;
  sandbox.addEventListener = function () {};
  sandbox.document = {
    getElementById: function (id) { return id === "messageInput" ? input : el(id); },
    addEventListener: function () {}, dispatchEvent: function () {},
  };
  sandbox.window.GBTurn = {
    SLOW_AFTER_MS: 12000,
    start: function () { calls.turnStart.push("start"); },
    forceHide: function () { calls.forceHide++; },
  };
  sandbox.window.CustomEvent = function (type, opts) { this.type = type; this.detail = opts.detail; };
  sandbox.window.dispatchEvent = function () {};
  h.vm.createContext(sandbox);
  [SEND_A, SEND_B].forEach(function (rel) {
    h.vm.runInContext(readSrc(rel), sandbox, { filename: rel });
  });
  return { sandbox: sandbox, state: state, input: input, calls: calls };
}

h.test("auth_required send resets state, restores the composer and never retries", function () {
  var t = makeChat("auth_required", false);
  t.state.pendingResponseAt = 123;
  t.input.value = "hello";
  t.sandbox.sendMessage();
  h.assertEqual(t.calls.forceHide, 1, "the stuck turn was cleared");
  h.assertEqual(t.calls.turnStart.length, 0, "no new turn was started");
  h.assertEqual(t.state.pendingResponseAt, null, "pending state was cleared");
  h.assertEqual(t.input.value, "hello", "composer keeps the message for a retry");
  h.assertEqual(t.calls.notify.length, 1, "exactly one user-visible failure");
  h.assert(t.calls.notify[0].message.indexOf("LOC:" + KEY + ":") === 0,
    "failure uses the existing auth-failure key: " + t.calls.notify[0].message);
  h.assertEqual(t.calls.notify[0].type, "error", "failure is an error, not an info toast");
  h.assertEqual(t.calls.queued, 0, "no offline queue / silent reconnect loop");
  h.assertEqual(t.calls.sent.length, 0, "nothing was written into a socket");
  h.assertEqual(t.state.offlineQueue.length, 0, "no payload was queued");
});

h.test("probe_error send resets state without starting a turn", function () {
  var t = makeChat("probe_error", false);
  t.state.pendingResponseAt = 456;
  t.input.value = "still here";
  t.sandbox.sendMessage();
  h.assertEqual(t.calls.forceHide, 1, "the stuck turn was cleared");
  h.assertEqual(t.calls.turnStart.length, 0, "no turn was started");
  h.assertEqual(t.state.pendingResponseAt, null, "pending state was cleared");
  h.assertEqual(t.input.value, "still here", "composer keeps the message");
  h.assertEqual(t.calls.queued, 0, "no silent retry");
  h.assertEqual(t.calls.sent.length, 0, "nothing was sent");
});

h.test("allowed access keeps the authenticated send path unchanged", function () {
  var t = makeChat("allowed", true);
  t.sandbox.sendMessage("normal message");
  h.assertEqual(t.calls.turnStart.length, 1, "the normal turn still starts");
  h.assertEqual(t.calls.sent.length, 1, "the message reaches the open socket");
  h.assert(t.calls.sent[0].indexOf("normal message") !== -1, "payload carries the text");
  h.assertEqual(t.state.pendingResponseAt !== null, true, "pending state is armed");
  h.assertEqual(t.input.value, "", "the composer was cleared after a real send");
  h.assertEqual(t.calls.notify.length, 0, "no auth failure for a signed-in user");
});

/** Probe sandbox: real shipped WS module, failing /api/bot/public fetch. */
function makeProbe(composerRestored) {
  var calls = { reset: 0, restored: 0, notify: [] };
  var state = { botAccess: null, currentBotName: "default", offlineQueue: [] };
  var sandbox = {
    console: { warn: function () {}, error: function () {}, info: function (m) { calls.notify.push({ message: m, type: "info" }); } },
    Math: Math, Date: Date, JSON: JSON, Array: Array, Object: Object, String: String,
    encodeURIComponent: encodeURIComponent,
    ChatState: state,
    gbT: function (key, fallback) { return "LOC:" + key + ":" + fallback; },
    fetch: function () { return { then: function () { return this; }, catch: function (fn) { fn(new Error("probe_failed")); return this; } }; },
  };
  sandbox.window = sandbox;
  sandbox.window.GBSecurity = { getToken: function () { return null; } };
  sandbox.window.clearBlockedChatTurn = function () { calls.reset++; };
  sandbox.window.gbRestoreBlockedComposer = function () { calls.restored++; return composerRestored; };
  h.vm.createContext(sandbox);
  h.vm.runInContext(readSrc(WS_A), sandbox, { filename: WS_A });
  return { sandbox: sandbox, state: state, calls: calls };
}

h.test("probe_failed clears the stuck turn even when a later dial drops the callback", function () {
  var t = makeProbe(true);
  // Simulate connectWebSocket_b's seq guard: the done() result is ignored, so
  // showProbeError()/showAuthRequired() never get a chance to reset anything.
  t.sandbox.probeBotAccess(function () { /* stale callback */ });
  h.assertEqual(t.calls.reset, 1, "reset ran at the probe source, not in a stale UI handler");
  h.assertEqual(t.calls.restored, 1, "composer recovery was attempted at the source");
  h.assertEqual(t.state.botAccess, "probe_error", "the terminal state is still recorded");
  h.assertEqual(t.calls.notify.length, 1, "one honest failure is surfaced");
  h.assert(t.calls.notify[0].message.indexOf("LOC:chat-connect-retrying:") === 0,
    "the failure reuses an existing catalogue key: " + t.calls.notify[0].message);
});

h.test("terminal access does not queue or promise a delivery that cannot happen", function () {
  var t = makeChat("auth_required", false);
  var payload = { content: "stranded text" };
  t.state.lastUserTurnPayload = payload;
  t.sandbox.queueOfflineMessage(payload);
  h.assertEqual(t.state.offlineQueue.length, 0, "the message was not queued");
  h.assertEqual(t.input.value, "stranded text", "the typed text is back in the composer");
  h.assertEqual(t.calls.notify.length, 1, "one visible failure");
  h.assert(t.calls.notify[0].message.indexOf("LOC:" + KEY + ":") === 0,
    "the existing auth-failure key is used: " + t.calls.notify[0].message);
  h.assert(t.calls.notify[0].message.indexOf("will be sent when the connection is back") === -1,
    "no false reconnect promise");
});

h.test("composer recovery is idempotent and drops the stranded payload", function () {
  var t = makeChat("probe_error", false);
  var payload = { content: "stranded" };
  t.state.lastUserTurnPayload = payload;
  t.state.offlineQueue = [payload];
  h.assertEqual(t.sandbox.gbRestoreBlockedComposer(), true, "recovery applies to a stranded turn");
  h.assertEqual(t.input.value, "stranded", "text restored to the composer");
  h.assertEqual(t.state.offlineQueue.length, 0, "payload removed so a later socket cannot duplicate it");
  h.assertEqual(t.sandbox.gbRestoreBlockedComposer(), false, "a second call is a no-op");
});

h.staticTest("non-vacuity: shipped modules, pinned APIs and required guard", function () {
  [SEND_A, SEND_B, WS_A, INIT_A].forEach(function (rel) {
    var src = readSrc(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped module");
  });
  contains(readSrc(SEND_A), "function sendMessage(messageContent, extra) {", "send API preserved");
  contains(readSrc(SEND_A), "function armChatPending(text) {", "pending watchdog preserved");
  contains(readSrc(SEND_B), 'window.sendMessage = sendMessage;', "global send export preserved");
  contains(readSrc(SEND_B), "function queueOfflineMessage(payload) {", "offline queue API preserved");
  contains(readSrc(WS_A), "function showBotUnavailable(botName) {", "missing-bot handler preserved");
  h.assertEqual(countIn(readSrc(INIT_A), ".focus("), 1,
    "guard 75's single-focus pin in chat-init_a is untouched");
  contains(readSrc(RUNNER), 'require("./webtest-office-fidelity/82_chat_access_failure")',
    "this guard must be required by the fidelity runner");
});

h.note("#chat-access-failure: offline static + stub-vm proof only — no browser/server/cargo, no locale edit. Driver owns the live Chrome UAT and issue closure.");
