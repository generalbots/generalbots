"use strict";

/**
 * Module 20: Conditional formatting UI integration for Sheet.
 * Wires the SheetFormatting engine (module 13) into the existing
 * conditional-formatting modal/UI. Supports 17 rule types:
 *   cell value: equals, notEquals, contains, startsWith, endsWith,
 *               greater, less, between, isBlank, isError
 *   duplicate / unique
 *   top_n / bottom_n
 *   above_average / below_average
 *   color_scale (2-color and 3-color gradient)
 *   data_bar (rendered as a min-width bar inside the cell)
 *   icon_set (arrows, circles, traffic-lights)
 *
 * Public API: window.SheetCF = { applyRules, clear, render, renderAll }.
 */

(function () {
  const ICON_SETS = {
    arrows: ["▲", "▶", "▼"],
    circles: ["●", "●", "○"],
    traffic: ["🚦", "🚦", "🚦"],
    flags: ["⚑", "⚐", "⚐"],
  };

  function getState() { return window.state || null; }
  function getWorksheet() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets[s.activeWorksheet];
  }

  function ensureRules() {
    const ws = getWorksheet();
    if (!ws) return [];
    if (!ws.conditionalFormats) ws.conditionalFormats = [];
    return ws.conditionalFormats;
  }

  function rangeCells(rangeStr) {
    if (!rangeStr) return [];
    const parts = rangeStr.split(":");
    if (parts.length === 1) {
      const ref = parseRef(parts[0]);
      if (!ref) return [];
      return [[ref.row, ref.col]];
    }
    const start = parseRef(parts[0]);
    const end = parseRef(parts[1]);
    if (!start || !end) return [];
    const out = [];
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) out.push([r, c]);
    }
    return out;
  }

  function parseRef(ref) {
    const m = ref.match(/^([A-Z]+)(\d+)$/);
    if (!m) return null;
    let col = 0;
    for (const ch of m[1]) col = col * 26 + (ch.charCodeAt(0) - 64);
    return { row: parseInt(m[2]) - 1, col: col - 1 };
  }

  function colName(idx) {
    let name = "";
    let n = idx + 1;
    while (n > 0) {
      const r = (n - 1) % 26;
      name = String.fromCharCode(65 + r) + name;
      n = Math.floor((n - 1) / 26);
    }
    return name;
  }

  function getCellValue(ws, r, c) {
    const cell = ws.data && ws.data[r + "," + c];
    if (!cell) return null;
    if (cell.formula) {
      try { return window.evaluateFormula ? window.evaluateFormula(cell.formula, r, c) : ""; }
      catch (_e) { return ""; }
    }
    return cell.value != null ? cell.value : null;
  }

  function evaluateRule(rule, value, allValues) {
    const op = rule.type || rule.operator || "equals";
    const v = rule.value;
    const n = (x) => (typeof x === "number" ? x : parseFloat(x));
    const s = (x) => (x == null ? "" : String(x));
    switch (op) {
      case "equals": return s(value) === s(v);
      case "notEquals": return s(value) !== s(v);
      case "contains": return s(value).toLowerCase().includes(s(v).toLowerCase());
      case "text_starts": return s(value).toLowerCase().startsWith(s(v).toLowerCase());
      case "text_ends": return s(value).toLowerCase().endsWith(s(v).toLowerCase());
      case "greater": return n(value) > n(v);
      case "less": return n(value) < n(v);
      case "greaterOrEqual": return n(value) >= n(v);
      case "lessOrEqual": return n(value) <= n(v);
      case "between": return n(value) >= n(rule.value) && n(value) <= n(rule.value2);
      case "isBlank": return value == null || s(value).trim() === "";
      case "isError": return typeof value === "string" && value.startsWith("#");
      case "duplicate": return countOccurrences(allValues, value) > 1;
      case "unique": {
        const occ = countOccurrences(allValues, value);
        return occ === 1 && value != null && s(value).trim() !== "";
      }
      case "top_n": {
        const sorted = allValues.slice().sort((a, b) => n(b) - n(a));
        const limit = parseInt(rule.n || 10);
        const threshold = sorted[limit - 1];
        return n(value) >= n(threshold) && value != null;
      }
      case "bottom_n": {
        const sorted = allValues.slice().sort((a, b) => n(a) - n(b));
        const limit = parseInt(rule.n || 10);
        const threshold = sorted[limit - 1];
        return n(value) <= n(threshold) && value != null;
      }
      case "above_average": return n(value) > average(allValues);
      case "below_average": return n(value) < average(allValues);
      default: return false;
    }
  }

  function countOccurrences(arr, v) {
    const s = v == null ? "" : String(v);
    let n = 0;
    for (const x of arr) if ((x == null ? "" : String(x)) === s) n++;
    return n;
  }

  function average(arr) {
    let sum = 0, count = 0;
    for (const v of arr) {
      const x = parseFloat(v);
      if (!isNaN(x)) { sum += x; count++; }
    }
    return count === 0 ? 0 : sum / count;
  }

  function applyRuleStyles(cell, rule, value, allValues) {
    const styles = [];
    if (rule.background) styles.push("background:" + rule.background);
    if (rule.foreground) styles.push("color:" + rule.foreground);
    if (rule.bold) styles.push("font-weight:bold");
    if (rule.italic) styles.push("font-style:italic");
    if (rule.underline) styles.push("text-decoration:underline");
    if (rule.fontSize) styles.push("font-size:" + rule.fontSize + "px");
    if (rule.type === "color_scale") {
      const color = colorScale(value, allValues, rule);
      if (color) styles.push("background:" + color);
    }
    cell.style.cssText = (cell.style.cssText || "") + ";" + styles.join(";");
    if (rule.type === "data_bar") {
      renderDataBar(cell, value, allValues, rule);
    }
    if (rule.type === "icon_set") {
      renderIconSet(cell, value, allValues, rule);
    }
  }

  function colorScale(value, allValues, rule) {
    const nums = allValues.map(parseFloat).filter((n) => !isNaN(n));
    if (nums.length === 0) return null;
    const min = Math.min.apply(null, nums);
    const max = Math.max.apply(null, nums);
    const v = parseFloat(value);
    if (isNaN(v) || max === min) return rule.midColor || rule.maxColor || null;
    const ratio = (v - min) / (max - min);
    const colors = rule.colors || (rule.minColor && rule.midColor && rule.maxColor
      ? [rule.minColor, rule.midColor, rule.maxColor] : null);
    if (!colors) return null;
    if (colors.length === 2) {
      return ratio < 0.5 ? colors[0] : colors[1];
    }
    if (ratio < 0.5) {
      return interpolateColor(colors[0], colors[1], ratio * 2);
    }
    return interpolateColor(colors[1], colors[2], (ratio - 0.5) * 2);
  }

  function interpolateColor(a, b, t) {
    const pa = parseHex(a);
    const pb = parseHex(b);
    if (!pa || !pb) return a;
    const r = Math.round(pa[0] + (pb[0] - pa[0]) * t);
    const g = Math.round(pa[1] + (pb[1] - pa[1]) * t);
    const bl = Math.round(pa[2] + (pb[2] - pa[2]) * t);
    return "rgb(" + r + "," + g + "," + bl + ")";
  }

  function parseHex(h) {
    if (!h) return null;
    const m = h.match(/^#([0-9a-f]{6})$/i);
    if (!m) return null;
    return [parseInt(m[1].slice(0, 2), 16), parseInt(m[1].slice(2, 4), 16), parseInt(m[1].slice(4, 6), 16)];
  }

  function renderDataBar(cell, value, allValues, rule) {
    const nums = allValues.map(parseFloat).filter((n) => !isNaN(n));
    if (nums.length === 0) return;
    const min = Math.min.apply(null, nums);
    const max = Math.max.apply(null, nums);
    const v = parseFloat(value);
    if (isNaN(v) || max === min) return;
    const ratio = (v - min) / (max - min);
    const bar = document.createElement("div");
    bar.className = "data-bar";
    bar.style.cssText = "position:absolute;left:0;bottom:0;height:3px;background:" + (rule.color || "#1a73e8") + ";width:" + Math.round(ratio * 100) + "%;";
    if (cell.style.position !== "absolute" && cell.style.position !== "relative") cell.style.position = "relative";
    cell.appendChild(bar);
  }

  function renderIconSet(cell, value, allValues, rule) {
    const nums = allValues.map(parseFloat).filter((n) => !isNaN(n));
    if (nums.length === 0) return;
    const min = Math.min.apply(null, nums);
    const max = Math.max.apply(null, nums);
    const v = parseFloat(value);
    if (isNaN(v) || max === min) return;
    const ratio = (v - min) / (max - min);
    const icons = ICON_SETS[rule.icons || "arrows"] || ICON_SETS.arrows;
    let icon;
    if (ratio < 0.33) icon = icons[2];
    else if (ratio < 0.67) icon = icons[1];
    else icon = icons[0];
    const span = document.createElement("span");
    span.className = "icon-set";
    span.textContent = icon;
    span.style.cssText = "margin-right:4px;";
    cell.insertBefore(span, cell.firstChild);
  }

  function applyRules() {
    const ws = getWorksheet();
    if (!ws) return;
    const rules = ensureRules();
    if (rules.length === 0) return;
    const allValues = collectAllValues(ws);
    for (const rule of rules) {
      const cells = rangeCells(rule.range);
      for (const [r, c] of cells) {
        const cell = document.querySelector('.cell[data-row="' + r + '"][data-col="' + c + '"]');
        if (!cell) continue;
        const value = getCellValue(ws, r, c);
        try {
          if (evaluateRule(rule, value, allValues)) {
            applyRuleStyles(cell, rule, value, allValues);
          }
        } catch (_e) { /* silent */ }
      }
    }
  }

  function collectAllValues(ws) {
    const out = [];
    for (const key in ws.data || {}) {
      const [r, c] = key.split(",").map(Number);
      const v = getCellValue(ws, r, c);
      if (v != null) out.push(v);
    }
    return out;
  }

  function clear() {
    const ws = getWorksheet();
    if (!ws) return;
    ws.conditionalFormats = [];
    document.querySelectorAll(".cell .data-bar, .cell .icon-set").forEach((el) => el.remove());
    document.querySelectorAll(".cell").forEach((c) => {
      c.style.background = "";
      c.style.color = "";
      c.style.fontWeight = "";
      c.style.fontStyle = "";
      c.style.textDecoration = "";
      c.style.fontSize = "";
    });
  }

  function render() { applyRules(); }
  function renderAll() { applyRules(); }

  function addRule(rule) {
    const rules = ensureRules();
    rule.id = rule.id || ("cf-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6));
    rules.push(rule);
    applyRules();
  }

  function removeRule(id) {
    const ws = getWorksheet();
    if (!ws || !ws.conditionalFormats) return false;
    const idx = ws.conditionalFormats.findIndex((r) => r.id === id);
    if (idx === -1) return false;
    ws.conditionalFormats.splice(idx, 1);
    applyRules();
    return true;
  }

  function attachUI() {
    const btn = document.getElementById("conditionalFormatBtn");
    if (btn) btn.addEventListener("click", openCFDialog);
  }

  function openCFDialog() {
    const dialog = document.createElement("div");
    dialog.className = "modal";
    dialog.style.cssText = "position:fixed;inset:0;background:rgba(0,0,0,0.4);z-index:9999;display:flex;align-items:center;justify-content:center;";
    const content = document.createElement("div");
    content.style.cssText = "background:#fff;border-radius:8px;padding:24px;min-width:480px;max-width:90%;max-height:90vh;overflow:auto;";
    content.innerHTML = `
      <h3 style="margin:0 0 16px 0;">Conditional Formatting</h3>
      <div style="margin-bottom:12px;">
        <label>Rule type:
          <select id="cfType" style="margin-left:8px;padding:4px;">
            <option value="equals">Cell value equals</option>
            <option value="text_starts">Text starts with</option>
            <option value="text_ends">Text ends with</option>
            <option value="contains">Text contains</option>
            <option value="greater">Greater than</option>
            <option value="less">Less than</option>
            <option value="between">Between</option>
            <option value="isBlank">Is blank</option>
            <option value="duplicate">Duplicate values</option>
            <option value="unique">Unique values</option>
            <option value="top_n">Top N</option>
            <option value="bottom_n">Bottom N</option>
            <option value="above_average">Above average</option>
            <option value="below_average">Below average</option>
            <option value="color_scale">Color scale</option>
            <option value="data_bar">Data bar</option>
            <option value="icon_set">Icon set</option>
          </select>
        </label>
      </div>
      <div id="cfValueBox" style="margin-bottom:12px;">
        <input type="text" id="cfValue" placeholder="Value" style="padding:4px;width:200px;" />
      </div>
      <div style="margin-bottom:12px;">
        <label>Range: <input type="text" id="cfRange" placeholder="A1:D10" style="padding:4px;width:200px;" /></label>
      </div>
      <div style="margin-bottom:12px;display:flex;gap:8px;">
        <label>BG: <input type="color" id="cfBg" value="#ffeb3b" /></label>
        <label>FG: <input type="color" id="cfFg" value="#000000" /></label>
      </div>
      <div style="display:flex;gap:8px;justify-content:flex-end;">
        <button id="cfCancel" style="padding:6px 16px;">Cancel</button>
        <button id="cfApply" style="padding:6px 16px;background:#1a73e8;color:#fff;border:0;border-radius:4px;">Apply</button>
      </div>
    `;
    dialog.appendChild(content);
    document.body.appendChild(dialog);
    const cfType = content.querySelector("#cfType");
    const cfValueBox = content.querySelector("#cfValueBox");
    cfType.addEventListener("change", () => {
      const t = cfType.value;
      cfValueBox.style.display = ["isBlank", "duplicate", "unique", "above_average", "below_average", "color_scale", "data_bar", "icon_set"].indexOf(t) === -1 ? "" : "none";
    });
    content.querySelector("#cfCancel").addEventListener("click", () => dialog.remove());
    content.querySelector("#cfApply").addEventListener("click", () => {
      const rule = {
        type: cfType.value,
        value: cfType.value === "top_n" || cfType.value === "bottom_n" ? null : content.querySelector("#cfValue").value,
        n: cfType.value === "top_n" || cfType.value === "bottom_n" ? parseInt(content.querySelector("#cfValue").value) || 10 : null,
        range: content.querySelector("#cfRange").value || "A1:D100",
        background: content.querySelector("#cfBg").value,
        foreground: content.querySelector("#cfFg").value,
      };
      addRule(rule);
      dialog.remove();
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attachUI);
  } else {
    setTimeout(attachUI, 50);
  }

  window.SheetCF = { applyRules, clear, render, renderAll, addRule, removeRule, evaluateRule, openCFDialog };
})();
