"use strict";
/* Sheet advanced module: 90_validation_manager — in-app data-validation rule
   manager (define / list / remove rules, highlight invalid grid cells).
   Loaded last; plain IIFE, no CDN. With no sheet app present the module is a
   no-op (only passive listeners). HTMX-safe: every DOM lookup is lazy, so a
   re-targeted window or replaced grid is re-discovered on demand. */
(function () {
  var APP_SEL = '[data-gb-app="sheets"], #sheet-app';
  var PANEL_ID = "gb-dv-manager", LAUNCHER_ID = "gb-dv-launcher";
  var REG_KEY = "validation_rule_registry", INVALID_ATTR = "data-gb-dv-invalid";
  var state = { rules: [], marked: [], grid: null, bodyInner: null, seq: 0, open: false };

  function t(key, fallback, params) {
    if (typeof window.gbT === "function") {
      try { return window.gbT(key, fallback, params) || fallback; } catch (e) { return fallback; }
    }
    return fallback;
  }
  function hasApp() {
    return !!(typeof document !== "undefined" && document.querySelector && document.querySelector(APP_SEL));
  }
  function esc(v) {
    return String(v == null ? "" : v).replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function notify(msg, isError) {
    if (typeof window.sheetNotify === "function") { window.sheetNotify(msg, !!isError); return; }
    try { console[isError ? "error" : "log"](msg); } catch (e) { /* best effort */ }
  }
  function grid() {
    return (window.SheetCore && typeof window.SheetCore.getGrid === "function")
      ? window.SheetCore.getGrid() : (window.SheetVirtualGrid || null);
  }
  function worksheet() {
    var s = window.__LOADED_SHEET;
    if (!s || !s.worksheets || !s.worksheets.length) return null;
    var i = (window.SheetCore && typeof window.SheetCore.wsIndex === "function") ? window.SheetCore.wsIndex() : 0;
    return s.worksheets[i] || null;
  }
  function validations(ws, create) {
    if (!ws) return null;
    if (!ws.validations || typeof ws.validations !== "object") { if (!create) return null; ws.validations = {}; }
    return ws.validations;
  }
  function registry(ws) {
    if (!ws) return state.rules;
    if (!Array.isArray(ws[REG_KEY])) ws[REG_KEY] = [];
    return ws[REG_KEY];
  }
  function colName(idx) {
    var n = idx + 1, out = "";
    while (n > 0) { out = String.fromCharCode(65 + (n - 1) % 26) + out; n = Math.floor((n - 1) / 26); }
    return out;
  }
  function parseCell(token) {
    var m = /^\$?([A-Za-z]+)\$?(\d+)$/.exec(String(token || "").trim());
    if (!m) return null;
    var letters = m[1].toUpperCase(), col = 0;
    for (var i = 0; i < letters.length; i++) col = col * 26 + (letters.charCodeAt(i) - 64);
    var row = parseInt(m[2], 10) - 1;
    return (col < 1 || row < 0 || isNaN(row)) ? null : { row: row, col: col - 1 };
  }
  function parseRange(ref) {
    var parts = String(ref || "").trim().split(":"), a = parseCell(parts[0]);
    if (!a) return null;
    var b = parts.length > 1 ? parseCell(parts[1]) : a;
    if (!b) return null;
    return { start_row: Math.min(a.row, b.row), end_row: Math.max(a.row, b.row),
      start_col: Math.min(a.col, b.col), end_col: Math.max(a.col, b.col) };
  }
  function serverType(u) {
    return u === "whole" ? "integer" : u === "decimal" ? "number"
      : u === "list" ? "list" : u === "date" ? "date" : "any";
  }
  function syncRules() {
    var ws = worksheet();
    if (!ws) { state.rules = []; return; }
    var reg = registry(ws), vals = validations(ws, false);
    if (vals) Object.keys(vals).forEach(function (key) {
      var cr = vals[key];
      if (!cr || !cr.id || reg.some(function (x) { return x.id === cr.id; })) return;
      reg.push({ id: cr.id, range: cr.range || key,
        ui_type: cr.ui_type || (cr.validation_type === "integer" ? "whole" : cr.validation_type),
        validation_type: cr.validation_type, operator: cr.operator, value1: cr.value1,
        value2: cr.value2, allowed_values: cr.allowed_values, error_message: cr.error_message });
    });
    state.rules = reg;
    reg.forEach(function (rule) {
      var n = parseInt(String(rule.id || "").replace(/^vr-/, ""), 10);
      if (!isNaN(n) && n > state.seq) state.seq = n;
    });
  }
  function mirror(ws, rule, b) {
    var vals = validations(ws, true);
    for (var r = b.start_row; r <= b.end_row; r++) for (var c = b.start_col; c <= b.end_col; c++) {
      vals[r + "," + c] = { id: rule.id, range: rule.range, ui_type: rule.ui_type,
        validation_type: rule.validation_type, operator: rule.operator, value1: rule.value1,
        value2: rule.value2, allowed_values: rule.allowed_values, error_message: rule.error_message };
    }
  }
  function cmp(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
  function againstBounds(value, rule) {
    var op = rule.operator || "any";
    if (op === "any") return true;
    var lo = rule.value1 == null ? "" : String(rule.value1).trim();
    var hi = rule.value2 == null ? "" : String(rule.value2).trim();
    if (lo === "" && hi === "") return true;
    var num = typeof value === "number";
    var l = lo ? (num ? Number(lo) : lo) : null, h = hi ? (num ? Number(hi) : hi) : null;
    if (num && ((lo && isNaN(l)) || (hi && isNaN(h)))) return true;
    switch (op) {
      case "between": return (!lo || cmp(value, l) >= 0) && (!hi || cmp(value, h) <= 0);
      case ">=": case "greaterEqual": return !!lo && cmp(value, l) >= 0;
      case "<=": case "lessEqual": return !!lo && cmp(value, l) <= 0;
      case ">": case "greaterThan": return !!lo && cmp(value, l) > 0;
      case "<": case "lessThan": return !!lo && cmp(value, l) < 0;
      case "=": case "equal": return !!lo && cmp(value, l) === 0;
      case "<>": case "notEqual": return !!lo && cmp(value, l) !== 0;
      default: return true;
    }
  }
  function evaluate(raw, rule) {
    var v = raw == null ? "" : String(raw).trim();
    if (v === "") return true;
    var type = rule.validation_type || serverType(rule.ui_type);
    if (type === "list") { var list = rule.allowed_values || []; return !list.length || list.indexOf(v) >= 0; }
    if (type === "integer") { if (!/^-?\d+$/.test(v)) return false; return againstBounds(Number(v), rule); }
    if (type === "number") { if (isNaN(Number(v))) return false; return againstBounds(Number(v), rule); }
    if (type === "date") { if (!/^\d{4}-\d{2}-\d{2}$/.test(v)) return false; return againstBounds(v, rule); }
    return true;
  }
  function unmark(node) {
    if (!node || !node.getAttribute || node.getAttribute(INVALID_ATTR) !== "1") return;
    for (var i = state.marked.length - 1; i >= 0; i--) if (state.marked[i].node === node) {
      var s = state.marked[i];
      if (s.style) { s.style.backgroundColor = s.bg; s.style.outline = s.outline; }
      state.marked.splice(i, 1);
    }
    if (node.removeAttribute) node.removeAttribute(INVALID_ATTR);
  }
  function mark(node) {
    if (!node || !node.setAttribute || node.getAttribute(INVALID_ATTR) === "1") return;
    var style = node.style || (node.style = {});
    state.marked.push({ node: node, style: style, bg: style.backgroundColor || "", outline: style.outline || "" });
    style.backgroundColor = "rgba(239, 68, 68, 0.28)";
    style.outline = "2px solid #ef4444";
    node.setAttribute(INVALID_ATTR, "1");
  }
  function clearMarks() {
    state.marked.slice().forEach(function (e) { unmark(e.node); });
    state.marked = [];
  }
  function ruleForCell(key) {
    var ws = worksheet(), vals = ws && ws.validations;
    return vals && vals[key] ? vals[key] : null;
  }
  function apply() {
    if (!hasApp()) return;
    syncRules(); clearMarks();
    if (!state.rules.length) return;
    var g = grid();
    if (!g || !g.bodyInner || !g.cells || typeof g.bodyInner.querySelector !== "function") return;
    var rows = typeof g.visibleRowRange === "function" ? g.visibleRowRange() : { start: 0, end: g.totalRows || 200 };
    var cols = typeof g.visibleColRange === "function" ? g.visibleColRange() : { start: 0, end: g.totalCols || 26 };
    for (var r = rows.start; r < rows.end; r++) for (var c = cols.start; c < cols.end; c++) {
      var key = r + "," + c, rule = ruleForCell(key);
      if (!rule) continue;
      var d = g.cells.get(key), raw = d ? d.value : "";
      if (evaluate(raw, rule)) continue;
      var node = g.bodyInner.querySelector('[data-row="' + r + '"][data-col="' + c + '"]');
      if (node) mark(node);
    }
  }
  function clear() { if (hasApp()) clearMarks(); }
  function onGridPointer(e) {
    if (!hasApp() || !state.rules.length) return;
    var target = e.target;
    if (target && target.dataset && target.dataset.row != null && target.dataset.col != null) apply();
  }
  function hookGrid() {
    var g = grid();
    if (!g || !g.bodyInner) return;
    // An HTMX re-target can swap the grid's bodyInner while keeping the same
    // grid object. Re-hook when EITHER changes, otherwise the new cell layer
    // never receives the delegated pointerdown listener (#2474 UAT).
    if (state.grid === g && state.bodyInner === g.bodyInner && g.bodyInner.__gbDvBound) return;
    state.grid = g;
    state.bodyInner = g.bodyInner;
    if (typeof g.render === "function" && !g.__gbDvWrapped) {
      var original = g.render;
      g.__gbDvRender = original;
      g.render = function () { var res = original.apply(this, arguments); apply(); return res; };
      g.__gbDvWrapped = true;
    }
    if (!g.bodyInner.__gbDvBound) {
      g.bodyInner.__gbDvBound = true;
      g.bodyInner.addEventListener("pointerdown", onGridPointer, true);
    }
  }
  function ensureLauncher() {
    if (document.getElementById(LAUNCHER_ID)) return;
    var anchor = document.getElementById("dataValidationBtn");
    var host = anchor && anchor.parentNode ? anchor.parentNode : document.querySelector(".sheet-toolbar");
    if (!host || typeof host.appendChild !== "function") return;
    var btn = document.createElement("button");
    btn.id = LAUNCHER_ID; btn.type = "button"; btn.className = "btn-icon";
    btn.setAttribute("data-vm-act", "open");
    btn.setAttribute("title", t("sheet-data-validation", "Data Validation rules"));
    btn.setAttribute("aria-label", t("sheet-data-validation", "Data Validation rules"));
    btn.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M9 11l3 3L22 4"></path><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>';
    host.appendChild(btn);
  }
  function opt(v, label) { return '<option value="' + esc(v) + '">' + esc(label) + "</option>"; }
  function panelHtml() {
    var inp = "width:100%;box-sizing:border-box;padding:6px;margin-top:3px;background:var(--sheet-bg,#0f172a);border:1px solid var(--sheet-border,#334155);border-radius:4px;color:var(--sheet-fg,#f8fafc);";
    var lab = "display:block;color:var(--sheet-fg,#f8fafc);font-size:12px;margin-bottom:8px;";
    var btn = "padding:7px 10px;border:1px solid var(--sheet-border,#334155);border-radius:4px;background:var(--sheet-bg,#0f172a);color:var(--sheet-fg,#f8fafc);cursor:pointer;font-size:12px;";
    var types = opt("whole", t("sheet-dv-whole", "Whole number")) + opt("decimal", t("sheet-dv-decimal", "Decimal")) +
      opt("list", t("sheet-dv-list", "List")) + opt("date", t("label-date", "Date"));
    var ops = opt("any", t("sheet-dv-any", "Any value")) + opt("between", t("sheet-dv-between", "Between")) +
      opt(">=", t("sheet-dv-gte", "Greater than or equal")) + opt("<=", t("sheet-dv-lte", "Less than or equal")) +
      opt(">", t("sheet-dv-gt", "Greater than")) + opt("<", t("sheet-dv-lt", "Less than")) +
      opt("=", t("sheet-dv-eq", "Equal to")) + opt("<>", t("sheet-dv-neq", "Not equal to"));
    function field(label, control) { return '<label style="' + lab + '">' + esc(label) + control + "</label>"; }
    function text(id, ph) { return '<input id="' + id + '" type="text" placeholder="' + esc(ph || "") + '" style="' + inp + '" />'; }
    return '' +
      '<div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--sheet-border,#334155);padding-bottom:8px;margin-bottom:10px;">' +
        '<strong data-i18n="sheet-data-validation">' + esc(t("sheet-data-validation", "Data Validation")) + '</strong>' +
        '<button type="button" data-vm-act="close" data-i18n-aria-label="action-close" aria-label="' + esc(t("action-close", "Close")) + '" style="' + btn + '">×</button></div>' +
      field(t("sheet-dv-range", "Cell range (e.g. A1:C10)"), text("vm-range", "A1:C10")) +
      field(t("label-type", "Type"), '<select id="vm-type" style="' + inp + '">' + types + "</select>") +
      field(t("sheet-dv-operator", "Operator"), '<select id="vm-op" style="' + inp + '">' + ops + "</select>") +
      field(t("sheet-dv-value", "Value"), text("vm-value1")) +
      field(t("sheet-dv-value2", "Second value (for Between)"), text("vm-value2")) +
      field(t("sheet-dv-list", "Allowed values (comma-separated)"), text("vm-list")) +
      field(t("sheet-dv-error", "Error message"), text("vm-msg")) +
      '<div style="display:flex;gap:6px;margin:6px 0 12px;">' +
        '<button type="button" data-vm-act="add" data-i18n="action-add" style="' + btn + '">' + esc(t("action-add", "Add")) + '</button>' +
        '<button type="button" data-vm-act="apply" data-i18n="action-apply" style="' + btn + '">' + esc(t("action-apply", "Apply")) + '</button>' +
        '<button type="button" data-vm-act="clear" data-i18n="action-clear" style="' + btn + '">' + esc(t("action-clear", "Clear")) + '</button></div>' +
      '<div data-i18n="sheet-validations" style="font-weight:600;margin-bottom:6px;">' + esc(t("sheet-validations", "Validations")) + '</div><div id="vm-rules"></div>';
  }
  function renderList() {
    syncRules();
    var host = document.getElementById("vm-rules");
    if (!host || typeof host.innerHTML === "undefined") return;
    if (!state.rules.length) { host.textContent = t("label-no-data", "No validation rules defined"); return; }
    var rowStyle = "display:flex;align-items:center;gap:6px;padding:5px 0;border-bottom:1px solid var(--sheet-border,#334155);font-size:12px;";
    var idStyle = "font-family:monospace;color:#93c5fd;min-width:44px;";
    var btn = "margin-left:auto;padding:4px 8px;border:1px solid var(--sheet-border,#334155);border-radius:4px;background:transparent;color:#fca5a5;cursor:pointer;font-size:11px;";
    host.innerHTML = state.rules.map(function (rule) {
      return '<div style="' + rowStyle + '"><span style="' + idStyle + '">' + esc(rule.id) + '</span>' +
        '<span>' + esc(rule.range) + '</span>' +
        '<span style="color:var(--sheet-muted,#94a3b8);">' + esc(rule.ui_type || rule.validation_type) + '</span>' +
        '<button type="button" data-vm-act="remove" data-vm-id="' + esc(rule.id) + '" data-i18n="action-remove" style="' + btn + '">' + esc(t("action-remove", "Remove")) + '</button></div>';
    }).join("");
  }
  function buildPanel() {
    var host = document.getElementById("sheet-modal-container") || document.body;
    if (!host || typeof host.appendChild !== "function") return;
    var existing = document.getElementById(PANEL_ID);
    if (existing && existing.remove) existing.remove();
    var panel = document.createElement("div");
    panel.id = PANEL_ID;
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", t("sheet-data-validation", "Data Validation"));
    panel.style.cssText = "position:fixed;right:16px;bottom:16px;z-index:var(--z-dropdown,1000);width:340px;max-height:80vh;overflow:auto;background:var(--sheet-bg-elevated,#1e293b);color:var(--sheet-fg,#f8fafc);border:1px solid var(--sheet-border,#334155);border-radius:8px;padding:14px;font-size:13px;box-shadow:0 8px 28px rgba(0,0,0,.45);";
    panel.innerHTML = panelHtml();
    host.appendChild(panel);
    if (window.SheetI18n && typeof window.SheetI18n.localize === "function") window.SheetI18n.localize(panel);
    renderList();
  }
  function open() {
    if (!hasApp()) return;
    ensureLauncher(); hookGrid(); buildPanel(); state.open = true; apply();
  }
  function close() {
    var panel = document.getElementById(PANEL_ID);
    if (panel && panel.remove) panel.remove();
    state.open = false; clearMarks();
  }
  function toggle() {
    if (state.open && document.getElementById(PANEL_ID)) close(); else open();
  }
  function addRule() {
    if (!hasApp()) return;
    var ws = worksheet();
    if (!ws) { notify(t("sheet-sidebar-load-first", "Load a spreadsheet first."), true); return; }
    var rangeEl = document.getElementById("vm-range");
    var rangeStr = rangeEl && rangeEl.value ? String(rangeEl.value).trim() : "";
    var bounds = parseRange(rangeStr);
    if (!bounds) { notify(t("validation-required", "Enter a valid cell range"), true); return; }
    var sel = document.getElementById("vm-type"), opSel = document.getElementById("vm-op");
    var uiType = sel && sel.value ? sel.value : "whole", operator = opSel && opSel.value ? opSel.value : "any";
    function val(id) { var el = document.getElementById(id); return el && el.value ? String(el.value).trim() : ""; }
    var v1 = val("vm-value1"), v2 = val("vm-value2"), listRaw = val("vm-list"), msg = val("vm-msg");
    var allowed = uiType === "list" ? listRaw.split(",").map(function (s) { return s.trim(); }).filter(Boolean) : null;
    var rule = { id: "vr-" + (++state.seq), range: rangeStr, ui_type: uiType,
      validation_type: serverType(uiType), operator: operator };
    if (v1) rule.value1 = v1;
    if (v2) rule.value2 = v2;
    if (allowed && allowed.length) rule.allowed_values = allowed;
    if (msg) rule.error_message = msg;
    var reg = registry(ws);
    reg.push(rule); state.rules = reg; mirror(ws, rule, bounds);
    renderList(); apply(); notify(t("action-add", "Validation rule added"), false);
  }
  function removeRule(id) {
    if (!hasApp() || !id) return;
    var ws = worksheet();
    if (!ws) return;
    var reg = registry(ws), index = -1;
    reg.forEach(function (rule, i) { if (rule.id === id) index = i; });
    if (index < 0) return;
    reg.splice(index, 1);
    var vals = validations(ws, false);
    if (vals) Object.keys(vals).forEach(function (key) { if (vals[key] && vals[key].id === id) delete vals[key]; });
    state.rules = reg; renderList(); clearMarks(); apply();
    notify(t("action-remove", "Validation rule removed"), false);
  }
  function onDocPointer(e) {
    if (!hasApp()) return;
    var target = e.target;
    if (!target || typeof target.closest !== "function") return;
    var btn = target.closest("[data-vm-act]");
    if (!btn) return;
    var act = btn.getAttribute("data-vm-act");
    if (act === "open") { e.preventDefault(); open(); }
    else if (act === "close") { e.preventDefault(); close(); }
    else if (act === "add") { e.preventDefault(); addRule(); }
    else if (act === "apply") { e.preventDefault(); apply(); }
    else if (act === "clear") { e.preventDefault(); clear(); }
    else if (act === "remove") { e.preventDefault(); removeRule(btn.getAttribute("data-vm-id")); }
  }
  function refresh() {
    if (!hasApp()) return;
    syncRules(); hookGrid();
    if (state.open && document.getElementById(PANEL_ID)) { renderList(); apply(); }
  }
  window.SheetValidationManager = {
    open: open, close: close, toggle: toggle, add: addRule, remove: removeRule,
    apply: apply, clear: clear, refresh: refresh,
    list: function () { syncRules(); return state.rules.slice(); }
  };
  if (typeof document !== "undefined" && document.addEventListener) {
    document.addEventListener("pointerdown", onDocPointer, true);
    ["htmx:afterSwap", "htmx:afterSettle", "gb-sheet-locale"].forEach(function (name) {
      document.addEventListener(name, refresh);
    });
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", refresh);
    else refresh();
  }
})();
