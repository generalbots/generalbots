/**
 * RUNTIME probes of REAL shipped suite JS (#2389).
 *
 * clock.js and settings-actions.js are loaded verbatim (no reimplementation)
 * into a Node vm. Intl is the real Node/ICU implementation, so IANA offsets and
 * DST transitions are whatever the shipped helpers actually compute. Wall time
 * is never read: every probe passes an explicit instant. Results are published
 * to METRICS so the report shows measured values, not assumed ones.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;

var clock = h.makeClockEngine();
var app = clock.app;

var WINTER = new Date(Date.UTC(2026, 0, 15, 12, 0, 0));
var SUMMER = new Date(Date.UTC(2026, 6, 15, 12, 0, 0));
var UNKNOWN = "Not/AZone";

test("[RUNTIME] shipped clock.js exports pure zoneOffsetMinutes/timezoneDiff", function () {
  assert(app, "window.GBClockApp missing after loading shipped clock.js");
  equal(typeof app.zoneOffsetMinutes, "function", "zoneOffsetMinutes export");
  equal(typeof app.timezoneDiff, "function", "timezoneDiff export");
});

test("[RUNTIME] shipped zoneOffsetMinutes is DST-correct for America/New_York", function () {
  var winter = app.zoneOffsetMinutes("America/New_York", WINTER);
  var summer = app.zoneOffsetMinutes("America/New_York", SUMMER);
  equal(winter, -300, "NY winter offset (EST)");
  equal(summer, -240, "NY summer offset (EDT)");
  assert(winter !== summer, "shipped helper ignores DST");
});

test("[RUNTIME] shipped zoneOffsetMinutes resolves fixed and UTC zones", function () {
  equal(app.zoneOffsetMinutes("UTC", WINTER), 0, "UTC offset");
  equal(app.zoneOffsetMinutes("Asia/Tokyo", WINTER), 540, "Tokyo offset");
  equal(app.zoneOffsetMinutes("Europe/London", WINTER), 0, "London winter (GMT)");
  equal(app.zoneOffsetMinutes("Europe/London", SUMMER), 60, "London summer (BST)");
});

test("[RUNTIME] unknown zone yields null in the shipped clock helper (no silent UTC)", function () {
  equal(app.zoneOffsetMinutes(UNKNOWN, WINTER), null, "unknown-zone offset");
});

test("[RUNTIME] shipped timezoneDiff produces an offset label for a real zone", function () {
  var label = app.timezoneDiff("America/New_York", SUMMER);
  assert(typeof label === "string" && label.length > 0, "empty timezoneDiff label");
  assert(label.indexOf("GMT") >= 0, "label lacks a GMT offset: " + label);
});

var listEngine = h.makeSettingsListEngine();
var zones = listEngine.zones;

test("[RUNTIME] shipped settings timezoneList() returns the IANA list, not 8 or 12 zones", function () {
  assert(Array.isArray(zones), "timezoneList() did not return an array");
  assert(zones.length >= 400, "runtime IANA zone count too small: " + zones.length);
  assert(zones.indexOf("America/New_York") >= 0, "New York missing from runtime list");
  assert(zones.indexOf("Europe/London") >= 0, "London missing from runtime list");
  assert(zones.indexOf("Australia/Lord_Howe") >= 0, "30-minute-DST zone missing");
});

test("[RUNTIME] runtime IANA list is larger than every shipped fallback list", function () {
  [12, 10, 15].forEach(function (fallbackSize) {
    assert(zones.length > fallbackSize,
      "IANA list (" + zones.length + ") not larger than fallback " + fallbackSize);
  });
});

h.updateMetrics({
  runtimeIanaZones: zones.length,
  runtimeNyWinterOffset: app.zoneOffsetMinutes("America/New_York", WINTER),
  runtimeNySummerOffset: app.zoneOffsetMinutes("America/New_York", SUMMER),
  runtimeUnknownZoneOffset: app.zoneOffsetMinutes(UNKNOWN, WINTER),
});
