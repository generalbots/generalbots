/**
 * Static self-test for the gbgov -> classic Suite agent-runtime port.
 * Run with: node botui/ui/suite/tests/webtest-agent-runtime-parity.js
 *
 * No browser, no server, no cargo, no network: only the shipped sources and
 * the ten Fluent catalogues are inspected. Non-vacuous: removing a nav item,
 * a module registration, a status region or any locale key makes it fail.
 */
"use strict";

var assert = require("assert");
var fs = require("fs");
var path = require("path");

var ROOT = path.resolve(__dirname, "../../../..");
var LOCALES = ["en", "it", "ru", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var CATALOGS = ["settings-agent-runtime.ftl", "settings-licenses.ftl", "chat-agent-gate.ftl"];

function read(relative) {
  var file = path.join(ROOT, relative);
  assert(fs.existsSync(file), "missing file: " + relative);
  return fs.readFileSync(file, "utf8");
}
function has(source, needle, message) {
  assert(source.indexOf(needle) !== -1, message + ": <" + needle + ">");
}
function keysOf(source) {
  var found = [];
  var re = /\bt\(\s*"([a-z0-9][a-z0-9-]*)"/g;
  var match;
  while ((match = re.exec(source)) !== null) found.push(match[1]);
  var html = /data-i18n(?:-title|-aria-label|-placeholder)?="([a-z0-9][a-z0-9-]*)"/g;
  while ((match = html.exec(source)) !== null) found.push(match[1]);
  return found.filter(function (key, index, all) { return all.indexOf(key) === index; });
}

var SETTINGS_INDEX = read("botui/ui/suite/settings/index.html");
var NAV = read("botui/ui/suite/settings/nav-registry.js");
var RUNTIME = read("botui/ui/suite/settings/settings-agent-runtime.js");
var RUNTIME_TEXT = read("botui/ui/suite/settings/settings-agent-runtime-text.js");
var LICENSES = read("botui/ui/suite/settings/settings-licenses.js");
var AGENT_PARTIAL = read("botui/ui/suite/settings/partials/18_agent_runtime.html");
var LICENSE_PARTIAL = read("botui/ui/suite/settings/partials/19_licenses.html");
var GATE = read("botui/ui/suite/chat/assistant-parity/08_agent_gate.js");
var INIT = read("botui/ui/suite/chat/assistant-parity/99_init.js");
var SOURCES = [RUNTIME, RUNTIME_TEXT, LICENSES, AGENT_PARTIAL, LICENSE_PARTIAL, GATE];

// 1. Settings nav registry + index wiring (additive: existing sections untouched).
has(NAV, '{ id: "agent-runtime"', "agent-runtime is a registered settings node");
has(NAV, '{ id: "licenses"', "licenses is a registered settings node");
has(SETTINGS_INDEX, "showSection('agent-runtime', this)", "Settings nav opens the agent-runtime section");
has(SETTINGS_INDEX, "showSection('licenses', this)", "Settings nav opens the licenses section");
has(SETTINGS_INDEX, "/suite/settings/partials/18_agent_runtime.html", "agent-runtime partial is loaded");
has(SETTINGS_INDEX, "/suite/settings/partials/19_licenses.html", "licenses partial is loaded");
["settings-agent-runtime-text.js", "settings-agent-runtime.js", "settings-licenses.js"].forEach(function (name) {
  has(SETTINGS_INDEX, "/suite/settings/" + name, name + " is wired into Settings");
});
has(AGENT_PARTIAL, 'id="agent-runtime-card" role="status" aria-live="polite"', "agent status is announced");
has(LICENSE_PARTIAL, 'id="licenses-card"', "licenses section has a render target");

// 2. Runtime contract: existing API, in-memory polling, Tauri bridge only.
has(RUNTIME, 'fetch("/api/product"', "status reuses the existing product endpoint");
has(RUNTIME, "1500", "poll interval is 1.5 s");
has(RUNTIME, 'agent_runtime_status', "Tauri status command");
has(RUNTIME, 'agent_runtime_install', "Tauri install command");
has(RUNTIME, 'agent_runtime_remove', "Tauri remove command");
has(RUNTIME, 'window.GBSettingsAgentRuntime', "public runtime API is exposed");
has(RUNTIME, 'aria-live="polite"', "progress region is announced");
has(RUNTIME, "GBSettingsOnSection", "runtime hooks the existing Settings nav");

// 3. Chat parity: no duplicate picker, no second socket, in-memory state.
has(GATE, "window.GBChatAssistantParity", "chat gate registers on the parity namespace");
has(GATE, "P.agentRuntimeReady", "Assistant's agentRuntimeReady equivalent exists");
has(GATE, "window.ChatState", "gate reads classic chat state");
has(GATE, "window.GBChatControls", "gate reads the existing chat controls");
has(GATE, 'setAttribute("aria-live", "polite")', "chat hint is announced");
has(GATE, "/settings#agent-runtime", "hint links to Settings without an extra surface");
assert(!/localStorage\s*[.[]/.test(GATE), "gate state stays in memory (no localStorage use)");
assert(!/new\s+WebSocket|["']\/ws["']/.test(GATE), "gate never opens a second WebSocket");
has(INIT, "08_agent_gate.js", "99_init registers the ordered gate module");
has(INIT, 'safe("agentGate")', "99_init mounts the gate module");

// 4. Licenses come from the local suite copy only.
has(LICENSES, 'BASE = "/suite/about/licenses/"', "licenses read the local suite copy");
has(LICENSES, 'readText("NOTICE")', "NOTICE is rendered");
has(LICENSES, 'readText("LICENSES-desktop.md")', "component record is rendered");
assert(!/https?:\/\//.test(LICENSES), "licenses module performs no external fetch");

// 5. Size limit and hygiene.
SOURCES.forEach(function (source, index) {
  var lines = source.split(/\r?\n/).length;
  assert(lines < 450, "source " + index + " is " + lines + " lines (limit 450)");
  assert(!/https?:\/\/(cdn|unpkg|jsdelivr)/.test(source), "no CDN reference in source " + index);
});

// 6. Every literal key used by the new UI is catalogued in ALL ten locales.
var usedKeys = [];
SOURCES.forEach(function (source) {
  keysOf(source).forEach(function (key) {
    if (usedKeys.indexOf(key) === -1) usedKeys.push(key);
  });
});
assert(usedKeys.length >= 20, "key extraction is non-vacuous (" + usedKeys.length + " keys)");
var catalogs = {};
LOCALES.forEach(function (locale) {
  catalogs[locale] = {};
  CATALOGS.forEach(function (name) {
    var source = read("botlib/locales/" + locale + "/" + name);
    source.split(/\r?\n/).forEach(function (line) {
      var match = /^([a-z0-9][a-z0-9-]*)\s*=\s*(.+)$/.exec(line);
      if (match) catalogs[locale][match[1]] = match[2].trim();
    });
  });
});
usedKeys.forEach(function (key) {
  assert(/^[a-z0-9]+(-[a-z0-9]+)*$/.test(key), key + " is a hyphenated literal key");
  assert(key.indexOf("auto-") !== 0, key + " is not machine-generated");
  LOCALES.forEach(function (locale) {
    var value = catalogs[locale][key];
    assert(value && value.length > 0, locale + "/" + key + " has a non-empty value");
  });
});

console.log("PASS: Settings + chat agent-runtime parity wired; " + usedKeys.length +
  " keys x " + LOCALES.length + " locales; no second socket, no localStorage, no CDN.");
