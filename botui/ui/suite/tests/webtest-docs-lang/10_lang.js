/**
 * #2484 offline verification of the pure window.DocsLang helpers.
 *
 * The four target modules are IIFEs; every access to document/window happens
 * inside function bodies, so they load in a vm sandbox with stubbed window,
 * document and gbT. The shipped core module is loaded only to supply the real
 * namespace, state and dictionary accessors that spellcheck/report read.
 * RUNTIME tests call the real exports. STATIC tests are labelled explicitly
 * and cover only properties that cannot be observed through those exports.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;
var loadScripts = h.loadScripts;
var memoryStorage = h.memoryStorage;

var SOURCES = [
  "docs/modules/25_lang_core.js",
  "docs/modules/25b_lang_data.js",
  "docs/modules/26_lang_spell.js",
  "docs/modules/28_lang_thesaurus.js",
  "docs/modules/29_lang_readability.js",
];

/** Fresh sandbox per test: real modules, in-memory storage, inert DOM/gbT. */
function fixture() {
  var windowStub = {
    gbT: function (key, fallback) { return fallback; },
    localStorage: memoryStorage(),
  };
  var documentStub = {
    getElementById: function () { return null; },
    querySelector: function () { return null; },
    createRange: function () { return {}; },
    createTextNode: function (value) { return { nodeValue: String(value) }; },
  };
  loadScripts(SOURCES, {
    console: console,
    window: windowStub,
    document: documentStub,
    NodeFilter: { SHOW_TEXT: 4 },
  });
  return { NS: windowStub.DocsLang, window: windowStub, document: documentStub };
}

// ── Load / export surface ────────────────────────────────────────────

test("RUNTIME #2484: shipped modules load with stubbed DOM and export the pure helpers", function () {
  var ns = fixture().NS;
  ["wordTokens", "editDistance", "matchCase", "suggestionsFor", "checkSpelling",
    "thesaurusLookup", "autoformatText", "countSyllables", "sentenceParts",
    "roundMetric", "buildReport", "languageName"].forEach(function (name) {
    assert(typeof ns[name] === "function", name + " must be bound onto window.DocsLang");
  });
  assert(Array.isArray(ns.LANGUAGES), "LANGUAGES catalog must be exported");
  assert(Array.isArray(ns.BASE_WORDS) && ns.BASE_WORDS.length > 0, "CORE must be exported as BASE_WORDS");
  assert(ns.THESAURUS && typeof ns.THESAURUS === "object", "THESAURUS table must be exported");
  assert(ns.DOMAIN_TERMS && typeof ns.DOMAIN_TERMS === "object", "DOMAIN_TERMS must be exported");
});

// ── wordTokens ───────────────────────────────────────────────────────

test("RUNTIME #2484: wordTokens splits on boundaries and reports exact offsets", function () {
  var ns = fixture().NS;
  var text = "alpha, beta.";
  var tokens = ns.wordTokens(text);
  assert(tokens.length === 2, "punctuation must not be part of a token");
  assert(tokens[0].word === "alpha" && tokens[0].start === 0, "first token and offset");
  assert(tokens[1].word === "beta" && tokens[1].start === 7, "second token and offset");
  tokens.forEach(function (token) {
    assert(text.slice(token.start, token.end) === token.word, "offsets must slice back to the word");
  });
});

test("RUNTIME #2484: wordTokens keeps accented Latin text and intra-word marks", function () {
  var ns = fixture().NS;
  var words = ns.wordTokens("Café São state-of-the-art don't").map(function (token) {
    return token.word;
  });
  assert(words.join("|") === "Café|São|state-of-the-art|don't",
    "accents, internal hyphen and apostrophe must stay inside a word");
});

// ── Levenshtein distance (exported as editDistance) ─────────────────

test("RUNTIME #2484: editDistance is symmetric, zero for equals and counts substitutions", function () {
  var ns = fixture().NS;
  assert(ns.editDistance("kitten", "kitten") === 0, "equal strings have distance zero");
  assert(ns.editDistance("cat", "cut") === 1, "one substitution costs one edit");
  assert(ns.editDistance("cat", "cut") === ns.editDistance("cut", "cat"), "distance must be symmetric");
  assert(ns.editDistance("kitten", "sitting") === 3, "classic three-edit case");
  assert(ns.editDistance("", "abc") === 3, "inserting into an empty string costs its length");
});

test("RUNTIME #2484: editDistance compares case-insensitively", function () {
  var ns = fixture().NS;
  assert(ns.editDistance("Cat", "cat") === 0, "casing must not change distance");
});

// ── matchCase ────────────────────────────────────────────────────────

test("RUNTIME #2484: matchCase preserves casing of identical lower/title/upper forms", function () {
  var ns = fixture().NS;
  ["mac", "Mac", "MAC"].forEach(function (word) {
    assert(ns.matchCase(word, word) === word, "identity must preserve casing of " + word);
  });
});

test("RUNTIME #2484: matchCase capitalizes only a capitalized source", function () {
  var ns = fixture().NS;
  assert(ns.matchCase("Mac", "word") === "Word", "capitalized source must capitalize the candidate");
  assert(ns.matchCase("word", "word") === "word", "lowercase source must leave the candidate alone");
});

// ── suggestionsFor ───────────────────────────────────────────────────

test("RUNTIME #2484: suggestionsFor ranks by edit distance within the limit", function () {
  var ns = fixture().NS;
  var known = { apples: true, appel: true, zebra: true };
  var got = ns.suggestionsFor("apple", known, 2);
  assert(got.length === 2, "limit must cap the result list");
  assert(got[0] === "apples", "one-edit candidate must outrank a two-edit candidate");
  assert(got[1] === "appel", "second candidate must be the two-edit match");
});

test("RUNTIME #2484: suggestionsFor excludes the exact word and honours the limit", function () {
  var ns = fixture().NS;
  var known = { cat: true, cot: true, cut: true, car: true, cap: true, can: true };
  var got = ns.suggestionsFor("cat", known, 2);
  assert(got.length === 2, "only the requested number of suggestions may be returned");
  assert(got.indexOf("cat") < 0, "the queried word must never be suggested");
});

// ── checkSpelling and its state inputs ──────────────────────────────

test("[UAT:DOC-09] RUNTIME #2484: checkSpelling flags misspelled words with ranges and suggestion fields", function () {
  var ns = fixture().NS;
  // Build the fixture from the shipped dictionary so a CORE edit cannot turn a
  // "known" fixture word into a second false-positive issue.
  var known = ns.BASE_WORDS.filter(function (word) { return /^[a-z]{3,}$/.test(word); });
  var unknown = "zzqxwv";
  var text = known[0] + " " + known[1] + " " + unknown + " " + known[2];
  var start = text.indexOf(unknown);
  var issues = ns.checkSpelling(text);
  assert(issues.length === 1, "exactly one unknown token must be flagged");
  assert(issues[0].kind === "spelling" && issues[0].group === "spelling", "issue kind/group");
  assert(issues[0].original === unknown, "issue must carry the original word");
  assert(issues[0].start === start && issues[0].end === start + unknown.length,
    "issue must carry exact offsets");
  assert(Array.isArray(issues[0].suggestions), "issue must expose a suggestion array");
  assert(Object.prototype.hasOwnProperty.call(issues[0], "replacement"), "issue must expose a replacement slot");
});

test("RUNTIME #2484: checkSpelling does not flag CORE dictionary words", function () {
  var ns = fixture().NS;
  assert(ns.checkSpelling(ns.BASE_WORDS.slice(0, 8).join(" ")).length === 0,
    "words from the shipped dictionary must be accepted");
});

test("RUNTIME #2484: checkSpelling honours the loaded domain dictionary in state", function () {
  var f = fixture();
  var ns = f.NS;
  var term = ns.DOMAIN_TERMS.legal[0];
  var was = ns.state.domains.legal;
  try {
    ns.state.domains.legal = false;
    assert(ns.checkSpelling(term).length === 1, "domain term must be flagged before the list is loaded");
    ns.state.domains.legal = true;
    assert(ns.checkSpelling(term).length === 0, "domain term must be accepted once the list is loaded");
  } finally {
    ns.state.domains.legal = was;
  }
});

test("RUNTIME #2484: checkSpelling honours the ignored-once state", function () {
  var ns = fixture().NS;
  var word = "zzqonce";
  assert(ns.checkSpelling(word).length === 1, "word must be flagged before ignore-once");
  ns.state.ignoredOnce[word] = true;
  assert(ns.checkSpelling(word).length === 0, "ignored-once word must be accepted");
  delete ns.state.ignoredOnce[word];
});

test("RUNTIME #2484: ignoreAll persists a custom word and clears it from spelling issues", function () {
  var f = fixture();
  var ns = f.NS;
  var word = "zzqall";
  assert(ns.checkSpelling(word).length === 1, "word must be flagged before ignore-all");
  assert(ns.ignoreAll(word) === true, "ignoreAll must report success");
  assert(ns.state.ignoredAllCache.indexOf(word) >= 0, "ignore-all cache must record the word");
  assert(ns.checkSpelling(word).length === 0, "ignored-all word must no longer be flagged");
  var stored = JSON.parse(f.window.localStorage.getItem("docs2484:custom-words"));
  assert(Array.isArray(stored) && stored.indexOf(word) >= 0, "custom word must be persisted locally");
});

// ── thesaurus ────────────────────────────────────────────────────────

test("RUNTIME #2484: thesaurusLookup returns synonyms for a known word and a defensive copy", function () {
  var ns = fixture().NS;
  var key = Object.keys(ns.THESAURUS)[0];
  var synonyms = ns.thesaurusLookup("  " + key.toUpperCase() + "  ");
  assert(synonyms.length === ns.THESAURUS[key].length, "lookup must return every synonym");
  assert(synonyms[0] === ns.THESAURUS[key][0], "first synonym must match the catalog");
  synonyms.push("zzqsentinel");
  assert(ns.THESAURUS[key].indexOf("zzqsentinel") < 0, "caller must not be able to mutate the catalog");
});

test("RUNTIME #2484: thesaurusLookup returns an empty list for unknown or empty input", function () {
  var ns = fixture().NS;
  assert(ns.thesaurusLookup("zzqunknown").length === 0, "unknown word must have no synonyms");
  assert(ns.thesaurusLookup("").length === 0, "empty input must have no synonyms");
});

test("RUNTIME #2484: thesaurusSetResults applies source casing to the result state", function () {
  var f = fixture();
  var ns = f.NS;
  var result = ns.thesaurusSetResults("Help");
  assert(Array.isArray(result) && result.length > 0, "results must be a non-empty list");
  assert(ns.state.thesaurus === result, "results must be stored in state for the panel");
  assert(result[0].charAt(0) === result[0].charAt(0).toUpperCase(), "capitalized source must capitalize results");
});

// ── autoformatText ───────────────────────────────────────────────────

test("RUNTIME #2484: autoformatText converts straight quotes and -- to an em dash", function () {
  var ns = fixture().NS;
  var source = "He said \"hello\" -- then left.";
  var result = ns.autoformatText(source);
  assert(result.text === "He said \u201Chello\u201D \u2014 then left.", "curly quotes and em dash expected");
  assert(result.changes.length === 3, "each replacement must be reported as a reviewable change");
  result.changes.forEach(function (change) {
    var original = source.slice(change.start, change.end);
    assert(original === "\"" || original === "--", "change ranges must point at the original punctuation");
  });
});

test("RUNTIME #2484: autoformatText converts apostrophes and list markers", function () {
  var ns = fixture().NS;
  var result = ns.autoformatText("don't\n- item");
  assert(result.text === "don\u2019t\n\u2022 item", "apostrophe and list marker expected");
  assert(result.changes.length === 2, "both replacements must be reported");
});

test("RUNTIME #2484: autoformatText leaves already-correct text untouched", function () {
  var ns = fixture().NS;
  var source = "He said \u201Chello\u201D \u2014 then left.";
  var result = ns.autoformatText(source);
  assert(result.changes.length === 0, "no change may be proposed for correct text");
  assert(result.text === source, "already-correct text must come back byte-for-byte");
});

// ── readability ──────────────────────────────────────────────────────

test("RUNTIME #2484: countSyllables handles empty and simple words", function () {
  var ns = fixture().NS;
  [["", 0], ["cat", 1], ["machine", 2], ["education", 4]].forEach(function (pair) {
    assert(ns.countSyllables(pair[0]) === pair[1], "syllables for " + (pair[0] || "empty") + " must be " + pair[1]);
  });
});

test("RUNTIME #2484: sentenceParts splits on .,!? and drops empty parts", function () {
  var ns = fixture().NS;
  assert(ns.sentenceParts("One. Two! Three? Four.").join("|") === "One|Two|Three|Four",
    "all four terminal marks must split sentences");
  assert(ns.sentenceParts("   ").length === 0, "blank text must produce no sentences");
});

test("RUNTIME #2484: roundMetric rounds deterministically", function () {
  var ns = fixture().NS;
  assert(ns.roundMetric(1.23456, 2) === 1.23, "two decimal places expected");
  assert(ns.roundMetric(2, 0) === 2, "zero digits expected");
});

test("RUNTIME #2484: buildReport returns deterministic Flesch-style fields", function () {
  var ns = fixture().NS;
  ns.state.language = "en";
  ns.state.suggestions = [{}, {}];
  var report = ns.buildReport("cat sat. dog ran.");
  assert(report.words === 4 && report.sentences === 2 && report.syllables === 4, "count fields");
  assert(report.averageSentence === 2 && report.flesch === 120.2, "metric fields");
  assert(report.language === "en" && report.issues === 2, "state fields");
  var empty = ns.buildReport("");
  assert(empty.words === 0 && empty.flesch === null, "empty text must have no Flesch score");
});

test("RUNTIME #2484: languageName maps every catalog code and echoes unknown codes", function () {
  var ns = fixture().NS;
  ns.LANGUAGES.forEach(function (entry) {
    assert(ns.languageName(entry[0]) === entry[2], "code " + entry[0] + " must resolve via the catalog");
  });
  assert(ns.languageName("zzq") === "zzq", "unknown code must be echoed");
});

// ── reference data ───────────────────────────────────────────────────

test("RUNTIME #2484: the eight-language catalog has a key and English label per entry", function () {
  var ns = fixture().NS;
  assert(ns.LANGUAGES.length === 8, "catalog must contain eight languages");
  var seen = {};
  ns.LANGUAGES.forEach(function (entry) {
    assert(Array.isArray(entry) && entry.length === 3, "each entry is [code, key, label]");
    assert(typeof entry[0] === "string" && entry[0] && !seen[entry[0]], "codes must be unique");
    seen[entry[0]] = true;
    assert(/^docs-lang-lang-[a-z]+$/.test(entry[1]), "entry must carry a catalog key");
    assert(typeof entry[2] === "string" && entry[2].length > 0, "entry must carry an English label");
  });
  ["legal", "medical", "financial", "public"].forEach(function (domain) {
    assert(Array.isArray(ns.DOMAIN_TERMS[domain]) && ns.DOMAIN_TERMS[domain].length > 0,
      domain + " domain list must ship with terms");
  });
});

// ── explicitly labelled static guards ───────────────────────────────

test("STATIC #2484: pure language modules contain no network or CDN references", function () {
  SOURCES.forEach(function (relPath) {
    var src = readFile(relPath);
    assert(!/\bfetch\s*\(/.test(src), relPath + " must not call fetch");
    assert(src.indexOf("XMLHttpRequest") < 0, relPath + " must not use XMLHttpRequest");
    assert(src.indexOf("WebSocket") < 0, relPath + " must not use WebSocket");
    assert(!/https?:\/\//.test(src), relPath + " must not embed a remote URL");
  });
});

test("STATIC #2484: autoformatText returns ranges and never mutates the article", function () {
  var src = readFile("docs/modules/28_lang_thesaurus.js");
  var start = src.indexOf("function autoformatText");
  var end = src.indexOf("NS.autoformatText", start);
  assert(start >= 0 && end > start, "autoformatText body must be present in the source");
  var body = src.slice(start, end);
  ["getArticle", "innerHTML", "replaceRange", "applyChange"].forEach(function (symbol) {
    assert(body.indexOf(symbol) < 0, "autoformatText must not reference " + symbol);
  });
});

test("STATIC #2484: the four target modules are IIFEs bound to window.DocsLang", function () {
  SOURCES.slice(1).forEach(function (relPath) {
    var src = readFile(relPath);
    assert(src.indexOf("(function () {") >= 0, relPath + " must be an IIFE");
    assert(/var NS = window\.DocsLang/.test(src), relPath + " must bind to window.DocsLang");
  });
});

// ── regressions for the two provable segmentation/punctuation bugs ───

test("RUNTIME #2484: wordTokens excludes multiplication and division symbols", function () {
  var ns = fixture().NS;
  assert(ns.wordTokens("a×b÷c").map(function (token) { return token.word; }).join("|") === "a|b|c",
    "× and ÷ are symbols, not letters, and must split words");
  assert(ns.wordTokens("Café São").map(function (token) { return token.word; }).join("|") === "Café|São",
    "accented Latin letters must still stay inside a word");
});

test("RUNTIME #2484: comma spacing rule never rewrites a thousands separator", function () {
  var windowStub = {
    gbT: function (key, fallback) { return fallback; },
    localStorage: memoryStorage(),
  };
  var documentStub = {
    getElementById: function () { return null; },
    querySelector: function () { return null; },
    createRange: function () { return {}; },
    createTextNode: function (value) { return { nodeValue: String(value) }; },
  };
  var ns = loadScripts(SOURCES.concat(["docs/modules/27_lang_style.js"]), {
    console: console,
    window: windowStub,
    document: documentStub,
    NodeFilter: { SHOW_TEXT: 4 },
  }).window.DocsLang;
  assert(ns.checkStyle("Cost is 1,000 dollars.").length === 0,
    "a digit-to-digit comma must stay untouched");
  var prose = ns.checkStyle("hello,world");
  assert(prose.length === 1 && prose[0].replacement === ", w",
    "a missing prose comma space must still be suggested");
});
