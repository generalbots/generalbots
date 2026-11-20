/**
 * #2485 offline verification of docs/modules/21_find_replace.js.
 * The module is an IIFE that binds helpers onto window.DocsNav. All DOM access
 * lives inside functions, so it loads in a vm sandbox with stubbed window,
 * document and NodeFilter; the matcher/scope/undo/validation paths are then
 * exercised as real runtime behaviour. A few source-order checks are labelled
 * STATIC where an internal array cannot be observed from outside the IIFE.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;
var loadScript = h.loadScript;

var SOURCE = "docs/modules/21_find_replace.js";

function textNode(data, tagName) {
  return {
    data: data,
    textContent: data,
    parentElement: { tagName: tagName || "P", style: {} },
  };
}

/** Build a fresh module instance with a deterministic fake document. */
function fixture(options) {
  var opts = options || {};
  var events = [];
  var statuses = [];
  var validations = [];
  var validation = opts.validation || { ok: true, error: "" };

  function makeArticle(texts) {
    var nodes = texts.map(function (text) { return textNode(text, "P"); });
    return {
      __textNodes: nodes,
      querySelectorAll: function () { return []; },
      cloneNode: function () {
        return makeArticle(nodes.map(function (node) { return node.data; }));
      },
      get innerHTML() {
        return "<p>" + nodes.map(function (node) { return node.data; }).join("") + "</p>";
      },
      set innerHTML(value) {
        events.push("write");
        var raw = String(value).replace(/^<p>/, "").replace(/<\/p>$/, "");
        nodes.length = 0;
        nodes.push(textNode(raw, "P"));
      },
    };
  }

  var article = makeArticle(opts.texts || ["Alpha beta ALPHA"]);

  var NS = {
    tr: function (key, fallback, params) {
      var text = fallback || key;
      if (params && typeof params.n !== "undefined") {
        text = text.replace("{$n}", String(params.n));
      }
      return text;
    },
    escapeHtml: function (value) { return String(value); },
    status: function (message) { statuses.push(message); },
    save: function () { events.push("save"); },
    getArticle: function () { return article; },
    validateHtml: function (html, original) {
      events.push("validate");
      validations.push({ html: html, original: original });
      return validation;
    },
    searchScopes: function (target) {
      return [{ scope: "body", node: target, text: target.textContent || "" }];
    },
  };

  var documentStub = {
    createTreeWalker: function (root) {
      var list = (root && root.__textNodes) ? root.__textNodes : [];
      var index = -1;
      return {
        currentNode: null,
        nextNode: function () {
          index += 1;
          if (index >= list.length) {
            this.currentNode = null;
            return null;
          }
          this.currentNode = list[index];
          return this.currentNode;
        },
      };
    },
    getElementById: function () { return null; },
    addEventListener: function () {},
    removeEventListener: function () {},
  };

  loadScript(SOURCE, {
    window: { DocsNav: NS },
    document: documentStub,
    NodeFilter: { SHOW_TEXT: 4 },
    console: console,
  });

  function dialog(values) {
    var v = values || {};
    var fields = {
      "#fr-find": { value: v.find || "" },
      "#fr-replace": { value: v.replace || "" },
      "#fr-regex": { checked: !!v.regex },
      "#fr-word": { checked: !!v.wholeWord },
      "#fr-case": { checked: !!v.matchCase },
      "#fr-style": { value: v.style || "any" },
      "#fr-scope": { value: v.scope || "all" },
      "#fr-result": { innerHTML: "" },
    };
    return {
      querySelector: function (selector) {
        return Object.prototype.hasOwnProperty.call(fields, selector) ? fields[selector] : null;
      },
    };
  }

  return {
    NS: NS,
    article: article,
    events: events,
    statuses: statuses,
    validations: validations,
    dialog: dialog,
    setScopes: function (factory) { NS.searchScopes = factory; },
  };
}

function scopeNode(data) {
  var node = textNode(data, "P");
  return { __textNodes: [node], textContent: data };
}

// ── Load / export surface ────────────────────────────────────────────

test("RUNTIME #2485: helper module loads with stubbed DOM and exports its API", function () {
  var f = fixture();
  ["findMatches", "replaceAll", "undoReplace", "openFindReplace", "closeFindReplace"].forEach(function (name) {
    assert(typeof f.NS[name] === "function", name + " must be exported onto DocsNav");
  });
});

// ── Matcher construction (flags, escaping, whole word) ──────────────

test("RUNTIME #2485: plain mode matches case-insensitively by default", function () {
  var f = fixture({ texts: ["alpha ALPHA Alpha"] });
  assert(f.NS.findMatches("alpha", {}).length === 3, "default plain search must find 3 case variants");
});

test("RUNTIME #2485: match case narrows a plain search to exact case", function () {
  var f = fixture({ texts: ["alpha ALPHA Alpha"] });
  assert(f.NS.findMatches("alpha", { matchCase: true }).length === 1,
    "match case must find only the lowercase occurrence");
});

test("RUNTIME #2485: plain mode escapes regex metacharacters", function () {
  var f = fixture({ texts: ["a.b axb"] });
  assert(f.NS.findMatches("a.b", {}).length === 1, "plain dot must be literal, not a wildcard");
});

test("RUNTIME #2485: whole word restricts plain matches to word boundaries", function () {
  var f = fixture({ texts: ["cat catalog cat"] });
  assert(f.NS.findMatches("cat", { wholeWord: true }).length === 2,
    "whole word must skip the occurrence inside 'catalog'");
});

test("RUNTIME #2485: regex mode builds a global, case-insensitive pattern", function () {
  var f = fixture({ texts: ["a1 a2", "A3"] });
  assert(f.NS.findMatches("a\\d", { regex: true }).length === 3,
    "regex flags gi must return all three occurrences across text nodes");
});

test("RUNTIME #2485: regex mode honours match case", function () {
  var f = fixture({ texts: ["a1 a2", "A3"] });
  assert(f.NS.findMatches("a\\d", { regex: true, matchCase: true }).length === 2,
    "regex flag g without i must skip the uppercase occurrence");
});

// ── Regex validation before mutation ────────────────────────────────

test("RUNTIME #2485: invalid regex yields no matches", function () {
  var f = fixture({ texts: ["Alpha ("] });
  assert(f.NS.findMatches("(", { regex: true }).length === 0, "invalid pattern must produce no matches");
});

test("RUNTIME #2485: invalid regex never reaches validation, mutation or undo", function () {
  var f = fixture({ texts: ["Alpha ("] });
  var before = f.article.innerHTML;
  f.NS.replaceAll(f.dialog({ find: "(", replace: "X", regex: true }));
  assert(f.events.indexOf("validate") === -1, "invalid regex must not call schema validation");
  assert(f.events.indexOf("write") === -1, "invalid regex must not mutate the document");
  assert(f.article.innerHTML === before, "invalid regex must leave the HTML unchanged");
  assert(f.NS.undoReplace() === false, "invalid regex must not add an undo step");
  assert(f.statuses.join(" ").indexOf("Invalid") >= 0, "invalid regex must report a status");
});

// ── Scope selection ──────────────────────────────────────────────────

test("RUNTIME #2485: scope selector restricts matches to body/headers/footnotes/comments", function () {
  var f = fixture({ texts: [] });
  f.setScopes(function () {
    return [
      { scope: "body", node: scopeNode("BODY_TOKEN") },
      { scope: "headers", node: scopeNode("HEAD_TOKEN") },
      { scope: "footnotes", node: scopeNode("FOOT_TOKEN") },
      { scope: "comments", node: scopeNode("COMM_TOKEN") },
    ];
  });
  assert(f.NS.findMatches("TOKEN", { scope: "all" }).length === 4, "all scopes must be searched");
  [["body", "BODY_TOKEN"], ["headers", "HEAD_TOKEN"],
    ["footnotes", "FOOT_TOKEN"], ["comments", "COMM_TOKEN"]].forEach(function (pair) {
    var found = f.NS.findMatches("TOKEN", { scope: pair[0] });
    assert(found.length === 1 && found[0].node.textContent === pair[1],
      pair[0] + " scope must return only its own match");
  });
});

// ── One undo step per replace-all ───────────────────────────────────

test("RUNTIME #2485: replace-all pushes exactly one undo step and restores it", function () {
  var f = fixture({ texts: ["Alpha beta"] });
  var before = f.article.innerHTML;
  f.NS.replaceAll(f.dialog({ find: "Alpha", replace: "Z" }));
  assert(f.article.innerHTML !== before, "replace-all must mutate the document");
  assert(f.NS.undoReplace() === true, "one undo must be available after replace-all");
  assert(f.article.innerHTML === before, "undo must restore the exact pre-replace HTML");
  assert(f.NS.undoReplace() === false, "there must be exactly one undo step per replace-all");
});

test("RUNTIME #2485: two replace-all actions produce two undo steps", function () {
  var f = fixture({ texts: ["Alpha beta"] });
  var before = f.article.innerHTML;
  f.NS.replaceAll(f.dialog({ find: "Alpha", replace: "Z" }));
  f.NS.replaceAll(f.dialog({ find: "Z", replace: "Y" }));
  assert(f.NS.undoReplace() === true, "first undo must revert the latest action");
  assert(f.article.innerHTML.indexOf("Z beta") >= 0, "first undo restores the intermediate state");
  assert(f.NS.undoReplace() === true, "second undo must revert the earlier action");
  assert(f.article.innerHTML === before, "second undo restores the original HTML");
  assert(f.NS.undoReplace() === false, "no third undo step may exist");
});

// ── Schema validation before mutation ───────────────────────────────

test("RUNTIME #2485: schema validation runs once before the innerHTML write", function () {
  var f = fixture({ texts: ["Alpha"] });
  f.NS.replaceAll(f.dialog({ find: "Alpha", replace: "X" }));
  var v = f.events.indexOf("validate");
  var w = f.events.indexOf("write");
  assert(v >= 0 && w >= 0 && v < w, "validation must be invoked before any innerHTML mutation");
  assert(f.validations.length === 1, "validation must be invoked exactly once per action");
});

test("RUNTIME #2485: failed validation blocks mutation and undo", function () {
  var f = fixture({ texts: ["Alpha"], validation: { ok: false, error: "blocked" } });
  var before = f.article.innerHTML;
  f.NS.replaceAll(f.dialog({ find: "Alpha", replace: "X" }));
  assert(f.events.indexOf("write") === -1, "failed validation must not mutate the document");
  assert(f.article.innerHTML === before, "failed validation must leave HTML unchanged");
  assert(f.NS.undoReplace() === false, "failed validation must not add an undo step");
  assert(f.statuses.indexOf("blocked") >= 0, "validation error must be surfaced");
});

// ── Source-order guards (internal undoStack is not externally visible) ─

test("STATIC #2485: regex guard, validation and undo snapshot precede mutation", function () {
  var src = readFile(SOURCE);
  var guard = src.indexOf("options.regex && !matcher(query, options)");
  var validation = src.indexOf("NS.validateHtml(clone.innerHTML, article.innerHTML)");
  var push = src.indexOf("undoStack.push(article.innerHTML)");
  var write = src.indexOf("article.innerHTML = clone.innerHTML");
  assert(guard >= 0 && guard < validation, "regex guard must precede schema validation");
  assert(validation >= 0 && validation < push, "schema validation must precede undo snapshot");
  assert(push >= 0 && push < write, "undo snapshot must precede the innerHTML mutation");
});
