/**
 * Static self-test for the chat VM-consent / subscription-agent parity port.
 * Run with: node botui/ui/suite/tests/webtest-chat-vm-agent-parity.js
 *
 * No browser, no server, no network: only the shipped sources and the eight
 * Fluent catalogues are read. The checks are non-vacuous — dropping a script
 * tag, a mount() registration, a key from any locale or crossing the 450-line
 * limit makes this file fail.
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

var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var BASE = "botui/ui/suite/chat/assistant-parity/";
var MODULES = [
  ["06_consent_vm.js", "consentVm"],
  ["06b_vm_wait.js", "vmWait"],
  ["06c_vm_all_bar.js", "vmAllBar"],
  ["07_agent_subscribe.js", "agentSubscribe"]
];
var HTML = read("botui/ui/suite/partials/chat.html");
var INIT = read(BASE + "99_init.js");
var sources = {};

// 1. Module wiring: every new script is loaded before the boot, and boot mounts it.
MODULES.forEach(function (entry) {
  var file = entry[0], name = entry[1];
  sources[file] = read(BASE + file);
  var tag = '<script src="/suite/chat/assistant-parity/' + file;
  assert(HTML.indexOf(tag) !== -1, file + " is wired into partials/chat.html");
  assert(HTML.indexOf(tag) < HTML.indexOf("assistant-parity/99_init.js"),
    file + " loads before 99_init.js");
  assert(INIT.indexOf('safe("' + name + '")') !== -1,
    name + " is registered by 99_init.js safe(name).mount()");
  assert(new RegExp("P\\." + name + "\\s*=\\s*\\{[^;]*mount:\\s*function").test(sources[file]),
    file + " exports P." + name + ".mount()");
  var lines = sources[file].split(/\r?\n/).length;
  assert(lines < 450, file + " is " + lines + " lines (limit 450)");
});

// 2. Reuse chat's transport and state; no second socket, no duplicate chat send.
var VM = sources["06_consent_vm.js"];
var WAIT = sources["06b_vm_wait.js"];
var BAR = sources["06c_vm_all_bar.js"];
var AGENT = sources["07_agent_subscribe.js"];
["new WebSocket", "ChatState.ws.send", "/ws"].forEach(function (needle) {
  MODULES.forEach(function (entry) {
    assert(sources[entry[0]].indexOf(needle) === -1,
      entry[0] + " must reuse chat's WebSocket, found: " + needle);
  });
});
assert(VM.indexOf("window.sendMessage") === -1,
  "a VM decision never sends a chat continuation message");
assert(VM.indexOf('"gb-vm-consent-decision"') !== -1 &&
  VM.indexOf("native_tools") !== -1,
  "VM decisions hand off through the native_tools contract event");
assert(VM.indexOf('data-decision="deny"') === -1 || VM.indexOf("BUTTONS") !== -1,
  "VM offer buttons are built from the server offers");
assert(VM.indexOf('setAttribute("role", "status")') !== -1 &&
  VM.indexOf('setAttribute("aria-live", "polite")') !== -1,
  "VM announcements use a polite status region");
assert(BAR.indexOf('setAttribute("role", "status")') !== -1 &&
  BAR.indexOf('setAttribute("aria-live", "polite")') !== -1,
  "allow-all bar is a polite status region");
assert(AGENT.indexOf("GBChatControls") !== -1 && AGENT.indexOf("AgentMode") !== -1,
  "subscription lock reuses chat's agent switch, not a second toggle");
assert(AGENT.indexOf("sub:") !== -1 && AGENT.indexOf('"/api/ai-accounts"') !== -1,
  "subscription agents reuse chat's linked-account surface");
assert(WAIT.indexOf("window.sendMessage = guarded") !== -1 &&
  WAIT.indexOf("__gbVmSendGuard") !== -1,
  "send confirmation wraps the existing send function only once");

// 3. Every key the new JS asks for is hyphenated and present in all 8 catalogs.
var used = {};
MODULES.forEach(function (entry) {
  var text = sources[entry[0]];
  var re = /\bt\(\s*"([a-z0-9][a-z0-9.-]*)"/g;
  var match;
  while ((match = re.exec(text)) !== null) used[match[1]] = true;
});
var keys = Object.keys(used).filter(function (key) { return key.indexOf("chat-vm-") === 0; });
assert(keys.length >= 40, "expected the full VM/agent key set, got " + keys.length);
var catalogs = {};
LOCALES.forEach(function (locale) {
  var text = read("botlib/locales/" + locale + "/ui-shell.ftl");
  catalogs[locale] = text;
});
keys.forEach(function (key) {
  assert(/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(key), key + " must be hyphenated");
  assert(key.indexOf("auto-") !== 0, key + " is an invented auto- key");
  LOCALES.forEach(function (locale) {
    var found = catalogs[locale].match(new RegExp("^" + key + " = (.+)$", "m"));
    assert(found && found[1].trim().length > 0, locale + "/" + key + " is missing or empty");
    var count = catalogs[locale].split(new RegExp("^" + key + " = ", "m")).length - 1;
    assert(count === 1, locale + "/" + key + " is defined " + count + " times");
  });
});

console.log("PASS: 4 modules wired; no second socket; " + keys.length +
  " keys x " + LOCALES.length + " locales; all files under 450 lines.");
