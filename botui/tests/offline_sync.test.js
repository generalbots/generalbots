"use strict";
/* GB6-22 (#1573) — local-folder sync guarantees. Zero-dependency test
 * (run: node tests/offline_sync.test.js)
 *
 * A headless run cannot pick a real folder, but the property that matters most
 * is pure and testable: when both sides changed, the local edit is preserved
 * under a distinct conflict name while Drive stays canonical. The module also
 * must report "no folder" instead of failing when no handle exists. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(__dirname, "..", "ui", "suite", "drive", "modules", "12_offline_sync.js");

let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

const store = {};
const localStorage = {
  getItem: (k) => (Object.prototype.hasOwnProperty.call(store, k) ? store[k] : null),
  setItem: (k, v) => { store[k] = String(v); },
  removeItem: (k) => { delete store[k]; },
};
const window = {
  localStorage: localStorage,
  addEventListener: function () {},
  indexedDB: {
    open: function () {
      // Never resolves in the shim: the cache is irrelevant to the conflict
      // naming and folder-state assertions below.
      return { onupgradeneeded: null, onsuccess: null, onerror: null };
    },
  },
};
const document = {
  addEventListener: function () {},
  getElementById: function () { return null; },
  querySelector: function () { return null; },
  createElement: function () { return { style: {}, appendChild: function () {}, addEventListener: function () {} }; },
};
const sandbox = {
  window: window,
  document: document,
  localStorage: localStorage,
  indexedDB: window.indexedDB,
  navigator: { onLine: true },
  console: console,
  setTimeout: function () {},
  clearTimeout: function () {},
  Date: Date,
  JSON: JSON,
  Object: Object,
  Array: Array,
  String: String,
  Number: Number,
  Promise: Promise,
  Uint8Array: Uint8Array,
  atob: (s) => Buffer.from(s, "base64").toString("binary"),
  fetch: function () { return Promise.resolve({ ok: true, json: function () { return Promise.resolve({}); } }); },
  CustomEvent: function () {},
};
vm.runInNewContext(fs.readFileSync(MODULE, "utf8"), sandbox, { filename: MODULE });
const sync = window.DriveOfflineSync;

// 1. The conflict copy keeps the original name and extension and is distinct.
const when = new Date("2026-11-01T12:34:56.000Z");
const name = sync.conflictName("report.pdf", when);
check(name.indexOf("report") === 0, "the base name is kept");
check(name.indexOf(".pdf") === name.length - 4, "the extension is kept");
check(/conflict/.test(name), "the copy is labelled as a conflict");
check(name !== "report.pdf", "the local copy does not overwrite the synced file");

const later = sync.conflictName("report.pdf", new Date("2026-11-01T12:35:56.000Z"));
check(name !== later, "each conflict gets a distinct name");

// 2. Extension-less files are still preserved.
const bare = sync.conflictName("README", when);
check(bare.indexOf("README (conflict ") === 0, "an extension-less file keeps its name");

// 3. Without the File System Access API the module must not promise folder sync.
check(sync.supportsFolderSync() === false, "folder sync is unavailable without the File System Access API");
check(typeof sync.syncNow === "function", "the two-way sync entry point exists");
check(typeof sync.getStatus === "function", "a status reporter exists for the toolbar");
check(typeof sync.getSyncFolder === "function", "the chosen folder is readable");

console.log(`offline_sync: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
