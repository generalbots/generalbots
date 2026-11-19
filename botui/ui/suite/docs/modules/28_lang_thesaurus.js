"use strict";
/* Docs language tools — thesaurus substitutions and local autoformat (#2484).
 *
 * Autoformat produces a list of changes that goes through the same reviewable,
 * single-undo path as every other suggestion. Nothing is edited silently. */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  function lookup(word) {
    var clean = String(word || "").trim().toLowerCase();
    if (!clean) return [];
    var table = NS.THESAURUS || {};
    if (Object.prototype.hasOwnProperty.call(table, clean)) {
      return (table[clean] || []).slice();
    }
    return [];
  }
  NS.thesaurusLookup = lookup;

  function setResults(word) {
    var alternatives = lookup(word);
    NS.state.thesaurus = alternatives.map(function (candidate) {
      return NS.matchCase(word, candidate);
    });
    return NS.state.thesaurus;
  }
  NS.thesaurusSetResults = setResults;

  /* Scan the text once and record non-overlapping character ranges. The output
   * text is derived from the ranges so the harness can assert both at once. */
  function autoformatText(text, options) {
    var source = String(text == null ? "" : text);
    var opts = options || {};
    var changes = [];
    var i = 0;
    while (i < source.length) {
      var ch = source.charAt(i);
      if (ch === "-" && source.charAt(i + 1) === "-") {
        var end = i;
        while (source.charAt(end) === "-") end++;
        changes.push({ start: i, end: end, replacement: "\u2014" });
        i = end;
        continue;
      }
      if (ch === '"') {
        var close = source.indexOf('"', i + 1);
        if (close > i) {
          changes.push({ start: i, end: i + 1, replacement: "\u201C" });
          changes.push({ start: close, end: close + 1, replacement: "\u201D" });
          i = close + 1;
          continue;
        }
        i++;
        continue;
      }
      if (ch === "'") {
        var prev = i > 0 ? source.charAt(i - 1) : "";
        var next = i + 1 < source.length ? source.charAt(i + 1) : "";
        if (/[A-Za-zÀ-ÿ]/.test(prev) && /[A-Za-zÀ-ÿ]/.test(next)) {
          changes.push({ start: i, end: i + 1, replacement: "\u2019" });
          i++;
          continue;
        }
        var closeSingle = source.indexOf("'", i + 1);
        if (closeSingle > i) {
          changes.push({ start: i, end: i + 1, replacement: "\u2018" });
          changes.push({ start: closeSingle, end: closeSingle + 1, replacement: "\u2019" });
          i = closeSingle + 1;
          continue;
        }
        i++;
        continue;
      }
      if (opts.lists !== false && (ch === "-" || ch === "*") &&
          (i === 0 || source.charAt(i - 1) === "\n") && source.charAt(i + 1) === " ") {
        changes.push({ start: i, end: i + 2, replacement: "\u2022 " });
        i += 2;
        continue;
      }
      i++;
    }
    var result = source;
    changes.slice().sort(function (a, b) { return b.start - a.start; }).forEach(function (change) {
      result = result.slice(0, change.start) + change.replacement + result.slice(change.end);
    });
    return { text: result, changes: changes };
  }
  NS.autoformatText = autoformatText;

  function applyAutoformat(changes) {
    var article = NS.getArticle();
    if (!article || !changes || !changes.length) return 0;
    var original = article.innerHTML;
    var clone = article.cloneNode(true);
    var ordered = changes.slice().sort(function (a, b) { return b.start - a.start; });
    var index;
    for (index = 0; index < ordered.length; index++) {
      if (!NS.replaceRange(clone, ordered[index].start, ordered[index].end, ordered[index].replacement)) return 0;
    }
    if (!NS.validResult(clone, original)) {
      NS.status(NS.tr("docs-find-schema-error", "Replacement would corrupt the document"));
      return 0;
    }
    NS.state.undoSnapshot = original;
    for (index = 0; index < ordered.length; index++) {
      NS.replaceRange(article, ordered[index].start, ordered[index].end, ordered[index].replacement);
    }
    NS.save();
    NS.status(NS.tr("docs-lang-autoformat-done", "Autoformat applied - use Undo to revert."));
    return ordered.length;
  }
  NS.applyAutoformat = applyAutoformat;

  function autoformatArticle() {
    var article = NS.getArticle();
    if (!article) return 0;
    var text = typeof article.innerText === "string" && article.innerText ? article.innerText : (article.textContent || "");
    var result = autoformatText(text, { lists: true });
    if (!result.changes.length) {
      NS.status(NS.tr("docs-lang-autoformat-none", "Nothing to reformat."));
      return 0;
    }
    return applyAutoformat(result.changes);
  }
  NS.autoformatArticle = autoformatArticle;

  // Opt-in as-you-type hook: only runs when the privacy panel toggle is on,
  // only at the caret's text node, and never across element boundaries.
  function maybeAutoformatAtCaret() {
    if (!NS.state.autoformat) return 0;
    var selection = window.getSelection && window.getSelection();
    if (!selection || selection.rangeCount < 1) return 0;
    var range = selection.getRangeAt(0);
    if (!range.collapsed || range.startContainer.nodeType !== 3) return 0;
    var node = range.startContainer;
    var article = NS.getArticle();
    if (!article || !article.contains || !article.contains(node)) return 0;
    var caret = range.startOffset;
    var before = (node.nodeValue || "").slice(0, caret);
    var last = before.slice(-3);
    var replacement = null;
    var length = 0;
    if (/--$/.test(last)) { replacement = "\u2014"; length = 2; }
    else if (/"$/.test(last)) { replacement = "\u201D"; length = 1; }
    if (!replacement) return 0;
    NS.state.undoSnapshot = article.innerHTML;
    node.deleteData(caret - length, length);
    node.insertData(caret - length, replacement);
    NS.save();
    var next = document.createRange();
    next.setStart(node, caret - length + replacement.length);
    next.collapse(true);
    selection.removeAllRanges();
    selection.addRange(next);
    return 1;
  }
  NS.maybeAutoformatAtCaret = maybeAutoformatAtCaret;
})();
