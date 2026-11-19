"use strict";
/* Docs language tools — spelling and grammar/style panels (#2484). */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;
  var tr = NS.tr;
  var esc = NS.escapeHtml;

  function optionsFor(item, selected) {
    var choices = (item.suggestions || []).slice();
    if (item.replacement && choices.indexOf(item.replacement) < 0) choices.unshift(item.replacement);
    return choices.map(function (value) {
      return '<option value="' + esc(value) + '"' + (value === selected ? " selected" : "") + ">" + esc(value) + "</option>";
    }).join("");
  }

  function row(item, index) {
    var category = item.groupKey ? tr(item.groupKey, item.groupFallback) : tr("docs-lang-tab-spelling", "Spelling");
    var replace = "";
    if (item.replacement) {
      var hasChoices = item.suggestions && item.suggestions.length > 1;
      if (hasChoices) {
        replace = '<select id="docs-lang-sugg-' + index + '" aria-label="' + esc(tr("docs-lang-suggestions", "Suggestions")) + '">' +
          optionsFor(item, item.replacement) + "</select>";
      } else {
        replace = "<code>" + esc(item.replacement) + "</code>";
      }
    } else {
      replace = '<span class="docs-lang-guidance">' + esc(category) + "</span>";
    }
    var buttons = [];
    if (item.replacement) {
      buttons.push('<button type="button" class="docs-lang-btn docs-lang-primary" data-action="accept" data-index="' + index + '">' +
        esc(tr("docs-lang-accept", "Accept")) + "</button>");
    }
    buttons.push('<button type="button" class="docs-lang-btn" data-action="ignore" data-index="' + index + '">' +
      esc(tr("docs-lang-ignore", "Ignore")) + "</button>");
    if (item.kind === "spelling") {
      buttons.push('<button type="button" class="docs-lang-btn" data-action="ignore-all" data-index="' + index + '">' +
        esc(tr("docs-lang-ignore-all", "Ignore all")) + "</button>");
    }
    return '<div class="docs-lang-issue" data-index="' + index + '">' +
      '<div class="docs-lang-issue-main"><code>' + esc(item.original) + "</code> &rarr; " + replace + "</div>" +
      '<div class="docs-lang-issue-meta">' + esc(category) + "</div>" +
      '<div class="docs-lang-issue-actions">' + buttons.join("") + "</div></div>";
  }

  function indexed(predicate) {
    return (NS.state.suggestions || []).map(function (item, index) {
      return { item: item, index: index };
    }).filter(function (entry) { return predicate(entry.item); });
  }

  function issuesList(entries, emptyKey, emptyFallback) {
    if (!entries.length) {
      return '<div class="docs-lang-empty">' + esc(tr(emptyKey, emptyFallback)) + "</div>";
    }
    return entries.map(function (entry) { return row(entry.item, entry.index); }).join("");
  }

  function domainChecks() {
    var domains = [
      ["legal", "docs-lang-domain-legal", "Legal terms (example)"],
      ["medical", "docs-lang-domain-medical", "Medical terms (example)"],
      ["financial", "docs-lang-domain-financial", "Financial terms (example)"],
      ["public", "docs-lang-domain-public", "Public-sector terms (example)"]
    ];
    return domains.map(function (entry) {
      var checked = NS.state.domains && NS.state.domains[entry[0]];
      return '<label class="docs-lang-check"><input type="checkbox" data-action="toggle-domain" data-domain="' + entry[0] + '"' +
        (checked ? " checked" : "") + "> " + esc(tr(entry[1], entry[2])) + "</label>";
    }).join("");
  }

  NS.renderSpelling = function () {
    var entries = indexed(function (item) { return item.kind === "spelling"; });
    var words = NS.customWords();
    return [
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-spelling-hint", "Review suggestions below. Nothing changes until you accept.")) + "</p>",
      '<div class="docs-lang-section"><button type="button" class="docs-lang-btn" data-action="refresh">' + esc(tr("docs-lang-refresh", "Refresh")) + "</button></div>",
      '<div class="docs-lang-list">' + issuesList(entries, "docs-lang-spelling-none", "No spelling issues found.") + "</div>",
      '<div class="docs-lang-section"><h4>' + esc(tr("docs-lang-custom-title", "Custom dictionary")) + "</h4>",
      '<div class="docs-lang-inline"><input id="docs-lang-new-word" type="text" placeholder="' + esc(tr("docs-lang-word-placeholder", "New word")) + '" aria-label="' + esc(tr("docs-lang-word-placeholder", "New word")) + '">' +
      '<button type="button" class="docs-lang-btn" data-action="add-word">' + esc(tr("docs-lang-add-word", "Add word")) + "</button></div>",
      '<div class="docs-lang-chips">' + (words.length ? words.map(function (word) {
        return '<span class="docs-lang-chip">' + esc(word) +
          ' <button type="button" data-action="remove-word" data-word="' + esc(word) + '" aria-label="' + esc(tr("docs-lang-remove", "Remove")) + '">&times;</button></span>';
      }).join("") : esc(tr("docs-lang-custom-empty", "No custom words yet"))) + "</div></div>",
      '<div class="docs-lang-section"><h4>' + esc(tr("docs-lang-domain-title", "Example domain dictionaries")) + "</h4>",
      '<div class="docs-lang-checks">' + domainChecks() + "</div></div>"
    ].join("");
  };

  NS.renderStyle = function () {
    var entries = indexed(function (item) { return item.kind !== "spelling"; });
    return [
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-spelling-hint", "Review suggestions below. Nothing changes until you accept.")) + "</p>",
      '<div class="docs-lang-section"><button type="button" class="docs-lang-btn" data-action="refresh">' + esc(tr("docs-lang-refresh", "Refresh")) + "</button></div>",
      '<div class="docs-lang-list">' + issuesList(entries, "docs-lang-style-none", "No grammar or style suggestions.") + "</div>"
    ].join("");
  };

  NS.acceptIssue = function (index) {
    var item = (NS.state.suggestions || [])[index];
    if (!item || !item.replacement) return;
    var select = document.getElementById("docs-lang-sugg-" + index);
    var replacement = select && select.value ? select.value : item.replacement;
    if (NS.applyChange({ start: item.start, end: item.end, replacement: replacement })) {
      NS.refresh();
      NS.rerender();
    }
  };

  NS.ignoreIssue = function (index, all) {
    var item = (NS.state.suggestions || [])[index];
    if (!item) return;
    if (item.kind === "spelling") {
      if (all) NS.ignoreAll(item.original);
      else NS.ignoreOnce(item.original);
    } else {
      NS.state.suggestions.splice(index, 1);
    }
    NS.rerender();
  };
})();
