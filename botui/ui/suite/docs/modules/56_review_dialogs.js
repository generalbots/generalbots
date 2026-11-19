"use strict";
/* Docs enterprise review workflow (#2425), focus-trapped dialogs + workspace.
 *
 * One reusable dialog shell provides Esc-to-close, Tab focus wrap and focus
 * restore, backed by the pure NS.focusMove contract. The reply/compare/
 * dashboard/permission surfaces are tabs inside the review workspace so a
 * reviewer never needs a native browser input dialog or a second window. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;
  var UI = (NS.dialog = NS.dialog || {});

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }

  function i18n(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    return node;
  }

  UI.shell = function (opts) {
    var options = opts || {};
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = el("div", "gb-review-backdrop");
    var dialog = el("div", "gb-review-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = NS.uid("gb-review-title");
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = el("h3", "gb-review-dialog-title");
    heading.id = titleId;
    i18n(heading, options.titleKey || "docs-review-title", options.titleFallback || "Review workspace");
    dialog.appendChild(heading);
    var note = el("p", "gb-review-local-note");
    note.textContent = NS.localNote();
    dialog.appendChild(note);
    var body = el("div", "gb-review-dialog-body");
    dialog.appendChild(body);
    var status = el("div", "gb-review-status");
    status.id = "docs-review-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var footer = el("div", "gb-review-dialog-footer");
    var close = el("button", "gb-review-btn");
    close.setAttribute("type", "button");
    i18n(close, "docs-review-close", "Close");
    footer.appendChild(close);
    dialog.appendChild(footer);
    backdrop.appendChild(dialog);

    var focusSelector = "button,input,select,textarea,[tabindex]:not([tabindex='-1'])";
    function focusables() {
      var list = dialog.querySelectorAll ? dialog.querySelectorAll(focusSelector) : [];
      return Array.prototype.filter.call(list, function (node) { return !node.disabled; });
    }
    function closeDialog() {
      document.removeEventListener("keydown", onKey, true);
      if (backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      if (restore && restore.focus) restore.focus();
    }
    function onKey(event) {
      var list = focusables();
      var current = document.activeElement;
      var action = NS.focusMove(list, current, event);
      if (action.action === "close") { event.preventDefault(); closeDialog(); return; }
      if (action.action === "focus") { event.preventDefault(); list[action.index].focus(); }
    }
    close.addEventListener("click", closeDialog);
    backdrop.addEventListener("click", function (event) {
      if (event.target === backdrop) closeDialog();
    });
    document.addEventListener("keydown", onKey, true);
    host.appendChild(backdrop);
    var first = focusables()[0];
    if (first) first.focus();
    return { body: body, status: status, close: closeDialog, root: dialog };
  };

  UI.open = function (opts) {
    var options = opts || {};
    var shell = UI.shell({
      titleKey: options.titleKey,
      titleFallback: options.titleFallback
    });
    var label = i18n(el("label", "gb-review-field"), options.placeholderKey || "docs-review-comment-placeholder",
      options.placeholderFallback || "Write a comment…");
    var textarea = el("textarea", "gb-review-textarea");
    textarea.id = NS.uid("gb-review-compose");
    textarea.setAttribute("data-i18n-placeholder", options.placeholderKey || "docs-review-comment-placeholder");
    textarea.placeholder = NS.tr(options.placeholderKey || "docs-review-comment-placeholder",
      options.placeholderFallback || "Write a comment…");
    label.appendChild(textarea);
    shell.body.appendChild(label);
    var submit = el("button", "gb-review-btn gb-review-btn-primary");
    submit.setAttribute("type", "button");
    i18n(submit, options.submitKey || "docs-review-add-thread", options.submitFallback || "Add comment");
    shell.body.appendChild(submit);
    function commit() {
      var text = textarea.value.trim();
      if (!text) return;
      var keep = typeof options.onSubmit === "function" ? options.onSubmit(text) : false;
      if (keep !== false) shell.close();
    }
    submit.addEventListener("click", commit);
    textarea.focus();
    return shell.close;
  };

  /* ---- change list ------------------------------------------------------ */
  function changeCard(change, refresh) {
    var card = el("div", "gb-review-change gb-review-change-" + change.type);
    card.setAttribute("data-review-change-card", change.id);
    var head = el("div", "gb-review-change-head");
    var author = el("strong");
    author.textContent = change.author;
    head.appendChild(author);
    var time = el("time", "gb-review-time");
    time.textContent = change.time;
    head.appendChild(time);
    var kind = el("span", "gb-review-kind");
    kind.textContent = NS.typeLabel(change.type);
    head.appendChild(kind);
    card.appendChild(head);
    if (change.before) {
      var del = el("del", "gb-review-del");
      del.textContent = change.before;
      card.appendChild(del);
    }
    if (change.after) {
      var ins = el("ins", "gb-review-ins");
      ins.textContent = change.after;
      card.appendChild(ins);
    }
    var actions = el("div", "gb-review-change-actions");
    var accept = el("button", "gb-review-btn");
    i18n(accept, "docs-review-accept", "Accept");
    accept.setAttribute("type", "button");
    accept.addEventListener("click", function () {
      NS.acceptChange(change.id);
      NS.statusKey("docs-review-status-accepted-one", "Change accepted (others untouched)");
      refresh();
      NS.renderMarkup();
    });
    var reject = el("button", "gb-review-btn");
    i18n(reject, "docs-review-reject", "Reject");
    reject.setAttribute("type", "button");
    reject.addEventListener("click", function () {
      NS.rejectChange(change.id);
      NS.statusKey("docs-review-status-rejected-one", "Change rejected");
      refresh();
      NS.renderMarkup();
    });
    actions.appendChild(accept);
    actions.appendChild(reject);
    card.appendChild(actions);
    return card;
  }

  function renderChanges(host) {
    var panel = host || document.getElementById("docs-review-changes");
    if (!panel) return;
    panel.textContent = "";
    var list = el("div", "gb-review-change-list");
    function refresh() {
      list.textContent = "";
      var open = NS.state.changes.filter(function (c) { return c.status === "open"; });
      if (!open.length) {
        var empty = el("p", "gb-review-empty");
        i18n(empty, "docs-review-no-changes", "No tracked changes yet");
        list.appendChild(empty);
      }
      open.forEach(function (change) { list.appendChild(changeCard(change, refresh)); });
    }
    panel.appendChild(list);
    var bulk = el("div", "gb-review-bulk");
    NS.changeAuthors().forEach(function (author) {
      var row = el("div", "gb-review-bulk-row");
      var name = el("span");
      name.textContent = author;
      row.appendChild(name);
      var accept = el("button", "gb-review-btn");
      i18n(accept, "docs-review-accept-all-author", "Accept all by { $author }");
      accept.textContent = NS.tr("docs-review-accept-all-author", "Accept all by { $author }", { author: author });
      accept.setAttribute("data-i18n-params", JSON.stringify({ author: author }));
      accept.setAttribute("type", "button");
      accept.addEventListener("click", function () {
        var count = NS.acceptByAuthor(author);
        NS.statusKey("docs-review-status-accepted-author", "{ $count } changes accepted",
          { count: count });
        refresh();
        NS.renderMarkup();
      });
      var reject = el("button", "gb-review-btn");
      i18n(reject, "docs-review-reject-all-author", "Reject all by { $author }");
      reject.textContent = NS.tr("docs-review-reject-all-author", "Reject all by { $author }", { author: author });
      reject.setAttribute("data-i18n-params", JSON.stringify({ author: author }));
      reject.setAttribute("type", "button");
      reject.addEventListener("click", function () {
        NS.rejectByAuthor(author);
        NS.statusKey("docs-review-status-rejected-author", "All changes by { $author } rejected",
          { author: author });
        refresh();
        NS.renderMarkup();
      });
      row.appendChild(accept);
      row.appendChild(reject);
      bulk.appendChild(row);
    });
    panel.appendChild(bulk);
    refresh();
  }
  NS.renderChanges = renderChanges;

  /* ---- workspace -------------------------------------------------------- */
  var TABS = [
    ["changes", "docs-review-changes", "Changes", "renderChanges", "docs-review-changes"],
    ["comments", "docs-review-comments", "Comments", "renderCommentPanel", "docs-review-comments"],
    ["compare", "docs-review-compare", "Compare", "renderCompare", "docs-review-compare"],
    ["dashboard", "docs-review-dashboard", "Dashboard", "renderDashboard", "docs-review-dashboard"],
    ["permissions", "docs-review-permissions", "Permissions", "renderPermissions", "docs-review-permissions"]
  ];

  NS.renderWorkspace = function (host) {
    var panel = host || document.getElementById("docs-review-workspace");
    if (!panel) return;
    panel.textContent = "";
    panel.className = "gb-review-workspace";
    var bar = el("div", "gb-review-modebar");
    var modeLabel = i18n(el("label"), "docs-review-mode-label", "View mode");
    var modeSelect = el("select", "gb-review-mode-select");
    NS.VIEW_MODES.forEach(function (mode) {
      var option = el("option");
      option.value = mode;
      i18n(option, NS.MODE_LABELS[mode][0], NS.MODE_LABELS[mode][1]);
      modeSelect.appendChild(option);
    });
    modeSelect.value = NS.state.viewMode;
    modeLabel.appendChild(modeSelect);
    bar.appendChild(modeLabel);
    [["docs-review-suggest", "docs-review-suggest", "Suggestion mode", NS.toggleSuggestionMode,
      "suggestionMode"],
     ["docs-review-protect", "docs-review-protect", "Protect (comments only)", NS.toggleProtectMode,
      "protectMode"]].forEach(function (spec) {
      var button = el("button", "gb-review-btn");
      button.setAttribute("type", "button");
      button.setAttribute("aria-pressed", NS.state[spec[4]] ? "true" : "false");
      i18n(button, spec[0], spec[2]);
      button.addEventListener("click", function () {
        var on = spec[3]();
        button.setAttribute("aria-pressed", on ? "true" : "false");
      });
      bar.appendChild(button);
    });
    var undo = el("button", "gb-review-btn");
    undo.setAttribute("type", "button");
    i18n(undo, "docs-review-undo", "Undo last action");
    undo.addEventListener("click", function () {
      NS.undo();
      NS.statusKey("docs-review-status-undone", "Last action undone");
      refreshAll();
    });
    bar.appendChild(undo);
    panel.appendChild(bar);

    var tabBar = el("div", "gb-review-tabs");
    var panes = el("div", "gb-review-tabpanes");
    var tabs = [];
    function selectTab(index) {
      tabs.forEach(function (tab, i) { tab.setAttribute("aria-selected", i === index ? "true" : "false"); });
      Array.prototype.forEach.call(panes.children, function (pane, i) {
        pane.hidden = i !== index;
      });
      var spec = TABS[index];
      if (NS[spec[3]]) NS[spec[3]]();
    }
    TABS.forEach(function (spec, index) {
      var tab = el("button", "gb-review-tab");
      tab.setAttribute("type", "button");
      tab.setAttribute("role", "tab");
      tab.setAttribute("data-i18n", spec[1]);
      tab.textContent = NS.tr(spec[1], spec[2]);
      tab.addEventListener("click", function () { selectTab(index); });
      tabBar.appendChild(tab);
      tabs.push(tab);
      var pane = el("div", "gb-review-tabpane");
      pane.id = spec[4];
      panes.appendChild(pane);
    });
    function refreshAll() {
      TABS.forEach(function (spec) { if (NS[spec[3]]) NS[spec[3]](); });
    }
    modeSelect.addEventListener("change", function () {
      NS.setViewMode(modeSelect.value);
      NS.renderMarkup();
    });
    panel.appendChild(tabBar);
    panel.appendChild(panes);
    selectTab(0);
  };

  NS.openPanel = function () {
    if (!UI.shell) return;
    var shell = UI.shell({ titleKey: "docs-review-title", titleFallback: "Review workspace" });
    NS.renderWorkspace(shell.body);
    NS.renderMarkup();
    return shell.close;
  };
})(window);
