/**
 * #2511 RUNTIME verification of shipped docs language helpers that are the
 * only translation-adjacent glossary/variable/variant code in the trio.
 *
 * 25_lang_core, 27_lang_style, 23_quickparts, 25b_lang_data and
 * 29_lang_readability are read from disk and executed in a vm sandbox. These
 * helpers are LOCAL/browser-only; the [STATIC] module records what is missing
 * for enterprise translation (org termbase, linked variants, review, TM).
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var makeLangFixture = h.makeLangFixture;

// ── glossary / termbase (local instance) ─────────────────────────────

test("RUNTIME #2511: glossary term is normalised, persisted and de-duplicated", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  assertEqual(ns.addGlossaryTerm("  Force Majeure ", "  Superior Force  "), true, "new term must be accepted");
  var list = ns.glossary();
  assertEqual(list.length, 1, "exactly one term must be stored");
  assertEqual(list[0].term, "force majeure", "term must be trimmed and lowercased");
  assertEqual(list[0].preferred, "Superior Force", "preferred translation must keep its casing");
  var stored = JSON.parse(f.window.localStorage.getItem("docs2484:glossary"));
  assertEqual(stored[0].preferred, "Superior Force", "glossary must be persisted to localStorage");
  assertEqual(ns.addGlossaryTerm("FORCE MAJEURE", "Other"), false, "duplicate term must be rejected");
  assertEqual(ns.glossary().length, 1, "duplicate must not grow the glossary");
});

test("RUNTIME #2511: glossary consistency check reports deviations with the approved replacement", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  ns.addGlossaryTerm("force majeure", "Superior Force");
  var issues = ns.checkConsistency("The force majeure clause applies.");
  assertEqual(issues.length, 1, "one non-approved term occurrence must be reported");
  assertEqual(issues[0].kind, "consistency", "issue kind must be consistency");
  assertEqual(issues[0].group, "consistency", "issue group must be consistency");
  assertEqual(issues[0].original, "force majeure", "deviation must carry the source term");
  assertEqual(issues[0].replacement, "Superior Force", "deviation must propose the approved term");
  assertEqual(issues[0].groupFallback, "Glossary consistency", "deviation must be labelled for review");
  var start = "The force majeure clause applies.".indexOf("force majeure");
  assertEqual(issues[0].start, start, "deviation start offset must be exact");
  assertEqual(issues[0].end, start + "force majeure".length, "deviation end offset must be exact");
});

test("RUNTIME #2511: removing a glossary term removes its consistency enforcement", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  ns.addGlossaryTerm("force majeure", "Superior Force");
  assertEqual(ns.checkConsistency("The force majeure clause applies.").length, 1,
    "precondition: term is enforced");
  assertEqual(ns.removeGlossaryTerm("Force Majeure"), true, "removal must be reported");
  assertEqual(ns.glossary().length, 0, "glossary must be empty");
  assertEqual(ns.checkConsistency("The force majeure clause applies.").length, 0,
    "removed term must no longer be flagged");
});

test("RUNTIME #2511: glossary terms are fed into the local spellcheck known-words set", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  ns.addGlossaryTerm("zzqterm", "Zzq Approved");
  assert(ns.knownWords().zzqterm === true, "glossary term must be known to the spellchecker");
  assert(ns.knownWords().approved === true, "approved translation words must be known too");
});

// ── variable handling (quick parts) ──────────────────────────────────

test("RUNTIME #2511: quick-part variables are discovered uniquely and in document order", function () {
  var f = makeLangFixture();
  var qp = f.window.DocsNav;
  var content = "<p>{{amount}} for {{name}} / {{amount}} ({{ title }})</p>";
  var names = qp.quickPartVariables(content);
  assertEqual(names.length, 3, "duplicate placeholders must be collapsed");
  assertEqual(names[0], "amount", "first variable");
  assertEqual(names[1], "name", "second variable");
  assertEqual(names[2], "title", "spaced placeholder must be trimmed");
});

test("RUNTIME #2511: quick-part variable fill replaces known names and preserves unknown ones", function () {
  var f = makeLangFixture();
  var qp = f.window.DocsNav;
  var content = "{{amount}} - {{name}} - {{missing}}";
  var filled = qp.fillQuickPartVariables(content, { amount: "1000", name: "Acme" });
  assertEqual(filled, "1000 - Acme - {{missing}}", "only supplied variables may be substituted");
  assertEqual(qp.fillQuickPartVariables(content, {}), content, "empty values must leave all placeholders intact");
});

// ── language variant state (single selected language, not linked versions) ──

test("RUNTIME #2511: language state defaults to en and setLanguage persists per document", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  assertEqual(ns.state.language, "en", "default language must be English");
  ns.setLanguage("pt");
  assertEqual(ns.state.language, "pt", "setLanguage must update in-memory state");
  assertEqual(f.window.localStorage.getItem("docs2484:doc-lang-1:language"), JSON.stringify("pt"),
    "language must be persisted under the per-document key");
  ns.state.language = "zz";
  ns.loadState();
  assertEqual(ns.state.language, "pt", "loadState must restore the persisted language");
  ns.setLanguage("");
  assertEqual(ns.state.language, "en", "empty language must fall back to English");
});

test("RUNTIME #2511: language catalog and languageName resolve all eight catalog codes", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  assertEqual(ns.LANGUAGES.length, 8, "eight languages must be catalogued");
  ns.LANGUAGES.forEach(function (row) {
    assertEqual(ns.languageName(row[0]), row[2], "code " + row[0] + " must resolve to its label");
    assert(/^docs-lang-lang-[a-z]+$/.test(row[1]), "each entry must carry a catalog key");
  });
  assertEqual(ns.languageName("zzq"), "zzq", "unknown code must be echoed verbatim");
  assertEqual(ns.languageName("it"), "it", "observed: Italian has no catalog entry in the shipped language table");
});

test("RUNTIME #2511: language helpers keep a single-step undo independent of translation undo", function () {
  var f = makeLangFixture();
  var ns = f.NS;
  var before = "<p>Before</p>";
  var after = "<p>After</p>";
  f.article.innerHTML = before;
  ns.state.undoSnapshot = before;
  f.article.innerHTML = after;
  assertEqual(ns.undo(), true, "a stored snapshot must be undoable");
  assertEqual(f.article.innerHTML, before, "undo must restore the exact snapshot");
  assertEqual(ns.state.undoSnapshot, null, "undo must clear the snapshot");
  assertEqual(ns.undo(), false, "a second undo must be a no-op");
});
