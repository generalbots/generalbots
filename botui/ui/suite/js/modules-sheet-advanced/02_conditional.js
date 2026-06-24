"use strict";
/* SheetAdvanced module 02: conditional formats + data validation + named ranges + tables */
(function (window) {
  const P = window.SheetAdvancedProto;
  const H = window.SheetAdvancedHelpers;
  if (!P) { console.error("Load 01_core.js first"); return; }

  const COND_KEY = "gb-sheet-cond";
  const VAL_KEY = "gb-sheet-validation";
  const NAME_KEY = "gb-sheet-names";
  const TABLE_KEY = "gb-sheet-tables";

  function readObj(k) { try { return JSON.parse(localStorage.getItem(k) || "{}"); } catch (_) { return {}; } }
  function writeObj(k, obj) { try { localStorage.setItem(k, JSON.stringify(obj)); } catch (_) {} }

  P.addConditionalFormat = function (range, type, opts) {
    const map = readObj(COND_KEY);
    if (!map[this.sheetId]) map[this.sheetId] = [];
    map[this.sheetId].push({ range: range, type: type, opts: opts || {} });
    writeObj(COND_KEY, map);
    this._renderConditionalFormat(map[this.sheetId][map[this.sheetId].length - 1]);
  };
  P.listConditionalFormats = function () { return readObj(COND_KEY)[this.sheetId] || []; };
  P._renderConditionalFormat = function (cf) {
    const r = H.parseRange(cf.range);
    if (!r) return;
    for (let row = r.start.row; row <= r.end.row; row++) {
      for (let col = r.start.col; col <= r.end.col; col++) {
        const cell = this.grid.querySelector("[data-row='" + row + "'][data-col='" + col + "']");
        if (!cell) continue;
        const v = parseFloat(cell.textContent.replace(",", ".")) || 0;
        if (cf.type === "databar") {
          const all = this._readRangeValues(cf.range);
          const max = Math.max.apply(null, all) || 1;
          const pct = (v / max * 100).toFixed(0);
          cell.style.background = "linear-gradient(to right, #3b82f6 " + pct + "%, transparent " + pct + "%)";
        } else if (cf.type === "color-scale") {
          const all = this._readRangeValues(cf.range);
          const min = Math.min.apply(null, all);
          const max = Math.max.apply(null, all);
          const t = (v - min) / ((max - min) || 1);
          const r_c = Math.round(255 * (1 - t));
          const g_c = Math.round(255 * t);
          cell.style.background = "rgb(" + r_c + "," + g_c + ",0)";
          cell.style.color = t > 0.5 ? "#000" : "#fff";
        } else if (cf.type === "gt") {
          if (v > (cf.opts.value || 0)) cell.style.background = cf.opts.color || "#10b981";
        } else if (cf.type === "lt") {
          if (v < (cf.opts.value || 0)) cell.style.background = cf.opts.color || "#ef4444";
        } else if (cf.type === "between") {
          if (v >= (cf.opts.min || 0) && v <= (cf.opts.max || 100)) cell.style.background = cf.opts.color || "#fbbf24";
        } else if (cf.type === "icon-set") {
          const all = this._readRangeValues(cf.range);
          const max = Math.max.apply(null, all) || 1;
          const t = v / max;
          cell.innerHTML = (t > 0.66 ? "🟢" : t > 0.33 ? "🟡" : "🔴") + " " + cell.innerHTML;
        }
      }
    }
  };
  P._renderAllConditionalFormats = function () {
    const self = this;
    (readObj(COND_KEY)[this.sheetId] || []).forEach(cf => self._renderConditionalFormat(cf));
  };

  P.addDataValidation = function (range, type, opts) {
    const map = readObj(VAL_KEY);
    if (!map[this.sheetId]) map[this.sheetId] = [];
    map[this.sheetId].push({ range: range, type: type, opts: opts || {} });
    writeObj(VAL_KEY, map);
  };
  P.listValidations = function () { return readObj(VAL_KEY)[this.sheetId] || []; };
  P.validateCell = function (cellRef, value) {
    const vals = readObj(VAL_KEY)[this.sheetId] || [];
    for (const v of vals) {
      const r = H.parseRange(v.range);
      if (!r) continue;
      const cr = H.parseA1Ref(cellRef);
      if (!cr) continue;
      if (cr.row >= r.start.row && cr.row <= r.end.row && cr.col >= r.start.col && cr.col <= r.end.col) {
        if (v.type === "list") {
          if (v.opts.items && v.opts.items.indexOf(String(value)) < 0) return { ok: false, msg: "Valor deve estar na lista: " + v.opts.items.join(", ") };
        } else if (v.type === "number") {
          const n = parseFloat(value);
          if (isNaN(n)) return { ok: false, msg: "Deve ser numérico" };
          if (v.opts.min !== undefined && n < v.opts.min) return { ok: false, msg: "Mínimo: " + v.opts.min };
          if (v.opts.max !== undefined && n > v.opts.max) return { ok: false, msg: "Máximo: " + v.opts.max };
        } else if (v.type === "text-length") {
          if (v.opts.min !== undefined && value.length < v.opts.min) return { ok: false, msg: "Mínimo " + v.opts.min + " caracteres" };
          if (v.opts.max !== undefined && value.length > v.opts.max) return { ok: false, msg: "Máximo " + v.opts.max + " caracteres" };
        } else if (v.type === "date") {
          const d = new Date(value);
          if (isNaN(d.getTime())) return { ok: false, msg: "Data inválida" };
        }
      }
    }
    return { ok: true };
  };
  P._renderAllValidations = function () {};

  P.addNamedRange = function (name, range) {
    const map = readObj(NAME_KEY);
    if (!map[this.sheetId]) map[this.sheetId] = {};
    map[this.sheetId][name] = range;
    writeObj(NAME_KEY, map);
  };
  P.resolveNamedRange = function (name) {
    const map = readObj(NAME_KEY);
    return (map[this.sheetId] && map[this.sheetId][name]) || null;
  };
  P.listNamedRanges = function () {
    const map = readObj(NAME_KEY);
    return (map && map[this.sheetId]) || {};
  };
  P._renderAllNamedRanges = function () {};

  P.createTable = function (name, range) {
    const map = readObj(TABLE_KEY);
    if (!map[this.sheetId]) map[this.sheetId] = {};
    map[this.sheetId][name] = range;
    writeObj(TABLE_KEY, map);
    const r = H.parseRange(range);
    if (!r) return;
    for (let row = r.start.row; row <= r.end.row; row++) {
      for (let col = r.start.col; col <= r.end.col; col++) {
        const cell = this.grid.querySelector("[data-row='" + row + "'][data-col='" + col + "']");
        if (cell) {
          if (row === r.start.row) cell.style.background = "#334155";
          else if (row % 2 === 0) cell.style.background = "#1e293b";
          cell.style.borderRight = "1px solid #0f172a";
        }
      }
    }
  };
  P.listTables = function () { return readObj(TABLE_KEY)[this.sheetId] || {}; };
  P._renderAllTables = function () {};
})(window);
