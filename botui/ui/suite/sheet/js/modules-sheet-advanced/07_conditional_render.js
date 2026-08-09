"use strict";
/* Sheet advanced module: 07_conditional_render — render conditional formatting rules on the grid */

(function () {
  let wrapped = false;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function rulesFor() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return [];
    const ws = sheet.worksheets[wsIndex()];
    return ws.conditional_formats || [];
  }

  function validationsFor() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return {};
    const ws = sheet.worksheets[wsIndex()];
    return ws.validations || {};
  }

  function isNum(v) {
    if (v == null || v === "") return false;
    return !isNaN(Number(v));
  }

  function validateCellValue(value, rule) {
    const v = value == null ? "" : String(value);
    switch (rule.validation_type) {
      case "number":
        return isNum(v);
      case "integer":
        return /^-?\d+$/.test(v.trim());
      case "list":
        if (rule.allowed_values && rule.allowed_values.length) return rule.allowed_values.indexOf(v) >= 0;
        return true;
      case "date":
        return /^\d{4}-\d{2}-\d{2}$/.test(v);
      case "text_length": {
        const len = v.length;
        const min = rule.value1 != null && rule.value1 !== "" ? parseInt(rule.value1, 10) : 0;
        const max = rule.value2 != null && rule.value2 !== "" ? parseInt(rule.value2, 10) : Number.MAX_SAFE_INTEGER;
        return len >= min && len <= max;
      }
      case "custom":
        return true;
      default:
        return true;
    }
  }

  function validationForCell(row, col) {
    const vals = validationsFor();
    return vals[row + "," + col] || null;
  }

  function validateEdit(row, col, value) {
    const rule = validationForCell(row, col);
    if (!rule) return { valid: true, message: null };
    const valid = validateCellValue(value, rule);
    return { valid: valid, message: valid ? null : (rule.error_message || "Valor inválido") };
  }

  function num(v) {
    if (v == null || v === "") return NaN;
    return Number(v);
  }

  function str(v) {
    return v == null ? "" : String(v);
  }

  function evalRule(rule, cellValue) {
    const cond = rule.condition || "";
    const cv = str(cellValue);
    const n = num(cellValue);
    if (rule.rule_type === "colorScale") return true;
    if (rule.rule_type === "duplicates") return true;
    if (cond.indexOf("text:contains:") === 0) return cv.toLowerCase().indexOf(cond.slice(14).toLowerCase()) >= 0;
    if (cond.indexOf("text:startswith:") === 0) return cv.toLowerCase().indexOf(cond.slice(16).toLowerCase()) === 0;
    if (cond.indexOf("text:endswith:") === 0) return cv.toLowerCase().endsWith(cond.slice(14).toLowerCase());
    if (cond.indexOf("between:") === 0) {
      const parts = cond.slice(8).split(":");
      const lo = num(parts[0]);
      const hi = num(parts[1]);
      return !isNaN(n) && n >= lo && n <= hi;
    }
    const m = cond.match(/^(>=|<=|<>|>|<|=)\s*(.+)$/);
    if (!m) return false;
    const op = m[1];
    const rhs = m[2].trim();
    const rhsNum = num(rhs);
    if (!isNaN(rhsNum) && !isNaN(n)) {
      switch (op) {
        case ">": return n > rhsNum;
        case "<": return n < rhsNum;
        case ">=": return n >= rhsNum;
        case "<=": return n <= rhsNum;
        case "=": return n === rhsNum;
        case "<>": return n !== rhsNum;
      }
    }
    switch (op) {
      case ">": return cv > rhs;
      case "<": return cv < rhs;
      case ">=": return cv >= rhs;
      case "<=": return cv <= rhs;
      case "=": return cv === rhs;
      case "<>": return cv !== rhs;
    }
    return false;
  }

  function countInRange(rule, value) {
    const g = grid();
    let count = 0;
    for (let r = rule.start_row; r <= rule.end_row; r++) {
      for (let c = rule.start_col; c <= rule.end_col; c++) {
        const d = g.cells.get(r + "," + c);
        const v = d ? (d.value != null ? String(d.value) : "") : "";
        if (v === String(value)) count++;
      }
    }
    return count;
  }

  function applyRules() {
    const g = grid();
    const rules = rulesFor();
    if (!g || !rules.length) return;
    const visible = g.visibleRowRange();
    for (let i = 0; i < rules.length; i++) {
      const rule = rules[i];
      for (let r = Math.max(rule.start_row, visible.start); r <= Math.min(rule.end_row, visible.end - 1); r++) {
        for (let c = rule.start_col; c <= rule.end_col; c++) {
          const d = g.cells.get(r + "," + c);
          if (!d) continue;
          const display = d.value != null ? String(d.value) : "";
          let hit = evalRule(rule, display);
          if (hit && rule.rule_type === "duplicates") {
            hit = countInRange(rule, display) > 1 && display !== "";
          }
          if (!hit) continue;
          const node = g.bodyInner.querySelector('[data-row="' + r + '"][data-col="' + c + '"]');
          if (!node) continue;
          applyRuleStyle(node, rule.style);
        }
      }
    }
    applyValidationMarkers(g, visible);
  }

  function applyValidationMarkers(g, visible) {
    const vals = validationsFor();
    const hasVals = Object.keys(vals).length > 0;
    for (let r = visible.start; r < visible.end; r++) {
      for (let c = 0; c < g.totalCols; c++) {
        const key = r + "," + c;
        const d = g.cells.get(key);
        const hasVal = hasVals && !!vals[key];
        const hasComment = d && (d.has_comment || d.note);
        if (!hasVal && !hasComment) continue;
        const node = g.bodyInner.querySelector('[data-row="' + r + '"][data-col="' + c + '"]');
        if (!node) continue;
        if (hasVal && !node.querySelector(".ss-validation-dot")) {
          const dot = document.createElement("span");
          dot.className = "ss-validation-dot";
          dot.style.cssText = "position:absolute;right:2px;top:2px;width:6px;height:6px;border-radius:50%;background:#ef4444;";
          node.style.position = "relative";
          node.appendChild(dot);
        }
        if (hasComment && !node.querySelector(".ss-comment-marker")) {
          const mark = document.createElement("span");
          mark.className = "ss-comment-marker";
          mark.style.cssText = "position:absolute;right:2px;bottom:2px;width:0;height:0;border-right:6px solid transparent;border-bottom:6px solid #f59e0b;";
          node.style.position = "relative";
          node.appendChild(mark);
          if (d.note) {
            mark.title = String(d.note);
          }
        }
      }
    }
  }

  function applyRuleStyle(node, style) {
    if (!style) return;
    if (style.background) node.style.backgroundColor = style.background;
    if (style.color) node.style.color = style.color;
    if (style.font_weight) node.style.fontWeight = style.font_weight;
    if (style.font_style) node.style.fontStyle = style.font_style;
    if (style.text_decoration) node.style.textDecoration = style.text_decoration;
  }

  function wrapRender() {
    const g = grid();
    if (!g || wrapped) return;
    const orig = g.render.bind(g);
    g.render = function () {
      orig();
      applyRules();
    };
    wrapped = true;
  }

  function wire() {
    if (!grid() || !grid().render) {
      setTimeout(wire, 100);
      return;
    }
    wrapRender();
    applyRules();
  }

  window.SheetConditionalRender = {
    apply: applyRules,
    wire: wire,
  };

  if (window.SheetCore) {
    window.SheetCore.conditionalRender = applyRules;
    window.SheetCore.validateEdit = validateEdit;
    window.SheetCore.validationForCell = validationForCell;
  }

  setTimeout(wire, 0);
})();