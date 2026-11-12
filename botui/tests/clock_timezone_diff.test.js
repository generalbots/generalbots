"use strict";
/* Clock timezone diff (#2261 CLOCK-01) — zero-dependency unit test.
 * Run: node tests/clock_timezone_diff.test.js
 * Loads ui/suite/clock/clock.js in a minimal window/document shim and asserts
 * the pure offset/diff helpers. The reader's zone is pinned with TZ so the
 * "vs local" numbers are deterministic; November 12 2026 has no DST in any
 * zone used below. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");
const { spawnSync } = require("child_process");

const WANTED = "America/Sao_Paulo";
if (process.env.GB_CLOCK_LOCAL_TZ !== WANTED) {
  // The default zone is fixed by the process environment, before any Intl use.
  const child = spawnSync(process.execPath, [__filename], {
    stdio: "inherit",
    env: Object.assign({}, process.env, { TZ: WANTED, GB_CLOCK_LOCAL_TZ: WANTED }),
  });
  process.exit(child.status === null ? 1 : child.status);
}

const SRC = path.join(__dirname, "..", "ui", "suite", "clock", "clock.js");

let passed = 0;
let failed = 0;

function assertEqual(actual, expected, label) {
  if (actual === expected) {
    passed += 1;
  } else {
    failed += 1;
    console.error("FAIL " + label + "\n  expected: " + expected + "\n  actual:   " + actual);
  }
}

function assertTrue(cond, label) {
  if (cond) passed += 1;
  else {
    failed += 1;
    console.error("FAIL " + label);
  }
}

function loadApp() {
  const sandbox = {
    window: {},
    document: {
      readyState: "loading",
      addEventListener: function () {},
      getElementById: function () { return null; },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
    },
    Intl: Intl,
    Date: Date,
    Math: Math,
    String: String,
    Number: Number,
    console: console,
    setInterval: function () { return 0; },
    clearInterval: function () {},
  };
  sandbox.window = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(SRC, "utf8"), sandbox, { filename: SRC });
  return sandbox.GBClockApp;
}

const app = loadApp();
const AT = new Date("2026-11-12T12:00:00Z");

assertTrue(typeof app.timezoneDiff === "function", "timezoneDiff is exposed");

// Same zone as the reader: the diff is zero and must still be printed.
assertEqual(app.timezoneDiff("America/Sao_Paulo", AT), "GMT-3 · +0h vs local",
  "local zone shows GMT-3 and +0h");
assertEqual(app.timezoneDiff("", AT), "GMT-3 · +0h vs local",
  "no zone selected behaves as local");

// Ahead of the reader (positive diff).
assertEqual(app.timezoneDiff("Asia/Tokyo", AT), "GMT+9 · +12h vs local",
  "Tokyo is twelve hours ahead of Sao Paulo");
assertEqual(app.timezoneDiff("Europe/London", AT), "GMT+0 · +3h vs local",
  "London in November is GMT+0, three hours ahead");
assertEqual(app.timezoneDiff("Asia/Kolkata", AT), "GMT+5:30 · +8h 30m vs local",
  "half-hour offsets keep their minutes");
assertEqual(app.timezoneDiff("UTC", AT), "GMT+0 · +3h vs local",
  "UTC (bare GMT label) resolves to offset 0");

// Behind the reader (negative diff): the sign must survive.
assertEqual(app.timezoneDiff("America/Los_Angeles", AT), "GMT-8 · -5h vs local",
  "Los Angeles is five hours behind");

assertTrue(app.zoneOffsetMinutes("UTC", AT) === 0, "UTC offset minutes is zero");
assertTrue(app.zoneOffsetMinutes("America/Sao_Paulo", AT) === -180,
  "Sao Paulo offset minutes is -180");
assertTrue(app.zoneOffsetMinutes("", AT) === -180,
  "empty zone reports the reader's own offset");

console.log((failed ? "FAILED " : "OK ") + passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
