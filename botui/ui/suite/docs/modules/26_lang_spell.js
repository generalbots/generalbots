"use strict";
/* Docs language tools — local spellcheck with reviewable suggestions (#2484).
 *
 * Words are compared against the built-in list, the user dictionary, any
 * enabled example domain list and the local glossary. No network request. */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  function wordTokens(text) {
    var source = String(text == null ? "" : text);
    var pattern = /[A-Za-zÀ-ÿ][A-Za-zÀ-ÿ'’-]*/g;
    var result = [];
    var match;
    while ((match = pattern.exec(source)) !== null) {
      result.push({ word: match[0], start: match.index, end: match.index + match[0].length });
    }
    return result;
  }
  NS.wordTokens = wordTokens;

  function addWords(target, list) {
    (list || []).forEach(function (word) {
      var clean = String(word || "").trim().toLowerCase();
      if (clean && clean.indexOf(" ") < 0) target[clean] = true;
    });
  }

  function knownWords() {
    var known = Object.create(null);
    addWords(known, NS.BASE_WORDS);
    addWords(known, NS.customWords());
    Object.keys(NS.state.domains || {}).forEach(function (domain) {
      if (NS.state.domains[domain]) addWords(known, NS.DOMAIN_TERMS[domain]);
    });
    NS.glossary().forEach(function (entry) {
      addWords(known, entry.term.split(/\s+/));
      addWords(known, entry.preferred.split(/\s+/));
    });
    addWords(known, NS.state.ignoredAllCache);
    Object.keys(NS.state.ignoredOnce).forEach(function (word) { known[word] = true; });
    return known;
  }
  NS.knownWords = knownWords;

  function distance(a, b) {
    var x = String(a).toLowerCase();
    var y = String(b).toLowerCase();
    if (x === y) return 0;
    if (!x.length) return y.length;
    if (!y.length) return x.length;
    var previous = [];
    var current = [];
    var i;
    var j;
    for (j = 0; j <= y.length; j++) previous[j] = j;
    for (i = 1; i <= x.length; i++) {
      current[0] = i;
      for (j = 1; j <= y.length; j++) {
        var cost = x.charAt(i - 1) === y.charAt(j - 1) ? 0 : 1;
        current[j] = Math.min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + cost);
      }
      for (j = 0; j <= y.length; j++) previous[j] = current[j];
    }
    return previous[y.length];
  }
  NS.editDistance = distance;

  function matchCase(source, candidate) {
    if (!source || source.charAt(0) !== source.charAt(0).toUpperCase()) return candidate;
    return candidate.charAt(0).toUpperCase() + candidate.slice(1);
  }
  NS.matchCase = matchCase;

  function suggestionsFor(word, known, limit) {
    var lower = word.toLowerCase();
    var pool = Object.keys(known);
    var ranked = [];
    for (var i = 0; i < pool.length; i++) {
      var candidate = pool[i];
      if (candidate === lower) continue;
      if (Math.abs(candidate.length - lower.length) > 2) continue;
      var d = distance(lower, candidate);
      if (d > 2) continue;
      ranked.push({ word: candidate, distance: d, prefix: candidate.charAt(0) === lower.charAt(0) ? 0 : 1 });
    }
    ranked.sort(function (a, b) {
      return a.distance - b.distance || a.prefix - b.prefix || a.word.localeCompare(b.word);
    });
    return ranked.slice(0, limit || 3).map(function (entry) { return matchCase(word, entry.word); });
  }
  NS.suggestionsFor = suggestionsFor;

  function checkSpelling(text) {
    var known = knownWords();
    var issues = [];
    wordTokens(text).forEach(function (token) {
      var lower = token.word.toLowerCase();
      if (known[lower]) return;
      if (lower.length < 3) return;
      if (/^\d/.test(lower)) return;
      var suggestions = suggestionsFor(token.word, known, 3);
      issues.push({
        kind: "spelling",
        group: "spelling",
        start: token.start,
        end: token.end,
        original: token.word,
        replacement: suggestions.length ? suggestions[0] : null,
        suggestions: suggestions
      });
    });
    return issues;
  }
  NS.checkSpelling = checkSpelling;

  function ignoreOnce(word) {
    var clean = String(word || "").toLowerCase();
    if (!clean) return false;
    NS.state.ignoredOnce[clean] = true;
    var items = NS.state.suggestions || [];
    for (var i = 0; i < items.length; i++) {
      if (items[i].kind === "spelling" && String(items[i].original).toLowerCase() === clean) {
        items.splice(i, 1);
        return true;
      }
    }
    return false;
  }
  NS.ignoreOnce = ignoreOnce;

  function ignoreAll(word) {
    var clean = String(word || "").toLowerCase();
    if (!clean) return false;
    if (NS.state.ignoredAllCache.indexOf(clean) < 0) NS.state.ignoredAllCache.push(clean);
    NS.addCustomWord(clean);
    NS.state.suggestions = (NS.state.suggestions || []).filter(function (item) {
      return !(item.kind === "spelling" && String(item.original).toLowerCase() === clean);
    });
    return true;
  }
  NS.ignoreAll = ignoreAll;
})();
