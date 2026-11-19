"use strict";
/* Docs language tools — readability metrics and document report (#2484).
 * English Flesch reading ease is an estimate; other languages still get word,
 * sentence and suggestion counts. */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  function countSyllables(word) {
    var clean = String(word || "").toLowerCase().replace(/[^a-z]/g, "");
    if (!clean) return 0;
    if (clean.length <= 3) return 1;
    var trimmed = clean.replace(/(?:[^laeiouy]es|ed|[^laeiouy]e)$/, "").replace(/^y/, "");
    var groups = trimmed.match(/[aeiouy]{1,2}/g);
    return Math.max(1, groups ? groups.length : 1);
  }
  NS.countSyllables = countSyllables;

  function sentenceParts(text) {
    var source = String(text || "").trim();
    if (!source) return [];
    return source.split(/[.!?]+(?:\s+|$)/).map(function (part) { return part.trim(); }).filter(Boolean);
  }
  NS.sentenceParts = sentenceParts;

  function round(value, digits) {
    var factor = Math.pow(10, digits || 0);
    return Math.round(value * factor) / factor;
  }
  NS.roundMetric = round;

  function buildReport(text) {
    var source = String(text == null ? "" : text);
    var tokens = NS.wordTokens(source);
    var words = tokens.length;
    var parts = sentenceParts(source);
    var sentences = parts.length;
    var syllables = tokens.reduce(function (total, token) {
      return total + countSyllables(token.word);
    }, 0);
    var avg = sentences ? round(words / sentences, 1) : 0;
    var flesch = null;
    if (words > 0 && sentences > 0) {
      flesch = round(206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words), 1);
    }
    var issues = NS.state.suggestions || [];
    return {
      words: words,
      sentences: sentences,
      syllables: syllables,
      averageSentence: avg,
      flesch: flesch,
      language: NS.state.language,
      issues: issues.length
    };
  }
  NS.buildReport = buildReport;

  function refreshReport() {
    var article = NS.getArticle();
    if (!article) return null;
    var text = typeof article.innerText === "string" && article.innerText ? article.innerText : (article.textContent || "");
    NS.state.report = buildReport(text);
    return NS.state.report;
  }
  NS.refreshReport = refreshReport;

  function languageName(code) {
    var languages = NS.LANGUAGES || [];
    for (var i = 0; i < languages.length; i++) {
      if (languages[i][0] === code) return NS.tr(languages[i][1], languages[i][2]);
    }
    return code || "";
  }
  NS.languageName = languageName;
})();
