"use strict";

// Calculator app state, keypads and storage (suite/calculator).
// Rendering lives in 02_render.js, interaction in 03_events.js.

window.GBCalc = window.GBCalc || {};

(function (app) {
  var STORAGE_KEYS = {
    history: "gb-calc-history",
    memory: "gb-calc-memory",
    angle: "gb-calc-angle",
  };

  app.state = {
    expr: "",
    tab: "standard",
    angle: "deg",
    memory: 0,
    historyOpen: false,
    lastResult: null,
  };

  // value is appended to the expression verbatim; label is what the user
  // sees; cls styles the key.
  function k(value, label, cls) {
    return { value: value, label: label || value, cls: cls || "" };
  }

  app.STANDARD_KEYS = [
    [k("C", "C", "warn"), k("⌫", "⌫", "warn"), k("%", "%", "op"), k("/", "÷", "op"), k("^", "xʸ", "op")],
    [k("7"), k("8"), k("9"), k("*", "×", "op"), k("(", "(", "op")],
    [k("4"), k("5"), k("6"), k("-", "−", "op"), k(")", ")", "op")],
    [k("1"), k("2"), k("3"), k("+", "+", "op"), k("sqrt(", "√", "fn")],
    [k("±", "±"), k("0"), k("."), k("=", "=", "equals"), k("π", "π", "fn")],
  ];

  app.SCI_KEYS = [
    [k("sin(", "sin", "fn"), k("cos(", "cos", "fn"), k("tan(", "tan", "fn"), k("ln(", "ln", "fn"), k("log(", "log", "fn")],
    [k("asin(", "sin⁻¹", "fn"), k("acos(", "cos⁻¹", "fn"), k("atan(", "tan⁻¹", "fn"), k("sqrt(", "√", "fn"), k("^2", "x²", "fn")],
    [k("e", "e", "fn"), k("abs(", "|x|", "fn"), k("exp(", "eˣ", "fn"), k("1/", "1/x", "fn"), k("!", "n!", "fn")],
  ];

  app.CONV_CATEGORIES = {
    Length: {
      base: "m",
      units: { m: 1, km: 1000, cm: 0.01, mm: 0.001, mi: 1609.344, yd: 0.9144, ft: 0.3048, in: 0.0254 },
    },
    Mass: {
      base: "g",
      units: { kg: 1000, g: 1, mg: 0.001, t: 1000000, lb: 453.59237, oz: 28.349523125 },
    },
    Temperature: {
      special: true,
      units: ["°C", "°F", "K"],
    },
  };

  app.readStore = function (key, fallback) {
    try {
      var raw = localStorage.getItem(STORAGE_KEYS[key]);
      return raw === null ? fallback : JSON.parse(raw);
    } catch (e) {
      return fallback;
    }
  };

  app.writeStore = function (key, value) {
    try {
      localStorage.setItem(STORAGE_KEYS[key], JSON.stringify(value));
    } catch (e) {}
  };

  app.loadState = function () {
    var history = app.readStore("history", []);
    if (!Array.isArray(history)) history = [];
    app.history = history;
    app.state.memory = Number(app.readStore("memory", 0)) || 0;
    var angle = app.readStore("angle", "deg");
    app.state.angle = angle === "rad" ? "rad" : "deg";
  };

  app.pushHistory = function (exprText, resultText) {
    app.history.unshift({ expr: exprText, result: resultText, ts: Date.now() });
    if (app.history.length > 50) app.history.length = 50;
    app.writeStore("history", app.history);
  };

  app.clearHistory = function () {
    app.history = [];
    app.writeStore("history", app.history);
  };
})(window.GBCalc);
