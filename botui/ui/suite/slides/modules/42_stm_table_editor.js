"use strict";
/* Slides table & media authoring (#2487) — in-canvas cell editing, structural
 * table operations and a single-step undo/redo scoped to table edits. */
(function () {
  var STM = window.SlidesTableMedia = window.SlidesTableMedia || {};
  var state = {
    activeId: null, model: null, options: null, cell: null,
    mergeMode: false, anchor: null, panel: null, undo: [], redo: [], editing: null
  };
  STM.tableState = state;

  function clone(value) { return STM.cloneTable(value); }

  function selectCell(cell) {
    if (!cell) return;
    state.cell = cell;
    var table = cell.closest("table");
    if (!table) return;
    Array.prototype.slice.call(table.querySelectorAll("td,th")).forEach(function (item) {
      item.classList.toggle("stm-cell-selected", item === cell);
      if (item === cell) item.setAttribute("data-stm-selected", "1");
      else item.removeAttribute("data-stm-selected");
    });
  }

  function syncFrom(table) {
    state.activeId = STM.elementIdOf(table);
    state.model = STM.tableModelFromDom(table);
    state.options = clone(state.model ? state.model.options : STM.tableDefaults());
    var pos = state.cell ? cellCoords(state.cell) : null;
    if (pos && table) {
      var fresh = table.querySelector('td[data-stm-r="' + pos.row + '"][data-stm-c="' + pos.col + '"]');
      if (fresh) selectCell(fresh);
    }
    if (state.panel) STM.syncTablePanel();
  }

  STM.tableActivate = function (table) {
    if (!table) return;
    STM.restoreTable(table);
    syncFrom(table);
    if (state.panel) {
      state.panel.hidden = false;
      STM.syncTablePanel();
    }
  };

  STM.tableCommit = function (table, before, after, key, fallback) {
    if (!table || !after) return;
    var oldModel = clone(before.model);
    var oldOptions = clone(before.options || before.model.options);
    var newModel = clone(after.model);
    var newOptions = clone(after.options || after.model.options);
    state.undo.push({ id: STM.elementIdOf(table), before: oldModel, beforeOptions: oldOptions,
      after: newModel, afterOptions: newOptions });
    if (state.undo.length > 100) state.undo.shift();
    state.redo.length = 0;
    STM.tableRender(table, newModel, newOptions);
    STM.persistTable(table, newModel, newOptions);
    syncFrom(table);
    STM.status(key, fallback);
  };

  function applyMutation(mutator, key, fallback) {
    var table = STM.tableById(state.activeId);
    if (!table || !state.model) { STM.status("slides-stm-table-empty", "Select a table on the slide to edit it."); return; }
    var before = { model: clone(state.model), options: clone(state.options) };
    var result = mutator(clone(state.model), clone(state.options), state.cell);
    if (!result) return;
    var after = result.model ? result : { model: result, options: state.options };
    STM.tableCommit(table, before, after, key, fallback);
  }

  function cellCoords(cell) {
    return {
      row: parseInt(cell.getAttribute("data-stm-r"), 10) || 0,
      col: parseInt(cell.getAttribute("data-stm-c"), 10) || 0
    };
  }

  function currentCellModel() {
    var table = STM.tableById(state.activeId);
    var live = table && table.querySelector('[data-stm-r][data-stm-c][data-stm-selected="1"]');
    var cell = live || state.cell;
    return cell ? cellCoords(cell) : { row: 0, col: 0 };
  }

  function finishEdit(commit) {
    var editing = state.editing;
    if (!editing) return;
    state.editing = null;
    var cell = editing.cell;
    cell.removeEventListener("keydown", editing.onKey);
    cell.removeEventListener("blur", editing.onBlur);
    if (cell.isContentEditable) cell.contentEditable = "false";
    var text = commit ? cell.textContent : editing.original;
    cell.textContent = text;
    var table = STM.tableById(state.activeId);
    if (table) {
      var before = { model: clone(state.model), options: clone(state.options) };
      var modelBeforeDom = STM.tableModelFromDom(table);
      var afterModel = STM.tableSetCell(modelBeforeDom, editing.row, editing.col, text);
      if (before.model.grid[editing.row] && before.model.grid[editing.row][editing.col] &&
          before.model.grid[editing.row][editing.col].text !== text) {
        STM.tableCommit(table, before, { model: afterModel, options: state.options },
          "slides-stm-table-cell-saved", "Cell updated");
      }
    }
    selectCell(cell);
  }

  function selectAllText(cell) {
    if (!window.getSelection || !document.createRange) return;
    var range = document.createRange();
    range.selectNodeContents(cell);
    var selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
  }

  function beginEdit(cell) {
    if (state.editing && state.editing.cell === cell) return;
    finishEdit(true);
    var coords = cellCoords(cell);
    var original = cell.textContent || "";
    cell.setAttribute("data-stm-selected", "1");
    selectCell(cell);
    cell.contentEditable = "true";
    cell.spellcheck = true;
    selectAllText(cell);
    var onKey = function (event) {
      if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); finishEdit(true); }
      else if (event.key === "Escape") { event.preventDefault(); finishEdit(false); }
      else if (event.key === "Tab") {
        event.preventDefault();
        finishEdit(true);
        var table = STM.tableById(state.activeId);
        if (!table) return;
        var cells = Array.prototype.slice.call(table.querySelectorAll("td,th"));
        var index = cells.indexOf(cell);
        var next = cells[index + (event.shiftKey ? -1 : 1)];
        if (next) { next.setAttribute("data-stm-selected", "1"); beginEdit(next); }
      }
    };
    var onBlur = function () { finishEdit(true); };
    cell.addEventListener("keydown", onKey);
    cell.addEventListener("blur", onBlur);
    state.editing = { cell: cell, original: original, row: coords.row, col: coords.col, onKey: onKey, onBlur: onBlur };
    cell.focus();
  }

  function handleMergeSelect(cell) {
    var coords = cellCoords(cell);
    if (!state.anchor) {
      state.anchor = coords;
      selectCell(cell);
      STM.status("slides-stm-table-merge-first", "Choose the second cell to merge.");
      return;
    }
    var result = STM.tableMerge(state.model, state.anchor.row, state.anchor.col, coords.row, coords.col);
    state.anchor = null;
    state.mergeMode = false;
    if (state.panel && state.panel.mergeBtn) state.panel.mergeBtn.setAttribute("aria-pressed", "false");
    applyMutation(function () { return result; }, "slides-stm-table-merged", "Cells merged");
  }

  function onCellClick(event) {
    var cell = event.target.closest ? event.target.closest("td,th") : null;
    if (!cell || !cell.closest("table.stm-table")) return;
    var table = cell.closest("table");
    if (STM.elementIdOf(table) !== state.activeId) STM.tableActivate(table);
    event.stopPropagation();
    if (state.mergeMode) { handleMergeSelect(cell); return; }
    beginEdit(cell);
  }
  function onCellMouseDown(event) {
    var cell = event.target.closest ? event.target.closest("td,th") : null;
    if (cell && cell.closest("table.stm-table")) event.stopPropagation();
  }

  STM.tableUndoLast = function () {
    var action = state.undo.pop();
    if (!action) { STM.status("slides-stm-undo-empty", "Nothing to undo."); return; }
    var table = STM.tableById(action.id);
    if (table) {
      STM.tableRender(table, action.before, action.beforeOptions);
      STM.persistTable(table, action.before, action.beforeOptions);
      syncFrom(table);
    }
    state.redo.push(action);
    STM.status("slides-stm-undo-done", "Undo applied");
  };
  STM.tableRedoLast = function () {
    var action = state.redo.pop();
    if (!action) { STM.status("slides-stm-redo-empty", "Nothing to redo."); return; }
    var table = STM.tableById(action.id);
    if (table) {
      STM.tableRender(table, action.after, action.afterOptions);
      STM.persistTable(table, action.after, action.afterOptions);
      syncFrom(table);
    }
    state.undo.push(action);
    STM.status("slides-stm-redo-done", "Redo applied");
  };

  function labelledControl(tag, attr, key, fallback) {
    var el = document.createElement(tag);
    STM.labelled(el, attr, key, fallback);
    return el;
  }

  function buildPanel() {
    var panel = document.createElement("div");
    panel.className = "stm-table-panel";
    panel.id = "stm-table-panel";
    panel.hidden = true;
    panel.setAttribute("role", "toolbar");
    STM.labelled(panel, "data-i18n-aria-label", "slides-stm-table-title", "Table editing");
    var hint = labelledControl("p", "data-i18n", "slides-stm-table-hint", "Click a cell to edit it. Each change is one undoable step.");
    hint.className = "stm-panel-hint";
    panel.appendChild(hint);

    var groups = document.createElement("div");
    groups.className = "stm-panel-grid";
    panel.appendChild(groups);

    function group(key, fallback) {
      var wrap = document.createElement("div");
      wrap.className = "stm-control-group";
      var legend = labelledControl("span", "data-i18n", key, fallback);
      legend.className = "stm-control-legend";
      wrap.appendChild(legend);
      groups.appendChild(wrap);
      return wrap;
    }
    function action(parent, key, fallback, handler) {
      var b = STM.button(key, fallback, handler);
      parent.appendChild(b);
      return b;
    }
    function toggle(parent, key, fallback, checked, onChange) {
      var label = document.createElement("label");
      label.className = "stm-check";
      var input = document.createElement("input");
      input.type = "checkbox";
      input.checked = !!checked;
      input.addEventListener("change", function () { onChange(input.checked); });
      label.appendChild(input);
      label.appendChild(labelledControl("span", "data-i18n", key, fallback));
      parent.appendChild(label);
      return input;
    }

    var rows = group("slides-stm-table-rows", "Rows");
    action(rows, "slides-stm-table-row-insert-above", "Insert row above", function () {
      applyMutation(function (model) { return STM.tableInsertRow(model, currentCellModel().row, false); },
        "slides-stm-table-row-added", "Row inserted");
    });
    action(rows, "slides-stm-table-row-insert-below", "Insert row below", function () {
      applyMutation(function (model) { return STM.tableInsertRow(model, currentCellModel().row, true); },
        "slides-stm-table-row-added", "Row inserted");
    });
    action(rows, "slides-stm-table-row-delete", "Delete row", function () {
      STM.confirmDialog("slides-stm-table-title", "Table editing", "slides-stm-table-delete-confirm", "Delete this row?", function () {
        applyMutation(function (model) { return STM.tableDeleteRow(model, currentCellModel().row); },
          "slides-stm-table-row-removed", "Row deleted");
      });
    });

    var cols = group("slides-stm-table-columns", "Columns");
    action(cols, "slides-stm-table-col-insert-left", "Insert column left", function () {
      applyMutation(function (model) { return STM.tableInsertCol(model, currentCellModel().col, false); },
        "slides-stm-table-col-added", "Column inserted");
    });
    action(cols, "slides-stm-table-col-insert-right", "Insert column right", function () {
      applyMutation(function (model) { return STM.tableInsertCol(model, currentCellModel().col, true); },
        "slides-stm-table-col-added", "Column inserted");
    });
    action(cols, "slides-stm-table-col-delete", "Delete column", function () {
      STM.confirmDialog("slides-stm-table-title", "Table editing", "slides-stm-table-delete-col-confirm", "Delete this column?", function () {
        applyMutation(function (model) { return STM.tableDeleteCol(model, currentCellModel().col); },
          "slides-stm-table-col-removed", "Column deleted");
      });
    });

    var cells = group("slides-stm-table-cells", "Cells");
    panel.mergeBtn = action(cells, "slides-stm-table-merge", "Merge cells", function () {
      finishEdit(true);
      state.mergeMode = !state.mergeMode;
      state.anchor = null;
      panel.mergeBtn.setAttribute("aria-pressed", state.mergeMode ? "true" : "false");
      STM.status("slides-stm-table-merge-start", "Choose the first cell to merge.");
    });
    panel.mergeBtn.setAttribute("aria-pressed", "false");
    action(cells, "slides-stm-table-split", "Split cell", function () {
      var pos = currentCellModel();
      if (!pos) { STM.status("slides-stm-table-select-cell", "Select a cell first"); return; }
      applyMutation(function (model) { return STM.tableSplit(model, pos.row, pos.col); },
        "slides-stm-table-split-done", "Cell split");
    });

    var look = group("slides-stm-table-appearance", "Appearance");
    panel.headerInput = toggle(look, "slides-stm-table-header", "Header row", state.options && state.options.header, function (value) {
      applyMutation(function (model, options) { options.header = value; return { model: model, options: options }; },
        "slides-stm-table-style-saved", "Table style updated");
    });
    panel.bandedInput = toggle(look, "slides-stm-table-banded", "Banded rows", false, function (value) {
      applyMutation(function (model, options) { options.banded = value; return { model: model, options: options }; },
        "slides-stm-table-style-saved", "Table style updated");
    });
    panel.bordersInput = toggle(look, "slides-stm-table-borders", "Borders", true, function (value) {
      applyMutation(function (model, options) { options.borders = value; return { model: model, options: options }; },
        "slides-stm-table-style-saved", "Table style updated");
    });

    var paddingLabel = labelledControl("label", "data-i18n", "slides-stm-table-padding", "Padding");
    paddingLabel.className = "stm-field-label";
    var padding = document.createElement("input");
    padding.type = "number";
    padding.min = "0";
    padding.max = "40";
    padding.step = "1";
    padding.value = "4";
    STM.labelled(padding, "data-i18n-aria-label", "slides-stm-table-padding", "Padding");
    padding.addEventListener("change", function () {
      applyMutation(function (model, options) { options.padding = Number(padding.value) || 0; return { model: model, options: options }; },
        "slides-stm-table-style-saved", "Table style updated");
    });
    paddingLabel.appendChild(padding);
    look.appendChild(paddingLabel);

    var alignLabel = labelledControl("label", "data-i18n", "slides-stm-table-align", "Alignment");
    alignLabel.className = "stm-field-label";
    var align = document.createElement("select");
    [["left", "slides-stm-table-align-left", "Left"], ["center", "slides-stm-table-align-center", "Center"],
      ["right", "slides-stm-table-align-right", "Right"]].forEach(function (entry) {
      var option = document.createElement("option");
      option.value = entry[0];
      STM.labelled(option, "data-i18n", entry[1], entry[2]);
      align.appendChild(option);
    });
    STM.labelled(align, "data-i18n-aria-label", "slides-stm-table-align", "Alignment");
    align.addEventListener("change", function () {
      applyMutation(function (model, options) { options.align = align.value; return { model: model, options: options }; },
        "slides-stm-table-style-saved", "Table style updated");
    });
    alignLabel.appendChild(align);
    look.appendChild(alignLabel);

    var styleLabel = labelledControl("label", "data-i18n", "slides-stm-table-widths", "Column widths");
    styleLabel.className = "stm-field-label";
    panel.widths = document.createElement("div");
    panel.widths.className = "stm-widths";
    styleLabel.appendChild(panel.widths);
    look.appendChild(styleLabel);

    var styleSelectLabel = labelledControl("label", "data-i18n", "slides-stm-table-style", "Theme style");
    styleSelectLabel.className = "stm-field-label";
    var styles = document.createElement("select");
    [["plain", "slides-stm-table-style-plain", "Plain"], ["striped", "slides-stm-table-style-striped", "Striped"],
      ["grid", "slides-stm-table-style-grid", "Grid"], ["minimal", "slides-stm-table-style-minimal", "Minimal"]].forEach(function (entry) {
      var option = document.createElement("option");
      option.value = entry[0];
      STM.labelled(option, "data-i18n", entry[1], entry[2]);
      styles.appendChild(option);
    });
    styles.addEventListener("change", function () {
      applyMutation(function (model, options) { options.style = styles.value; return { model: model, options: options }; },
        "slides-stm-table-style-saved", "Table style updated");
    });
    styleSelectLabel.appendChild(styles);
    look.appendChild(styleSelectLabel);
    panel.styleSelect = styles;
    panel.alignSelect = align;
    panel.paddingInput = padding;

    var history = group("slides-stm-table-history", "History");
    action(history, "slides-stm-undo", "Undo", STM.tableUndoLast);
    action(history, "slides-stm-redo", "Redo", STM.tableRedoLast);
    action(history, "slides-stm-close", "Close", function () { panel.hidden = true; });

    var note = document.createElement("p");
    STM.labelled(note, "data-i18n", "slides-stm-local-only", "Local only — not saved to the server");
    note.className = "stm-local-note";
    panel.appendChild(note);

    document.body.appendChild(panel);
    return panel;
  }

  STM.openTableEditor = function () {
    if (!state.panel) state.panel = buildPanel();
    var tables = STM.tableElements();
    if (!tables.length) {
      STM.status("slides-stm-table-empty", "Select a table on the slide to edit it.");
      return;
    }
    STM.tableActivate(tables[0]);
    STM.syncTablePanel();
    state.panel.hidden = false;
  };

  function bind() {
    document.addEventListener("click", onCellClick, true);
    document.addEventListener("mousedown", onCellMouseDown, true);
    document.addEventListener("keydown", function (event) {
      var editingInput = event.target && event.target.closest &&
        event.target.closest("input,textarea,select,[contenteditable='true']");
      if (!state.activeId || editingInput) return;
      if (!(event.ctrlKey || event.metaKey)) return;
      var key = (event.key || "").toLowerCase();
      if (key === "z" && !event.shiftKey) { event.preventDefault(); event.stopImmediatePropagation(); STM.tableUndoLast(); }
      else if ((key === "z" && event.shiftKey) || key === "y") {
        event.preventDefault(); event.stopImmediatePropagation(); STM.tableRedoLast();
      }
    }, true);
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind);
  else bind();
})();
