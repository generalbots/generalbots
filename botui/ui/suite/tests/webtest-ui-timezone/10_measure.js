/**
 * [BASELINE-OBSERVE] measurements of the CURRENT tree for #2389.
 *
 * These lock what is actually shipped today so a future change surfaces for
 * review. They are observations, not a ratchet: no baseline file is read or
 * written, and no target state is asserted. The issue-era figures (8-zone
 * table, 98 date_naive sites) are recorded in the test names as history only.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var repoFile = h.repoFile;
var suiteFile = h.suiteFile;
var repoExists = h.repoExists;
var count = h.count;
var countInTree = h.countInTree;
var readStringArray = h.readStringArray;

var SETTINGS_DIR = "botserver/crates/botsettings/src/";
var DIGEST = SETTINGS_DIR + "settings_digest.rs";
var TZ = SETTINGS_DIR + "timezone.rs";
var CARGO = "botserver/crates/botsettings/Cargo.toml";
var SETTINGS_JS = "settings/settings-actions.js";
var CLOCK_JS = "clock/clock.js";
var AUTO_JS = "automations/automations-state.js";
var LANGUAGE_HTML = "settings/partials/02_language.html";

var digestSrc = repoFile(DIGEST);
var tzSrc = repoExists(TZ) ? repoFile(TZ) : "";
var cargoSrc = repoFile(CARGO);
var settingsJs = suiteFile(SETTINGS_JS);
var clockJs = suiteFile(CLOCK_JS);
var autoJs = suiteFile(AUTO_JS);
var languageHtml = suiteFile(LANGUAGE_HTML);

// Fixed-offset arms of the shape `"Zone/Name" => -180,` (the #2389 table).
var FIXED_ARM_RE = /"[^"]+"\s*=>\s*-?\d+/g;
var DATE_NAIVE_RE = /date_naive\(\)/g;
var UTC_NOW_DATE_RE = /Utc::now\(\)\.date_naive/g;

var settingsFallbacks = readStringArray(settingsJs, "TIMEZONE_FALLBACKS") || [];
var clockFallbacks = readStringArray(clockJs, "FALLBACK_ZONES") || [];
var autoZones = readStringArray(autoJs, "AUTO_TIMEZONES") || [];

function supportedValuesOfFiles() {
  return [settingsJs, clockJs].filter(function (src) {
    return src.indexOf("Intl.supportedValuesOf") >= 0;
  }).length;
}

var METRICS = {
  rustFixedOffsetZones: count(digestSrc, FIXED_ARM_RE),
  timezoneModule: repoExists(TZ) ? 1 : 0,
  chronoTzDep: cargoSrc.indexOf("chrono-tz") >= 0 ? 1 : 0,
  ianaTzDep: cargoSrc.indexOf("iana-time-zone") >= 0 ? 1 : 0,
  zoneIanaVariant: tzSrc.indexOf("Iana(Tz)") >= 0 ? 1 : 0,
  parseTz: tzSrc.indexOf("parse::<Tz>()") >= 0 ? 1 : 0,
  warnOnUnknown: tzSrc.indexOf("log::warn!") >= 0 &&
    tzSrc.indexOf("falling back to UTC") >= 0 ? 1 : 0,
  localDateHelper: tzSrc.indexOf("pub fn local_date") >= 0 &&
    tzSrc.indexOf("pub fn today_in") >= 0 ? 1 : 0,
  dateNaiveCrates: countInTree(h.REPO_ROOT + "/botserver/crates", DATE_NAIVE_RE),
  dateNaiveBotserver: countInTree(h.REPO_ROOT + "/botserver", DATE_NAIVE_RE),
  dateNaiveRepo: countInTree(h.REPO_ROOT, DATE_NAIVE_RE),
  utcNowDateNaive: countInTree(h.REPO_ROOT + "/botserver", UTC_NOW_DATE_RE),
  uiSettingsFallbacks: settingsFallbacks.length,
  uiClockFallbacks: clockFallbacks.length,
  uiAutomationZones: autoZones.length,
  uiLanguageStaticOptions: count(languageHtml, /data-i18n\s*=\s*"settings-timezone-/g),
  uiSupportedValuesOf: supportedValuesOfFiles(),
  automationSilentUtcDefault: autoJs.indexOf("includes(localTz) ? localTz : \"UTC\"") >= 0 ? 1 : 0,
  runtimeIanaZones: 0,
  runtimeNyWinterOffset: 0,
  runtimeNySummerOffset: 0,
  runtimeUnknownZoneOffset: 0,
};

h.setMetrics(METRICS);

test("[BASELINE-OBSERVE] #2389 fixed-offset table is gone from settings_digest.rs (was 8)", function () {
  equal(METRICS.rustFixedOffsetZones, 0, "fixed-offset arms");
});

test("[BASELINE-OBSERVE] dedicated timezone.rs with chrono-tz and iana-time-zone ships", function () {
  equal(METRICS.timezoneModule, 1, "timezone.rs");
  equal(METRICS.chronoTzDep, 1, "chrono-tz dependency");
  equal(METRICS.ianaTzDep, 1, "iana-time-zone dependency");
});

test("[BASELINE-OBSERVE] IANA resolution, local_date helper and unknown-zone warning ship", function () {
  equal(METRICS.zoneIanaVariant, 1, "Zone::Iana(Tz)");
  equal(METRICS.parseTz, 1, "parse::<Tz>()");
  equal(METRICS.localDateHelper, 1, "local_date/today_in");
  equal(METRICS.warnOnUnknown, 1, "log::warn on unresolvable zone");
});

test("[BASELINE-OBSERVE] 107/108/109 date_naive() sites remain (crates/botserver/repo); 42 UTC-now", function () {
  equal(METRICS.dateNaiveCrates, 107, "botserver/crates date_naive count");
  equal(METRICS.dateNaiveBotserver, 108, "botserver date_naive count");
  equal(METRICS.dateNaiveRepo, 109, "repo date_naive count");
  equal(METRICS.utcNowDateNaive, 42, "Utc::now().date_naive count");
  assert(METRICS.dateNaiveCrates <= METRICS.dateNaiveBotserver &&
    METRICS.dateNaiveBotserver <= METRICS.dateNaiveRepo,
  "date_naive counts are not nested");
});

test("[BASELINE-OBSERVE] suite timezone lists: 12 settings / 10 clock / 15 automations / 5 static options", function () {
  equal(METRICS.uiSettingsFallbacks, 12, "TIMEZONE_FALLBACKS");
  equal(METRICS.uiClockFallbacks, 10, "FALLBACK_ZONES");
  equal(METRICS.uiAutomationZones, 15, "AUTO_TIMEZONES");
  equal(METRICS.uiLanguageStaticOptions, 5, "static language options");
});

test("[BASELINE-OBSERVE] 2 suite helpers consult Intl.supportedValuesOf; automation defaults unknown zone to UTC", function () {
  equal(METRICS.uiSupportedValuesOf, 2, "supportedValuesOf consumers");
  equal(METRICS.automationSilentUtcDefault, 1, "automation UTC default");
});

test("[BASELINE-OBSERVE] every measured metric is an integer", function () {
  Object.keys(METRICS).forEach(function (key) {
    assert(Number.isInteger(METRICS[key]), key + " is not an integer: " + METRICS[key]);
  });
});
