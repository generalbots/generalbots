/**
 * #2533 STATIC source assertions + queue-depth label regression test.
 *
 * STATIC checks read shipped source only: they document what is local-only or
 * explicitly deferred to a compiler/backend. They never fabricate a runtime
 * PASS. The regression test executes shipped code and asserts fixed behaviour.
 */
"use strict";

var h = require("./00_harness");
var dom = require("./01_dom");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var note = h.note;
var fixture = f.fixture;
var MODULES = f.MODULES;
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

function sources() {
  var out = {};
  MODULES.forEach(function (rel) { out[rel] = h.readFile(rel); });
  return out;
}

function findByI18nKey(node, key) {
  var found = null;
  dom.walk(node, function (candidate) {
    if (!found && candidate.getAttribute("data-i18n") === key) found = candidate;
  });
  return found;
}

test("STATIC #2533: all eight shipped IIFEs exist and are wired into docs.html", function () {
  var src = sources();
  equal(Object.keys(src).length, 8, "module count");
  var html = h.readFile("docs/docs.html");
  MODULES.forEach(function (rel) {
    assert(src[rel].indexOf("window.DocsLarge") >= 0, rel + " binds the shared namespace");
    var name = rel.split("/").pop();
    assert(html.indexOf("/suite/docs/modules/" + name) >= 0, rel + " has a script tag in docs.html");
  });
});

test("STATIC #2533: the local slice performs no server/export round-trip", function () {
  var src = sources();
  var forbidden = /fetch\s*\(|XMLHttpRequest|\.innerHTML\b|outerHTML|document\.write|\b(?:alert|confirm|prompt)\s*\(/;
  MODULES.forEach(function (rel) {
    assert(!forbidden.test(src[rel]), rel + " contains a forbidden network/HTML/dialog write");
  });
  assert(src[MODULES[0]].indexOf("Local only") >= 0, "state module labels the surface local-only");
  assert(src[MODULES[7]].indexOf("No server calls") >= 0, "init module states there are no server calls");
});

test("STATIC #2533: streaming/limits/compression/export/measured baseline are deferred", function () {
  var src = sources();
  assert(src[MODULES[0]].indexOf("DEFERRED") >= 0, "state module declares deferred work");
  var deferred = src[MODULES[0]];
  ["streaming", "server body limits", "gzip/brotli", "ETag/If-Match", "async export", "measured baseline"]
    .forEach(function (token) {
      assert(deferred.indexOf(token) >= 0, "deferred scope names " + token);
    });
  assert(src[MODULES[1]].indexOf("SIMULATED by block count") >= 0, "fixture declares its size simulated");
  assert(src[MODULES[1]].indexOf("not a real DOCX import") >= 0, "fixture is not a real import");
  assert(src[MODULES[3]].indexOf("simulation") >= 0, "save queue declares itself a simulation");
  assert(src[MODULES[5]].indexOf("LOCAL ESTIMATE") >= 0, "baseline declares a local estimate");
  note("DEFERRED (not exercisable offline): streaming DOCX import, real server body limits/413, gzip/brotli transport, real ETag/If-Match enforcement, async export with progress/cancel and the measured RSS/heap/TTFP baseline. Browser/CDP UAT and CI remain authoritative.");
});

test("STATIC #2533: every literal docs-large key used by shipped modules exists in all catalogs", function () {
  var all = MODULES.map(function (rel) { return sources()[rel]; }).join("\n");
  var keys = {};
  var pattern = /["'](docs-large-[a-z0-9]+(?:-[a-z0-9]+)*)["']/g;
  var match;
  while ((match = pattern.exec(all)) !== null) keys[match[1]] = true;
  var list = Object.keys(keys);
  assert(list.length >= 35, "shipped modules reference a full catalog surface");
  var domIds = { "docs-large-status": true };
  list.forEach(function (key) {
    assert(key.indexOf("auto-") < 0, "no machine-generated key: " + key);
    if (domIds[key]) return;
    LOCALES.forEach(function (locale) {
      var text = h.readRepo("botlib/locales/" + locale + "/docs-large.ftl");
      assert(new RegExp("^" + key + "\\s*=", "m").test(text),
        locale + " missing " + key);
    });
  });
  equal(LOCALES.length, 8, "eight locale catalogs checked");
});

test("RUNTIME #2533: queue-depth label has no raw placeholder (regression)", function () {
  var fx = fixture();
  f.article(fx, "doc-A");
  var shell = fx.NS.dialog.shell();
  fx.NS.dialog.render(shell);
  fx.NS.dialog.refresh();
  var label = findByI18nKey(shell.body, "docs-large-queue-depth");
  assert(label, "queue-depth label found by walking data-i18n attributes");
  assert(label.textContent.indexOf("{ $n }") < 0,
    "label must not show a raw placeholder");
  equal(label.textContent, "Queue depth", "label is the plain localized string");
  var value = label.parentNode.querySelectorAll(".gb-large-value")[0];
  assert(value && value.textContent === "0",
    "the numeric count lives in the sibling value node");
});
