"use strict";
/* Sheets enterprise controls (#2437) — range-level ACL policy editor.
 * This module only authors a local policy preview; server-side enforcement
 * on every read/write endpoint is explicitly out of scope. */

(function () {
  var C = window.SheetControls;

  var STATUS = {
    "sheet-ctrl-acl-visible": "The role can read this range in the policy.",
    "sheet-ctrl-acl-invisible": "The role is hidden from this range in the policy.",
    "sheet-ctrl-acl-nomatch": "No rule covers this role and range.",
  };

  function statusText(key) {
    return C.t(key, STATUS[key] || STATUS["sheet-ctrl-acl-nomatch"]);
  }

  function roles(value) {
    return String(value || "").split(/[;,]/).map(function (item) {
      return item.trim();
    }).filter(function (item) { return item.length > 0; });
  }

  function add(range, readable, hidden) {
    var clean = String(range || "").trim().toUpperCase();
    if (!clean) return false;
    C.mutate(function (state) {
      state.acl = state.acl.filter(function (rule) { return rule.range !== clean; });
      state.acl.push({ range: clean, readable: roles(readable), hidden: roles(hidden) });
    });
    return true;
  }

  function remove(range) {
    C.mutate(function (state) {
      state.acl = state.acl.filter(function (rule) { return rule.range !== String(range || "").trim().toUpperCase(); });
    });
    return true;
  }

  function preview(range, role) {
    var clean = String(range || "").trim().toUpperCase();
    var who = String(role || "").trim();
    if (!clean || !who) return "sheet-ctrl-acl-nomatch";
    var match = C.state().acl.filter(function (rule) { return rule.range === clean; })[0];
    if (!match) return "sheet-ctrl-acl-nomatch";
    if (match.hidden.indexOf(who) !== -1) return "sheet-ctrl-acl-invisible";
    if (match.readable.indexOf(who) !== -1) return "sheet-ctrl-acl-visible";
    return "sheet-ctrl-acl-nomatch";
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var grid = make("div", "ss-ctrl-form");
    var note = make("p", "ss-ctrl-scope-note", C.t("sheet-ctrl-acl-note",
      "Policy editor only. Access is enforced on the server and is outside this preview."));
    note.setAttribute("data-i18n", "sheet-ctrl-acl-note");
    grid.appendChild(note);

    var list = make("ul", "ss-ctrl-list");
    var rules = C.state().acl;
    if (!rules.length) {
      var empty = make("li", "ss-ctrl-empty", C.t("sheet-ctrl-acl-empty", "No range access rules yet."));
      empty.setAttribute("data-i18n", "sheet-ctrl-acl-empty");
      list.appendChild(empty);
    } else {
      rules.forEach(function (rule) {
        var item = make("li", "ss-ctrl-list-item");
        item.textContent = rule.range + " | " + rule.readable.join(", ") + " | " + rule.hidden.join(", ");
        item.setAttribute("data-range", rule.range);
        var button = make("button", "ss-ctrl-button-small", C.t("sheet-ctrl-remove", "Remove"));
        button.type = "button";
        button.setAttribute("data-i18n", "sheet-ctrl-remove");
        button.addEventListener("click", function () {
          remove(rule.range);
          ctx.rerender();
        });
        item.appendChild(button);
        list.appendChild(item);
      });
    }
    grid.appendChild(list);

    var range = make("input", "ss-ctrl-input");
    range.type = "text";
    range.id = "ss-ctrl-acl-range";
    range.placeholder = C.t("sheet-ctrl-acl-range", "Named range");
    range.setAttribute("data-i18n-placeholder", "sheet-ctrl-acl-range");
    var readable = make("input", "ss-ctrl-input");
    readable.type = "text";
    readable.id = "ss-ctrl-acl-readable";
    readable.placeholder = C.t("sheet-ctrl-acl-readable", "Roles allowed to read");
    readable.setAttribute("data-i18n-placeholder", "sheet-ctrl-acl-readable");
    var hidden = make("input", "ss-ctrl-input");
    hidden.type = "text";
    hidden.id = "ss-ctrl-acl-hidden";
    hidden.placeholder = C.t("sheet-ctrl-acl-hidden", "Roles hidden from range");
    hidden.setAttribute("data-i18n-placeholder", "sheet-ctrl-acl-hidden");
    var add = make("button", "ss-ctrl-button", C.t("sheet-ctrl-acl-add", "Add access rule"));
    add.type = "button";
    add.id = "ss-ctrl-acl-add";
    add.setAttribute("data-i18n", "sheet-ctrl-acl-add");
    var addRow = make("div", "ss-ctrl-row");
    addRow.appendChild(range);
    addRow.appendChild(readable);
    addRow.appendChild(hidden);
    addRow.appendChild(add);
    grid.appendChild(addRow);

    var role = make("input", "ss-ctrl-input");
    role.type = "text";
    role.id = "ss-ctrl-acl-role";
    role.placeholder = C.t("sheet-ctrl-acl-role", "Role to preview");
    role.setAttribute("data-i18n-placeholder", "sheet-ctrl-acl-role");
    var previewButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-acl-preview", "Preview role"));
    previewButton.type = "button";
    previewButton.id = "ss-ctrl-acl-preview";
    previewButton.setAttribute("data-i18n", "sheet-ctrl-acl-preview");
    var previewRow = make("div", "ss-ctrl-row");
    previewRow.appendChild(role);
    previewRow.appendChild(previewButton);
    grid.appendChild(previewRow);
    container.appendChild(grid);

    add.addEventListener("click", function () {
      var ok = addRule(range.value, readable.value, hidden.value);
      ctx.setStatus(ok ? C.t("sheet-ctrl-status-ready", "Ready.") : C.t("sheet-ctrl-input-needs", "Enter a range and a value."), !ok);
      if (ok) ctx.rerender();
    });
    previewButton.addEventListener("click", function () {
      var key = preview(range.value, role.value);
      ctx.setStatus(statusText(key), false);
    });
  }

  function addRule(range, readable, hidden) { return add(range, readable, hidden); }

  window.SheetControlsAcl = {
    roles: roles,
    add: add,
    remove: remove,
    preview: preview,
    render: render,
  };
})();
