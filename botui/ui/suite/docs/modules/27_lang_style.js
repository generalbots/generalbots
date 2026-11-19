"use strict";
/* Docs language tools — basic grammar and house-style rules (#2484).
 *
 * Rules surface reviewable suggestions; they never rewrite the document on
 * their own. Passive voice is guidance only (no safe automatic rewrite). */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  function groupLabel(group) {
    var groups = NS.STYLE_GROUPS || [];
    for (var i = 0; i < groups.length; i++) {
      if (groups[i][0] === group) return [groups[i][1], groups[i][2]];
    }
    return ["docs-lang-style-plain", "Plain language"];
  }
  NS.styleGroupLabel = groupLabel;

  function expand(template, match) {
    return String(template).replace(/\$(\d)/g, function (whole, index) {
      var value = match[parseInt(index, 10)];
      return value == null ? "" : value;
    });
  }
  NS.expandReplacement = expand;

  function escapeRegExp(value) {
    return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  NS.escapeRegExp = escapeRegExp;

  function checkStyle(text) {
    var source = String(text == null ? "" : text);
    var issues = [];
    (NS.STYLE_RULES || []).forEach(function (rule) {
      var pattern = rule.pattern;
      if (!pattern || typeof pattern.exec !== "function") return;
      pattern.lastIndex = 0;
      var match;
      var guard = 0;
      while ((match = pattern.exec(source)) !== null && guard < 500) {
        guard++;
        var original = match[0];
        var replacement = rule.replacement == null ? null : expand(rule.replacement, match);
        if (rule.replacement != null && rule.group !== "punctuation") {
          replacement = NS.matchCase(original, replacement);
        }
        var label = groupLabel(rule.group);
        issues.push({
          kind: "style",
          group: rule.group,
          groupKey: label[0],
          groupFallback: label[1],
          start: match.index,
          end: match.index + original.length,
          original: original,
          replacement: replacement,
          suggestions: []
        });
        if (match[0].length === 0) pattern.lastIndex++;
      }
    });
    return issues;
  }
  NS.checkStyle = checkStyle;

  function checkConsistency(text) {
    var source = String(text == null ? "" : text);
    var issues = [];
    var label = ["docs-lang-style-consistency", "Glossary consistency"];
    NS.glossary().forEach(function (entry) {
      if (!entry.term || !entry.preferred) return;
      var pattern;
      try {
        pattern = new RegExp("\\b" + escapeRegExp(entry.term.toLowerCase()) + "\\b", "gi");
      } catch (error) {
        return;
      }
      var match;
      var guard = 0;
      while ((match = pattern.exec(source)) !== null && guard < 200) {
        guard++;
        issues.push({
          kind: "consistency",
          group: "consistency",
          groupKey: label[0],
          groupFallback: label[1],
          start: match.index,
          end: match.index + match[0].length,
          original: match[0],
          replacement: NS.matchCase(match[0], entry.preferred),
          suggestions: []
        });
        if (match[0].length === 0) pattern.lastIndex++;
      }
    });
    return issues;
  }
  NS.checkConsistency = checkConsistency;

  function checkAll(text) {
    var all = [];
    all = all.concat(NS.checkSpelling(text) || []);
    all = all.concat(NS.checkStyle(text) || []);
    all = all.concat(NS.checkConsistency(text) || []);
    all.sort(function (a, b) { return a.start - b.start || a.end - b.end; });
    return all;
  }
  NS.checkAll = checkAll;

  function refresh() {
    var article = NS.getArticle();
    if (!article) return [];
    var text = typeof article.innerText === "string" && article.innerText ? article.innerText : (article.textContent || "");
    NS.state.suggestions = checkAll(text);
    return NS.state.suggestions;
  }
  NS.refresh = refresh;
})();
