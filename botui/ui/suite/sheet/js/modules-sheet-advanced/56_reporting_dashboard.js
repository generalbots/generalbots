"use strict";
/* Sheets print, reporting and scheduled delivery (#2438) — dashboard canvas.
 * Widgets point at existing named ranges/charts by name; nothing is executed
 * against the workbook and the canvas is a local, print-friendly preview. */

(function () {
  var C = window.SheetReportingCore;
  var t = C.t;
  var TYPES = ["chart", "table", "kpi", "sparkline"];
  var TYPE_KEYS = {
    chart: "sheet-rpt-type-chart",
    table: "sheet-rpt-type-table",
    kpi: "sheet-rpt-type-kpi",
    sparkline: "sheet-rpt-type-sparkline",
  };
  var TYPE_FALLBACKS = {
    chart: "Chart",
    table: "Table",
    kpi: "KPI card",
    sparkline: "Sparkline",
  };

  function makeId(prefix) {
    return prefix + "-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e5).toString(36);
  }

  function clamp(value, min, max, fallback) {
    var parsed = Number(value);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.min(Math.max(Math.round(parsed), min), max);
  }

  function validateWidget(fields) {
    var title = String((fields && fields.title) || "").trim();
    var source = String((fields && fields.source) || "").trim();
    var type = String((fields && fields.type) || "");
    if (!title || !source || TYPES.indexOf(type) < 0) {
      return { ok: false, key: "sheet-rpt-invalid-widget", fallback: "Enter a title, a source and a widget type." };
    }
    return { ok: true, title: title.slice(0, 80), source: source.slice(0, 120), type: type };
  }

  function createDashboard(title) {
    var name = String(title == null ? "" : title).trim().slice(0, 80);
    if (!name) return { ok: false, key: "sheet-rpt-dash-needs-title", fallback: "Enter a dashboard title." };
    var created = { id: makeId("dash"), title: name, widgets: [] };
    C.mutate(function (state) { state.dashboards.push(created); });
    return { ok: true, dashboard: created };
  }

  function addWidget(dashboardId, fields) {
    var valid = validateWidget(fields);
    if (!valid.ok) return valid;
    var dashboard = null;
    C.mutate(function (state) {
      dashboard = state.dashboards.filter(function (item) { return item.id === dashboardId; })[0] || null;
      if (dashboard && dashboard.widgets.length < C.MAX_ITEMS) {
        dashboard.widgets.push({
          id: makeId("widget"),
          type: valid.type,
          title: valid.title,
          source: valid.source,
          col: clamp(fields.col, 1, 12, 1),
          row: clamp(fields.row, 1, 12, 1),
          w: clamp(fields.w, 1, 12, 3),
          h: clamp(fields.h, 1, 6, 2),
        });
        var count = dashboard.widgets.length;
        dashboard.widgets[dashboard.widgets.length - 1].col = ((count - 1) * 3) % 12 + 1;
        dashboard.widgets[dashboard.widgets.length - 1].row = Math.floor((count - 1) / 4) * 2 + 1;
      }
    });
    if (!dashboard) return { ok: false, key: "sheet-rpt-dash-missing", fallback: "Create a dashboard first." };
    return valid;
  }

  function removeWidget(dashboardId, widgetId) {
    C.mutate(function (state) {
      state.dashboards.forEach(function (dashboard) {
        if (dashboard.id === dashboardId) {
          dashboard.widgets = dashboard.widgets.filter(function (widget) { return widget.id !== widgetId; });
        }
      });
    });
    return true;
  }

  function sparklinePoints(seed) {
    var text = String(seed || "");
    var points = [];
    var value = 20;
    for (var i = 0; i < 8; i++) {
      var code = text.charCodeAt(i % Math.max(1, text.length)) || 65;
      value = Math.max(2, Math.min(28, value + (code % 7) - 3));
      points.push(i * 10 + "," + (30 - value));
    }
    return points.join(" ");
  }

  function widgetNode(widget, ctx) {
    var card = C.element("div", "ss-rpt-widget ss-rpt-widget-" + widget.type);
    card.style.gridColumn = String(widget.col) + " / span " + String(widget.w);
    card.style.gridRow = String(widget.row) + " / span " + String(widget.h);
    card.appendChild(C.element("h4", "ss-rpt-widget-title", widget.title));
    card.appendChild(C.element("p", "ss-rpt-widget-source", widget.source));
    if (widget.type === "sparkline") {
      var svg = document.createElementNS
        ? document.createElementNS("http://www.w3.org/2000/svg", "svg")
        : C.element("span", "ss-rpt-sparkline-fallback");
      if (svg.setAttribute) {
        svg.setAttribute("viewBox", "0 0 70 30");
        svg.setAttribute("aria-hidden", "true");
        var line = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
        line.setAttribute("points", sparklinePoints(widget.source));
        line.setAttribute("fill", "none");
        line.setAttribute("stroke", "currentColor");
        svg.appendChild(line);
      }
      card.appendChild(svg);
    } else if (widget.type === "kpi") {
      card.appendChild(C.element("p", "ss-rpt-widget-kpi", t("sheet-rpt-dash-kpi-value", "Local KPI preview")));
    } else if (widget.type === "table") {
      card.appendChild(C.element("div", "ss-rpt-widget-table", t("sheet-rpt-dash-table-value", "Range preview: {range}", { range: widget.source })));
    }
    if (ctx && ctx.onRemove) {
      var remove = C.element("button", "ss-rpt-button-quiet", t("sheet-rpt-dash-remove", "Remove widget"));
      remove.type = "button";
      remove.setAttribute("data-i18n", "sheet-rpt-dash-remove");
      remove.addEventListener("click", ctx.onRemove);
      card.appendChild(remove);
    }
    return card;
  }

  function render(container, ctx) {
    container.textContent = "";
    var state = C.state();
    var note = C.element("p", "ss-rpt-dash-note", t("sheet-rpt-dash-note",
      "Compose existing named ranges and charts on a local canvas. Print-friendly layout preview only."));
    note.setAttribute("data-i18n", "sheet-rpt-dash-note");
    container.appendChild(note);
    var form = C.element("div", "ss-rpt-form");
    var title = C.element("input", "ss-rpt-input");
    title.type = "text";
    title.placeholder = t("sheet-rpt-dash-title-ph", "Monthly overview");
    title.setAttribute("data-i18n-placeholder", "sheet-rpt-dash-title-ph");
    var create = C.element("button", "ss-rpt-button", t("sheet-rpt-dash-create", "Create dashboard"));
    create.type = "button";
    create.setAttribute("data-i18n", "sheet-rpt-dash-create");
    create.addEventListener("click", function () {
      var result = createDashboard(title.value);
      ctx.setStatus(t(result.key, result.fallback), !result.ok);
      if (result.ok) { title.value = ""; ctx.rerender(); }
    });
    form.appendChild(title);
    form.appendChild(create);
    container.appendChild(form);

    if (!state.dashboards.length) {
      container.appendChild(C.element("p", "ss-rpt-empty", t("sheet-rpt-dash-empty", "No dashboards yet.")));
      return;
    }
    state.dashboards.forEach(function (dashboard) {
      var wrap = C.element("section", "ss-rpt-section");
      var heading = C.element("h3", "ss-rpt-section-title", dashboard.title);
      wrap.appendChild(heading);
      var controls = C.element("div", "ss-rpt-row");
      var type = C.element("select", "ss-rpt-input");
      TYPES.forEach(function (name) {
        var option = C.element("option", "", t(TYPE_KEYS[name], TYPE_FALLBACKS[name]));
        option.value = name;
        option.setAttribute("data-i18n", TYPE_KEYS[name]);
        type.appendChild(option);
      });
      var widgetTitle = C.element("input", "ss-rpt-input");
      widgetTitle.type = "text";
      widgetTitle.placeholder = t("sheet-rpt-dash-widget-title-ph", "Widget title");
      widgetTitle.setAttribute("data-i18n-placeholder", "sheet-rpt-dash-widget-title-ph");
      var source = C.element("input", "ss-rpt-input");
      source.type = "text";
      source.placeholder = t("sheet-rpt-dash-source-ph", "SalesChart or A1:C20");
      source.setAttribute("data-i18n-placeholder", "sheet-rpt-dash-source-ph");
      var add = C.element("button", "ss-rpt-button-small", t("sheet-rpt-dash-add-widget", "Add widget"));
      add.type = "button";
      add.setAttribute("data-i18n", "sheet-rpt-dash-add-widget");
      add.addEventListener("click", function () {
        var result = addWidget(dashboard.id, { type: type.value, title: widgetTitle.value, source: source.value });
        ctx.setStatus(t(result.key, result.fallback), !result.ok);
        if (result.ok) ctx.rerender();
      });
      controls.appendChild(type);
      controls.appendChild(widgetTitle);
      controls.appendChild(source);
      controls.appendChild(add);
      wrap.appendChild(controls);
      var canvas = C.element("div", "ss-rpt-dash-canvas");
      canvas.setAttribute("aria-label", t("sheet-rpt-dash-preview", "Local dashboard preview"));
      if (!dashboard.widgets.length) {
        canvas.appendChild(C.element("p", "ss-rpt-empty", t("sheet-rpt-dash-empty-widgets", "No widgets on this dashboard yet.")));
      }
      dashboard.widgets.forEach(function (widget) {
        var remove = function () {
          removeWidget(dashboard.id, widget.id);
          ctx.setStatus(t("sheet-rpt-widget-removed", "Widget removed from the local dashboard."), false);
          ctx.rerender();
        };
        canvas.appendChild(widgetNode(widget, { onRemove: remove }));
      });
      wrap.appendChild(canvas);
      container.appendChild(wrap);
    });
  }

  window.SheetReportingDashboard = {
    render: render,
    TYPES: TYPES,
    validateWidget: validateWidget,
    createDashboard: createDashboard,
    addWidget: addWidget,
    removeWidget: removeWidget,
    sparklinePoints: sparklinePoints,
    widgetNode: widgetNode,
  };
})();
