"use strict";

// Calculator app rendering (suite/calculator). Reads window.GBCalc state,
// draws keypad grids, screen, history and the unit converter.

(function (app) {
  var engine = window.GBCalcEngine;

  function root() {
    return document.getElementById("gb-calc-root");
  }

  function buildPad(el, keys) {
    if (!el) return;
    el.innerHTML = "";
    keys.forEach(function (row) {
      row.forEach(function (key) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "gb-calc-key " + key.cls;
        btn.textContent = key.label;
        btn.setAttribute("data-key", key.value);
        el.appendChild(btn);
      });
    });
  }

  app.renderScreen = function () {
    var box = root();
    if (!box) return;
    var input = box.querySelector("#gb-calc-expr");
    var preview = box.querySelector("#gb-calc-preview");
    if (input) input.value = app.state.expr;
    if (!preview) return;
    preview.innerHTML = "&nbsp;";
    if (!app.state.expr || !engine) return;
    try {
      var value = engine.evaluate(app.state.expr, { angle: app.state.angle });
      preview.textContent = "= " + engine.format(value);
      app.state.lastResult = value;
    } catch (e) {
      app.state.lastResult = null;
    }
  };

  // Preview-only refresh: keeps the input caret untouched while typing.
  app.updatePreview = function () {
    var box = root();
    if (!box) return;
    var preview = box.querySelector("#gb-calc-preview");
    if (!preview) return;
    preview.innerHTML = "&nbsp;";
    app.state.lastResult = null;
    if (!app.state.expr || !engine) return;
    try {
      var value = engine.evaluate(app.state.expr, { angle: app.state.angle });
      preview.textContent = "= " + engine.format(value);
      app.state.lastResult = value;
    } catch (e) { /* incomplete expression: no preview */ }
  };

  app.renderPads = function () {
    var box = root();
    if (!box) return;
    buildPad(box.querySelector("#gb-calc-pad"), app.STANDARD_KEYS);
    buildPad(box.querySelector("#gb-calc-sci"), app.SCI_KEYS);
  };

  app.renderTabs = function () {
    var box = root();
    if (!box) return;
    var isConv = app.state.tab === "convert";
    var isSci = app.state.tab === "scientific";
    box.querySelectorAll(".gb-calc-tab").forEach(function (tab) {
      tab.classList.toggle("active", tab.getAttribute("data-tab") === app.state.tab);
    });
    var sci = box.querySelector("#gb-calc-sci");
    var conv = box.querySelector("#gb-calc-converter");
    var pad = box.querySelector("#gb-calc-pad");
    if (sci) sci.hidden = !isSci;
    if (conv) conv.hidden = !isConv;
    if (pad) pad.hidden = isConv;
  };

  app.renderAngle = function () {
    var box = root();
    var btn = box ? box.querySelector("#gb-calc-angle") : null;
    if (btn) btn.textContent = app.state.angle.toUpperCase();
  };

  app.renderMemory = function () {
    var box = root();
    var btn = box ? box.querySelector("#gb-calc-memory-indicator") : null;
    if (!btn) return;
    var hasMemory = Number(app.state.memory) !== 0;
    btn.classList.toggle("active", hasMemory);
    btn.title = hasMemory
      ? "Memory: " + String(app.state.memory)
      : "Calculator memory";
  };

  app.renderHistory = function () {
    var box = root();
    if (!box) return;
    var panel = box.querySelector("#gb-calc-history");
    var list = box.querySelector("#gb-calc-history-list");
    var toggle = box.querySelector("#gb-calc-history-btn");
    if (panel) panel.hidden = !app.state.historyOpen;
    if (toggle) toggle.classList.toggle("active", app.state.historyOpen);
    if (!list) return;
    list.innerHTML = "";
    if (!app.history.length) {
      var empty = document.createElement("li");
      empty.className = "gb-calc-history-empty";
      empty.textContent = "No calculations yet.";
      list.appendChild(empty);
      return;
    }
    app.history.forEach(function (item) {
      var li = document.createElement("li");
      li.className = "gb-calc-history-item";
      li.setAttribute("data-result", item.result);
      var exprLine = document.createElement("span");
      exprLine.className = "h-expr";
      exprLine.textContent = item.expr;
      var resultLine = document.createElement("span");
      resultLine.className = "h-result";
      resultLine.textContent = item.result;
      li.appendChild(exprLine);
      li.appendChild(resultLine);
      list.appendChild(li);
    });
  };

  // ── Converter ────────────────────────────────────────────────

  function convertTemperature(value, from, to) {
    var celsius;
    if (from === "°C") celsius = value;
    else if (from === "°F") celsius = ((value - 32) * 5) / 9;
    else celsius = value - 273.15;
    if (to === "°C") return celsius;
    if (to === "°F") return (celsius * 9) / 5 + 32;
    return celsius + 273.15;
  }

  function fillSelect(select, options) {
    if (!select) return;
    select.innerHTML = "";
    options.forEach(function (opt) {
      var option = document.createElement("option");
      option.value = opt;
      option.textContent = opt;
      select.appendChild(option);
    });
  }

  app.renderConverter = function () {
    var box = root();
    if (!box) return;
    var catSelect = box.querySelector("#gb-calc-conv-cat");
    if (!catSelect || catSelect.options.length) return;
    fillSelect(catSelect, Object.keys(app.CONV_CATEGORIES));
    catSelect.value = "Length";
    app.onConverterCategory();
  };

  app.onConverterCategory = function () {
    var box = root();
    if (!box) return;
    var catName = box.querySelector("#gb-calc-conv-cat").value;
    var category = app.CONV_CATEGORIES[catName];
    var names = category.special
      ? category.units
      : Object.keys(category.units);
    fillSelect(box.querySelector("#gb-calc-conv-from"), names);
    fillSelect(box.querySelector("#gb-calc-conv-to"), names);
    if (names.length > 1) {
      box.querySelector("#gb-calc-conv-to").value = names[1];
    }
    app.computeConverter();
  };

  app.computeConverter = function () {
    var box = root();
    if (!box) return;
    var out = box.querySelector("#gb-calc-conv-out");
    if (!out) return;
    var amount = parseFloat(box.querySelector("#gb-calc-conv-amt").value);
    var from = box.querySelector("#gb-calc-conv-from").value;
    var to = box.querySelector("#gb-calc-conv-to").value;
    if (isNaN(amount) || !from || !to) {
      out.textContent = "—";
      return;
    }
    var result;
    if (app.CONV_CATEGORIES[box.querySelector("#gb-calc-conv-cat").value].special) {
      result = convertTemperature(amount, from, to);
    } else {
      var units = app.CONV_CATEGORIES[box.querySelector("#gb-calc-conv-cat").value].units;
      result = (amount * units[from]) / units[to];
    }
    out.textContent = engine ? engine.format(result) : String(result);
  };
})(window.GBCalc);
