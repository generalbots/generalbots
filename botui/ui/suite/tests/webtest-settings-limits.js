/**
 * Offline static guard for the Settings "Usage & limits" port (from gbgov's
 * 00_engine_names.js + 10k_limits_refresh.js / 10i_settings_limits.js).
 *
 * Run with: node botui/ui/suite/tests/webtest-settings-limits.js
 *
 * Loads the two shipped modules in a VM sandbox (no browser, no HTTP) and
 * checks the shared engine vocabulary and the pure limit helpers. Then scans
 * the wiring in index.html and proves every i18n key used by the port exists
 * in all eight locale catalogs. No runtime PASS is fabricated.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE = path.resolve(__dirname, "..");
var SETTINGS = path.join(SUITE, "settings");
var LOCALES = path.resolve(SUITE, "..", "..", "..", "botlib", "locales");
var LOCALE_IDS = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

var PASS = 0;
var FAIL = 0;
function check(name, condition, detail) {
  if (condition) { PASS++; console.log("[PASS] " + name); }
  else { FAIL++; console.log("[FAIL] " + name + (detail ? " — " + detail : "")); }
}
function read(rel) { return fs.readFileSync(rel, "utf8"); }
function lines(rel) { return read(rel).split(/\r?\n/).length; }

/* A gbT stub that records keys and returns the English fallback (or the key
   when no fallback is given) so pure helpers are deterministic. */
function gbT(keys) {
  return function (key, fallback) {
    keys.push(key);
    return fallback && fallback.length ? fallback : key;
  };
}
function load(file, sandbox) {
  vm.createContext(sandbox);
  vm.runInContext(read(file), sandbox, { filename: file });
}

var nameKeys = [];
var nameBox = { window: {}, console: console };
nameBox.window.gbT = gbT(nameKeys);
load(path.join(SETTINGS, "settings-engine-names.js"), nameBox);
var N = nameBox.window.GBSettingsEngineNames;

check("GBSettingsEngineNames published", !!N);
check("four engine types declared", !!(N && JSON.stringify(N.TYPES) === JSON.stringify(["local", "free", "own_key", "acp"])));
if (N) {
  check("group tier includes subscription", N.engineName("acp", "group") === "Subscriptions", N.engineName("acp", "group"));
  check("short tier includes subscription", N.engineName("acp", "short") === "Subscription");
  check("detail tier reuses mdl-own", N.engineName("own_key", "detail") === "Your server or provider key");
  check("detail tier names subscription agent", N.engineName("acp", "detail") === "Subscription agent");
  N.engineName("local", "detail");
  check("limit kind subscription maps to acp", N.engineNameByLimitKind("subscription", "group") === "Subscriptions");
  check("limit kind api_key maps to own_key", N.engineNameByLimitKind("api_key", "short") === "Own key");
  check("unknown type falls back", N.engineName("nope", "short") === "nope");
  check("detail tier asks gbT for eng-detail-acp", nameKeys.indexOf("eng-detail-acp") >= 0);
  check("detail tier asks gbT for mdl-local", nameKeys.indexOf("mdl-local") >= 0);
}

var limitKeys = [];
var limitBox = { window: { GBSettingsEngineNames: N }, console: console, Promise: Promise, Math: Math, Date: Date, String: String, Number: Number };
limitBox.window.gbT = gbT(limitKeys);
load(path.join(SETTINGS, "settings-limits.js"), limitBox);
var L = limitBox.window.GBSettingsLimits;

check("GBSettingsLimits pure helpers published", !!L);
check("loadLimitsPanel hook published", typeof limitBox.window.loadLimitsPanel === "function");
if (L) {
  check("pctOf reads used_percent", L.pctOf({ used_percent: 90 }) === 90);
  check("pctOf reads limit/remaining", L.pctOf({ limit: 100, remaining: 40 }) === 60);
  check("pctOf clamps over 100", L.pctOf({ used_percent: 140 }) === 100);
  check("pctOf returns null without data", L.pctOf({ name: "5h" }) === null);
  check("tightest returns highest window", L.tightest({ windows: [{ used_percent: 30 }, { used_percent: 75 }] }) === 75);
  check("status text is localized", L.statusText("rejected") === "lim-full");
  check("window name is localized", L.windowName("weekly") === "Weekly limit");
  check("updated text carries a time", /Updated at \d\d:\d\d/.test(L.updatedText(Date.now())));
  check("reset text empty for the past", L.resetText("2000-01-01T00:00:00Z") === "");
  check("value text renders remaining", L.valueText({ limit: 100, remaining: 40 }) === "40 of 100 left");
  var bars = L.bars({ windows: [{ name: "5h", used_percent: 90, status: "allowed_warning", resets_at: "2999-01-02T10:00:00" }] });
  check("bars drops windows with no data", L.bars({ windows: [{ name: "5h" }] }).length === 0);
  check("bars builds one row with a share", bars.length === 1 && bars[0].pct === 90, JSON.stringify(bars));
  check("bar status is localized", bars.length === 1 && bars[0].value.indexOf("lim-close") >= 0);
  check("kind map has four engines", JSON.stringify(Object.keys(L.KIND).sort()) === JSON.stringify(["acp", "free", "local", "own_key"]));
}

/* Wiring in index.html and the partial. */
var index = read(path.join(SETTINGS, "index.html"));
check("index loads engine vocabulary", index.indexOf("/suite/settings/settings-engine-names.js") >= 0);
check("index loads limits module", index.indexOf("/suite/settings/settings-limits.js") >= 0);
check("index loads limits css", index.indexOf("/suite/settings/settings-limits.css") >= 0);
check("index has limits nav anchor", index.indexOf('href="#limits"') >= 0);
check("index loads limits partial", index.indexOf("/suite/settings/partials/17_limits.html") >= 0);
var partial = read(path.join(SETTINGS, "partials", "17_limits.html"));
check("partial owns limits-section", partial.indexOf('id="limits-section"') >= 0);
var model = read(path.join(SETTINGS, "settings-model.js"));
check("model panel delegates to shared vocabulary", model.indexOf("GBSettingsEngineNames") >= 0 && model.indexOf('names.engineName(id, "detail")') >= 0);

/* File size gate (450 lines) and no panics / CDN in the new files. */
["settings-engine-names.js", "settings-limits.js", "settings-limits.css", "partials/17_limits.html"].forEach(function (rel) {
  var file = path.join(SETTINGS, rel);
  check("under 450 lines: " + rel, lines(file) < 450, String(lines(file)));
  var src = read(file);
  check("no CDN link: " + rel, !/(https?:)?\/\/(cdn|unpkg|jsdelivr)/i.test(src));
});

/* Every i18n key used by the port must exist in all eight catalogs. */
var used = {};
function collect(src) {
  var re = /["']((?:(?:lim|eng|mdl)-[a-z0-9-]+|settings-limits))["']/g;
  var m;
  while ((m = re.exec(src))) { used[m[1]] = true; }
}
collect(read(path.join(SETTINGS, "settings-engine-names.js")));
collect(read(path.join(SETTINGS, "settings-limits.js")));
collect(partial);
LOCALE_IDS.forEach(function (loc) {
  var catalog = read(path.join(LOCALES, loc, "ui-settings.ftl"));
  var missing = Object.keys(used).filter(function (key) {
    return catalog.indexOf("\n" + key + " = ") < 0 && catalog.indexOf("^" + key + " = ") < 0;
  });
  check("locale " + loc + " has all " + Object.keys(used).length + " keys", missing.length === 0, missing.join(", "));
});

console.log("=".repeat(64));
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("=".repeat(64));
process.exitCode = FAIL === 0 ? 0 : 1;
