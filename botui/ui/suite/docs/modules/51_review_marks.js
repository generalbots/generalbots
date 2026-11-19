"use strict";
/* Docs enterprise review workflow (#2425), view modes + in-document markup.
 *
 * View modes only toggle a class on the article; the CSS decides how each
 * `<span data-review-change>` and its margin balloon is displayed. That makes
 * "all markup / simple / none / original / final" a pure state change and
 * keeps neighbouring text nodes untouched. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;

  var MODE_LABELS = {
    all: ["docs-review-mode-all", "All markup"],
    simple: ["docs-review-mode-simple", "Simple markup"],
    none: ["docs-review-mode-none", "No markup"],
    original: ["docs-review-mode-original", "Original"],
    final: ["docs-review-mode-final", "Final"]
  };
  NS.MODE_LABELS = MODE_LABELS;

  function modeLabel(mode) {
    var spec = MODE_LABELS[mode] || MODE_LABELS.all;
    return NS.tr(spec[0], spec[1]);
  }
  NS.modeLabel = modeLabel;

  function root() { return NS.getArticle(); }

  function applyModeClass(node) {
    if (!node || !node.classList) return;
    NS.VIEW_MODES.forEach(function (mode) { node.classList.remove("gb-review-mode-" + mode); });
    node.classList.add("gb-review-mode-" + NS.state.viewMode);
  }
  NS.applyModeClass = applyModeClass;

  function setViewMode(mode) {
    if (NS.VIEW_MODES.indexOf(mode) < 0) return false;
    NS.state.viewMode = mode;
    NS.persist();
    applyModeClass(root());
    NS.statusKey("docs-review-status-mode", "Mode: { $mode }", { mode: modeLabel(mode) });
    return true;
  }
  NS.setViewMode = setViewMode;

  function toggleSuggestionMode(on) {
    NS.state.suggestionMode = on === undefined ? !NS.state.suggestionMode : !!on;
    if (NS.state.suggestionMode) NS.state.protectMode = false;
    NS.persist();
    NS.statusKey(NS.state.suggestionMode
      ? "docs-review-suggest-on" : "docs-review-suggest-off",
      NS.state.suggestionMode
        ? "Suggestion mode is on — edits are recorded as suggestions"
        : "Suggestion mode is off");
    return NS.state.suggestionMode;
  }
  NS.toggleSuggestionMode = toggleSuggestionMode;

  function toggleProtectMode(on) {
    NS.state.protectMode = on === undefined ? !NS.state.protectMode : !!on;
    if (NS.state.protectMode) NS.state.suggestionMode = false;
    NS.persist();
    NS.statusKey(NS.state.protectMode
      ? "docs-review-protect-on" : "docs-review-protect-off",
      NS.state.protectMode ? "Protected — comments only" : "Protection off");
    return NS.state.protectMode;
  }
  NS.toggleProtectMode = toggleProtectMode;

  /* Pure proposal gate: in protect mode nothing is written; in suggestion mode
   * the edit becomes an open change; otherwise the caller applies it directly. */
  function proposeEdit(fields) {
    if (NS.state.protectMode) {
      NS.statusKey("docs-review-suggest-blocked", "Document is protected — comments only");
      return { recorded: false, blocked: true, applied: false };
    }
    if (NS.state.suggestionMode) {
      var input = Object.assign({}, fields || {}, { suggestOnly: true });
      var change = NS.addChange(input);
      NS.statusKey("docs-review-status-suggested", "Suggestion recorded for review");
      return { recorded: true, blocked: false, applied: false, change: change };
    }
    return { recorded: false, blocked: false, applied: true };
  }
  NS.proposeEdit = proposeEdit;

  function typeLabel(type) {
    var key = "docs-review-type-" + (type || "insert");
    var fallbacks = { insert: "Insert", delete: "Delete", format: "Format" };
    return NS.tr(key, fallbacks[type] || "Insert");
  }
  NS.typeLabel = typeLabel;

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }

  function labelledText(parent, key, fallback, value, tag) {
    var node = el(tag || "span");
    node.setAttribute("data-i18n", key);
    node.textContent = value != null ? value : NS.tr(key, fallback);
    parent.appendChild(node);
    return node;
  }

  function actionButton(parent, key, fallback, handler) {
    var button = el("button", "gb-review-balloon-btn");
    button.setAttribute("type", "button");
    button.setAttribute("data-i18n", key);
    button.textContent = NS.tr(key, fallback);
    button.addEventListener("click", handler);
    parent.appendChild(button);
    return button;
  }

  function balloonFor(change) {
    var card = el("div", "gb-review-balloon gb-review-balloon-" + change.type);
    card.setAttribute("data-review-balloon", change.id);
    var head = el("div", "gb-review-balloon-head");
    labelledText(head, "docs-review-change-author", "Author", change.author, "strong");
    var time = el("time", "gb-review-balloon-time");
    time.textContent = change.time;
    head.appendChild(time);
    card.appendChild(head);
    var meta = el("div", "gb-review-balloon-meta");
    labelledText(meta, "docs-review-change-type", "Type");
    var kind = el("span", "gb-review-kind");
    kind.textContent = typeLabel(change.type);
    meta.appendChild(kind);
    card.appendChild(meta);
    var diff = el("div", "gb-review-balloon-diff");
    if (change.before) {
      var del = el("del", "gb-review-del");
      del.textContent = change.before;
      diff.appendChild(del);
    }
    if (change.after) {
      var ins = el("ins", "gb-review-ins");
      ins.textContent = change.after;
      diff.appendChild(ins);
    }
    card.appendChild(diff);
    var actions = el("div", "gb-review-balloon-actions");
    actionButton(actions, "docs-review-accept", "Accept", function () {
      NS.acceptChange(change.id);
      NS.statusKey("docs-review-status-accepted-one", "Change accepted (others untouched)");
      renderMarkup();
    });
    actionButton(actions, "docs-review-reject", "Reject", function () {
      NS.rejectChange(change.id);
      NS.statusKey("docs-review-status-rejected-one", "Change rejected");
      renderMarkup();
    });
    card.appendChild(actions);
    return card;
  }

  function renderMarkup(node) {
    var article = node || root();
    if (!article) return;
    applyModeClass(article);
    var old = article.parentNode && article.parentNode.querySelector
      ? article.parentNode.querySelector(".gb-review-margin") : null;
    if (old && old.parentNode) old.parentNode.removeChild(old);
    var margin = el("aside", "gb-review-margin");
    margin.setAttribute("aria-label", NS.tr("docs-review-changes", "Changes"));
    margin.setAttribute("data-i18n-aria-label", "docs-review-changes");
    NS.state.changes.forEach(function (change) {
      if (change.status !== "open") return;
      var marker = change.anchorId && article.querySelector
        ? article.querySelector('[data-review-anchor="' + change.anchorId + '"]') : null;
      if (marker) marker.setAttribute("data-review-change", change.id);
      margin.appendChild(balloonFor(change));
    });
    if (article.parentNode) article.parentNode.appendChild(margin);
  }
  NS.renderMarkup = renderMarkup;
})(window);
