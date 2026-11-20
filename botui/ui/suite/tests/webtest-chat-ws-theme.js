/**
 * Offline verification for #2547 UAT follow-up: plain-text bot replies must
 * never be passed to JSON.parse in the WebSocket onmessage handler.
 *
 * The handler used to call JSON.parse(data.content) on every string looking
 * for an embedded `change_theme` event. Normal answers put plain text in
 * data.content (frames::chunk / frames::final_frame), so every reply threw a
 * SyntaxError that was reported as a CLIENT error on the server.
 *
 * Loads the REAL shipped botui/ui/suite/chat/chat-websocket_b.js verbatim in a
 * Node vm. No production logic is reimplemented here.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..");
var SRC = path.join(SUITE_DIR, "chat", "chat-websocket_b.js");
var PASS = 0;
var FAIL = 0;

function test(name, fn) {
  try { fn(); PASS++; console.log("[PASS] " + name); }
  catch (e) { FAIL++; console.log("[FAIL] " + name + " :: " + e.message); }
}
function assert(cond, msg) { if (!cond) throw new Error(msg || "Assertion failed"); }

var sandbox = { window: {}, document: {}, console: console };
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(SRC, "utf8"), sandbox, { filename: SRC });
var parse = sandbox.parseThemeEventContent;

test("shipped module exports the guarded theme-content parser", function () {
  assert(typeof parse === "function", "parseThemeEventContent not exported");
});

test("plain-text replies of any shape are ignored, never parsed", function () {
  assert(parse("Hello, how can I help?") === null, "plain prose must be ignored");
  assert(parse("Unexpected token 'P' — not valid JSON") === null, "error-like text must be ignored");
  assert(parse("") === null, "empty string must be ignored");
  assert(parse("   ") === null, "whitespace must be ignored");
  assert(parse("Here is {not JSON at all}") === null, "inline braces must be ignored");
  assert(parse("[1,2,3]") === null, "a JSON array is not a theme event");
  assert(parse(null) === null, "null must be ignored");
  assert(parse(42) === null, "non-string must be ignored");
});

test("JSON objects without change_theme are ignored without throwing", function () {
  assert(parse('{"some":"payload"}') === null, "unrelated JSON object must be ignored");
  assert(parse('{"broken":') === null, "malformed JSON object must fail closed");
});

test("a genuine embedded change_theme event is still decoded", function () {
  var evt = parse('  {"event":"change_theme","data":{"theme":"sentient"}}  ');
  assert(evt && evt.event === "change_theme", "valid theme event was not decoded");
  assert(evt.data.theme === "sentient", "theme payload was not preserved");
});

console.log("============================================================");
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("============================================================");
process.exit(FAIL ? 1 : 0);
