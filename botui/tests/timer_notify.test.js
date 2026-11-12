"use strict";
/* Timer bot-notify (#2261 TIMER-01) — zero-dependency unit test.
 * Run: node tests/timer_notify.test.js
 * Loads ui/suite/timer/timer.js in a minimal window/document shim and asserts
 * that a completed session reaches the chat send path (window.sendMessage),
 * and that a closed chat surface is opened and then retried. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const SRC = path.join(__dirname, "..", "ui", "suite", "timer", "timer.js");

let passed = 0;
let failed = 0;

function assertEqual(actual, expected, label) {
  if (actual === expected) {
    passed += 1;
  } else {
    failed += 1;
    console.error("FAIL " + label + "\n  expected: " + JSON.stringify(expected) + "\n  actual:   " + JSON.stringify(actual));
  }
}

function assertTrue(cond, label) {
  if (cond) passed += 1;
  else {
    failed += 1;
    console.error("FAIL " + label);
  }
}

function makeEl(id) {
  return {
    id: id,
    textContent: "",
    dataset: {},
    style: {},
    classList: { add: function () {}, remove: function () {}, toggle: function () {} },
    addEventListener: function () {},
  };
}

function load(hasComposer) {
  const elements = {};
  ["timerDisplay", "timerStart", "timerReset", "timerLabel", "timerSessions"].forEach(function (id) {
    elements[id] = makeEl(id);
  });
  if (hasComposer) elements.messageInput = makeEl("messageInput");
  const sent = [];
  const events = [];
  const intervals = [];
  const sandbox = {
    window: {},
    document: {
      readyState: "loading",
      addEventListener: function () {},
      getElementById: function (id) { return elements[id] || null; },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      title: "",
    },
    localStorage: {
      store: {},
      getItem: function (k) { return Object.prototype.hasOwnProperty.call(this.store, k) ? this.store[k] : null; },
      setItem: function (k, v) { this.store[k] = String(v); },
    },
    console: console,
    Math: Math,
    String: String,
    Number: Number,
    Date: Date,
    setInterval: function (fn, ms) { intervals.push({ fn: fn, ms: ms }); return intervals.length; },
    clearInterval: function (id) { intervals[id - 1] = null; },
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    CustomEvent: function (name, opts) { this.type = name; this.detail = (opts || {}).detail; },
    Notification: function () {},
  };
  sandbox.window = sandbox;
  sandbox.sendMessage = function (text) { sent.push(text); };
  sandbox.dispatchEvent = function (ev) { events.push(ev); };
  sandbox.openDeepLinkCalls = [];
  sandbox.openDeepLink = function (appId) { sandbox.openDeepLinkCalls.push(appId); };
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(SRC, "utf8"), sandbox, { filename: SRC });
  return { timer: sandbox.GBTimer, sent: sent, events: events, intervals: intervals, box: sandbox };
}

// 1. Chat surface mounted: the completion is handed straight to the bot.
const open = load(true);
assertTrue(typeof open.timer.notify === "function", "notify is exposed on GBTimer");
assertEqual(open.timer.notify("[Timer] Focus finished (25 min)."), true,
  "notify reports delivery when the composer exists");
assertEqual(open.sent.length, 1, "the bot received exactly one message");
assertTrue(String(open.sent[0]).indexOf("[Timer]") === 0, "the message is tagged as a timer event");
assertEqual(open.events.length, 1, "a gb-timer-complete event is dispatched");
assertEqual(open.events[0].type, "gb-timer-complete", "event name");
assertEqual(open.events[0].detail.delivered, true, "event reports delivery");
assertEqual(open.box.openDeepLinkCalls.length, 0, "no chat window is opened when chat is up");

// 2. Chat surface closed: open it, then send as soon as the composer appears.
const closed = load(false);
assertEqual(closed.timer.notify("[Timer] Short Break finished (5 min)."), false,
  "notify reports no delivery while chat is closed");
assertEqual(closed.box.openDeepLinkCalls.length, 1, "the chat app is opened");
assertEqual(closed.box.openDeepLinkCalls[0], "chat", "the chat app is the one opened");
assertEqual(closed.intervals.length, 1, "one bounded retry is scheduled");
assertEqual(closed.sent.length, 0, "nothing is sent before the composer exists");

// Composer still missing -> retry keeps waiting (bounded).
closed.intervals[0].fn();
assertEqual(closed.sent.length, 0, "a missing composer does not send");

// The composer appears -> the pending notification is delivered.
const send = closed.box.sendMessage;
closed.box.document.getElementById = (function (original) {
  return function (id) {
    if (id === "messageInput") return makeEl("messageInput");
    return original.call(closed.box.document, id);
  };
})(closed.box.document.getElementById);
closed.intervals[0].fn();
assertEqual(closed.sent.length, 1, "the retry delivers once the composer exists");
assertTrue(send === closed.box.sendMessage, "the send path is stable across retries");

// 3. A completed session (tick -> stop(true)) goes through the same path.
const run = load(true);
run.timer.setMode(0, "Focus");
run.timer.start();
assertEqual(run.intervals.length, 1, "the countdown interval is armed");
run.intervals[0].fn();
assertEqual(run.sent.length, 1, "time's up notifies the bot");
assertTrue(String(run.sent[0]).indexOf("Focus finished") > -1, "the label and duration travel with it");
assertEqual(run.box.document.getElementById("timerSessions").textContent, "1 session completed",
  "the session counter still works");

console.log((failed ? "FAILED " : "OK ") + passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
