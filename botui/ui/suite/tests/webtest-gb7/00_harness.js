/**
 * GB7 Web Automation Test Suite
 * Tests all GB7 features and BBSIA review fixes.
 * Run with: node botui/ui/suite/tests/webtest_gb7.js
 *
 * Shared harness; the tests live in ordered modules under ./webtest-gb7/ (#1731).
 */
"use strict";

var fs = require("fs");
var path = require("path");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var TESTS_DIR = path.resolve(__dirname, "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var PENDING = [];

function test(name, fn) {
  try {
    var output = fn();
    if (output && typeof output.then === "function") {
      PENDING.push(
        output.then(
          function () {
            PASS++;
            RESULTS.push({ name: name, status: "PASS" });
          },
          function (e) {
            FAIL++;
            RESULTS.push({ name: name, status: "FAIL", error: e.message });
          },
        ),
      );
      return;
    }
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (e) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: e.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

var featureFiles = [
  "chat/chat-sanitize.js",
  "chat/chat-artifacts.js",
  "chat/chat-render.js",
  "chat/chat-messages.js",
  "chat/chat-send.js",
  "chat/chat-session.js",
  "chat/modules/30_citations.js",
  "chat/modules/50_memory.js",
  "chat/modules/60_computer_use.js",
  "chat/modules/61_terminal_use.js",
  "chat/modules/62_sandbox.js",
  "drive/modules/05_files.js",
  "drive/modules/08_modals.js",
  "drive/modules/11_external.js",
  "drive/modules/12_offline_sync.js",
  "drive/modules/98_actions.js",
  "js/auto-update.js",
  "js/auto-update-settings.js",
  "js/data-export.js",
  "js/usage-stats.js",
  "js/privacy-controls.js",
  "js/accessibility.js",
  "desktop.html",
];

var allJsFiles = featureFiles.filter(function (f) { return f.endsWith(".js"); });

var vm = require("vm");

function loadScript(relPath, sandbox) {
  var context = vm.createContext(sandbox);
  vm.runInContext(readFile(relPath), context, { filename: relPath });
  return context;
}

function memoryStorage() {
  var store = {};
  return {
    getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
  };
}

function report() {
console.log("\n" + "=".repeat(60));
console.log("GB7 WEB AUTOMATION TEST RESULTS");
console.log("=".repeat(60));
RESULTS.forEach(function (r) {
  var icon = r.status === "PASS" ? "PASS" : "FAIL";
  console.log("[" + icon + "] " + r.name + (r.error ? " — " + r.error : ""));
});
console.log("=".repeat(60));
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("=".repeat(60));

process.exit(FAIL > 0 ? 1 : 0);
}

function finish() {
  if (PENDING.length) {
    Promise.all(PENDING).then(report);
  } else {
    report();
  }
}

module.exports = {
  fs,
  path,
  SUITE_DIR,
  TESTS_DIR,
  test,
  assert,
  readFile,
  fileExists,
  featureFiles,
  allJsFiles,
  loadScript,
  memoryStorage,
  finish,
};
