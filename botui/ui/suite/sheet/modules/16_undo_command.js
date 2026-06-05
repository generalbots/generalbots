"use strict";

/**
 * Module 16: Command-pattern undo/redo for Sheet.
 * Replaces the JSON.stringify(state.worksheets) snapshot strategy with
 * an inverse-operation history. Each user action (cell edit, row/col
 * insert/delete, merge/unmerge, format change, etc.) is captured as a
 * Command with do() / undo() methods. Edits are batched within a short
 * time window (250ms) into a single history entry to coalesce keystrokes.
 *
 * Public API: window.SheetUndo = {
 *   execute(command), undo(), redo(), canUndo(), canRedo(),
 *   beginTransaction(), endTransaction(), saveCheckpoint(),
 *   clear(), getStackInfo()
 * }.
 */

(function () {
  const STACK_LIMIT = 200;
  const BATCH_WINDOW_MS = 250;

  const stack = [];
  let redoStack = [];
  let pending = null;
  let pendingTimer = null;
  let inTransaction = false;
  let transactionCmds = [];

  function makeId() {
    return "cmd-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8);
  }

  function getWorksheet() {
    if (!window.state || !window.state.worksheets) return null;
    return window.state.worksheets[window.state.activeWorksheet];
  }

  function push(cmd) {
    if (inTransaction) {
      transactionCmds.push(cmd);
      return;
    }
    stack.push(cmd);
    if (stack.length > STACK_LIMIT) stack.shift();
    redoStack = [];
    notifyChange();
  }

  function flushPending() {
    if (pending) {
      push(pending);
      pending = null;
    }
    if (pendingTimer) {
      clearTimeout(pendingTimer);
      pendingTimer = null;
    }
  }

  function execute(command) {
    if (!command) return;
    if (command.do) command.do();
    if (pending && pending.canCoalesce && pending.canCoalesce(command)) {
      pending.coalesce(command);
      if (pendingTimer) clearTimeout(pendingTimer);
      pendingTimer = setTimeout(flushPending, BATCH_WINDOW_MS);
    } else {
      flushPending();
      pending = command;
      pendingTimer = setTimeout(flushPending, BATCH_WINDOW_MS);
    }
  }

  function undo() {
    flushPending();
    const cmd = stack.pop();
    if (!cmd) return false;
    if (cmd.undo) cmd.undo();
    redoStack.push(cmd);
    notifyChange();
    return true;
  }

  function redo() {
    const cmd = redoStack.pop();
    if (!cmd) return false;
    if (cmd.do) cmd.do();
    stack.push(cmd);
    notifyChange();
    return true;
  }

  function canUndo() {
    flushPending();
    return stack.length > 0;
  }

  function canRedo() {
    return redoStack.length > 0;
  }

  function beginTransaction(name) {
    flushPending();
    inTransaction = true;
    transactionCmds = [];
    transactionCmds.__name = name || "transaction";
  }

  function endTransaction() {
    if (!inTransaction) return null;
    inTransaction = false;
    if (transactionCmds.length === 0) return null;
    const batch = {
      id: makeId(),
      type: "batch",
      name: transactionCmds.__name,
      commands: transactionCmds.slice(),
      do() {
        for (let i = 0; i < this.commands.length; i++) {
          if (this.commands[i].do) this.commands[i].do();
        }
      },
      undo() {
        for (let i = this.commands.length - 1; i >= 0; i--) {
          if (this.commands[i].undo) this.commands[i].undo();
        }
      },
    };
    transactionCmds = [];
    push(batch);
    return batch;
  }

  function saveCheckpoint(name) {
    flushPending();
    const ws = getWorksheet();
    if (!ws) return null;
    const snapshot = JSON.stringify({ data: ws.data, merges: ws.merges, validations: ws.validations });
    const cmd = {
      id: makeId(),
      type: "checkpoint",
      name: name || "snapshot",
      before: null,
      after: snapshot,
      do() { /* no-op */ },
      undo() { /* no-op */ },
    };
    push(cmd);
    return cmd;
  }

  function clear() {
    stack.length = 0;
    redoStack.length = 0;
    pending = null;
    if (pendingTimer) clearTimeout(pendingTimer);
    pendingTimer = null;
    notifyChange();
  }

  function getStackInfo() {
    return {
      undoCount: stack.length,
      redoCount: redoStack.length,
      lastCommand: stack.length ? stack[stack.length - 1].name || stack[stack.length - 1].type : null,
      inTransaction,
    };
  }

  function notifyChange() {
    document.dispatchEvent(new CustomEvent("sheetUndoChange", { detail: getStackInfo() }));
  }

  function makeSetCellValueCommand(row, col, newValue, oldValue) {
    return {
      id: makeId(),
      type: "setCellValue",
      name: "Edit cell " + (col + 1) + (row + 1),
      row, col, newValue, oldValue,
      do() { applyCellValue(this.row, this.col, this.newValue); },
      undo() { applyCellValue(this.row, this.col, this.oldValue); },
      canCoalesce(other) {
        return other.type === "setCellValue" && other.row === this.row && other.col === this.col;
      },
      coalesce(other) { this.newValue = other.newValue; },
    };
  }

  function applyCellValue(row, col, value) {
    if (!window.setCellValue) return;
    window.setCellValue(row, col, value, { skipValidation: true, skipHistory: true });
  }

  function makeInsertRowCommand(insertAt) {
    const ws = getWorksheet();
    if (!ws) return null;
    const affected = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (r >= insertAt) affected.push({ key, value: JSON.parse(JSON.stringify(ws.data[key])) });
    }
    return {
      id: makeId(),
      type: "insertRow",
      name: "Insert row at " + (insertAt + 1),
      insertAt, affected,
      do() { shiftRows(this.insertAt, 1); },
      undo() { shiftRows(this.insertAt, -1); restoreCells(this.affected); },
    };
  }

  function makeInsertColCommand(insertAt) {
    const ws = getWorksheet();
    if (!ws) return null;
    const affected = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c >= insertAt) affected.push({ key, value: JSON.parse(JSON.stringify(ws.data[key])) });
    }
    return {
      id: makeId(),
      type: "insertCol",
      name: "Insert column at " + (insertAt + 1),
      insertAt, affected,
      do() { shiftCols(this.insertAt, 1); },
      undo() { shiftCols(this.insertAt, -1); restoreCells(this.affected); },
    };
  }

  function makeDeleteRowCommand(row) {
    const ws = getWorksheet();
    if (!ws) return null;
    const removed = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (r === row) removed.push({ key, value: JSON.parse(JSON.stringify(ws.data[key])) });
    }
    const below = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (r > row) below.push({ key, value: JSON.parse(JSON.stringify(ws.data[key])) });
    }
    return {
      id: makeId(),
      type: "deleteRow",
      name: "Delete row " + (row + 1),
      row, removed, below,
      do() {
        removeCellsByRow(this.row);
        shiftRows(this.row, -1);
      },
      undo() {
        shiftRows(this.row, 1);
        restoreCells(this.removed);
      },
    };
  }

  function makeDeleteColCommand(col) {
    const ws = getWorksheet();
    if (!ws) return null;
    const removed = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c === col) removed.push({ key, value: JSON.parse(JSON.stringify(ws.data[key])) });
    }
    const right = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c > col) right.push({ key, value: JSON.parse(JSON.stringify(ws.data[key])) });
    }
    return {
      id: makeId(),
      type: "deleteCol",
      name: "Delete column " + (col + 1),
      col, removed, right,
      do() {
        removeCellsByCol(this.col);
        shiftCols(this.col, -1);
      },
      undo() {
        shiftCols(this.col, 1);
        restoreCells(this.removed);
      },
    };
  }

  function makeMergeCommand(startRow, startCol, endRow, endCol) {
    const ws = getWorksheet();
    if (!ws) return null;
    if (!ws.merges) ws.merges = [];
    const existing = (ws.merges || []).slice();
    return {
      id: makeId(),
      type: "merge",
      name: "Merge cells",
      startRow, startCol, endRow, endCol,
      do() {
        if (!ws.merges) ws.merges = [];
        const exists = ws.merges.find(m =>
          m.startRow === this.startRow && m.startCol === this.startCol &&
          m.endRow === this.endRow && m.endCol === this.endCol);
        if (!exists) ws.merges.push({ startRow: this.startRow, startCol: this.startCol, endRow: this.endRow, endCol: this.endCol });
      },
      undo() { ws.merges = existing; },
    };
  }

  function makeUnmergeCommand(mergeIndex) {
    const ws = getWorksheet();
    if (!ws || !ws.merges || !ws.merges[mergeIndex]) return null;
    const removed = ws.merges[mergeIndex];
    const existing = ws.merges.slice();
    return {
      id: makeId(),
      type: "unmerge",
      name: "Unmerge cells",
      mergeIndex, removed, existing,
      do() {
        ws.merges = (ws.merges || []).filter((_, i) => i !== this.mergeIndex);
      },
      undo() { ws.merges = this.existing; },
    };
  }

  function shiftRows(atRow, delta) {
    const ws = getWorksheet();
    if (!ws) return;
    const updates = {};
    const remove = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (r >= atRow) {
        const newKey = (r + delta) + "," + c;
        updates[newKey] = ws.data[key];
        remove.push(key);
      }
    }
    for (const k of remove) delete ws.data[k];
    Object.assign(ws.data, updates);
  }

  function shiftCols(atCol, delta) {
    const ws = getWorksheet();
    if (!ws) return;
    const updates = {};
    const remove = [];
    for (const key in ws.data) {
      const [r, c] = key.split(",").map(Number);
      if (c >= atCol) {
        const newKey = r + "," + (c + delta);
        updates[newKey] = ws.data[key];
        remove.push(key);
      }
    }
    for (const k of remove) delete ws.data[k];
    Object.assign(ws.data, updates);
  }

  function removeCellsByRow(row) {
    const ws = getWorksheet();
    if (!ws) return;
    for (const key in ws.data) {
      const [r] = key.split(",").map(Number);
      if (r === row) delete ws.data[key];
    }
  }

  function removeCellsByCol(col) {
    const ws = getWorksheet();
    if (!ws) return;
    for (const key in ws.data) {
      const [, c] = key.split(",").map(Number);
      if (c === col) delete ws.data[key];
    }
  }

  function restoreCells(list) {
    const ws = getWorksheet();
    if (!ws) return;
    for (const item of list) ws.data[item.key] = item.value;
  }

  function installKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
      if (!(e.ctrlKey || e.metaKey)) return;
      const key = e.key.toLowerCase();
      if (key === "z" && !e.shiftKey) {
        if (canUndo()) { e.preventDefault(); undo(); }
      } else if ((key === "z" && e.shiftKey) || key === "y") {
        if (canRedo()) { e.preventDefault(); redo(); }
      }
    });
  }

  installKeyboardShortcuts();

  window.SheetUndo = {
    execute,
    undo,
    redo,
    canUndo,
    canRedo,
    beginTransaction,
    endTransaction,
    saveCheckpoint,
    clear,
    getStackInfo,
    commands: {
      setCellValue: makeSetCellValueCommand,
      insertRow: makeInsertRowCommand,
      insertCol: makeInsertColCommand,
      deleteRow: makeDeleteRowCommand,
      deleteCol: makeDeleteColCommand,
      merge: makeMergeCommand,
      unmerge: makeUnmergeCommand,
    },
  };
})();
