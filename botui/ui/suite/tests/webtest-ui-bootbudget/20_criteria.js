/**
 * [STATIC] characterizations for the #2495 criteria that are NOT shipped.
 *
 * GAP assertions pass by confirming the gap still exists; they are honest
 * observations, not claims that the criterion is met. When production ships
 * the criterion, flip the assertion in the same PR. Criteria that ARE shipped
 * (runtime defer loader, CSS media queries) are asserted positively.
 *
 * No browser, Lighthouse, trace or transfer-size measurement is attempted:
 * those remain open and are reported, not faked.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var all = h.all;
var measure = h.measure;
var readFile = h.readFile;
var fileExists = h.fileExists;
var statBytes = h.statBytes;

var DESKTOP = "desktop.html";
var CHAT = "partials/chat.html";
var LOADER = "suite/js/boot-budget.js";

var DESKTOP_DEFERRED = [
  "screensavers.js", "snap-assist.js", "virtual-desktops.js",
  "control-center.js", "lock-screen.js", "power-menu.js", "spotlight.js",
];
var CHAT_DEFERRED = [
  "chart.umd.min.js", "chat-artifacts.js", "20_tabs", "multichat/",
  "run_timeline", "surface_", "74_viz", "73_app_frame", "74_subagents",
  "76_voice", "77_camera", "78_voice_mode",
];

test("[STATIC][GAP] no defer/async attribute on either entry page's script tags", function () {
  equal(measure(DESKTOP).deferAsync, 0, "desktop deferAsync");
  equal(measure(CHAT).deferAsync, 0, "chat deferAsync");
});

test("[STATIC][GAP] zero defer/async adoption across all shipped suite HTML", function () {
  equal(all().total.deferAsync, 0, "suite-wide deferAsync");
  equal(all().total.blockingScriptSrc, all().total.scriptSrc, "all src scripts block");
});

test("[STATIC] partial form-factor split ships as CSS media queries only", function () {
  equal(measure(DESKTOP).mediaLink, 1, "desktop media-gated stylesheets");
  equal(measure(CHAT).mediaLink, 6, "chat media-gated stylesheets");
  var media = measure(DESKTOP).mediaAttrs.concat(measure(CHAT).mediaAttrs);
  assert(media.length === 7, "unexpected media-attribute count: " + media.length);
  media.forEach(function (value) {
    equal(value, "(min-width: 768px)", "media query");
  });
});

test("[STATIC][GAP] no script-level form-factor gating in shipped HTML", function () {
  equal(measure(DESKTOP).mediaScript, 0, "desktop media-gated scripts");
  equal(measure(CHAT).mediaScript, 0, "chat media-gated scripts");
  var hits = all().files.filter(function (rel) {
    return readFile("suite/" + rel).indexOf("matchMedia") >= 0;
  });
  equal(hits.length, 0, "HTML files carrying matchMedia");
});

test("[STATIC] runtime defer loader ships with matchMedia gating for both scopes", function () {
  assert(fileExists(LOADER), "boot-budget.js is missing");
  var src = readFile(LOADER);
  assert(src.indexOf('COMPACT_QUERY = "(max-width: 767px)"') >= 0, "compact query missing");
  assert(src.indexOf("global.matchMedia(COMPACT_QUERY)") >= 0, "matchMedia gate missing");
  assert(src.indexOf("var DEFERRED = {") >= 0, "DEFERRED map missing");
  assert(src.indexOf("desktop: [") >= 0 && src.indexOf("chat: [") >= 0, "scope keys missing");
  assert(src.indexOf("global.GBBootBudget = {") >= 0, "public entry point missing");
  assert(src.indexOf("deferred: DEFERRED") >= 0, "enumerated deferred set not exposed");
});

test("[STATIC] loader is wired into both entry pages via GBBootBudget.init", function () {
  ["suite/" + DESKTOP, "suite/" + CHAT].forEach(function (rel) {
    var src = readFile(rel);
    assert(src.indexOf("boot-budget.js") >= 0, "loader not referenced: " + rel);
    assert(src.indexOf('GBBootBudget.init(') >= 0, "init not called: " + rel);
  });
});

test("[STATIC] deferred module set is absent from static entry-page script tags", function () {
  var srcs = measure(DESKTOP).scriptSrcs.concat(measure(CHAT).scriptSrcs).map(function (s) {
    return s.toLowerCase();
  });
  DESKTOP_DEFERRED.concat(CHAT_DEFERRED).forEach(function (name) {
    var needle = name.toLowerCase();
    assert(!srcs.some(function (s) { return s.indexOf(needle) >= 0; }),
      "deferred module still statically linked: " + name);
  });
});

test("[STATIC][GAP] no CSS bundling: entry pages still link dozens of separate stylesheets", function () {
  var d = measure(DESKTOP);
  var c = measure(CHAT);
  assert(d.stylesheet > 1 && c.stylesheet > 1, "stylesheets already bundled");
  equal(d.stylesheet, new Set(d.stylesheetHrefs).size, "desktop distinct stylesheet refs");
  equal(c.stylesheet, new Set(c.stylesheetHrefs).size, "chat distinct stylesheet refs");
  var allHrefs = d.stylesheetHrefs.concat(c.stylesheetHrefs);
  assert(!allHrefs.some(function (href) { return /bundle|all\.css/i.test(href); }),
    "a bundled stylesheet unexpectedly ships");
});

test("[STATIC][GAP] no content hashing on stylesheet refs (only ?v=N cache-busting)", function () {
  equal(all().total.hashedCss, 0, "hashed stylesheet refs");
  equal(measure(DESKTOP).hashedCss, 0, "desktop hashed refs");
  equal(measure(CHAT).hashedCss, 0, "chat hashed refs");
  assert(all().total.versionedCss > 0, "expected version-query refs to be present");
  assert(measure(DESKTOP).versionedCss === 27 && measure(CHAT).versionedCss === 21,
    "entry-page version-query refs changed");
});

test("[STATIC][GAP] compressed mobile budget is a documented target, not a static measurement", function () {
  var src = readFile(LOADER);
  assert(src.indexOf("350 KB") >= 0 && src.indexOf("120 KB") >= 0 && src.indexOf("390px") >= 0,
    "stated 350 KB / 120 KB @390px budget missing from loader header");
  assert(src.indexOf("not a measured claim") >= 0,
    "budget no longer labelled as an unmeasured target");
  assert(src.indexOf("Lighthouse") >= 0,
    "loader no longer defers Lighthouse numbers to an external measurement");
});

test("[STATIC][GAP] runtime Tailwind vendor blob ships but no shipped HTML loads it", function () {
  assert(fileExists("suite/js/vendor/tailwindcss.js"), "vendor Tailwind blob missing");
  assert(statBytes("suite/js/vendor/tailwindcss.js") > 0, "vendor Tailwind blob is empty");
  assert(!all().total.scriptSrcs.some(function (s) { return s.toLowerCase().indexOf("tailwindcss") >= 0; }),
    "a shipped HTML unexpectedly links the Tailwind JIT blob");
});
