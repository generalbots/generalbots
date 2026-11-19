"use strict";
/* Sheets enterprise controls (#2437) — protection semantics and local
 * toolbar enforcement. Preview only: the password hash is not a security
 * control and no server protection is implied. */

(function () {
  var C = window.SheetControls;

  var ALLOWANCES = [
    { key: "selectLocked", label: "sheet-ctrl-prot-allow-select-locked", fallback: "Select locked cells" },
    { key: "selectUnlocked", label: "sheet-ctrl-prot-allow-select-unlocked", fallback: "Select unlocked cells" },
    { key: "format", label: "sheet-ctrl-prot-allow-format", fallback: "Format cells" },
    { key: "insertDeleteRows", label: "sheet-ctrl-prot-allow-rows", fallback: "Insert or delete rows" },
    { key: "sort", label: "sheet-ctrl-prot-allow-sort", fallback: "Sort" },
    { key: "filter", label: "sheet-ctrl-prot-allow-filter", fallback: "Filter" },
    { key: "usePivot", label: "sheet-ctrl-prot-allow-pivot", fallback: "Use pivot tables" },
  ];

  var ACTION_ALLOWANCE = {
    selectLocked: "selectLocked",
    selectUnlocked: "selectUnlocked",
    format: "format",
    insertRows: "insertDeleteRows",
    deleteRows: "insertDeleteRows",
    sort: "sort",
    filter: "filter",
    pivot: "usePivot",
  };

  var BUTTONS = {
    format: ["boldBtn", "italicBtn", "underlineBtn", "strikeBtn", "mergeCellsBtn", "customFormatBtn"],
    insertRows: ["insertRowsBtn"],
    deleteRows: ["deleteRowsBtn"],
    sort: ["sortBtn"],
    filter: ["filterBtn"],
    pivot: ["pivotSheetBtn"],
  };

  function parseRef(ref) {
    var match = /^\$?([A-Za-z]+)\$?([0-9]+)$/.exec(String(ref || "").trim());
    if (!match) return null;
    var letters = match[1].toUpperCase();
    var col = 0;
    for (var i = 0; i < letters.length; i++) col = col * 26 + (letters.charCodeAt(i) - 64);
    return { col: col - 1, row: parseInt(match[2], 10) - 1 };
  }

  function inRange(ref, range) {
    var parts = String(range || "").split("!");
    var cells = parts[parts.length - 1].split(":");
    if (cells.length !== 2) return false;
    var a = parseRef(cells[0]);
    var b = parseRef(cells[1]);
    var r = parseRef(String(ref || "").split("!").pop());
    if (!a || !b || !r) return false;
    return r.row >= Math.min(a.row, b.row) && r.row <= Math.max(a.row, b.row) &&
      r.col >= Math.min(a.col, b.col) && r.col <= Math.max(a.col, b.col);
  }

  function applies(ref) {
    var protection = C.state().protection;
    if (!protection.locked) return false;
    if (protection.scope === "range") {
      if (!ref) return true;
      return inRange(ref, protection.target);
    }
    return true;
  }

  function isAllowed(action, ref) {
    var allowance = ACTION_ALLOWANCE[action];
    if (!allowance) return true;
    if (!applies(ref)) return true;
    return Boolean(C.state().protection.allowances[allowance]);
  }

  function applyGuards() {
    var changed = 0;
    Object.keys(BUTTONS).forEach(function (action) {
      var allowed = isAllowed(action);
      BUTTONS[action].forEach(function (id) {
        var button = document.getElementById(id);
        if (!button || button.disabled === !allowed) return;
        button.disabled = !allowed;
        changed++;
      });
    });
    return changed;
  }

  function protect(scope, target, password) {
    if (!password) {
      return { ok: false, key: "sheet-ctrl-prot-need-password" };
    }
    C.mutate(function (state) {
      state.protection.scope = scope || "sheet";
      state.protection.target = target || "";
      state.protection.locked = true;
      state.protection.hash = C.hash(password);
    });
    applyGuards();
    return { ok: true };
  }

  function unprotect(password) {
    var current = C.state().protection;
    if (!current.locked) return true;
    if (C.hash(password) !== current.hash) return false;
    C.mutate(function (state) {
      state.protection.locked = false;
      state.protection.hash = "";
    });
    applyGuards();
    return true;
  }

  function setAllowance(key, value) {
    if (!(key in C.emptyState().protection.allowances)) return false;
    C.mutate(function (state) { state.protection.allowances[key] = Boolean(value); });
    applyGuards();
    return true;
  }

  function statusText() {
    return C.state().protection.locked
      ? C.t("sheet-ctrl-prot-status-locked", "Protected - preview active")
      : C.t("sheet-ctrl-prot-status-open", "Not protected");
  }

  function labelled(node, key) {
    node.setAttribute("data-i18n", key);
    return node;
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var protection = C.state().protection;
    var grid = make("div", "ss-ctrl-form");

    var scopeRow = make("div", "ss-ctrl-row");
    var scopeLabel = labelled(make("label", "ss-ctrl-label", C.t("sheet-ctrl-prot-scope", "Protection scope")), "sheet-ctrl-prot-scope");
    var scope = make("select", "ss-ctrl-input");
    scope.id = "ss-ctrl-prot-scope";
    [["workbook", "sheet-ctrl-prot-scope-workbook", "Workbook"],
      ["sheet", "sheet-ctrl-prot-scope-sheet", "Sheet"],
      ["range", "sheet-ctrl-prot-scope-range", "Named range"]].forEach(function (option) {
      var node = make("option", "", C.t(option[1], option[2]));
      node.value = option[0];
      node.setAttribute("data-i18n", option[1]);
      if (protection.scope === option[0]) node.selected = true;
      scope.appendChild(node);
    });
    scopeRow.appendChild(scopeLabel);
    scopeRow.appendChild(scope);
    grid.appendChild(scopeRow);

    var targetRow = make("div", "ss-ctrl-row");
    var targetLabel = labelled(make("label", "ss-ctrl-label", C.t("sheet-ctrl-prot-target", "Named range")), "sheet-ctrl-prot-target");
    var target = make("input", "ss-ctrl-input");
    target.type = "text";
    target.id = "ss-ctrl-prot-target";
    target.value = protection.target;
    target.setAttribute("data-i18n-placeholder", "sheet-ctrl-prot-target");
    target.placeholder = C.t("sheet-ctrl-prot-target", "Named range");
    targetRow.appendChild(targetLabel);
    targetRow.appendChild(target);
    grid.appendChild(targetRow);

    var passwordRow = make("div", "ss-ctrl-row");
    var passwordLabel = labelled(make("label", "ss-ctrl-label", C.t("sheet-ctrl-prot-password", "Password (preview)")), "sheet-ctrl-prot-password");
    var password = make("input", "ss-ctrl-input");
    password.type = "password";
    password.id = "ss-ctrl-prot-password";
    password.setAttribute("data-i18n-placeholder", "sheet-ctrl-prot-password-placeholder");
    password.placeholder = C.t("sheet-ctrl-prot-password-placeholder", "Enter a password");
    passwordRow.appendChild(passwordLabel);
    passwordRow.appendChild(password);
    grid.appendChild(passwordRow);

    var buttons = make("div", "ss-ctrl-actions");
    var lock = make("button", "ss-ctrl-button", C.t("sheet-ctrl-prot-lock", "Protect"));
    lock.type = "button";
    lock.id = "ss-ctrl-prot-lock";
    lock.setAttribute("data-i18n", "sheet-ctrl-prot-lock");
    var unlock = make("button", "ss-ctrl-button", C.t("sheet-ctrl-prot-unlock", "Unprotect"));
    unlock.type = "button";
    unlock.id = "ss-ctrl-prot-unlock";
    unlock.setAttribute("data-i18n", "sheet-ctrl-prot-unlock");
    buttons.appendChild(lock);
    buttons.appendChild(unlock);
    grid.appendChild(buttons);

    var allowBox = make("fieldset", "ss-ctrl-allowances");
    var checks = [];
    var legend = labelled(make("legend", "ss-ctrl-label", C.t("sheet-ctrl-prot-allowances", "Allowed actions")), "sheet-ctrl-prot-allowances");
    allowBox.appendChild(legend);
    ALLOWANCES.forEach(function (allowance) {
      var row = make("label", "ss-ctrl-check");
      var check = make("input");
      check.type = "checkbox";
      check.checked = Boolean(protection.allowances[allowance.key]);
      check.dataset.allowance = allowance.key;
      check.setAttribute("aria-label", C.t(allowance.label, allowance.fallback));
      check.setAttribute("data-i18n-aria-label", allowance.label);
      checks.push(check);
      row.appendChild(check);
      row.appendChild(labelled(make("span", "", C.t(allowance.label, allowance.fallback)), allowance.label));
      allowBox.appendChild(row);
    });
    grid.appendChild(allowBox);
    container.appendChild(grid);

    lock.addEventListener("click", function () {
      var result = protect(scope.value, target.value, password.value);
      ctx.setStatus(result.ok
        ? C.t("sheet-ctrl-prot-saved", "Protection preview saved.")
        : C.t(result.key, "Enter a password first."), !result.ok);
      ctx.rerender();
    });
    unlock.addEventListener("click", function () {
      var ok = unprotect(password.value);
      ctx.setStatus(ok
        ? C.t("sheet-ctrl-prot-status-open", "Not protected")
        : C.t("sheet-ctrl-prot-wrong-password", "That password does not match the local preview."), !ok);
      ctx.rerender();
    });
    checks.forEach(function (check) {
      check.addEventListener("change", function () {
        setAllowance(check.dataset.allowance, check.checked);
        ctx.setStatus(C.t("sheet-ctrl-prot-saved", "Protection preview saved."), false);
      });
    });
  }

  window.SheetControlsProtection = {
    ALLOWANCES: ALLOWANCES,
    ACTION_ALLOWANCE: ACTION_ALLOWANCE,
    inRange: inRange,
    isAllowed: isAllowed,
    applyGuards: applyGuards,
    protect: protect,
    unprotect: unprotect,
    setAllowance: setAllowance,
    statusText: statusText,
    render: render,
  };
})();
