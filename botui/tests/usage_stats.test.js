"use strict";
/* GB6-25 (#1576) — usage counts. Zero-dependency test
 * (run: node tests/usage_stats.test.js)
 *
 * Guards the two defects recorded in the issue: a streamed reply counted once
 * per frame, and a user message whose text was already cleared from the
 * composer when the usage event fired. Cost is never derived locally. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(__dirname, "..", "ui", "suite", "js", "usage-stats.js");

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
};
const listeners = { window: {}, document: {} };
const window = {
  localStorage: localStorage,
  addEventListener: (type, fn) => { listeners.window[type] = fn; },
};
const document = {
  getElementById: () => null,
  addEventListener: (type, fn) => { listeners.document[type] = fn; },
};
const sandbox = {
  window: window,
  document: document,
  localStorage: localStorage,
  setInterval: function () {},
  clearInterval: function () {},
  Date: Date,
  Math: Math,
  JSON: JSON,
  Object: Object,
  Array: Array,
  Number: Number,
  String: String,
  console: console,
};
vm.runInNewContext(fs.readFileSync(MODULE, "utf8"), sandbox, { filename: MODULE });
const usage = window.GBUsageStats;

// 1. A streamed reply is counted once per message id.
usage.track("bot", 400, "auto", "bot:message-1");
usage.track("bot", 400, "auto", "bot:message-1");
usage.track("bot", 400, "auto", "bot:message-1");
let summary = usage.getSummary(30);
check(summary.messages_received === 1, "a streamed reply counts once, not once per frame");
check(summary.tokens_estimated === 100, "tokens are estimated once for the same message");

// 2. Distinct replies still count.
usage.track("bot", 800, "auto", "bot:message-2");
summary = usage.getSummary(30);
check(summary.messages_received === 2, "a different message id is counted");
check(summary.tokens_estimated === 300, "token estimate adds up");

// 3. A user message uses the length carried by the event, because the composer
//    has already been cleared when the event fires.
listeners.window["gb-chat-message-sent"]({ detail: { content_length: 40, model: "fast" } });
summary = usage.getSummary(30);
check(summary.messages_sent === 1, "the sent message is counted");
check(summary.tokens_estimated === 310, "the sent message contributes its real length");

// 4. No cost is invented locally.
check(summary.cost === undefined, "local summary has no cost field");
const source = fs.readFileSync(MODULE, "utf8");
check(!/TOKEN_COST_PER_1K/.test(source), "no hardcoded price table remains");
check(!/cost_per_(1k|1m)|per_1k_tokens/i.test(source), "no per-token price constant remains");

console.log(`usage_stats: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
