"use strict";
/* Slides deck-wide find/replace (#2433): searches rendered slide text (titles,
 * text, table cells, chart titles) and locally stored notes. Per-match review
 * jumps to its slide; replace-all is one undoable action on local slide state. */
(function () {
  var O = window.GBSlidesOutline;
  var state = { matches: [], find: "", replace: "", cases: false, whole: false };

  function refs() { return O.elementRefs().concat(O.noteRefs()); }
  function keyFor(ref) {
    return [ref.kind, ref.slideIndex, ref.elementId, ref.elementIndex, ref.field, ref.noteId].join("|");
  }
  function positions(text, needle, cases, whole) {
    var out = [];
    if (!needle) return out;
    var hay = cases ? text : text.toLowerCase();
    var probe = cases ? needle : needle.toLowerCase();
    var from = 0, at;
    while ((at = hay.indexOf(probe, from)) !== -1) {
      var end = at + needle.length;
      var boundary = function (index) {
        if (!whole || index < 0 || index >= text.length) return true;
        return !/[\w\u00a0-]/.test(text.charAt(index));
      };
      if (boundary(at - 1) && boundary(end)) out.push({ start: at, end: end });
      from = at + needle.length;
    }
    return out;
  }
  function context(text, start, end) {
    var before = text.slice(Math.max(0, start - 26), start);
    var after = text.slice(end, end + 34);
    return (start > 26 ? "…" : "") + before + text.slice(start, end) + after + (end + 34 < text.length ? "…" : "");
  }

  function find() {
    state.find = O.$("#slides-fr-find").value;
    state.replace = O.$("#slides-fr-replace").value;
    state.cases = !!O.$("#slides-fr-case").checked;
    state.whole = !!O.$("#slides-fr-whole").checked;
    state.matches = [];
    if (!state.find) { renderList(); O.status(O.t("slides-outline-fr-empty-query", "Type text to find")); return; }
    refs().forEach(function (ref) {
      positions(ref.text, state.find, state.cases, state.whole).forEach(function (hit) {
        state.matches.push({
          ref: ref, start: hit.start, end: hit.end,
          key: keyFor(ref) + "|" + hit.start,
          snippet: context(ref.text, hit.start, hit.end)
        });
      });
    });
    renderList();
    O.status(O.t("slides-outline-fr-count", "{n} match(es) found", { n: state.matches.length }));
  }

  function writeRef(ref, before, after) {
    var target = { kind: ref.kind, slideIndex: ref.slideIndex, noteId: ref.noteId,
      elementId: ref.elementId, elementIndex: ref.elementIndex, field: ref.field, before: before };
    O.writeText(target, after);
  }

  function replaceMatch(match) {
    var before = match.ref.text;
    var after = before.slice(0, match.start) + state.replace + before.slice(match.end);
    writeRef(match.ref, before, after);
    O.pushUndo({
      label: O.t("slides-outline-fr-action-one", "replace one match"),
      undo: function () { writeRef(match.ref, after, before); },
      redo: function () { writeRef(match.ref, before, after); }
    });
    O.status(O.t("slides-outline-fr-replaced-one", "Replaced one match"));
    find();
  }

  function replaceAll() {
    if (!state.matches.length) { O.status(O.t("slides-outline-fr-no-matches", "No matches to replace")); return; }
    var groups = {};
    state.matches.forEach(function (match) {
      var key = keyFor(match.ref);
      if (!groups[key]) groups[key] = { ref: match.ref, before: match.ref.text };
    });
    var changes = [];
    Object.keys(groups).forEach(function (key) {
      var group = groups[key];
      var escaped = state.find.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      var flags = state.cases ? "g" : "gi";
      var pattern = state.whole
        ? new RegExp("(^|[^\\w])(" + escaped + ")(?=[^\\w]|$)", flags)
        : new RegExp(escaped, flags);
      var after = state.whole
        ? group.before.replace(pattern, function (full, lead) { return lead + state.replace; })
        : group.before.replace(pattern, function () { return state.replace; });
      if (after !== group.before) {
        changes.push({ ref: group.ref, before: group.before, after: after });
        writeRef(group.ref, group.before, after);
      }
    });
    if (!changes.length) { O.status(O.t("slides-outline-fr-no-changes", "Nothing changed")); return; }
    O.pushUndo({
      label: O.t("slides-outline-fr-action-all", "replace all matches"),
      undo: function () { changes.forEach(function (c) { writeRef(c.ref, c.after, c.before); }); },
      redo: function () { changes.forEach(function (c) { writeRef(c.ref, c.before, c.after); }); }
    });
    O.status(O.t("slides-outline-fr-replaced-all", "Replaced {n} item(s) — undo in one step", { n: changes.length }));
    find();
  }

  function matchRow(match, list) {
    var row = O.el("div", "slides-fr-match");
    var label = O.el("span", "slides-fr-context", match.snippet);
    row.appendChild(label);
    var jump = O.el("button", "slides-tools-btn", O.t("slides-outline-jump", "Go to slide"), "slides-outline-jump");
    jump.type = "button";
    jump.addEventListener("click", function () { O.jump(match.ref.slideIndex); });
    var one = O.el("button", "slides-tools-btn", O.t("slides-outline-fr-replace-one", "Replace"), "slides-outline-fr-replace-one");
    one.type = "button";
    one.addEventListener("click", function () { replaceMatch(match); });
    row.appendChild(jump);
    row.appendChild(one);
    list.appendChild(row);
  }
  function renderList() {
    var list = O.$("#slides-fr-results");
    if (!list) return;
    list.textContent = "";
    state.matches.forEach(function (match) { matchRow(match, list); });
    if (!state.matches.length) {
      list.appendChild(O.el("p", "slides-tools-empty",
        O.t("slides-outline-fr-none", "No matches"), "slides-outline-fr-none"));
    }
  }

  function open() {
    var panel = O.openPanel("slides-find-replace-panel", function () {
      var root = O.el("section", "slides-tools-panel slides-fr-panel");
      var head = O.el("div", "slides-tools-panel-head");
      head.appendChild(O.el("h2", "", O.t("slides-outline-fr-title", "Find and replace"), "slides-outline-fr-title"));
      head.appendChild(O.localBadge());
      var close = O.el("button", "slides-tools-close", O.t("slides-outline-close", "Close"), "slides-outline-close");
      close.type = "button"; close.addEventListener("click", O.closePanels);
      head.appendChild(close);
      root.appendChild(head);

      var form = O.el("div", "slides-fr-form");
      var findInput = document.createElement("input");
      findInput.type = "search"; findInput.id = "slides-fr-find";
      findInput.setAttribute("data-i18n-placeholder", "slides-outline-fr-find");
      findInput.placeholder = O.t("slides-outline-fr-find", "Find in deck");
      findInput.setAttribute("aria-label", O.t("slides-outline-fr-find", "Find in deck"));
      var replaceInput = document.createElement("input");
      replaceInput.type = "text"; replaceInput.id = "slides-fr-replace";
      replaceInput.setAttribute("data-i18n-placeholder", "slides-outline-fr-replace");
      replaceInput.placeholder = O.t("slides-outline-fr-replace", "Replace with");
      replaceInput.setAttribute("aria-label", O.t("slides-outline-fr-replace", "Replace with"));
      form.appendChild(findInput); form.appendChild(replaceInput);

      var options = O.el("div", "slides-fr-options");
      var cases = document.createElement("input");
      cases.type = "checkbox"; cases.id = "slides-fr-case";
      var casesLabel = O.el("label", "", O.t("slides-outline-fr-case", "Match case"), "slides-outline-fr-case");
      casesLabel.appendChild(cases);
      var whole = document.createElement("input");
      whole.type = "checkbox"; whole.id = "slides-fr-whole";
      var wholeLabel = O.el("label", "", O.t("slides-outline-fr-whole", "Whole words"), "slides-outline-fr-whole");
      wholeLabel.appendChild(whole);
      options.appendChild(casesLabel); options.appendChild(wholeLabel);
      form.appendChild(options);
      root.appendChild(form);

      var actions = O.el("div", "slides-tools-toolbar");
      var findBtn = O.el("button", "slides-tools-btn primary", O.t("slides-outline-fr-find-btn", "Find"), "slides-outline-fr-find-btn");
      findBtn.type = "button"; findBtn.addEventListener("click", find);
      var allBtn = O.el("button", "slides-tools-btn", O.t("slides-outline-fr-all", "Replace all"), "slides-outline-fr-all");
      allBtn.type = "button"; allBtn.addEventListener("click", replaceAll);
      var undoBtn = O.el("button", "slides-tools-btn slides-tools-undo", O.t("action-undo", "Undo"), "action-undo");
      undoBtn.type = "button"; undoBtn.addEventListener("click", O.undo);
      actions.appendChild(findBtn); actions.appendChild(allBtn); actions.appendChild(undoBtn);
      root.appendChild(actions);
      findInput.addEventListener("keydown", function (event) { if (event.key === "Enter") find(); });
      var results = O.el("div", "slides-fr-results"); results.id = "slides-fr-results";
      root.appendChild(results);
      return root;
    });
    renderList();
    O.$("#slides-fr-find").focus();
    return panel;
  }

  window.GBSlidesFindReplace = { open: open };
})();
