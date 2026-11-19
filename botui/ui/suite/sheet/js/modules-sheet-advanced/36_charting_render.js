"use strict";
/* Advanced charting — local SVG overlay integration (#2490).
 * These charts are renderings of chart definitions held in the browser. They
 * are explicitly labelled local only: no XLSX chart-part round trip, PDF/PNG
 * export, server named range or print parity is implied.
 */

(function () {
  const S = window.SheetChartingSvg;
  const registry = window.SheetChartingTypes.renderers;

  function t(key, fallback, params) {
    if (typeof window.gbT === "function") return window.gbT(key, fallback, params);
    if (window.SheetI18n && typeof window.SheetI18n.t === "function") return window.SheetI18n.t(key, fallback, params);
    return fallback;
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const value = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(value) || value < 0 ? 0 : value;
    } catch (_) {
      return 0;
    }
  }

  function sheetId() {
    if (window.__SHEET_INITIAL_ID) return window.__SHEET_INITIAL_ID;
    if (window.__LOADED_SHEET && window.__LOADED_SHEET.id) return window.__LOADED_SHEET.id;
    return "current";
  }

  function contextKey() {
    return sheetId() + ":" + wsIndex();
  }

  function store() {
    if (!window.__SHEET_CHARTING_LOCAL__) window.__SHEET_CHARTING_LOCAL__ = {};
    const key = contextKey();
    if (!window.__SHEET_CHARTING_LOCAL__[key]) window.__SHEET_CHARTING_LOCAL__[key] = [];
    return window.__SHEET_CHARTING_LOCAL__[key];
  }

  function isAdvanced(type) {
    return Object.prototype.hasOwnProperty.call(registry, type || "");
  }

  function typeKey(type) {
    return "sheet-charting" + "-type-" + (type || "");
  }

  function typeLabel(type) {
    return t(typeKey(type), type);
  }

  function titleOf(def, type) {
    return def && def.title ? def.title : typeLabel(type);
  }

  function altOf(def, model, type) {
    const source = model.range ? t("sheet-charting-alt-source", " from {range}", { range: model.range }) : "";
    return t("sheet-charting-alt", "{title}: {type} chart with {count} values{source}.", {
      title: titleOf(def, type),
      type: typeLabel(type),
      count: model.values.length,
      source: source,
    });
  }

  function renderSvg(definition, width, height) {
    const def = definition || {};
    const type = def.chart_type || "";
    const model = window.SheetChartingData.model(def);
    const colors = window.SheetChartingPalette.colors(model.series, def.options && def.options.palette);
    const geo = S.geometry(width, height);
    const ctx = {
      def: def, model: model, colors: colors, geo: geo, w: width, h: height, t: t,
      scale: S.verticalScale(0, Math.max(1, model.max * 1.1), geo, 4),
    };
    const renderer = registry[type];
    const parts = renderer ? renderer(ctx) : [];
    if (renderer && ctx.scale) {
      const map = {
        x: function (index) { return geo.x(index, Math.max(1, model.labels.length)); },
        y: function (value) { return ctx.scale.y(value); },
        left: geo.pad.l,
        right: geo.pad.l + geo.plotW,
        top: geo.pad.t,
        lastX: geo.pad.l + geo.plotW,
      };
      window.SheetChartingOverlays.apply(ctx, parts, map);
    }
    return S.frame(width, height, titleOf(def, type), altOf(def, model, type), parts);
  }

  function gridBody() {
    const grid = window.SheetCore && window.SheetCore.getGrid ? window.SheetCore.getGrid() : window.SheetVirtualGrid;
    return grid && grid.bodyInner ? grid.bodyInner : null;
  }

  function storedCharts() {
    const sheet = window.__LOADED_SHEET;
    const ws = sheet && sheet.worksheets && sheet.worksheets[wsIndex()];
    return (ws && ws.charts) || [];
  }

  function localCharts() {
    return store();
  }

  function addLocal(definition) {
    const def = definition || {};
    def.id = def.id || "local-" + Date.now() + "-" + Math.floor(Math.random() * 1000);
    def.localOnly = true;
    store().push(def);
    applyAll();
    return def;
  }

  function dataTable(def, model) {
    const table = document.createElement("table");
    table.className = "ss-charting-table";
    const caption = document.createElement("caption");
    caption.textContent = t("sheet-charting-data-table", "Chart data");
    caption.setAttribute("data-i18n", "sheet-charting-data-table");
    table.appendChild(caption);
    const head = document.createElement("thead");
    const headRow = document.createElement("tr");
    const labelHead = document.createElement("th");
    labelHead.scope = "col";
    labelHead.textContent = t("sheet-charting-column-label", "Label");
    labelHead.setAttribute("data-i18n", "sheet-charting-column-label");
    headRow.appendChild(labelHead);
    model.series.forEach(function (series) {
      const cell = document.createElement("th");
      cell.scope = "col";
      cell.textContent = series.label || t("sheet-charting-value", "Value");
      headRow.appendChild(cell);
    });
    head.appendChild(headRow);
    table.appendChild(head);
    const body = document.createElement("tbody");
    model.labels.forEach(function (label, index) {
      const row = document.createElement("tr");
      const name = document.createElement("th");
      name.scope = "row";
      name.textContent = label;
      row.appendChild(name);
      model.series.forEach(function (series) {
        const cell = document.createElement("td");
        const value = series.data[index];
        cell.textContent = value == null ? "" : S.fmt(value);
        row.appendChild(cell);
      });
      body.appendChild(row);
    });
    table.appendChild(body);
    table.style.display = "none";
    return table;
  }

  function decorate(layer, def, local, id) {
    const width = layer.offsetWidth || (def.position && def.position.width) || 300;
    const height = layer.offsetHeight || (def.position && def.position.height) || 180;
    const model = window.SheetChartingData.model(def);
    const handle = layer.querySelector(".ss-chart-resize");
    layer.innerHTML = "";

    const wrapper = document.createElement("div");
    wrapper.className = "ss-charting-svg";
    wrapper.innerHTML = renderSvg(def, width, height);
    layer.appendChild(wrapper);

    const badge = document.createElement("span");
    badge.className = "ss-charting-badge";
    badge.textContent = t("sheet-charting-local-only", "Local only");
    badge.setAttribute("data-i18n", "sheet-charting-local-only");
    badge.setAttribute("data-i18n-title", "sheet-charting-local-note");
    badge.setAttribute("role", "note");
    layer.appendChild(badge);

    const table = dataTable(def, model);
    layer.appendChild(table);
    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "ss-charting-table-toggle";
    let showing = false;
    function setToggle(visible) {
      showing = visible;
      table.style.display = visible ? "block" : "none";
      toggle.textContent = visible
        ? t("sheet-charting-hide-table", "Hide data table")
        : t("sheet-charting-show-table", "Show data table");
      toggle.setAttribute("data-i18n", visible ? "sheet-charting-hide-table" : "sheet-charting-show-table");
      toggle.setAttribute("aria-expanded", visible ? "true" : "false");
    }
    setToggle(false);
    toggle.addEventListener("click", function (event) {
      event.stopPropagation();
      setToggle(!showing);
    });
    layer.appendChild(toggle);
    if (handle) layer.appendChild(handle);

    layer.setAttribute("data-ss-charting", def.chart_type);
    layer.setAttribute("data-ss-charting-id", id);
    layer.setAttribute("title", t("sheet-charting-edit-hint", "Double-click to edit this local chart"));
    layer.addEventListener("dblclick", function (event) {
      event.preventDefault();
      event.stopImmediatePropagation();
      if (window.SheetChartingPicker) window.SheetChartingPicker.open(def, layer);
    }, true);
    return layer;
  }

  function makeLayer(def, local, id) {
    if (!window.SheetChartsRender || typeof window.SheetChartsRender.renderChart !== "function") return null;
    const layer = window.SheetChartsRender.renderChart(def);
    if (!layer) return null;
    return decorate(layer, def, local, id);
  }

  let busy = false;
  function applyAll() {
    if (busy) return;
    const body = gridBody();
    if (!body) return;
    busy = true;
    try {
      body.querySelectorAll("[data-ss-charting]").forEach(function (el) { el.remove(); });
      const stored = storedCharts();
      const coreLayers = Array.from(body.querySelectorAll(".ss-chart-float:not([data-ss-charting])"));
      for (let i = stored.length - 1; i >= 0; i--) {
        if (isAdvanced(stored[i].chart_type) && coreLayers[i]) coreLayers[i].remove();
      }
      stored.forEach(function (chart, index) {
        if (isAdvanced(chart.chart_type)) makeLayer(chart, false, "stored-" + index);
      });
      localCharts().forEach(function (chart) { makeLayer(chart, true, "local-" + chart.id); });
    } finally {
      busy = false;
    }
  }

  function needsApply() {
    const body = gridBody();
    if (!body) return false;
    const present = {};
    body.querySelectorAll("[data-ss-charting-id]").forEach(function (el) {
      present[el.getAttribute("data-ss-charting-id")] = true;
    });
    const stored = storedCharts();
    for (let i = 0; i < stored.length; i++) {
      const id = "stored-" + i;
      if (isAdvanced(stored[i].chart_type) && !present[id]) return true;
    }
    const local = localCharts();
    for (let i = 0; i < local.length; i++) {
      if (!present["local-" + local[i].id]) return true;
    }
    return false;
  }

  function schedule() {
    setTimeout(applyAll, 80);
  }

  window.SheetCharting = {
    renderSvg: renderSvg,
    renderAll: applyAll,
    addLocal: addLocal,
    localCharts: localCharts,
    isAdvanced: isAdvanced,
    types: registry,
    t: t,
  };

  ["gb-sheet-tab", "gb-sheet-selection", "gb-sheet-locale"].forEach(function (name) {
    document.addEventListener(name, schedule);
  });
  document.body.addEventListener("htmx:afterSwap", schedule);

  function observe() {
    const body = gridBody();
    if (!body || !window.MutationObserver) return;
    const observer = new MutationObserver(function () {
      if (!busy && needsApply()) setTimeout(applyAll, 0);
    });
    observer.observe(body, { childList: true });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", observe);
  else observe();
  setTimeout(applyAll, 0);
})();
