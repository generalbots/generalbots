"use strict";
/* #1813 — "Use all Drive files" Settings toggle.
 *
 * Zero-dependency unit tests (run: node tests/drive_knowledge_toggle.test.js).
 * The chat composer forwards `index_all_drive` from the same localStorage key;
 * this covers the Settings side: default on, persisted off/on, and the event
 * the desktop shell can react to. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(
  __dirname,
  "..",
  "ui",
  "suite",
  "settings",
  "main",
  "08_preview.js",
);

let passed = 0;
let failed = 0;

function assertEqual(actual, expected, label) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    passed++;
  } else {
    failed++;
    console.error("FAIL " + label + "\n  expected: " + e + "\n  actual:   " + a);
  }
}

function makeStorage(initial) {
  const data = Object.assign({}, initial || {});
  return {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(data, key) ? data[key] : null;
    },
    setItem: function (key, value) {
      data[key] = String(value);
    },
  };
}

function loadModule(storage) {
  const S = {
    showToast: function () {},
  };
  const checkbox = { checked: false };
  const events = [];
  const sandbox = {
    __GBSettingsMain: S,
    localStorage: storage,
    addEventListener: function () {},
    dispatchEvent: function (event) {
      events.push(event);
    },
    document: {
      addEventListener: function () {},
      dispatchEvent: function () {},
      getElementById: function (id) {
        return id === "settings-index-all-drive" ? checkbox : null;
      },
      querySelectorAll: function () {
        return [];
      },
    },
    CustomEvent: function (type, init) {
      return { type: type, detail: init && init.detail };
    },
    history: { replaceState: function () {} },
    console: console,
  };
  sandbox.window = sandbox;
  vm.runInNewContext(fs.readFileSync(MODULE, "utf8"), sandbox, {
    filename: MODULE,
  });
  return { S: S, window: sandbox, checkbox: checkbox, events: events };
}

// 1. Default on when the key is absent.
let loaded = loadModule(makeStorage());
loaded.S.syncIndexAllDrive();
assertEqual(loaded.checkbox.checked, true, "default is on when the key is absent");

// 2. Explicit false from a previous session survives the reload.
loaded = loadModule(makeStorage({ "gb-index-all-drive": "false" }));
loaded.S.syncIndexAllDrive();
assertEqual(loaded.checkbox.checked, false, "stored off survives a reload");

// 3. Turning it on persists true and notifies the shell.
loaded.checkbox.checked = true;
loaded.window.toggleIndexAllDrive(loaded.checkbox);
assertEqual(loaded.window.localStorage.getItem("gb-index-all-drive"), "true", "on is persisted");
assertEqual(loaded.events.length, 1, "one change event is fired");
assertEqual(loaded.events[0].type, "gb-index-all-drive-changed", "event name");
assertEqual(loaded.events[0].detail.enabled, true, "event says enabled");

// 4. Turning it off persists false.
loaded.checkbox.checked = false;
loaded.window.toggleIndexAllDrive(loaded.checkbox);
assertEqual(loaded.window.localStorage.getItem("gb-index-all-drive"), "false", "off is persisted");
assertEqual(loaded.events[1].detail.enabled, false, "event says disabled");

console.log(`drive_knowledge_toggle: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
