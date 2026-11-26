/**
 * Static self-test for the gbgov -> classic-chat parity ports (#2563).
 * Run with: node botui/ui/suite/tests/webtest-chat-parity-gaps.js
 *
 * No browser, no server, no cargo, no network: only literal substrings of the
 * shipped chat sources and the eight Fluent catalogues are inspected. The
 * assertions are non-vacuous — removing any of the three guards, their i18n
 * keys or the all-locale catalog entries makes this file fail.
 */
"use strict";

var assert = require("assert");
var fs = require("fs");
var path = require("path");

var ROOT = path.resolve(__dirname, "../../../..");
function read(relative) {
  var file = path.join(ROOT, relative);
  assert(fs.existsSync(file), "missing file: " + relative);
  return fs.readFileSync(file, "utf8");
}

var SEND = read("botui/ui/suite/chat/chat-send_b.js");
var MODEL = read("botui/ui/suite/chat/chat-model-tag.js");
var SOCKET = read("botui/ui/suite/chat/chat-websocket_b.js");
var TURN = read("botui/ui/suite/chat/chat-turn-state.js");
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

var KEYS = [
  "chat-busy-send-head", "chat-busy-send-text", "chat-busy-send-wait", "chat-busy-send-stop",
  "chat-engine-error-not-linked", "chat-engine-error-switch-off", "chat-engine-error-unavailable",
  "chat-engine-error-model-unavailable", "chat-engine-error-workspace-failed",
  "chat-engine-error-account-model-off", "chat-engine-error-free-unavailable",
  "chat-engine-error-limit-reached", "chat-engine-error-unsafe-version",
  "chat-engine-error-choose-model", "chat-engine-error-back-at",
];

function contains(source, needle, message) {
  assert(source.indexOf(needle) !== -1, message + ": <" + needle + ">");
}

// Non-vacuity: the three shipped files stay below the 450-line hard limit.
[["chat-send_b.js", SEND], ["chat-model-tag.js", MODEL], ["chat-websocket_b.js", SOCKET],
 ["chat-turn-state.js", TURN]].forEach(function (entry) {
  var lines = entry[1].split(/\r?\n/).length;
  assert(lines < 450, entry[0] + " is " + lines + " lines (limit 450)");
});

// 1. Busy-send guard: wraps the real send function, asks, then stops and sends.
contains(SEND, "window.sendMessage = guarded", "busy guard installs on the real send path");
contains(SEND, "__gbBusySendGuard", "busy guard is installed only once");
contains(SEND, "if (!busy()) return original.apply(this, arguments)", "unbusy sends pass through");
contains(SEND, "window.finalizeStreaming()", "stop-and-send cleans up the streaming bubble");
contains(SEND, "GBTurn.cancel", "stop-and-send cancels the running turn");
contains(SEND, "return original.apply(self, args)", "stop-and-send delivers the held message");
contains(SEND, 'setAttribute("role", "alertdialog")', "busy prompt is an accessible dialog");
assert(/head\.textContent = t\(/.test(SEND) && /text\.textContent = t\(/.test(SEND),
  "busy prompt text goes through gbT, never a bare literal");

// 2. Engine refusals: every server code has an actionable, localized message.
contains(MODEL, "__gbEngineErrors", "engine error wrapper is installed only once");
contains(MODEL, "window.processMessage = wrapped", "engine errors intercept the frame dispatcher");
contains(MODEL, "!data.error_code || !CODES[data.error_code]", "unknown codes are ignored");
contains(MODEL, "GBTurn.stale(data)", "a stale error frame never ends the live turn");
contains(MODEL, "window.finalizeStreaming()", "intercepted streaming bubble is cleaned up");
contains(MODEL, "cs.cancelledAt = Date.now()", "error card closes the late-frame window");
contains(MODEL, 'setAttribute("role", "alert")', "refusal card is announced");
contains(MODEL, "body.textContent = fill(", "server params are inserted as text (no HTML injection)");
contains(MODEL, "gbResendChatPayload", "retry reuses the structured original payload");
contains(MODEL, "gb-parity-model-btn", "choose-another-model opens chat's real model menu");
["agent_not_linked", "agent_switch_off", "agent_unavailable", "agent_model_unavailable",
 "agent_workspace_failed", "account_model_off", "free_model_unavailable",
 "agent_limit_reached", "agent_unsafe_version"].forEach(function (code) {
  contains(MODEL, code + ": [", "code " + code + " is actionable");
});

// 3. Cancellation: the turn stamps a bounded window; late frames are dropped.
contains(TURN, "current.cancelledAt = Date.now()", "turn state stamps the cancel window");
assert(TURN.split("current.cancelledAt = Date.now()").length - 1 === 2,
  "both cancel() and forceHide() stamp the window");
contains(SOCKET, "__gbCancelGuard", "cancel guard is installed only once");
contains(SOCKET, "window.processMessage = wrapped", "cancel guard wraps the frame dispatcher");
contains(SOCKET, "Date.now() - Number(cs.cancelledAt) > WINDOW_MS", "window is bounded");
contains(SOCKET, "GBTurn.active()", "a new turn is never suppressed");
contains(SOCKET, 'window.addEventListener("gb-chat-message-sent"', "next send clears the marker");

// 4. Every new user-facing key is actually used by JS and catalogued in all 8 locales.
var sources = SEND + MODEL + SOCKET;
KEYS.forEach(function (key) {
  assert(sources.indexOf('"' + key + '"') !== -1, key + " is used by chat JS");
  LOCALES.forEach(function (locale) {
    var catalog = read("botlib/locales/" + locale + "/ui-chat.ftl");
    var match = catalog.match(new RegExp("^" + key + " = (.+)$", "m"));
    assert(match && match[1].trim().length > 0, locale + "/" + key + " has a non-empty value");
  });
});

console.log("PASS: 3 chat parity guards shipped; " + KEYS.length + " keys x " +
  LOCALES.length + " locales; 4 files under 450 lines.");
