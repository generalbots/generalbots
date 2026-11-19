"use strict";
/* Docs language tools — thesaurus, glossary, language/privacy and report
 * panels plus the single delegated event handler (#2484). */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;
  var tr = NS.tr;
  var esc = NS.escapeHtml;

  function findOccurrences(word) {
    var clean = String(word || "").trim().toLowerCase();
    var article = NS.getArticle();
    if (!clean || !article) return [];
    var text = typeof article.innerText === "string" && article.innerText ? article.innerText : (article.textContent || "");
    var alternatives = NS.thesaurusLookup(clean);
    return NS.wordTokens(text).filter(function (token) {
      return token.word.toLowerCase() === clean;
    }).slice(0, 12).map(function (token) {
      return {
        kind: "thesaurus",
        start: token.start,
        end: token.end,
        original: token.word,
        replacement: alternatives.length ? alternatives[0] : null,
        suggestions: alternatives.slice()
      };
    });
  }
  NS.findThesaurusOccurrences = findOccurrences;

  NS.renderThesaurus = function () {
    var word = NS.state.thesaurusWord || "";
    var items = NS.state.thesaurus || [];
    var rows = items.map(function (item, index) {
      var options = item.suggestions.map(function (value) {
        return '<option value="' + esc(value) + '">' + esc(value) + "</option>";
      }).join("");
      return '<div class="docs-lang-issue"><div class="docs-lang-issue-main"><code>' + esc(item.original) + "</code> &rarr; " +
        '<select id="docs-lang-thes-' + index + '" aria-label="' + esc(tr("docs-lang-replace-with", "Replace with")) + '">' + options + "</select></div>" +
        '<div class="docs-lang-issue-actions"><button type="button" class="docs-lang-btn docs-lang-primary" data-action="thes-replace" data-index="' + index + '">' +
        esc(tr("docs-lang-replace-with", "Replace with")) + "</button></div></div>";
    }).join("");
    return [
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-thesaurus-hint", "Select a word to see alternatives.")) + "</p>",
      '<div class="docs-lang-inline"><input id="docs-lang-thes-word" type="text" value="' + esc(word) + '" placeholder="' + esc(tr("docs-lang-thesaurus-word", "Word")) + '" aria-label="' + esc(tr("docs-lang-thesaurus-word", "Word")) + '">' +
      '<button type="button" class="docs-lang-btn" data-action="thes-lookup">' + esc(tr("docs-lang-thesaurus-lookup", "Look up")) + "</button></div>",
      '<div class="docs-lang-list">' + (rows || '<div class="docs-lang-empty">' + esc(tr("docs-lang-thesaurus-none", "No alternatives found.")) + "</div>") + "</div>"
    ].join("");
  };

  NS.renderGlossary = function () {
    var list = NS.glossary();
    return [
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-glossary-local", "Local example glossary - stored only in this browser, never shared.")) + "</p>",
      '<div class="docs-lang-inline"><input id="docs-lang-gloss-term" type="text" placeholder="' + esc(tr("docs-lang-glossary-term", "Term")) + '" aria-label="' + esc(tr("docs-lang-glossary-term", "Term")) + '">' +
      '<input id="docs-lang-gloss-preferred" type="text" placeholder="' + esc(tr("docs-lang-glossary-preferred", "Preferred term")) + '" aria-label="' + esc(tr("docs-lang-glossary-preferred", "Preferred term")) + '">' +
      '<button type="button" class="docs-lang-btn" data-action="gloss-add">' + esc(tr("docs-lang-glossary-add", "Add term")) + "</button></div>",
      '<div class="docs-lang-list">' + (list.length ? list.map(function (entry) {
        return '<div class="docs-lang-gloss-row"><code>' + esc(entry.term) + "</code> &rarr; <code>" + esc(entry.preferred) + "</code>" +
          '<button type="button" class="docs-lang-btn" data-action="gloss-delete" data-term="' + esc(entry.term) + '">' +
          esc(tr("docs-lang-glossary-delete", "Delete")) + "</button></div>";
      }).join("") : '<div class="docs-lang-empty">' + esc(tr("docs-lang-glossary-empty", "No glossary terms yet")) + "</div>") + "</div>",
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-glossary-note", "Versioned organization dictionaries need a server endpoint and are not enabled here.")) + "</p>"
    ].join("");
  };

  NS.renderLanguage = function () {
    var options = (NS.LANGUAGES || []).map(function (language) {
      return '<option value="' + language[0] + '"' + (language[0] === NS.state.language ? " selected" : "") + ">" +
        esc(tr(language[1], language[2])) + "</option>";
    }).join("");
    return [
      '<div class="docs-lang-section"><label class="docs-lang-field">' + esc(tr("docs-lang-doc-language", "Document language")) +
      '<select id="docs-lang-language-select" aria-label="' + esc(tr("docs-lang-doc-language", "Document language")) + '">' + options + "</select></label>",
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-language-hint", "Used for spelling and readability.")) + "</p></div>",
      '<div class="docs-lang-section"><label class="docs-lang-check"><input type="checkbox" id="docs-lang-autoformat"' + (NS.state.autoformat ? " checked" : "") + "> " +
      esc(tr("docs-lang-autoformat", "Format smart quotes, dashes and lists as you type")) + "</label>",
      '<button type="button" class="docs-lang-btn" data-action="apply-autoformat">' + esc(tr("docs-lang-apply-autoformat", "Apply autoformat to document")) + "</button></div>",
      '<div class="docs-lang-section"><h4>' + esc(tr("docs-lang-privacy-title", "Privacy")) + "</h4>",
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-privacy-local", "Local/offline mode - all checks run in this browser. Document text is never sent anywhere.")) + "</p>",
      '<h4>' + esc(tr("docs-lang-ai-title", "AI assistance")) + "</h4>",
      '<label class="docs-lang-check"><input type="checkbox" id="docs-lang-ai-optin"' + (NS.state.aiOptIn ? " checked" : "") + "> " +
      esc(tr("docs-lang-ai-optin", "Enable AI assistance (needs an admin-configured endpoint)")) + "</label>",
      '<p class="docs-lang-hint">' + esc(tr("docs-lang-ai-disabled", "Disabled - no endpoint is configured. Document text is never sent.")) + "</p></div>"
    ].join("");
  };

  NS.renderReport = function () {
    var report = NS.state.report || {};
    var rows = [
      ["docs-lang-report-language", "Language: {$name}", { name: NS.languageName(report.language) }],
      ["docs-lang-report-words", "Words: {$n}", { n: report.words || 0 }],
      ["docs-lang-report-sentences", "Sentences: {$n}", { n: report.sentences || 0 }],
      ["docs-lang-report-syllables", "Syllables: {$n}", { n: report.syllables || 0 }],
      ["docs-lang-report-avg-sentence", "Average sentence length: {$n} words", { n: report.averageSentence || 0 }],
      ["docs-lang-report-flesch", "Flesch reading ease: {$score}", { score: report.flesch == null ? "-" : report.flesch }],
      ["docs-lang-report-issues", "Open suggestions: {$n}", { n: report.issues || 0 }]
    ];
    return '<h4>' + esc(tr("docs-lang-report-title", "Document language report")) + "</h4>" +
      '<div class="docs-lang-list">' + rows.map(function (row) {
        return '<div class="docs-lang-report-row">' + esc(tr(row[0], row[1], row[2])) + "</div>";
      }).join("") + "</div>" +
      '<button type="button" class="docs-lang-btn" data-action="refresh-report">' + esc(tr("docs-lang-refresh", "Refresh")) + "</button>";
  };

  function rerender() {
    NS.refreshReport();
    NS.rerender();
  }

  NS.handlePanelClick = function (event) {
    var target = event.target;
    var actionEl = target && target.closest ? target.closest("[data-action]") : null;
    if (!actionEl) return;
    var action = actionEl.getAttribute("data-action");
    var index = parseInt(actionEl.getAttribute("data-index"), 10);
    if (action === "refresh") { rerender(); return; }
    if (action === "refresh-report") { NS.refreshReport(); NS.rerender(); return; }
    if (action === "accept") { NS.acceptIssue(index); return; }
    if (action === "ignore") { NS.ignoreIssue(index, false); return; }
    if (action === "ignore-all") { NS.ignoreIssue(index, true); return; }
    if (action === "add-word") {
      var input = document.getElementById("docs-lang-new-word");
      var value = input ? String(input.value || "").trim() : "";
      if (!value) {
        NS.status(tr("docs-lang-enter-value", "Enter a value."));
        return;
      }
      var added = NS.addCustomWord(value);
      NS.status(tr(added ? "docs-lang-saved-locally" : "docs-lang-duplicate",
        added ? "Saved in this browser." : "Already in the list."));
      rerender();
      return;
    }
    if (action === "remove-word") { NS.removeCustomWord(actionEl.getAttribute("data-word")); rerender(); return; }
    if (action === "thes-lookup") {
      var wordEl = document.getElementById("docs-lang-thes-word");
      NS.state.thesaurusWord = wordEl ? wordEl.value || "" : "";
      NS.state.thesaurus = findOccurrences(NS.state.thesaurusWord);
      NS.rerender();
      return;
    }
    if (action === "thes-replace") {
      var item = (NS.state.thesaurus || [])[index];
      var select = document.getElementById("docs-lang-thes-" + index);
      var replacement = select && select.value ? select.value : (item && item.replacement);
      if (item && replacement && NS.applyChange({ start: item.start, end: item.end, replacement: replacement })) {
        NS.state.thesaurus = findOccurrences(NS.state.thesaurusWord);
        NS.rerender();
      }
      return;
    }
    if (action === "gloss-add") {
      var term = document.getElementById("docs-lang-gloss-term");
      var preferred = document.getElementById("docs-lang-gloss-preferred");
      var ok = NS.addGlossaryTerm(term ? term.value : "", preferred ? preferred.value : "");
      NS.status(tr(ok ? "docs-lang-saved-locally" : "docs-lang-duplicate",
        ok ? "Saved in this browser." : "Already in the list."));
      rerender();
      return;
    }
    if (action === "gloss-delete") { NS.removeGlossaryTerm(actionEl.getAttribute("data-term")); rerender(); return; }
    if (action === "apply-autoformat") { NS.autoformatArticle(); rerender(); }
  };

  NS.handlePanelChange = function (event) {
    var target = event.target;
    if (!target || !target.getAttribute) return;
    var action = target.getAttribute("data-action");
    if (action === "toggle-domain") {
      var domain = target.getAttribute("data-domain");
      NS.state.domains[domain] = target.checked === true;
      NS.writeGlobal("domains", NS.state.domains);
      rerender();
      return;
    }
    if (target.id === "docs-lang-language-select") {
      NS.setLanguage(target.value);
      NS.status(tr("docs-lang-saved-locally", "Saved in this browser."));
      rerender();
      return;
    }
    if (target.id === "docs-lang-autoformat") {
      NS.state.autoformat = target.checked === true;
      NS.writeGlobal("autoformat", NS.state.autoformat);
      NS.status(tr("docs-lang-saved-locally", "Saved in this browser."));
      return;
    }
    if (target.id === "docs-lang-ai-optin") {
      NS.state.aiOptIn = target.checked === true;
      NS.writeGlobal("ai-opt-in", NS.state.aiOptIn);
      NS.status(tr(NS.state.aiOptIn ? "docs-lang-ai-on" : "docs-lang-ai-off",
        NS.state.aiOptIn ? "AI assistance is enabled for this browser." : "AI assistance is off."));
    }
  };
})();
