"use strict";
/* Slides masters / layouts / theme tokens / brand kit (#2429).
 * Layer 1 — DOM-free local state. Deck-scoped localStorage, one full-state
 * checkpoint per mutation, nothing is written to the server. The pure cascade
 * and apply logic lives in 61_masters_cascade.js so an offline harness can
 * exercise every acceptance criterion without a browser. */
(function (window) {
  var M = (window.GBSlidesMasters = window.GBSlidesMasters || {});
  var STORE = "slides:masters:v1:";
  var LIMIT = 60;

  M.TOKEN_KEYS = [
    "color.dk1", "color.lt1", "color.dk2", "color.lt2",
    "color.accent1", "color.accent2", "color.accent3",
    "color.accent4", "color.accent5", "color.accent6",
    "font.major", "font.minor", "effect.style"
  ];
  M.COLOR_KEYS = M.TOKEN_KEYS.filter(function (k) { return k.indexOf("color.") === 0; });
  M.ROLES = ["title", "subtitle", "body", "picture", "chart", "table", "date", "footer", "number"];
  M.EFFECTS = ["flat", "subtle", "dramatic"];

  /* Placeholder roles reference theme tokens by default. */
  M.ROLE_BINDINGS = {
    title: { font: "font.major", color: "color.lt1" },
    subtitle: { font: "font.minor", color: "color.lt2" },
    body: { font: "font.minor", color: "color.lt1" },
    date: { font: "font.minor", color: "color.dk2" },
    footer: { font: "font.minor", color: "color.dk2" },
    number: { font: "font.minor", color: "color.dk2" },
    picture: { fill: "color.accent1" },
    chart: { fill: "color.accent1" },
    table: { fill: "color.accent2" }
  };

  M.tr = function (key, fallback, params) {
    var out = typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    if (params) {
      Object.keys(params).forEach(function (name) {
        out = String(out).split("{" + name + "}").join(params[name])
          .split("{ $" + name + " }").join(params[name]);
      });
    }
    return out;
  };
  M.clone = function (value) { return JSON.parse(JSON.stringify(value)); };
  M.uid = function (prefix) {
    return (prefix || "id") + "_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 7);
  };
  M.deckId = function () {
    return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) ||
      window.__SLIDES_PRESENTATION_ID || "current";
  };
  M.storeKey = function () { return STORE + M.deckId(); };

  function defaultFlat() {
    return {
      "color.dk1": "#0f172a", "color.lt1": "#f8fafc",
      "color.dk2": "#334155", "color.lt2": "#cbd5e1",
      "color.accent1": "#3b82f6", "color.accent2": "#8b5cf6",
      "color.accent3": "#10b981", "color.accent4": "#f59e0b",
      "color.accent5": "#ef4444", "color.accent6": "#06b6d4",
      "font.major": "Arial, Helvetica, sans-serif",
      "font.minor": "Georgia, serif",
      "effect.style": "flat"
    };
  }
  M.defaultFlat = defaultFlat;

  function layout(id, name, geometry) {
    return {
      id: id, name: name, background: "#0f172a", footer: "", number: false,
      themeOverrides: {}, placeholders: geometry
    };
  }
  M.fresh = function () {
    return {
      version: 1,
      master: { name: "Default master", defaultThemeId: "theme_default" },
      themes: {
        theme_default: { id: "theme_default", name: "Corporate", flat: defaultFlat() }
      },
      layouts: [
        layout("layout_title", "Title slide", {
          title: { x: 80, y: 180, w: 800, h: 120 },
          subtitle: { x: 80, y: 310, w: 800, h: 60 },
          footer: { x: 80, y: 500, w: 400, h: 24 },
          number: { x: 860, y: 500, w: 40, h: 24 }
        }),
        layout("layout_body", "Title and content", {
          title: { x: 60, y: 40, w: 840, h: 80 },
          body: { x: 60, y: 140, w: 840, h: 340 },
          footer: { x: 60, y: 500, w: 400, h: 24 },
          number: { x: 860, y: 500, w: 40, h: 24 }
        }),
        layout("layout_section", "Section header", {
          title: { x: 100, y: 220, w: 760, h: 100 },
          body: { x: 100, y: 330, w: 760, h: 80 },
          number: { x: 860, y: 500, w: 40, h: 24 }
        })
      ],
      slides: {},
      brand: {
        fonts: ["Arial, Helvetica, sans-serif", "Georgia, serif"],
        palette: ["#0f172a", "#f8fafc", "#3b82f6", "#8b5cf6"],
        logoUrl: "", locked: false, mode: "warn"
      },
      templates: []
    };
  };

  M.load = function () {
    var raw = null;
    try { raw = window.localStorage.getItem(M.storeKey()); } catch (e) { raw = null; }
    if (!raw) return M.fresh();
    try {
      var parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== "object" || !parsed.themes || !Array.isArray(parsed.layouts)) {
        return M.fresh();
      }
      if (!parsed.slides || typeof parsed.slides !== "object") parsed.slides = {};
      if (!Array.isArray(parsed.templates)) parsed.templates = [];
      if (!parsed.brand) parsed.brand = M.fresh().brand;
      if (!parsed.master) parsed.master = M.fresh().master;
      return parsed;
    } catch (e) { return M.fresh(); }
  };
  M.init = function () { if (!M.state) M.state = M.load(); return M.state; };
  M.save = function () {
    try { window.localStorage.setItem(M.storeKey(), JSON.stringify(M.state)); } catch (e) { /* disabled */ }
    M.emit("change");
  };
  M.reset = function () { M.state = M.fresh(); M.save(); return M.state; };

  M.listeners = {};
  M.on = function (name, fn) {
    M.listeners[name] = M.listeners[name] || [];
    M.listeners[name].push(fn);
  };
  M.emit = function (name, detail) {
    (M.listeners[name] || []).forEach(function (fn) { fn(detail); });
  };

  M.undoStack = [];
  M.redoStack = [];
  M.checkpoint = function () {
    M.init();
    M.undoStack.push(M.clone(M.state));
    if (M.undoStack.length > LIMIT) M.undoStack.shift();
    M.redoStack.length = 0;
  };
  M.undo = function () {
    if (!M.undoStack.length) return false;
    M.redoStack.push(M.clone(M.state));
    M.state = M.undoStack.pop();
    M.save(); return true;
  };
  M.redo = function () {
    if (!M.redoStack.length) return false;
    M.undoStack.push(M.clone(M.state));
    M.state = M.redoStack.pop();
    M.save(); return true;
  };
  M.canUndo = function () { return M.undoStack.length > 0; };
  M.canRedo = function () { return M.redoStack.length > 0; };

  M.theme = function (id) {
    M.init();
    var key = id || M.state.master.defaultThemeId;
    return M.state.themes[key] || null;
  };
  M.layout = function (id) {
    M.init();
    var found = null;
    M.state.layouts.forEach(function (l) { if (l.id === id) found = l; });
    return found;
  };
  M.slideKey = function (idx) { return String(idx == null ? 0 : idx); };
  M.slide = function (idx) {
    M.init();
    var key = M.slideKey(idx);
    if (!M.state.slides[key]) M.state.slides[key] = { themeId: null, layoutId: null, overrides: {} };
    var s = M.state.slides[key];
    if (!s.overrides || typeof s.overrides !== "object") s.overrides = {};
    return s;
  };
  M.slideCount = function () {
    M.init();
    var thumbs = (typeof document !== "undefined" && document.querySelectorAll(".sl-thumb")) || [];
    return Math.max(thumbs.length || 1, Object.keys(M.state.slides).length);
  };

  M.mutate = function (fn) {
    M.checkpoint();
    var result = fn(M.state);
    M.save();
    return result;
  };

  /* Change a master theme token. Every slide inheriting this theme with no
   * explicit override for the token resolves to the new value immediately. */
  M.setMasterToken = function (themeId, key, value) {
    return M.mutate(function (state) {
      var theme = state.themes[themeId || state.master.defaultThemeId];
      if (!theme) return [];
      theme.flat[key] = value;
      return Object.keys(state.slides).filter(function (slideKey) {
        var s = state.slides[slideKey];
        var usesTheme = (s.themeId || state.master.defaultThemeId) === theme.id;
        return usesTheme && !(s.overrides && Object.prototype.hasOwnProperty.call(s.overrides, key));
      });
    });
  };
})(typeof window !== "undefined" ? window : globalThis);
