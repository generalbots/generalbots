"use strict";
/** Non-vacuity guard split from 82_chat_access_failure.js to stay under 450 lines. */
var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var SEND_A = "botui/ui/suite/chat/chat-send_a.js";
var SEND_B = "botui/ui/suite/chat/chat-send_b.js";
var WS_A = "botui/ui/suite/chat/chat-websocket_a.js";
var INIT_A = "botui/ui/suite/chat/chat-init_a.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";

function readSrc(rel) {
  var probe = process.env.GB_CHAT_ACCESS_PROBE_ROOT;
  var candidate = probe ? path.join(probe, path.basename(rel)) : "";
  if (candidate && fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  return h.readRepo(rel);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }

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
  contains(readSrc(RUNNER), 'require("./webtest-office-fidelity/82b_chat_access_failure_nonvacuity")',
    "the split non-vacuity guard must be required by the fidelity runner");
});

h.note("#chat-access-failure: offline static + stub-vm proof only — no browser/server/cargo, no locale edit. Driver owns the live Chrome UAT and issue closure.");
