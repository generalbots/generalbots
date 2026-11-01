"use strict";
/* #1593 — web-search composer toggle. Zero-dependency unit tests
 * (run: node tests/web_search_toggle.test.js)
 *
 * Loads the IIFE with a minimal window/document/localStorage shim and asserts
 * the state contract: on by default, persisted per bot+user, aria-pressed and
 * the visible label always agree with the stored value. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(
  __dirname,
  "..",
  "ui",
  "suite",
  "chat",
  "modules",
  "45_web_search_toggle.js",
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
    data: data,
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(data, key) ? data[key] : null;
    },
    setItem: function (key, value) {
      data[key] = String(value);
    },
  };
}

function makeButton() {
  const attributes = {};
  const classes = new Set();
  const listeners = {};
  return {
    title: "",
    dataset: {},
    attributes: attributes,
    textContent: "Web on",
    addEventListener: function (type, handler) {
      listeners[type] = handler;
    },
    setAttribute: function (name, value) {
      attributes[name] = String(value);
    },
    getAttribute: function (name) {
      return attributes[name];
    },
    classList: {
      add: function (name) { classes.add(name); },
      remove: function (name) { classes.delete(name); },
      contains: function (name) { return classes.has(name); },
      toggle: function (name, force) {
        if (force === undefined) {
          if (classes.has(name)) classes.delete(name);
          else classes.add(name);
        } else if (force) {
          classes.add(name);
        } else {
          classes.delete(name);
        }
      },
    },
    querySelector: function (selector) {
      if (selector === ".web-search-label") return label;
      return null;
    },
    click: function () {
      if (listeners.click) listeners.click({ preventDefault: function () {} });
    },
  };
}

let label = null;

function loadModule(options) {
  const opts = options || {};
  const storage = opts.storage || makeStorage();
  const button = makeButton();
  label = { textContent: "Web on" };
  const window = {
    __INITIAL_BOT_NAME__: opts.bot || "demo",
    ChatState: { currentUserId: opts.user || "user-1" },
    localStorage: storage,
    dispatchEvent: function () {},
  };
  const sandbox = {
    window: window,
    document: {
      readyState: "complete",
      getElementById: function (id) {
        return id === "webSearchBtn" ? button : null;
      },
      addEventListener: function () {},
    },
    CustomEvent: function (type, init) {
      return { type: type, detail: init && init.detail };
    },
    console: console,
  };
  vm.runInNewContext(fs.readFileSync(MODULE, "utf8"), sandbox, {
    filename: MODULE,
  });
  return { api: window.ChatWebSearch, button: button, storage: storage, window: window };
}

// 1. Default is on, with the visible state matching.
let loaded = loadModule();
assertEqual(loaded.api.isEnabled(), true, "default is on");
assertEqual(loaded.button.getAttribute("aria-pressed"), "true", "aria-pressed is true by default");
assertEqual(loaded.button.title.indexOf("Web search: on"), 0, "tooltip says on");
assertEqual(loaded.button.classList.contains("is-off"), false, "not muted by default");

// 2. Turning it off persists and paints the off state.
loaded.api.setEnabled(false);
assertEqual(loaded.api.isEnabled(), false, "setEnabled(false) flips the state");
assertEqual(loaded.button.getAttribute("aria-pressed"), "false", "aria-pressed follows the state");
assertEqual(loaded.button.title.indexOf("Web search: off"), 0, "tooltip says off");
assertEqual(loaded.button.classList.contains("is-off"), true, "off state is muted");
assertEqual(label.textContent, "Web off", "visible label says off");
assertEqual(loaded.storage.getItem(loaded.api.storageKey()), "false", "choice is persisted");

// 3. A reload for the same bot+user starts off.
loaded = loadModule({ storage: loaded.storage });
assertEqual(loaded.api.isEnabled(), false, "reload keeps the persisted off state");
assertEqual(loaded.button.getAttribute("aria-pressed"), "false", "reload paints off");

// 4. Clicking the button toggles (keyboard Enter/Space reach the same handler).
loaded.button.click();
assertEqual(loaded.api.isEnabled(), true, "click turns search back on");
assertEqual(loaded.button.getAttribute("aria-pressed"), "true", "click repaints aria-pressed");

// 5. Per-bot/user key: another user starts at the default.
const other = loadModule({ storage: loaded.storage, user: "user-2" });
assertEqual(other.api.isEnabled(), true, "a different user keeps the default on");
assertEqual(
  loaded.api.storageKey() !== other.api.storageKey(),
  true,
  "storage keys are scoped per bot and user",
);

console.log(`web_search_toggle: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
