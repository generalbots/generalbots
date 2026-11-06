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

// 1. Default is off, with the visible state matching. The desktop assistant
// answers from the model and the knowledge base unless the user asks for live
// sources (the module comment documents the deliberate flip).
let loaded = loadModule();
assertEqual(loaded.api.isEnabled(), false, "default is off");
assertEqual(loaded.button.getAttribute("aria-pressed"), "false", "aria-pressed is false by default");
assertEqual(loaded.button.title.indexOf("Web search: off"), 0, "tooltip says off");
assertEqual(loaded.button.classList.contains("is-off"), true, "muted by default");

// 2. Turning it on persists and paints the on state.
loaded.api.setEnabled(true);
assertEqual(loaded.api.isEnabled(), true, "setEnabled(true) flips the state");
assertEqual(loaded.button.getAttribute("aria-pressed"), "true", "aria-pressed follows the state");
assertEqual(loaded.button.title.indexOf("Web search: on"), 0, "tooltip says on");
assertEqual(loaded.button.classList.contains("is-off"), false, "on state is not muted");
assertEqual(label.textContent, "Web on", "visible label says on");
assertEqual(loaded.storage.getItem(loaded.api.storageKey()), "true", "choice is persisted");

// 3. A reload for the same bot+user starts on.
loaded = loadModule({ storage: loaded.storage });
assertEqual(loaded.api.isEnabled(), true, "reload keeps the persisted on state");
assertEqual(loaded.button.getAttribute("aria-pressed"), "true", "reload paints on");

// 4. Clicking the button toggles (keyboard Enter/Space reach the same handler).
loaded.button.click();
assertEqual(loaded.api.isEnabled(), false, "click turns search back off");
assertEqual(loaded.button.getAttribute("aria-pressed"), "false", "click repaints aria-pressed");

// 5. Per-bot/user key: another user starts at the default.
const other = loadModule({ storage: loaded.storage, user: "user-2" });
assertEqual(other.api.isEnabled(), false, "a different user keeps the default off");
assertEqual(
  loaded.api.storageKey() !== other.api.storageKey(),
  true,
  "storage keys are scoped per bot and user",
);

console.log(`web_search_toggle: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
