"use strict";
/* Window fullscreen (#2104) — zero-dependency unit test.
 * run: node tests/window_fullscreen.test.js
 *
 * Desktop windows only offered minimize/maximize/close, so a user had to
 * resize by hand to read a large app. These assertions pin the new contract:
 * the button and F11 toggle a real fullscreen (Fullscreen API in the browser,
 * the Tauri window API in the desktop shell), the CSS fallback covers the
 * taskbar when the API is missing, Escape leaves first, the icon/tooltip
 * follow the platform (fullscreenchange), keys are ignored while typing, and
 * fixed popups keep no fullscreen at all.
 */
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const SRC = path.join(__dirname, "..", "ui", "suite", "js", "window-manager", "08_fullscreen.js");

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else { failed++; console.error("FAIL " + label); }
}

/* ── minimal DOM ─────────────────────────────────────────────────────────── */

function makeDot() {
  const attrs = {};
  return {
    attrs,
    setAttribute(name, value) { attrs[name] = String(value); },
    getAttribute(name) { return attrs[name]; },
  };
}

const elements = {};

function makeWindowEl(id, opts) {
  opts = opts || {};
  const classes = new Set();
  const dot = makeDot();
  const el = {
    id: "window-" + id,
    classes,
    style: {},
    dot,
    classList: {
      add: (name) => classes.add(name),
      remove: (name) => classes.delete(name),
      toggle: (name, on) => (on ? classes.add(name) : classes.delete(name)),
      contains: (name) => classes.has(name),
    },
    querySelector: (sel) => (sel === ".window-dot-fullscreen" ? dot : null),
  };
  if (opts.fullscreenApi) {
    el.requestFullscreen = () => {
      sandbox.document.fullscreenElement = el;
      return Promise.resolve();
    };
  }
  elements[el.id] = el;
  return el;
}

const listeners = {};
function on(type, fn) { (listeners[type] = listeners[type] || []).push(fn); }
function fire(type, event) { (listeners[type] || []).forEach((fn) => fn(event)); }

function keyEvent(key, target, extra) {
  return Object.assign({
    key,
    target: target || { nodeType: 1, tagName: "DIV" },
    defaultPrevented: false,
    preventDefault() { this.defaultPrevented = true; },
    stopPropagation() {},
  }, extra || {});
}

/* ── sandbox ─────────────────────────────────────────────────────────────── */

const sandbox = { console, Date, Promise, JSON };
sandbox.window = {
  navigator: { userAgent: "Mozilla/5.0 (X11; Linux x86_64)" },
  addEventListener() {},
  console,
};
sandbox.document = {
  fullscreenElement: null,
  getElementById: (id) => elements[id] || null,
  querySelector: (sel) => {
    if (sel.indexOf("window-fullscreen-fallback") === -1) return null;
    return Object.values(elements).find((el) => el.classes.has("window-fullscreen-fallback")) || null;
  },
  addEventListener: on,
  exitFullscreen() {
    sandbox.document.fullscreenElement = null;
    return Promise.resolve();
  },
};
sandbox.navigator = sandbox.window.navigator;

const WM = {
  WindowManager: function WindowManager() {},
  defineMethods: (proto, methods) => Object.assign(proto, methods),
};
sandbox.window.__GBWindowManager = WM;
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(SRC, "utf8"), sandbox, { filename: "08_fullscreen.js" });

/* ── fixtures ────────────────────────────────────────────────────────────── */

const wm = new WM.WindowManager();
sandbox.window.WindowManager = wm; // 99_init.js publishes the singleton
const chatEl = makeWindowEl("chat", { fullscreenApi: true });
const notesEl = makeWindowEl("notes");
wm.openWindows = [
  { id: "chat", isFullscreen: false, noMaximize: false },
  { id: "notes", isFullscreen: false, noMaximize: false },
  { id: "settings", isFullscreen: false, noMaximize: false },
  { id: "vibe-newproject", isFullscreen: false, noMaximize: true },
];
wm.activeWindowId = "chat";
wm.focus = (id) => { wm.activeWindowId = id; };

function apiPath() {
  delete sandbox.window.__TAURI__;
}

/* ── 1. Fullscreen API path (browser) ───────────────────────────────────── */

apiPath();
wm.toggleFullscreen("chat");
check(wm.isFullscreen("chat") === true, "api: the window reports fullscreen");
check(!chatEl.classes.has("window-fullscreen-fallback"),
  "api: no CSS fallback needed when the browser grants it");
check(chatEl.classes.has("window-fullscreen-active"), "api: the active class marks the window");
check(chatEl.dot.getAttribute("title") === "Exit fullscreen", "api: tooltip flips to exit");
check(chatEl.dot.getAttribute("data-i18n-title") === "window-exit-fullscreen",
  "api: tooltip key flips for localization");
check(chatEl.dot.getAttribute("aria-pressed") === "true", "api: aria-pressed reports the state");
check(wm.fullscreenWindowId() === "chat", "api: the manager resolves the fullscreen window");

fire("fullscreenchange", {});
check(wm.isFullscreen("chat") === true, "api: its own fullscreenchange keeps the state");

wm.toggleFullscreen("chat");
check(wm.isFullscreen("chat") === false, "api: the button toggles back out");
check(chatEl.dot.getAttribute("title") === "Fullscreen", "api: tooltip restored on exit");

/* ── 2. CSS fallback (no Fullscreen API) ────────────────────────────────── */

wm.toggleFullscreen("notes");
check(notesEl.classes.has("window-fullscreen-fallback"),
  "fallback: the window covers the viewport (taskbar hidden)");
check(wm.isFullscreen("notes") === true, "fallback: the window reports fullscreen");
check(notesEl.dot.getAttribute("data-i18n-title") === "window-exit-fullscreen",
  "fallback: the control shows the exit state");

/* ── 3. one fullscreen window at a time ─────────────────────────────────── */

apiPath();
wm.toggleFullscreen("chat");
check(wm.isFullscreen("chat") === true && wm.isFullscreen("notes") === false,
  "single: a new fullscreen window releases the previous one");
check(notesEl.dot.getAttribute("title") === "Fullscreen", "single: the released control resets");
wm.toggleFullscreen("chat");

/* ── 4. Tauri desktop path (OS window) ──────────────────────────────────── */

const osCalls = [];
sandbox.window.__TAURI__ = {
  window: {
    getCurrentWindow: () => ({
      setFullscreen: (value) => { osCalls.push(value); return Promise.resolve(); },
      isFullscreen: () => Promise.resolve(true),
      onResized: () => Promise.resolve(() => {}),
    }),
  },
};
wm.toggleFullscreen("notes");
check(osCalls[0] === true, "desktop: the OS window is asked to go fullscreen");
check(notesEl.classes.has("window-fullscreen-fallback"),
  "desktop: the page hides its own taskbar/desktop chrome");
wm.toggleFullscreen("notes");
check(osCalls[1] === false, "desktop: the OS window is released on exit");
check(!notesEl.classes.has("window-fullscreen-fallback"), "desktop: the desktop chrome returns");
apiPath();

/* ── 5. keys ─────────────────────────────────────────────────────────────── */

wm.toggleFullscreen("notes");
const f11 = keyEvent("F11");
fire("keydown", f11);
check(wm.isFullscreen("notes") === false, "keys: F11 leaves fullscreen");
check(f11.defaultPrevented === true, "keys: F11 is consumed by the shell");

fire("keydown", keyEvent("F11"));
check(wm.isFullscreen("notes") === true, "keys: F11 enters fullscreen again");

const typing = keyEvent("F11", { nodeType: 1, tagName: "INPUT" });
fire("keydown", typing);
check(wm.isFullscreen("notes") === true && typing.defaultPrevented === false,
  "keys: F11 in a text field belongs to the field");

const editor = keyEvent("F11", {
  nodeType: 1,
  tagName: "DIV",
  closest: (sel) => (sel.indexOf("monaco") !== -1 ? {} : null),
});
fire("keydown", editor);
check(wm.isFullscreen("notes") === true && editor.defaultPrevented === false,
  "keys: F11 inside the code editor belongs to the editor");

const escape = keyEvent("Escape");
fire("keydown", escape);
check(wm.isFullscreen("notes") === false, "keys: Escape leaves fullscreen");
check(escape.defaultPrevented === true, "keys: Escape is consumed so modals do not also close");

/* ── 6. platform-side exit (fullscreenchange) ───────────────────────────── */

apiPath();
wm.toggleFullscreen("chat");
sandbox.document.fullscreenElement = null; // the UA dropped it (Esc/gesture)
fire("fullscreenchange", {});
check(wm.openWindows[0].isFullscreen === false, "sync: a platform exit clears the flag");
check(chatEl.dot.getAttribute("title") === "Fullscreen", "sync: the control follows the exit");
check(!chatEl.classes.has("window-fullscreen-active"), "sync: the active class is removed");

/* ── 7. fixed popups keep no fullscreen ─────────────────────────────────── */

wm.toggleFullscreen("vibe-newproject");
check(wm.openWindows[3].isFullscreen === false, "popup: noMaximize windows refuse fullscreen");

makeWindowEl("settings").classes.add("window-settings-modal");
wm.toggleFullscreen("settings");
check(wm.openWindows[2].isFullscreen === false, "popup: fixed modal dialogs refuse fullscreen");

console.log(`window_fullscreen: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
