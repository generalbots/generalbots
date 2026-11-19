"use strict";
/* Sheets data connectivity (#2492) - ordered query steps with per-step preview.
 * window.SheetConnectQuery. Steps are stored as a reproducible JSON definition
 * in the workbook-scoped local state; nothing is sent to a server. */

(function () {
  var C = window.SheetConnectCore;
  var S = window.SheetConnectSteps;
  var F = window.SheetConnectFeed;
  var U = window.SheetConnectUi;
  var t = C.t;

  var TYPES = [
    { value: "select", key: "sheet-connect-step-select", fallback: "Select columns" },
    { value: "filter", key: "sheet-connect-step-filter", fallback: "Filter" },
    { value: "rename", key: "sheet-connect-step-rename", fallback: "Rename column" },
    { value: "type", key: "sheet-connect-step-type", fallback: "Change type" },
    { value: "unpivot", key: "sheet-connect-step-unpivot", fallback: "Unpivot" },
    { value: "group", key: "sheet-connect-step-group", fallback: "Group" },
  ];
  var OPS = [
    { value: "eq", key: "sheet-connect-op-eq", fallback: "equals" },
    { value: "neq", key: "sheet-connect-op-neq", fallback: "not equals" },
    { value: "contains", key: "sheet-connect-op-contains", fallback: "contains" },
    { value: "gt", key: "sheet-connect-op-gt", fallback: "greater than" },
    { value: "lt", key: "sheet-connect-op-lt", fallback: "less than" },
  ];
  var KINDS = [
    { value: "text", key: "sheet-connect-step-target-text", fallback: "text" },
    { value: "number", key: "sheet-connect-step-target-number", fallback: "number" },
    { value: "date", key: "sheet-connect-step-target-date", fallback: "date" },
  ];
  var AGGS = [
    { value: "sum", key: "sheet-connect-agg-sum", fallback: "sum" },
    { value: "count", key: "sheet-connect-agg-count", fallback: "count" },
  ];
  var DEFAULTS = {
    select: { columns: "date,region,amount" },
    filter: { field: "amount", op: "gt", value: "100" },
    rename: { from: "amount", to: "total" },
    type: { field: "amount", to: "number" },
    unpivot: { columns: "amount", categoryHeader: "series", valueHeader: "value" },
    group: { by: "region", aggField: "amount", agg: "sum" },
  };

  var previewIndex = -1;

  function typeLabel(type) {
    var found = TYPES.filter(function (item) { return item.value === type; })[0];
    return found ? t(found.key, found.fallback) : type;
  }

  function addStep(type, ctx) {
    var step = {
      id: "step-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e4).toString(36),
      type: type, enabled: true,
      config: Object.assign({}, DEFAULTS[type] || {}),
    };
    C.mutate(function (target) { target.steps.push(step); });
    previewIndex = C.state().steps.length - 1;
    ctx.setStatus(t("sheet-connect-step-saved", "Step added to the local query."));
    ctx.rerender();
  }

  function updateStep(id, fn, ctx) {
    C.mutate(function (target) {
      target.steps = target.steps.map(function (step) { return step.id === id ? fn(step) : step; });
    });
    ctx.rerender();
  }

  function reorder(steps, id, delta) {
    var index = steps.findIndex(function (step) { return step.id === id; });
    var next = index + delta;
    if (index < 0 || next < 0 || next >= steps.length) return false;
    var item = steps[index];
    steps[index] = steps[next];
    steps[next] = item;
    return true;
  }

  function move(id, delta, ctx) {
    var moved = false;
    C.mutate(function (target) {
      var index = target.steps.findIndex(function (step) { return step.id === id; });
      moved = reorder(target.steps, id, delta);
      if (!moved) return;
      if (previewIndex === index) previewIndex = index + delta;
      else if (previewIndex === index + delta) previewIndex = index;
    });
    if (!moved) return;
    ctx.setStatus(t("sheet-connect-step-moved", "Step order changed."));
    ctx.rerender();
  }

  function remove(id, ctx) {
    C.mutate(function (target) {
      target.steps = target.steps.filter(function (step) { return step.id !== id; });
    });
    previewIndex = -1;
    ctx.setStatus(t("sheet-connect-step-removed", "Step removed."));
    ctx.rerender();
  }

  function bindText(wrap, step, key, ctx) {
    wrap.input.addEventListener("change", function () {
      var value = wrap.input.value;
      updateStep(step.id, function (current) {
        current.config[key] = value;
        return current;
      }, ctx);
    });
  }

  function bindSelect(wrap, step, key, ctx) {
    wrap.input.addEventListener("change", function () {
      var value = wrap.input.value;
      updateStep(step.id, function (current) {
        current.config[key] = value;
        return current;
      }, ctx);
    });
  }

  function configFields(step, ctx) {
    var result = [];
    var config = step.config || {};
    if (step.type === "select") {
      var columns = U.field("sheet-connect-step-config-columns", "Columns (comma separated)", "text", config.columns);
      bindText(columns, step, "columns", ctx);
      result.push(columns);
    } else if (step.type === "filter") {
      var field = U.field("sheet-connect-step-config-field", "Field", "text", config.field);
      bindText(field, step, "field", ctx);
      result.push(field);
      var op = U.select("sheet-connect-step-config-op", "Operator", OPS, config.op);
      bindSelect(op, step, "op", ctx);
      result.push(op);
      var value = U.field("sheet-connect-step-config-value", "Value (or {param:key})", "text", config.value);
      bindText(value, step, "value", ctx);
      result.push(value);
    } else if (step.type === "rename") {
      var from = U.field("sheet-connect-step-config-from", "From", "text", config.from);
      bindText(from, step, "from", ctx);
      result.push(from);
      var to = U.field("sheet-connect-step-config-to", "To", "text", config.to);
      bindText(to, step, "to", ctx);
      result.push(to);
    } else if (step.type === "type") {
      var typeField = U.field("sheet-connect-step-config-field", "Field", "text", config.field);
      bindText(typeField, step, "field", ctx);
      result.push(typeField);
      var target = U.select("sheet-connect-step-config-target-type", "Target type", KINDS, config.to);
      bindSelect(target, step, "to", ctx);
      result.push(target);
    } else if (step.type === "unpivot") {
      var unpivotColumns = U.field("sheet-connect-step-config-columns", "Columns (comma separated)", "text", config.columns);
      bindText(unpivotColumns, step, "columns", ctx);
      result.push(unpivotColumns);
      var category = U.field("sheet-connect-step-config-value-header", "Category column", "text", config.categoryHeader);
      bindText(category, step, "categoryHeader", ctx);
      result.push(category);
      var values = U.field("sheet-connect-step-config-value-column", "Value column", "text", config.valueHeader);
      bindText(values, step, "valueHeader", ctx);
      result.push(values);
    } else if (step.type === "group") {
      var by = U.field("sheet-connect-step-config-by", "Group by", "text", config.by);
      bindText(by, step, "by", ctx);
      result.push(by);
      var aggField = U.field("sheet-connect-step-config-agg-field", "Aggregate field", "text", config.aggField);
      bindText(aggField, step, "aggField", ctx);
      result.push(aggField);
      var agg = U.select("sheet-connect-step-config-agg", "Aggregation", AGGS, config.agg);
      bindSelect(agg, step, "agg", ctx);
      result.push(agg);
    }
    return result;
  }

  function renderStepList(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-step-list", "Query steps"));
    var add = U.el("div", "ss-connect-type-row");
    TYPES.forEach(function (item) {
      add.appendChild(U.button(item.key, item.fallback, function () { addStep(item.value, ctx); }, "ss-connect-type-btn"));
    });
    container.appendChild(add);
    if (!target.steps.length) {
      container.appendChild(U.note("sheet-connect-step-empty", "No steps yet. Add a step to shape the local feed."));
      return;
    }
    var list = U.el("div", "ss-connect-step-list");
    target.steps.forEach(function (step, index) {
      var row = U.el("div", "ss-connect-step" + (step.enabled === false ? " is-disabled" : ""));
      var head = U.el("div", "ss-connect-step-head");
      head.appendChild(U.el("span", "ss-connect-step-index", String(index + 1)));
      head.appendChild(U.el("strong", "", typeLabel(step.type)));
      var enabled = U.checkbox("sheet-connect-step-enabled", "Enabled", step.enabled !== false, function () {
        updateStep(step.id, function (current) { current.enabled = !current.enabled; return current; }, ctx);
      });
      head.appendChild(enabled);
      head.appendChild(U.button("sheet-connect-step-preview", "Preview at this step", function () {
        previewIndex = index;
        ctx.rerender();
      }));
      head.appendChild(U.iconButton("sheet-connect-step-up", "Move up", "\u2191", function () { move(step.id, -1, ctx); }));
      head.appendChild(U.iconButton("sheet-connect-step-down", "Move down", "\u2193", function () { move(step.id, 1, ctx); }));
      head.appendChild(U.iconButton("sheet-connect-step-delete", "Remove step", "\u00d7", function () { remove(step.id, ctx); }));
      row.appendChild(head);
      var config = U.el("div", "ss-connect-step-config");
      configFields(step, ctx).forEach(function (node) { config.appendChild(node); });
      row.appendChild(config);
      list.appendChild(row);
    });
    container.appendChild(list);
  }

  function renderPreview(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-preview-title", "Preview"));
    var connector = F.activeConnector(target);
    if (!connector) {
      container.appendChild(U.note("sheet-connect-preview-empty", "No preview available. Add an active connector first."));
    } else {
      var table = previewIndex >= 0
        ? S.preview(target, connector, previewIndex + 1)
        : S.applyAll(target, connector).table;
      container.appendChild(U.table(table.columns, table.rows.map(function (row) {
        return row.map(function (cell) { return String(cell); });
      })));
    }
    container.appendChild(U.title("sheet-connect-definition", "Reproducible query definition (JSON, local)"));
    var pre = U.el("pre", "ss-connect-definition");
    pre.textContent = JSON.stringify(F.definition(target), null, 2);
    container.appendChild(pre);
  }

  function render(container, ctx) {
    var target = ctx.state();
    renderStepList(container, ctx, target);
    renderPreview(container, ctx, target);
  }

  window.SheetConnectQuery = { render: render, DEFAULTS: DEFAULTS, TYPES: TYPES, reorder: reorder };
})();
