"use strict";
/* Chat turn state machine (#2098) — zero-dependency unit test.
 * run: node tests/chat_turn_state.test.js
 *
 * The indicator used to be hidden by the first BOT_RESPONSE frame, so a slow
 * or tool-using turn looked dead. These assertions pin the new contract:
 * queued/thinking/tool keep the element, empty frames never hide it, visible
 * content replaces it with the streaming bubble, and a late/duplicate frame
 * from the previous turn cannot end the live one.
 */
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const CHAT_DIR = path.join(__dirname, "..", "ui", "suite", "chat");

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else { failed++; console.error("FAIL " + label); }
}

/* ── minimal DOM ─────────────────────────────────────────────────────────── */

class ClassList {
  constructor(node) { this.node = node; }
  add(...names) { this.node.className = merge(this.node.className, names); }
  remove(...names) {
    const drop = new Set(names);
    this.node.className = this.node.className.split(/\s+/)
      .filter((name) => name && !drop.has(name)).join(" ");
  }
  contains(name) { return this.node.className.split(/\s+/).indexOf(name) !== -1; }
}

function merge(current, names) {
  const out = current ? current.split(/\s+/) : [];
  names.forEach((name) => { if (name && out.indexOf(name) === -1) out.push(name); });
  return out.join(" ");
}

class DomNode {
  constructor(tagName) {
    this.tagName = String(tagName || "div").toUpperCase();
    this.id = "";
    this.className = "";
    this.children = [];
    this.parentNode = null;
    this.dataset = {};
    this.style = {};
    this.attributes = {};
    this.listeners = {};
    this.textContent = "";
    this.title = "";
    this.type = "";
    this.classList = new ClassList(this);
  }
  get isConnected() {
    let node = this;
    while (node.parentNode) node = node.parentNode;
    return node === document.root;
  }
  get lastElementChild() {
    return this.children.length ? this.children[this.children.length - 1] : null;
  }
  appendChild(child) {
    if (child.parentNode) child.parentNode.removeChild(child);
    child.parentNode = this;
    this.children.push(child);
    return child;
  }
  removeChild(child) {
    const at = this.children.indexOf(child);
    if (at !== -1) this.children.splice(at, 1);
    child.parentNode = null;
    return child;
  }
  setAttribute(name, value) { this.attributes[name] = String(value); }
  getAttribute(name) { return this.attributes[name]; }
  addEventListener(name, handler) { (this.listeners[name] || (this.listeners[name] = [])).push(handler); }
  matches(selector) {
    if (selector.charAt(0) === "#") return this.id === selector.slice(1);
    if (selector.charAt(0) === ".") return this.classList.contains(selector.slice(1));
    return this.tagName === selector.toUpperCase();
  }
  querySelector(selector) {
    for (const child of this.children) {
      if (child.matches(selector)) return child;
      const nested = child.querySelector(selector);
      if (nested) return nested;
    }
    return null;
  }
  click() {
    (this.listeners.click || []).forEach((handler) => handler({ preventDefault() {} }));
  }
}

const document = {
  root: null,
  createElement(tag) { return new DomNode(tag); },
  getElementById(id) {
    if (this.root && this.root.id === id) return this.root;
    return this.root ? this.root.querySelector("#" + id) : null;
  },
};

/* ── load both chat modules in one sandbox ──────────────────────────────── */

const sandbox = {
  console,
  Date,
  Math,
  JSON,
  setTimeout,
  clearTimeout,
  setInterval,
  clearInterval,
  document,
  navigator: { language: "en" },
  GBMultiChat: { tabs: {} },
  ChatState: { ws: null, currentSessionId: "session-1" },
};
sandbox.window = sandbox;
vm.createContext(sandbox);

for (const file of ["chat-turn-indicator.js", "chat-turn-state.js"]) {
  const source = fs.readFileSync(path.join(CHAT_DIR, file), "utf8");
  vm.runInContext(source, sandbox, { filename: file });
}

const GBTurn = sandbox.GBTurn;

function mountMessages() {
  const messages = new DomNode("div");
  messages.id = "messages";
  document.root = messages;
  return messages;
}

function indicator(messages) {
  return messages.querySelector("#thinking-indicator");
}

/* ── 1. start → queued, visible, turn-scoped ────────────────────────────── */

let messages = mountMessages();
GBTurn.start("hello", { sessionId: "session-1" });
check(GBTurn.state() === "queued", "start enters queued");
check(!!indicator(messages), "indicator is visible immediately after send");
check(indicator(messages).getAttribute("data-turn-state") === "queued",
  "indicator exposes data-turn-state=queued");
check(messages.dataset.turnState === "queued", "container exposes data-turn-state");

/* ── 2. thinking + empty frames keep it ─────────────────────────────────── */

check(GBTurn.thinking() === true, "thinking frame accepted");
check(GBTurn.state() === "thinking", "thinking state reached");
check(!!indicator(messages), "thinking frame does not hide the indicator");

GBTurn.observe({ message_type: 2, content: "", is_complete: false, thinking: true });
check(!!indicator(messages), "keepalive/empty bot frame keeps the indicator");

/* ── 3. tool/progress frame relabels instead of hiding ──────────────────── */

GBTurn.observe({ type: "TOOL_CALL_START", data: { name: "search" } });
check(GBTurn.state() === "tool", "tool call enters tool state");
check(!!indicator(messages), "tool frame keeps the indicator");
check(indicator(messages).querySelector(".thinking-label").textContent.indexOf("search") !== -1,
  "tool frame labels the running step");

/* ── 4. first visible content → streaming, spinner steps aside ──────────── */

check(GBTurn.visible() === true, "visible content accepted");
check(GBTurn.state() === "streaming", "streaming state reached");
check(!indicator(messages), "indicator yields to the streaming bubble");
check(GBTurn.visible() === true, "a second chunk stays streaming");

/* ── 5. completion ends the turn after the bubble ───────────────────────── */

check(GBTurn.complete({ content: "answer", session_id: "session-1" },
  { content: "answer" }) === true, "completion ends the turn");
check(GBTurn.state() === "idle", "turn is gone after completion");
check(GBTurn.complete({ content: "answer" }, { content: "answer" }) === false,
  "a second completion is refused");

/* ── 6. late frame from the previous turn cannot clear the new one ──────── */

GBTurn.start("next", { sessionId: "session-1" });
check(GBTurn.stale({ content: "answer", session_id: "session-1" }) === true,
  "duplicate completion of the previous turn is stale");
check(GBTurn.stale({ content: "answer", session_id: "other-session" }) === true,
  "frame from another conversation is stale");
check(GBTurn.stale({ content: "fresh", session_id: "session-1" }) === false,
  "the live turn's own completion is accepted");
check(!!indicator(messages), "stale frames leave the live indicator alone");
GBTurn.thinking();
check(GBTurn.stale({ content: "answer", session_id: "session-1" }) === false,
  "a legitimate repeat of the same text still lands once frames arrive");

/* ── 7. fail keeps an error element with retry ──────────────────────────── */

GBTurn.fail("Provider error");
check(GBTurn.state() === "error", "fail enters error state");
const errEl = indicator(messages);
check(!!errEl, "error keeps a visible element");
check(errEl.querySelector(".thinking-retry").style.display === "",
  "error shows the retry action");
check(errEl.querySelector(".thinking-stop").style.display === "none",
  "error hides the stop action");

/* ── 8. cancel ends the turn and tells the server ───────────────────────── */

const sent = [];
sandbox.ChatState.ws = { readyState: 1, send: (payload) => sent.push(JSON.parse(payload)) };
GBTurn.start("stop me", { sessionId: "session-1" });
check(GBTurn.cancel() === true, "cancel ends the active turn");
check(sent.length === 1 && sent[0].type === "run_cancel", "cancel sends run_cancel");
check(GBTurn.state() === "idle", "cancelled turn is not active");

/* ── 9. per-tab scoping ─────────────────────────────────────────────────── */

const stockPane = mountMessages();
const tabPane = new DomNode("div");
const tabMessages = new DomNode("div");
tabMessages.className = "chat-pane-messages";
tabPane.appendChild(tabMessages);
GBTurn.start("tab A", { tabId: "tab-a", pane: stockPane });
GBTurn.start("tab B", { tabId: "tab-b", pane: tabPane });
check(!!indicator(stockPane), "tab A indicator lives in its own pane");
check(!!indicator(tabMessages), "tab B indicator lives in its own pane");
check(indicator(stockPane).getAttribute("data-turn-id") !==
  indicator(tabMessages).getAttribute("data-turn-id"), "turns carry distinct ids");

/* ── 10. no resurrection after completion ───────────────────────────────── */

GBTurn.complete({ content: "done" }, { tabId: "tab-a", content: "done" });
check(GBTurn.thinking({ tabId: "tab-a" }) === false,
  "a frame right after completion does not resurrect the spinner");

GBTurn.forceHide();
check(!indicator(stockPane) && !indicator(tabMessages), "forceHide clears every pane");

console.log(`chat_turn_state: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
