"use strict";
/* Sheet advanced module: 24_a11y_checker — workbook accessibility report (#2493).
   Reports merged cells, colour-only conditional formats, images without alt
   text and missing header rows. Pure helpers stay DOM-free for offline tests.
   Loaded last so every other module is available. */

(function () {
  function t(key, fallback, params) {
    if (window.SheetDefaults && typeof window.SheetDefaults.a11yT === "function") {
      return window.SheetDefaults.a11yT(key, fallback, params);
    }
    if (typeof window.gbT === "function") return window.gbT(key, fallback, params);
    return fallback;
  }

  function asArray(value) {
    return Array.isArray(value) ? value : [];
  }
  function asObject(value) {
    return value && typeof value === "object" ? value : {};
  }
  function text(value) {
    return value == null ? "" : String(value);
  }
  function cellRef(r, c) {
    var letters = "";
    var n = Math.max(0, c) + 1;
    while (n > 0) {
      var rem = (n - 1) % 26;
      letters = String.fromCharCode(65 + rem) + letters;
      n = Math.floor((n - 1) / 26);
    }
    return letters + (Math.max(0, r) + 1);
  }
  function rangeRef(m) {
    var a = cellRef(Number(m.start_row) || 0, Number(m.start_col) || 0);
    var b = cellRef(Number(m.end_row) || 0, Number(m.end_col) || 0);
    return a === b ? a : a + ":" + b;
  }
  function finding(type, key, fallback, params, sheetName, severity) {
    return {
      type: type, key: key, fallback: fallback, params: params || {},
      sheetName: sheetName || "", severity: severity || "warning",
    };
  }

  function isColourOnly(style) {
    var s = asObject(style);
    var colourKeys = ["color", "background", "background_color", "bg_color", "fill", "text_color"];
    var nonColourKeys = ["icon", "pattern", "text_decoration", "font_weight", "border",
      "number_format", "label", "symbol", "font_style"];
    var hasColour = colourKeys.some(function (k) { return text(s[k]).trim() !== ""; });
    var hasAlternative = nonColourKeys.some(function (k) { return text(s[k]).trim() !== ""; });
    return hasColour && !hasAlternative;
  }

  function dataMapOf(ws) {
    var data = asObject(ws && ws.data);
    return data;
  }
  function firstRowHasHeader(data) {
    var rowZero = false;
    var dataBelow = false;
    Object.keys(data).forEach(function (key) {
      var parts = key.split(",");
      var r = parseInt(parts[0], 10);
      var cell = asObject(data[key]);
      var content = text(cell.value) || text(cell.formula);
      if (!content.trim()) return;
      if (r === 0) rowZero = true;
      else if (r > 0) dataBelow = true;
    });
    return !dataBelow || rowZero;
  }

  // Pure, defensive: a malformed worksheet is logged and skipped, never thrown.
  function collectFindings(workbook, sheetIndex) {
    var findings = [];
    var book = asObject(workbook);
    var sheets = asArray(book.worksheets);
    var index = Number.isInteger(sheetIndex) ? sheetIndex : 0;
    if (index < 0 || index >= sheets.length) index = 0;
    if (!sheets.length) return findings;

    var ws = asObject(sheets[index]);
    var sheetName = text(ws.name) || t("sheet-a11y-checker-sheet-fallback", "Sheet {index}",
      { index: String(index + 1) });

    try {
      asArray(ws.merged_cells).forEach(function (m) {
        if (!m || typeof m !== "object") return;
        findings.push(finding("merged", "sheet-a11y-finding-merged",
          "Merged cells at {range} can confuse screen readers; avoid merging in accessible workbooks.",
          { range: rangeRef(m) }, sheetName));
      });
    } catch (e) {
      console.warn("[sheet-a11y-checker] merged-cell scan failed", e);
    }

    try {
      asArray(ws.conditional_formats).forEach(function (rule) {
        if (!rule || !isColourOnly(rule.style)) return;
        var m = asObject(rule);
        findings.push(finding("color-only", "sheet-a11y-finding-color-only",
          "Conditional format at {range} uses colour only; add an icon, label or pattern.",
          { range: rangeRef(m) }, sheetName));
      });
    } catch (e) {
      console.warn("[sheet-a11y-checker] conditional-format scan failed", e);
    }

    try {
      var images = asArray(ws.images).concat(asArray(ws.drawings), asArray(book.images));
      images.forEach(function (image) {
        var img = asObject(image);
        var alt = text(img.alt || img.alt_text || img.description).trim();
        if (alt) return;
        findings.push(finding("no-alt", "sheet-a11y-finding-no-alt",
          "Image {name} has no alternative text.",
          { name: text(img.name || img.id) || t("sheet-a11y-unnamed-image", "unnamed") },
          sheetName));
      });
    } catch (e) {
      console.warn("[sheet-a11y-checker] image scan failed", e);
    }

    try {
      if (!firstRowHasHeader(dataMapOf(ws))) {
        findings.push(finding("no-header", "sheet-a11y-finding-no-header",
          "Worksheet {sheet} has data but no header row.",
          { sheet: sheetName }, sheetName));
      }
    } catch (e) {
      console.warn("[sheet-a11y-checker] header scan failed", e);
    }

    return findings;
  }

  function activeWorkbook() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    return (inst && inst.loadedSheet) || window.__LOADED_SHEET || null;
  }
  function activeIndex() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    if (inst && typeof inst.worksheetIndex === "number") return inst.worksheetIndex;
    return D ? D.WORKSHEET_INDEX : 0;
  }
  function activeHost() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    if (inst && inst.host) return inst.host;
    return document.getElementById("sheet-content");
  }

  function format(f) {
    var message = t(f.key, f.fallback, f.params);
    var label = t("sheet-a11y-severity-warning", "Warning");
    return label + ": " + message +
      (f.sheetName ? " (" + t("sheet-a11y-checker-sheet", "Sheet") + ": " + f.sheetName + ")" : "");
  }

  var panel = null;
  var opener = null;

  function ensurePanel() {
    if (panel && panel.isConnected) return panel;
    var host = activeHost();
    if (!host) return null;
    panel = document.createElement("section");
    panel.className = "sheet-a11y-panel";
    panel.setAttribute("role", "region");
    panel.setAttribute("aria-label", t("sheet-a11y-checker-title", "Accessibility check"));
    panel.setAttribute("tabindex", "-1");

    var header = document.createElement("div");
    header.className = "sheet-a11y-panel-header";
    var title = document.createElement("h2");
    title.textContent = t("sheet-a11y-checker-title", "Accessibility check");
    var close = document.createElement("button");
    close.type = "button";
    close.className = "sheet-a11y-panel-close";
    close.textContent = "\u00d7";
    close.setAttribute("aria-label", t("sheet-a11y-checker-close", "Close accessibility check"));
    close.addEventListener("click", function () {
      panel.style.display = "none";
      if (opener && opener.isConnected) opener.focus();
    });
    header.appendChild(title);
    header.appendChild(close);

    var summary = document.createElement("p");
    summary.className = "sheet-a11y-panel-summary";
    summary.setAttribute("role", "status");
    summary.setAttribute("aria-live", "polite");

    var list = document.createElement("ul");
    list.className = "sheet-a11y-panel-list";
    list.setAttribute("role", "list");

    panel.appendChild(header);
    panel.appendChild(summary);
    panel.appendChild(list);
    host.appendChild(panel);
    return panel;
  }

  function renderPanel(findings) {
    var p = ensurePanel();
    if (!p) return;
    var summary = p.querySelector(".sheet-a11y-panel-summary");
    var list = p.querySelector(".sheet-a11y-panel-list");
    if (!summary || !list) return;
    summary.textContent = findings.length
      ? t("sheet-a11y-checker-summary", "{count} accessibility issues found",
        { count: String(findings.length) })
      : t("sheet-a11y-checker-empty", "No accessibility issues found.");
    list.textContent = "";
    findings.forEach(function (f) {
      var li = document.createElement("li");
      li.className = "sheet-a11y-finding sheet-a11y-finding-" + f.type;
      var badge = document.createElement("span");
      badge.className = "sheet-a11y-finding-badge";
      badge.setAttribute("aria-hidden", "true");
      badge.textContent = "!";
      var textNode = document.createElement("span");
      textNode.textContent = format(f);
      li.appendChild(badge);
      li.appendChild(textNode);
      list.appendChild(li);
    });
    p.style.display = "block";
    p.focus({ preventScroll: true });
  }

  function run() {
    try {
      return collectFindings(activeWorkbook(), activeIndex());
    } catch (e) {
      console.warn("[sheet-a11y-checker] run failed", e);
      return [];
    }
  }

  function runAndRender() {
    var findings = run();
    renderPanel(findings);
    return findings;
  }

  function togglePanel() {
    if (panel && panel.isConnected && panel.style.display !== "none") {
      panel.style.display = "none";
      if (opener && opener.isConnected) opener.focus();
      return;
    }
    opener = (window.SheetDefaults && window.SheetDefaults.qInSheet
      ? window.SheetDefaults.qInSheet(null, "#a11yCheckBtn") : null) ||
      document.getElementById("a11yCheckBtn");
    runAndRender();
  }

  window.SheetA11yChecker = {
    collectFindings: collectFindings,
    run: run,
    runAndRender: runAndRender,
    togglePanel: togglePanel,
    formatFinding: format,
  };
})();
