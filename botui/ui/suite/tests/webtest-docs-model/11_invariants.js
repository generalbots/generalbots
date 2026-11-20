/**
 * Additional runtime invariants plus clearly-labelled STATIC source assertions.
 * Static checks never claim a runtime result: they only inspect shipped source
 * for private/DOM-only contracts and explicitly deferred backend behaviour.
 */
"use strict";

var h = require("./00_harness");
var loadModel = h.loadModel;
var test = h.test;
var note = h.note;
var assert = h.assert;
var equal = h.equal;
var readFile = h.readFile;
var readRepo = h.readRepo;

var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

// ── runtime invariants ─────────────────────────────────────────────────

test("RUNTIME fuzz: 200 deterministic ops keep the shipped schema valid", function () {
  var NS = loadModel().NS;
  var seed = 2423;
  function next() { seed = (seed * 1103515245 + 12345) & 0x7fffffff; return seed / 0x7fffffff; }
  function pick(list) { return list.length ? list[Math.floor(next() * list.length)] : null; }

  for (var i = 0; i < 200; i += 1) {
    var doc = NS.getDoc();
    var block = pick(doc.blocks);
    var span = block && pick(block.spans);
    var kind = i % 7;
    if (kind === 0) {
      NS.dispatch({ type: "insert", afterId: block ? block.id : null,
        block: NS.createBlock("paragraph", "fuzz " + i, "b-fuzz-" + i), author: "Fuzzer" });
    } else if (kind === 1 && span) {
      NS.dispatch({ type: next() < 0.5 ? "format" : "unformat",
        spanIds: [span.id], mark: { type: next() < 0.5 ? "bold" : "italic" }, author: "Fuzzer" });
    } else if (kind === 2 && block) {
      NS.dispatch({ type: "move", id: block.id, beforeId: pick(doc.blocks) ? pick(doc.blocks).id : null,
        author: "Fuzzer" });
    } else if (kind === 3 && block) {
      NS.dispatch({ type: "comment",
        comment: NS.createComment(block.id, "note " + i, "Fuzzer", "c-fuzz-" + i), author: "Fuzzer" });
    } else if (kind === 4 && doc.comments.length) {
      NS.dispatch({ type: "uncomment", id: pick(doc.comments).id, author: "Fuzzer" });
    } else if (kind === 5 && block) {
      NS.dispatch({ type: "replace", id: block.id,
        block: NS.createBlock("paragraph", "replaced " + i, "b-rep-" + i), author: "Fuzzer" });
    } else {
      NS.dispatch({ type: "insert", afterId: null,
        block: NS.createBlock("table", "", "b-tbl-" + i), author: "Fuzzer" });
    }
    var check = NS.validate(NS.getDoc());
    assert(check.ok, "fuzz step " + i + " produced " + JSON.stringify(check.errors));
  }
  equal(NS.history().length, 200, "one audited op per generated step");
});

test("RUNTIME inverse preview is a pure probe of the pre-op document", function () {
  var NS = loadModel().NS;
  var root = NS.state.doc.blocks[0];
  root.spans[0].text = "before";
  var probe = NS.previewInverse(NS.state.doc, { type: "replace", id: root.id,
    block: NS.createBlock("paragraph", "after", "b-probe") });
  equal(probe.type, "replace", "replace inverse type");
  equal(probe.payload.block.id, root.id, "inverse targets the original block");
  equal(NS.plainText(probe.payload.block), "before", "inverse carries original text");
  equal(NS.plainText(NS.getBlock(NS.getDoc(), root.id)), "before", "probe did not mutate the document");
  equal(NS.state.ops.length, 0, "probe did not write the log");
});

// ── STATIC source assertions (private/DOM contracts + deferred scope) ──

function sources() {
  var out = {};
  h.MODEL_MODULES().forEach(function (rel) { out[rel] = readFile(rel); });
  return out;
}

test("STATIC source: all seven shipped modules exist and are wired to DocsModel", function () {
  var src = sources();
  equal(Object.keys(src).length, 7, "module count");
  Object.keys(src).forEach(function (rel) {
    assert(src[rel].indexOf("window.DocsModel") >= 0, rel + " binds the shared namespace");
  });
  assert(src["docs/modules/68_model_state.js"].indexOf("NS.PREFIX = PREFIX") >= 0, "storage prefix exported");
  assert(src["docs/modules/74_model_init.js"].indexOf("DOMContentLoaded") >= 0, "boot handles loading state");
});

test("STATIC source: new modules write no innerHTML and call no native dialogs", function () {
  var src = sources();
  var forbidden = /\.innerHTML\b|outerHTML|insertAdjacentHTML|document\.write|\b(?:alert|confirm|prompt)\s*\(/;
  Object.keys(src).forEach(function (rel) {
    assert(!forbidden.test(src[rel]), rel + " contains a forbidden HTML/dialog write");
  });
  assert(src["docs/modules/72_model_dialog.js"].indexOf("role\", \"dialog") >= 0, "dialog role set via setAttribute");
  assert(src["docs/modules/73_model_inspector.js"].indexOf("textContent") >= 0, "inspector uses textContent");
});

test("STATIC source: dispatch is the only exported document write path", function () {
  var src = sources();
  var ops = src["docs/modules/70_model_ops.js"];
  assert(ops.indexOf("NS.dispatch = dispatch") >= 0, "dispatch exported");
  assert(ops.indexOf("NS.applyOpInternal = apply") >= 0, "single reducer exported for inverses");
  assert(ops.indexOf("NS.undo = undo") >= 0 && ops.indexOf("NS.redo = redo") >= 0, "undo/redo exported");
  ["71_model_projections.js", "72_model_dialog.js", "73_model_inspector.js", "74_model_init.js"].forEach(function (name) {
    assert(src["docs/modules/" + name].indexOf("NS.state.doc =") < 0, name + " never writes the document directly");
  });
  assert(ops.indexOf("NS.state.doc =") < 0, "ops module never replaces the document directly");
});

test("STATIC source: backend/scale work is declared deferred, not claimed", function () {
  var src = sources();
  var all = Object.keys(src).map(function (key) { return src[key]; }).join("\n");
  ["DOCX/ODT", "server validation", "HTML migration", "100k-word", "co-editing"].forEach(function (token) {
    assert(all.indexOf(token) >= 0, "deferred scope mentions " + token);
  });
  assert(src["docs/modules/73_model_inspector.js"].indexOf("docs-model-deferred") >= 0,
    "deferred-copy label is a catalog key, not a hardcoded literal");
  note("DEFERRED (not exercised offline): DOCX/ODT round-trip, server-side op validation, HTML-body migration, 100k-word scale, co-editing OT, IME and spellcheck ranges; browser/CDP UAT and CI remain the authoritative gates.");
});

test("STATIC i18n: every docs-model key used by the shipped UI exists in all 8 catalogs", function () {
  var src = sources();
  var ui = src["docs/modules/72_model_dialog.js"] + src["docs/modules/73_model_inspector.js"];
  var keys = {};
  var match = /["'](docs-model-[a-z-]+)["']/g;
  var found;
  while ((found = match.exec(ui)) !== null) keys[found[1]] = true;
  var keyList = Object.keys(keys);
  assert(keyList.length >= 30, "inspector/dialog reference catalog keys");
  var domIds = { "docs-model-status": true };
  keyList.forEach(function (key) {
    assert(key.indexOf("auto-") < 0, "no machine-generated keys");
    if (domIds[key]) return;
    LOCALES.forEach(function (locale) {
      var text = readRepo("botlib/locales/" + locale + "/docs-model.ftl");
      assert(new RegExp("^" + key + "\\s*=", "m").test(text), locale + " missing " + key);
    });
  });
  equal(LOCALES.length, 8, "locale catalogs checked");
});
