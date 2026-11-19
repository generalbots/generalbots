"use strict";
/* Single advanced find/replace dialog (#2485): regex, whole word, match case,
 * style/format filter, scope selector, preview and one undo step. The document
 * is only mutated after the candidate HTML passes schema validation. */

(function () {
  var NS = window.DocsNav;
  if (!NS) return;
  var tr = NS.tr;
  var undoStack = [];
  var lastFocus = null;
  var keyHandler = null;

  function optionsFrom(dialog) {
    function checked(id) {
      var el = dialog && dialog.querySelector("#" + id);
      return !!(el && el.checked);
    }
    function value(id) {
      var el = dialog && dialog.querySelector("#" + id);
      return el ? el.value : "";
    }
    return {
      regex: checked("fr-regex"),
      wholeWord: checked("fr-word"),
      matchCase: checked("fr-case"),
      style: value("fr-style") || "any",
      scope: value("fr-scope") || "all"
    };
  }

  function matcher(query, options) {
    try {
      if (options.regex) return new RegExp(query, options.matchCase ? "g" : "gi");
      var escaped = String(query).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      if (options.wholeWord) escaped = "\\b" + escaped + "\\b";
      return new RegExp(escaped, options.matchCase ? "g" : "gi");
    } catch (error) {
      return null;
    }
  }

  function styleAllows(node, filter) {
    if (!filter || filter === "any") return true;
    var el = node && node.parentElement;
    if (!el) return false;
    if (filter === "bold") return el.tagName === "B" || el.tagName === "STRONG" || parseInt(el.style.fontWeight, 10) >= 700;
    if (filter === "italic") return el.tagName === "I" || el.tagName === "EM" || el.style.fontStyle === "italic";
    if (filter === "heading") return /^H[1-6]$/.test(el.tagName || "");
    return true;
  }

  function matchesIn(article, query, options) {
    var result = [];
    if (!query || !article) return result;
    var opts = options || {};
    var pattern = matcher(query, opts);
    if (!pattern) return result;
    var scopes = NS.searchScopes(article);
    var headings = [];
    try {
      headings = Array.prototype.slice.call(article.querySelectorAll("h1,h2,h3,h4,h5,h6"));
    } catch (error) {
      headings = [];
    }
    scopes.forEach(function (scope) {
      if (opts.scope && opts.scope !== "all" && scope.scope !== opts.scope) return;
      var walker = document.createTreeWalker(scope.node, NodeFilter.SHOW_TEXT, null);
      while (walker.nextNode()) {
        var node = walker.currentNode;
        if (!styleAllows(node, opts.style)) continue;
        var text = node.textContent || "";
        var match;
        pattern.lastIndex = 0;
        while ((match = pattern.exec(text)) !== null) {
          result.push({
            node: node,
            start: match.index,
            end: match.index + match[0].length,
            text: match[0],
            scope: scope.scope,
            block: scope.node,
            headingIndex: headings.indexOf(scope.node)
          });
          if (match[0].length === 0) pattern.lastIndex++;
        }
      }
    });
    return result;
  }

  NS.findMatches = function (query, options) {
    return matchesIn(NS.getArticle(), query, options);
  };

  function applyToClone(clone, query, options, replacement) {
    var matches = matchesIn(clone, query, options);
    if (!matches.length) return 0;
    var cloneNodes = [];
    var walker = document.createTreeWalker(clone, NodeFilter.SHOW_TEXT, null);
    while (walker.nextNode()) cloneNodes.push(walker.currentNode);
    var groups = Object.create(null);
    matches.forEach(function (match) {
      var key = cloneNodes.indexOf(match.node);
      if (key < 0) return;
      if (!groups[key]) groups[key] = [];
      groups[key].push(match);
    });
    Object.keys(groups).forEach(function (key) {
      var node = cloneNodes[parseInt(key, 10)];
      var text = node.textContent || "";
      var edits = groups[key].sort(function (a, b) { return b.start - a.start; });
      edits.forEach(function (edit) {
        text = text.slice(0, edit.start) + replacement + text.slice(edit.end);
      });
      node.data = text;
    });
    return matches.length;
  }

  function replaceAll(dialog) {
    var article = NS.getArticle();
    if (!article) return;
    var query = dialog.querySelector("#fr-find").value || "";
    if (!query) {
      NS.status(tr("docs-find-enter-term", "Enter text to find"));
      return;
    }
    var options = optionsFrom(dialog);
    if (options.regex && !matcher(query, options)) {
      NS.status(tr("docs-find-invalid-regex", "Invalid regular expression"));
      return;
    }
    var replacement = dialog.querySelector("#fr-replace").value || "";
    var clone = article.cloneNode(true);
    var count = applyToClone(clone, query, options, replacement);
    if (!count) {
      NS.status(tr("docs-nav-no-results", "No matches"));
      return;
    }
    var validation = NS.validateHtml(clone.innerHTML, article.innerHTML);
    if (!validation.ok) {
      NS.status(validation.error);
      return;
    }
    undoStack.push(article.innerHTML);
    article.innerHTML = clone.innerHTML;
    NS.save();
    NS.status(tr("docs-find-replaced", "Replaced {$n} occurrences", { n: count }));
    if (NS.refreshPane) NS.refreshPane();
    renderPreview(dialog, []);
  }

  NS.undoReplace = function () {
    var article = NS.getArticle();
    if (!article || !undoStack.length) {
      NS.status(tr("docs-find-nothing-to-undo", "Nothing to undo"));
      return false;
    }
    article.innerHTML = undoStack.pop();
    NS.save();
    NS.status(tr("docs-find-undo-done", "Replacement undone"));
    if (NS.refreshPane) NS.refreshPane();
    return true;
  };

  function renderPreview(dialog, matches) {
    var host = dialog.querySelector("#fr-result");
    if (!host) return;
    if (!matches.length) {
      host.innerHTML = '<div class="gb-nav-empty">' + NS.escapeHtml(tr("docs-find-no-preview", "No matches to preview")) + "</div>";
      return;
    }
    var html = [];
    matches.slice(0, 30).forEach(function (match) {
      html.push('<div class="gb-fr-preview-row"><span class="gb-fr-scope">' + NS.escapeHtml(match.scope) +
        "</span> " + NS.escapeHtml(match.block.textContent.slice(0, 120)) + "</div>");
    });
    host.innerHTML = html.join("");
  }

  function preview(dialog) {
    var query = dialog.querySelector("#fr-find").value || "";
    var options = optionsFrom(dialog);
    if (!query) {
      renderPreview(dialog, []);
      NS.status("");
      return;
    }
    if (options.regex && !matcher(query, options)) {
      NS.status(tr("docs-find-invalid-regex", "Invalid regular expression"));
      return;
    }
    var matches = NS.findMatches(query, options);
    renderPreview(dialog, matches);
    NS.status(tr("docs-find-matches", "{$n} matches", { n: matches.length }));
    if (NS.navSetQuery) NS.navSetQuery(query);
    else NS.currentQuery = query;
  }

  function close() {
    var host = document.getElementById("docs-modal-container");
    if (host) host.innerHTML = "";
    if (keyHandler) {
      document.removeEventListener("keydown", keyHandler, true);
      keyHandler = null;
    }
    if (lastFocus && typeof lastFocus.focus === "function") lastFocus.focus();
    lastFocus = null;
  }

  function trapFocus(dialog, event) {
    var focusables = Array.prototype.slice.call(dialog.querySelectorAll(
      'button,input,select,textarea,[href],[tabindex]:not([tabindex="-1"])'
    )).filter(function (node) { return !node.disabled && node.offsetParent !== null; });
    if (!focusables.length) return;
    var first = focusables[0];
    var last = focusables[focusables.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  NS.openFindReplace = function () {
    var host = document.getElementById("docs-modal-container");
    if (!host) return;
    lastFocus = document.activeElement;
    var scopes = [
      ["all", "docs-find-scope-all", "All"],
      ["body", "docs-find-scope-body", "Body"],
      ["headers", "docs-find-scope-headers", "Headers"],
      ["footers", "docs-find-scope-footers", "Footers"],
      ["footnotes", "docs-find-scope-footnotes", "Footnotes"],
      ["comments", "docs-find-scope-comments", "Comments"],
      ["textboxes", "docs-find-scope-textboxes", "Text boxes"]
    ];
    var html = [
      '<div class="docs-modal gb-fr-backdrop" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
      '<div class="gb-fr-dialog" role="dialog" aria-modal="true" aria-labelledby="gb-fr-title" style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:620px;max-width:92vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:12px;">',
      '<h3 id="gb-fr-title" style="margin:0;font-size:16px;">' + NS.escapeHtml(tr("docs-find-title", "Find and replace")) + "</h3>",
      '<div class="gb-fr-grid">',
      '<label class="gb-fr-field">' + NS.escapeHtml(tr("docs-find", "Find")) + '<input id="fr-find" type="text" /></label>',
      '<label class="gb-fr-field">' + NS.escapeHtml(tr("docs-replace", "Replace")) + '<input id="fr-replace" type="text" /></label>',
      "</div>",
      '<div class="gb-fr-options">'
    ];
    [["fr-regex", "docs-find-regex", "Regular expression"],
     ["fr-word", "docs-find-whole-word", "Whole word"],
     ["fr-case", "docs-find-match-case", "Match case"]].forEach(function (opt) {
      html.push('<label class="gb-fr-check"><input id="' + opt[0] + '" type="checkbox" /> ' +
        NS.escapeHtml(tr(opt[1], opt[2])) + "</label>");
    });
    html.push("</div>");
    html.push('<div class="gb-fr-filters"><label class="gb-fr-field">' + NS.escapeHtml(tr("docs-find-style", "Style / format")) +
      '<select id="fr-style">' +
      [["any", "docs-find-style-any", "Any style"], ["bold", "docs-find-style-bold", "Bold"],
       ["italic", "docs-find-style-italic", "Italic"], ["heading", "docs-find-style-heading", "Heading"]]
        .map(function (o) { return '<option value="' + o[0] + '">' + NS.escapeHtml(tr(o[1], o[2])) + "</option>"; }).join("") +
      '</select></label><label class="gb-fr-field">' + NS.escapeHtml(tr("docs-find-scope", "Scope")) +
      '<select id="fr-scope">' +
      scopes.map(function (o) { return '<option value="' + o[0] + '">' + NS.escapeHtml(tr(o[1], o[2])) + "</option>"; }).join("") +
      "</select></label></div>");
    html.push('<div class="gb-fr-actions">' +
      '<button type="button" id="fr-preview-btn" class="gb-fr-btn">' + NS.escapeHtml(tr("docs-find-preview", "Preview")) + "</button>" +
      '<button type="button" id="fr-replace-all" class="gb-fr-btn gb-fr-primary">' + NS.escapeHtml(tr("docs-find-replace-all", "Replace all")) + "</button>" +
      '<button type="button" id="fr-undo" class="gb-fr-btn">' + NS.escapeHtml(tr("docs-find-undo", "Undo replace")) + "</button>" +
      '<button type="button" id="fr-close" class="gb-fr-btn">' + NS.escapeHtml(tr("docs-find-close", "Close")) + "</button></div>");
    html.push('<div class="gb-fr-preview" id="fr-result"></div>');
    html.push('<div class="gb-nav-status" id="docs-fr-status" role="status" aria-live="polite"></div>');
    html.push("</div></div>");
    host.innerHTML = html.join("");
    var dialog = host.querySelector(".gb-fr-dialog");
    dialog.querySelector("#fr-close").addEventListener("click", close);
    dialog.querySelector("#fr-preview-btn").addEventListener("click", function () { preview(dialog); });
    dialog.querySelector("#fr-replace-all").addEventListener("click", function () { replaceAll(dialog); });
    dialog.querySelector("#fr-undo").addEventListener("click", function () { NS.undoReplace(); });
    dialog.addEventListener("click", function (event) {
      if (event.target === dialog.parentNode) close();
    });
    keyHandler = function (event) {
      if (event.key === "Escape") {
        event.preventDefault();
        close();
      } else if (event.key === "Tab") {
        trapFocus(dialog, event);
      }
    };
    document.addEventListener("keydown", keyHandler, true);
    var input = dialog.querySelector("#fr-find");
    if (input) input.focus();
  };

  NS.replaceAll = replaceAll;
  NS.closeFindReplace = close;
})();
