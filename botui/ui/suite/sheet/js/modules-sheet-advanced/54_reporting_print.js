"use strict";
/* Sheets print, reporting and scheduled delivery (#2438) — print areas and
 * page setup. Pure logic (validation + pagination) is exported for offline
 * tests; rendering only builds local DOM and never calls a server. */

(function () {
  var C = window.SheetReportingCore;
  var t = C.t;
  var PAPER = {
    A4: { cols: 8, rows: 45 },
    A3: { cols: 12, rows: 60 },
    Letter: { cols: 9, rows: 42 },
    Legal: { cols: 9, rows: 52 },
  };
  var SIDE_KEYS = {
    top: "sheet-rpt-side-top",
    bottom: "sheet-rpt-side-bottom",
    left: "sheet-rpt-side-left",
    right: "sheet-rpt-side-right",
  };

  function makeId(prefix) {
    return prefix + "-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e5).toString(36);
  }

  function validateArea(fields) {
    var name = String((fields && fields.name) || "").trim();
    var range = String((fields && fields.range) || "").trim();
    if (!name || !range) return { ok: false, key: "sheet-rpt-needs-fields", fallback: "Enter a name and a range." };
    if (!C.parseRange(range)) return { ok: false, key: "sheet-rpt-invalid-range", fallback: "Enter a valid range such as A1:H60." };
    return { ok: true, name: name.slice(0, 80), range: range.toUpperCase() };
  }

  function addArea(fields) {
    var valid = validateArea(fields);
    if (!valid.ok) return valid;
    C.mutate(function (state) {
      state.printAreas.push({ id: makeId("area"), name: valid.name, range: valid.range });
    });
    return valid;
  }

  function clearArea(id) {
    C.mutate(function (state) {
      state.printAreas = state.printAreas.filter(function (area) { return area.id !== id; });
    });
    return true;
  }

  function clearAreas() {
    C.mutate(function (state) { state.printAreas = []; });
    return true;
  }

  function summary() {
    var areas = C.state().printAreas;
    return { count: areas.length, key: "sheet-rpt-area-summary", fallback: "{count} print area(s) defined locally." };
  }

  function updateSetup(patch) {
    C.mutate(function (state) {
      state.pageSetup = C.normalizeSetup(Object.assign({}, state.pageSetup, patch));
    });
    return C.state().pageSetup;
  }

  function addBreak(axis, before) {
    var value = axis === "col" ? C.colToNumber(String(before || "").toUpperCase()) : Number(before) - 1;
    if (!Number.isInteger(value) || value < 0) {
      return { ok: false, key: "sheet-rpt-invalid-break", fallback: "Enter a valid column letter or row number." };
    }
    C.mutate(function (state) {
      var setup = state.pageSetup;
      var exists = setup.manualBreaks.some(function (brk) { return brk.axis === axis && brk.before === value; });
      if (!exists) setup.manualBreaks.push({ axis: axis, before: value });
      setup.manualBreaks.sort(function (a, b) { return a.axis.localeCompare(b.axis) || a.before - b.before; });
    });
    return { ok: true, key: "sheet-rpt-break-added", fallback: "Manual page break added to the local preview." };
  }

  function clearBreaks() {
    C.mutate(function (state) { state.pageSetup.manualBreaks = []; });
    return true;
  }

  function autoBreaks(start, end, perPage, manual, axis) {
    var points = [start];
    var marks = manual.filter(function (brk) {
      return brk.axis === axis && brk.before > start && brk.before <= end;
    }).map(function (brk) { return brk.before; }).sort(function (a, b) { return a - b; });
    marks.forEach(function (point) { if (points.indexOf(point) < 0) points.push(point); });
    points.push(end + 1);
    points.sort(function (a, b) { return a - b; });
    var out = [];
    for (var i = 0; i < points.length - 1; i++) {
      var segStart = points[i];
      var segEnd = points[i + 1] - 1;
      if (segEnd < segStart) continue;
      for (var cursor = segStart; cursor <= segEnd; cursor += perPage) {
        out.push({
          start: cursor,
          end: Math.min(cursor + perPage - 1, segEnd),
          manual: marks.indexOf(cursor) >= 0,
        });
      }
    }
    return out.length ? out : [{ start: start, end: end, manual: false }];
  }

  /* Deterministic page math shared by the preview. This approximates a
   * paginated layout; it is not a PDF engine and makes no parity guarantee. */
  function paginate(area, setup) {
    var s = C.normalizeSetup(setup);
    var parsed = C.parseRange(area && area.range ? area.range : area);
    if (!parsed) {
      return { pages: 0, colPages: 0, rowPages: 0, colBreaks: [], rowBreaks: [],
        colsPerPage: 0, rowsPerPage: 0, repeatRows: s.repeatRows, reason: "no-area" };
    }
    var base = PAPER[s.paper] || PAPER.A4;
    var landscape = s.orientation === "landscape";
    var cols = landscape ? Math.round(base.cols * 1.5) : base.cols;
    var rows = landscape ? Math.round(base.rows * 0.7) : base.rows;
    cols = Math.max(2, cols - Math.ceil((s.margins.left + s.margins.right) * 2));
    rows = Math.max(5, rows - Math.ceil((s.margins.top + s.margins.bottom) * 2));
    var factor = s.scaleMode === "percent" ? s.percent / 100 : 1;
    if (s.scaleMode === "fit" && s.fitToWidth > 0) cols = Math.max(cols, Math.ceil(parsed.cols / s.fitToWidth));
    else cols = Math.max(2, Math.round(cols * factor));
    if (s.scaleMode === "fit" && s.fitToHeight > 0) rows = Math.max(rows, Math.ceil(parsed.rows / s.fitToHeight));
    else rows = Math.max(5, Math.round(rows * factor));
    var colBreaks = autoBreaks(parsed.startCol, parsed.endCol, cols, s.manualBreaks, "col");
    var rowBreaks = autoBreaks(parsed.startRow, parsed.endRow, rows, s.manualBreaks, "row");
    return {
      pages: colBreaks.length * rowBreaks.length,
      colPages: colBreaks.length,
      rowPages: rowBreaks.length,
      colBreaks: colBreaks,
      rowBreaks: rowBreaks,
      colsPerPage: cols,
      rowsPerPage: rows,
      repeatRows: s.repeatRows,
      reason: "ok",
    };
  }

  function fieldRow(labelText, control) {
    var row = C.element("label", "ss-rpt-field");
    row.appendChild(C.element("span", "ss-rpt-label", labelText));
    row.appendChild(control);
    return row;
  }

  function textInput(value, placeholderKey, placeholderFallback, onInput) {
    var input = C.element("input", "ss-rpt-input");
    input.type = "text";
    input.value = value || "";
    input.placeholder = t(placeholderKey, placeholderFallback);
    input.setAttribute("data-i18n-placeholder", placeholderKey);
    input.addEventListener("change", onInput);
    return input;
  }

  function selectInput(options, value, onChange) {
    var select = C.element("select", "ss-rpt-input");
    options.forEach(function (option) {
      var item = C.element("option", "", t(option.key, option.fallback));
      item.value = option.value;
      item.setAttribute("data-i18n", option.key);
      select.appendChild(item);
    });
    select.value = value;
    select.addEventListener("change", onChange);
    return select;
  }

  function section(titleKey, titleFallback) {
    var wrap = C.element("section", "ss-rpt-section");
    var heading = C.element("h3", "ss-rpt-section-title", t(titleKey, titleFallback));
    heading.setAttribute("data-i18n", titleKey);
    wrap.appendChild(heading);
    return wrap;
  }

  function renderAreas(container, ctx) {
    var state = C.state();
    var wrap = section("sheet-rpt-areas-title", "Print areas");
    var form = C.element("div", "ss-rpt-form");
    var name = textInput("", "sheet-rpt-area-name-ph", "Area name", function () {});
    var range = textInput("", "sheet-rpt-area-range-ph", "A1:H60", function () {});
    form.appendChild(fieldRow(t("sheet-rpt-area-name", "Area name"), name));
    form.appendChild(fieldRow(t("sheet-rpt-area-range", "Print range"), range));
    var add = C.element("button", "ss-rpt-button", t("sheet-rpt-area-add", "Add print area"));
    add.type = "button";
    add.setAttribute("data-i18n", "sheet-rpt-area-add");
    add.addEventListener("click", function () {
      var result = addArea({ name: name.value, range: range.value });
      ctx.setStatus(t(result.key, result.fallback), !result.ok);
      if (result.ok) { name.value = ""; range.value = ""; ctx.rerender(); }
    });
    form.appendChild(add);
    var list = C.element("ul", "ss-rpt-list");
    if (!state.printAreas.length) {
      list.appendChild(C.element("li", "ss-rpt-empty", t("sheet-rpt-areas-empty", "No print areas defined yet.")));
    }
    state.printAreas.forEach(function (area) {
      var item = C.element("li", "ss-rpt-list-item");
      item.appendChild(C.element("span", "ss-rpt-item-label", area.name + " - " + area.range));
      var remove = C.element("button", "ss-rpt-button-small", t("sheet-rpt-area-clear", "Clear"));
      remove.type = "button";
      remove.setAttribute("data-i18n", "sheet-rpt-area-clear");
      remove.addEventListener("click", function () {
        clearArea(area.id);
        ctx.setStatus(t("sheet-rpt-area-cleared", "Print area removed from the local preview."), false);
        ctx.rerender();
      });
      item.appendChild(remove);
      list.appendChild(item);
    });
    var count = summary();
    var note = C.element("p", "ss-rpt-summary", t(count.key, count.fallback, { count: count.count }));
    form.appendChild(list);
    form.appendChild(note);
    if (state.printAreas.length) {
      var clearAll = C.element("button", "ss-rpt-button-quiet", t("sheet-rpt-areas-clear-all", "Clear all"));
      clearAll.type = "button";
      clearAll.setAttribute("data-i18n", "sheet-rpt-areas-clear-all");
      clearAll.addEventListener("click", function () {
        clearAreas();
        ctx.setStatus(t("sheet-rpt-areas-cleared", "All print areas cleared locally."), false);
        ctx.refreshPreview();
        ctx.rerender();
      });
      form.appendChild(clearAll);
    }
    wrap.appendChild(form);
    container.appendChild(wrap);
  }

  function renderSetup(container, ctx) {
    var setup = C.state().pageSetup;
    var wrap = section("sheet-rpt-setup-title", "Page setup");
    var form = C.element("div", "ss-rpt-form");
    function commit(patch) {
      updateSetup(patch);
      ctx.setStatus(t("sheet-rpt-setup-saved", "Page setup updated in the local preview."), false);
      ctx.refreshPreview();
    }
    var sides = ["top", "bottom", "left", "right"];
    var options = {
      orientation: [
        { value: "portrait", key: "sheet-rpt-setup-portrait", fallback: "Portrait" },
        { value: "landscape", key: "sheet-rpt-setup-landscape", fallback: "Landscape" },
      ],
      scaleMode: [
        { value: "fit", key: "sheet-rpt-setup-fit", fallback: "Fit to pages" },
        { value: "percent", key: "sheet-rpt-setup-percent", fallback: "Scale percentage" },
      ],
    };
    form.appendChild(fieldRow(t("sheet-rpt-setup-orientation", "Orientation"),
      selectInput(options.orientation, setup.orientation, function (e) { commit({ orientation: e.target.value }); })));
    var paper = [
      { value: "A4", key: "sheet-rpt-paper-a4", fallback: "A4" },
      { value: "A3", key: "sheet-rpt-paper-a3", fallback: "A3" },
      { value: "Letter", key: "sheet-rpt-paper-letter", fallback: "Letter" },
      { value: "Legal", key: "sheet-rpt-paper-legal", fallback: "Legal" },
    ];
    form.appendChild(fieldRow(t("sheet-rpt-setup-paper", "Paper size"),
      selectInput(paper, setup.paper, function (e) { commit({ paper: e.target.value }); })));
    sides.forEach(function (side) {
      var input = C.element("input", "ss-rpt-input");
      input.type = "number";
      input.min = "0";
      input.max = "3";
      input.step = "0.1";
      input.value = String(setup.margins[side]);
      input.addEventListener("change", function (e) {
        var margins = Object.assign({}, setup.margins);
        margins[side] = Number(e.target.value);
        commit({ margins: margins });
        setup = C.state().pageSetup;
      });
      form.appendChild(fieldRow(t("sheet-rpt-margin", "Margin {side} (in)", { side: t(SIDE_KEYS[side], side) }), input));
    });
    form.appendChild(fieldRow(t("sheet-rpt-setup-scale-mode", "Scaling"),
      selectInput(options.scaleMode, setup.scaleMode, function (e) { commit({ scaleMode: e.target.value }); })));
    form.appendChild(fieldRow(t("sheet-rpt-setup-fit-width", "Fit to width (pages)"), numberInput(setup.fitToWidth, 0, 20, function (e) {
      commit({ fitToWidth: Number(e.target.value) });
    })));
    form.appendChild(fieldRow(t("sheet-rpt-setup-fit-height", "Fit to height (pages)"), numberInput(setup.fitToHeight, 0, 20, function (e) {
      commit({ fitToHeight: Number(e.target.value) });
    })));
    form.appendChild(fieldRow(t("sheet-rpt-setup-percent-value", "Scale percentage"), numberInput(setup.percent, 10, 400, function (e) {
      commit({ percent: Number(e.target.value) });
    })));
    form.appendChild(fieldRow(t("sheet-rpt-setup-repeat-rows", "Repeat title rows"),
      textInput(setup.repeatRows, "sheet-rpt-setup-repeat-ph", "1:3", function (e) { commit({ repeatRows: e.target.value }); })));
    var breaks = C.element("div", "ss-rpt-row");
    var axis = selectInput([
      { value: "col", key: "sheet-rpt-setup-break-col", fallback: "Before column" },
      { value: "row", key: "sheet-rpt-setup-break-row", fallback: "Before row" },
    ], "col", function () {});
    var before = textInput("", "sheet-rpt-setup-break-ph", "D or 20", function () {});
    var addBreakBtn = C.element("button", "ss-rpt-button-small", t("sheet-rpt-setup-add-break", "Add break"));
    addBreakBtn.type = "button";
    addBreakBtn.setAttribute("data-i18n", "sheet-rpt-setup-add-break");
    addBreakBtn.addEventListener("click", function () {
      var result = addBreak(axis.value, before.value);
      ctx.setStatus(t(result.key, result.fallback), !result.ok);
      if (result.ok) ctx.rerender();
    });
    breaks.appendChild(axis);
    breaks.appendChild(before);
    breaks.appendChild(addBreakBtn);
    if (setup.manualBreaks.length) {
      var reset = C.element("button", "ss-rpt-button-quiet", t("sheet-rpt-setup-clear-breaks", "Clear breaks"));
      reset.type = "button";
      reset.setAttribute("data-i18n", "sheet-rpt-setup-clear-breaks");
      reset.addEventListener("click", function () {
        clearBreaks();
        ctx.setStatus(t("sheet-rpt-break-cleared", "Manual page breaks cleared locally."), false);
        ctx.rerender();
      });
      breaks.appendChild(reset);
    }
    form.appendChild(fieldRow(t("sheet-rpt-setup-break", "Manual break"), breaks));
    wrap.appendChild(form);
    container.appendChild(wrap);
  }

  function numberInput(value, min, max, onChange) {
    var input = C.element("input", "ss-rpt-input");
    input.type = "number";
    input.min = String(min);
    input.max = String(max);
    input.value = String(value);
    input.addEventListener("change", onChange);
    return input;
  }

  function render(container, ctx) {
    container.textContent = "";
    renderAreas(container, ctx);
    renderSetup(container, ctx);
  }

  window.SheetReportingPrint = {
    render: render,
    addArea: addArea,
    clearArea: clearArea,
    clearAreas: clearAreas,
    summary: summary,
    updateSetup: updateSetup,
    paginate: paginate,
    addBreak: addBreak,
    clearBreaks: clearBreaks,
    validateArea: validateArea,
    PAPER: PAPER,
  };
})();
