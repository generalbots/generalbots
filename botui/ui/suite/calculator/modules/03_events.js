"use strict";

// Calculator app interaction (suite/calculator): keypad clicks, keyboard
// input, memory actions, tab switching and converter events.

(function (app) {
  function root() {
    return document.getElementById("gb-calc-root");
  }

  function pressKey(value) {
    var expr = app.state.expr;
    if (value === "C") {
      expr = "";
      app.state.lastResult = null;
    } else if (value === "⌫") {
      expr = expr.slice(0, -1);
    } else if (value === "=") {
      var engine = window.GBCalcEngine;
      if (!engine || !expr) return;
      try {
        var result = engine.format(engine.evaluate(expr, { angle: app.state.angle }));
        app.pushHistory(expr, result);
        expr = result;
        app.renderHistory();
        app.state.lastResult = null;
      } catch (e) {
        app.renderScreen();
        return;
      }
    } else if (value === "±") {
      // Toggle the sign of the final number in the expression.
      var match = expr.match(/(\d+\.?\d*)$/);
      if (match) {
        var start = expr.length - match[1].length;
        var before = expr.slice(0, start);
        expr = before.endsWith("-")
          ? before.slice(0, -1) + match[1]
          : before + "(-" + match[1];
      } else {
        expr = expr + "(-";
      }
    } else {
      expr += value;
    }
    app.state.expr = expr;
    app.renderScreen();
  }

  function copyResult() {
    var text = app.state.lastResult !== null && window.GBCalcEngine
      ? window.GBCalcEngine.format(app.state.lastResult)
      : app.state.expr;
    if (!text || !navigator.clipboard) return;
    navigator.clipboard.writeText(text).catch(function () {});
  }

  function toggleMemory(action) {
    var engine = window.GBCalcEngine;
    switch (action) {
      case "mc":
        app.state.memory = 0;
        break;
      case "mr":
        if (engine) {
          app.state.expr += String(app.state.memory);
          app.renderScreen();
        }
        break;
      case "mplus":
      case "mminus":
        if (!engine) return;
        try {
          var value = engine.evaluate(app.state.expr || "0", { angle: app.state.angle });
          var sign = action === "mplus" ? 1 : -1;
          app.state.memory = Number(app.state.memory) + sign * value;
        } catch (e) { /* keep previous memory */ }
        break;
      default:
        break;
    }
    app.writeStore("memory", app.state.memory);
    app.renderMemory();
  }

  function bindPadClicks(box) {
    box.addEventListener("click", function (e) {
      var keyBtn = e.target.closest("[data-key]");
      if (keyBtn) {
        pressKey(keyBtn.getAttribute("data-key"));
        return;
      }
      var historyItem = e.target.closest(".gb-calc-history-item");
      if (historyItem) {
        app.state.expr += historyItem.getAttribute("data-result");
        app.renderScreen();
      }
    });
  }

  function bindToolbar(box) {
    box.querySelectorAll(".gb-calc-tab").forEach(function (tab) {
      tab.addEventListener("click", function () {
        app.state.tab = tab.getAttribute("data-tab");
        app.renderTabs();
        if (app.state.tab === "convert") app.renderConverter();
      });
    });

    var angle = box.querySelector("#gb-calc-angle");
    if (angle) {
      angle.addEventListener("click", function () {
        app.state.angle = app.state.angle === "deg" ? "rad" : "deg";
        app.writeStore("angle", app.state.angle);
        app.renderAngle();
        app.renderScreen();
      });
    }

    var copy = box.querySelector("#gb-calc-copy");
    if (copy) copy.addEventListener("click", copyResult);

    var histBtn = box.querySelector("#gb-calc-history-btn");
    if (histBtn) {
      histBtn.addEventListener("click", function () {
        app.state.historyOpen = !app.state.historyOpen;
        app.renderHistory();
      });
    }

    var histClear = box.querySelector("#gb-calc-history-clear");
    if (histClear) {
      histClear.addEventListener("click", function () {
        app.clearHistory();
        app.renderHistory();
      });
    }

    var memIndicator = box.querySelector("#gb-calc-memory-indicator");
    if (memIndicator) {
      memIndicator.addEventListener("click", function () { toggleMemory("mr"); });
      memIndicator.addEventListener("contextmenu", function (e) {
        e.preventDefault();
        toggleMemory("mc");
      });
    }
  }

  function bindInput(box) {
    var input = box.querySelector("#gb-calc-expr");
    if (!input) return;
    input.addEventListener("input", function () {
      app.state.expr = input.value;
      app.updatePreview();
    });
    input.addEventListener("keydown", function (e) {
      if (e.key === "Enter") {
        e.preventDefault();
        pressKey("=");
      } else if (e.key === "Escape") {
        e.preventDefault();
        pressKey("C");
      } else if (
        e.key.length === 1 &&
        !"0123456789+-*/.()%^!".includes(e.key) &&
        !/[a-zπ]/i.test(e.key)
      ) {
        // Allow digits/operators and letters (function and constant names);
        // unknown names are rejected by the engine at evaluation time.
        e.preventDefault();
      }
    });
  }

  function bindConverter(box) {
    var cat = box.querySelector("#gb-calc-conv-cat");
    if (!cat) return;
    cat.addEventListener("change", app.onConverterCategory);
    var amt = box.querySelector("#gb-calc-conv-amt");
    if (amt) amt.addEventListener("input", app.computeConverter);
    box.querySelector("#gb-calc-conv-from").addEventListener("change", app.computeConverter);
    box.querySelector("#gb-calc-conv-to").addEventListener("change", app.computeConverter);
    var swap = box.querySelector("#gb-calc-conv-swap");
    if (swap) {
      swap.addEventListener("click", function () {
        var from = box.querySelector("#gb-calc-conv-from");
        var to = box.querySelector("#gb-calc-conv-to");
        var tmp = from.value;
        from.value = to.value;
        to.value = tmp;
        app.computeConverter();
      });
    }
  }

  app.bindEvents = function () {
    var box = root();
    if (!box) return;
    bindPadClicks(box);
    bindToolbar(box);
    bindInput(box);
    bindConverter(box);
  };

  app.pressKeyForTests = pressKey;
})(window.GBCalc);
