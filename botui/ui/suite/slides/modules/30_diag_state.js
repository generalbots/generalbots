"use strict";
/* Slides diagram authoring — local-only state, storage and one-step undo (#2488).
 * Authored diagrams live in deck-scoped localStorage. Nothing is written to the
 * server: the element model / PPTX connector export needs a Rust compile gate. */

function sdT(key, fallback, params) {
  return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
}

function sdClone(value) {
  return JSON.parse(JSON.stringify(value));
}

function sdUid(prefix) {
  return prefix + "_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 6);
}

function sdEl(tag, cls, text, key) {
  var node = document.createElement(tag);
  if (cls) node.className = cls;
  if (key) node.setAttribute("data-i18n", key);
  if (text !== undefined && text !== null) node.textContent = text;
  return node;
}

function sd$(selector, root) {
  return (root || document).querySelector(selector);
}

function sd$$(selector, root) {
  return Array.prototype.slice.call((root || document).querySelectorAll(selector));
}

var SD_STORE_PREFIX = "slides:diagrams:v1:";
var SD_UNDO_LIMIT = 60;

var GBSlidesDiagrams = {
  state: null,
  undoStack: [],
  redoStack: [],
  listeners: {},
  _activeId: null,

  deckId: function () {
    return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) ||
      window.__SLIDES_PRESENTATION_ID || "current";
  },

  slideIndex: function () {
    var cv = (window.SlideCanvas && window.SlideCanvas.canvas) || document.getElementById("slide-canvas");
    if (!cv) return 0;
    var raw = cv.dataset ? cv.dataset.slideId : cv.getAttribute("data-slide-id");
    var n = parseInt(raw, 10);
    return isFinite(n) && n >= 0 ? n : 0;
  },

  fresh: function () {
    return { slides: {} };
  },

  load: function () {
    var raw = null;
    try { raw = window.localStorage.getItem(SD_STORE_PREFIX + this.deckId()); } catch (e) { raw = null; }
    if (!raw) return this.fresh();
    try {
      var parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== "object" || !parsed.slides || typeof parsed.slides !== "object") {
        return this.fresh();
      }
      return parsed;
    } catch (e) {
      return this.fresh();
    }
  },

  init: function () {
    if (!this.state) this.state = this.load();
    return this.state;
  },

  save: function () {
    try {
      window.localStorage.setItem(SD_STORE_PREFIX + this.deckId(), JSON.stringify(this.state));
    } catch (e) { /* storage may be disabled */ }
    this.emit("saved");
  },

  emit: function (name, detail) {
    (this.listeners[name] || []).forEach(function (fn) { fn(detail); });
  },

  on: function (name, fn) {
    this.listeners[name] = this.listeners[name] || [];
    this.listeners[name].push(fn);
  },

  slideKey: function (idx) {
    return String(idx == null ? this.slideIndex() : idx);
  },

  list: function (idx) {
    this.init();
    var list = this.state.slides[this.slideKey(idx)];
    return Array.isArray(list) ? list : [];
  },

  get: function (id, idx) {
    var found = null;
    this.list(idx).forEach(function (diagram) {
      if (diagram.id === id) found = diagram;
    });
    return found;
  },

  /* Snapshot the whole local state before a mutation: one step per operation. */
  checkpoint: function () {
    this.init();
    this.undoStack.push(sdClone(this.state));
    if (this.undoStack.length > SD_UNDO_LIMIT) this.undoStack.shift();
    this.redoStack.length = 0;
  },

  undo: function () {
    if (!this.undoStack.length) return false;
    this.redoStack.push(sdClone(this.state));
    this.state = this.undoStack.pop();
    this.save();
    this.emit("change");
    return true;
  },

  redo: function () {
    if (!this.redoStack.length) return false;
    this.undoStack.push(sdClone(this.state));
    this.state = this.redoStack.pop();
    this.save();
    this.emit("change");
    return true;
  },

  canUndo: function () { return this.undoStack.length > 0; },
  canRedo: function () { return this.redoStack.length > 0; },

  upsert: function (diagram) {
    this.checkpoint();
    var key = this.slideKey(diagram.slideIndex);
    var list = this.state.slides[key] || (this.state.slides[key] = []);
    var at = -1;
    for (var i = 0; i < list.length; i++) {
      if (list[i].id === diagram.id) { at = i; break; }
    }
    if (at >= 0) list[at] = sdClone(diagram);
    else list.push(sdClone(diagram));
    this._activeId = diagram.id;
    this.save();
    this.emit("change");
    return diagram;
  },

  remove: function (id, idx) {
    var key = this.slideKey(idx);
    var list = this.state.slides[key];
    if (!Array.isArray(list)) return;
    this.checkpoint();
    this.state.slides[key] = list.filter(function (diagram) { return diagram.id !== id; });
    if (this._activeId === id) this._activeId = null;
    this.save();
    this.emit("change");
  },

  setActive: function (id) {
    this._activeId = id || null;
    this.emit("active", this._activeId);
  },

  active: function () {
    var found = this._activeId ? this.get(this._activeId) : null;
    if (found) return found;
    var list = this.list();
    return list.length ? list[0] : null;
  },

  localBadge: function () {
    return sdEl("span", "sl-diag-local-badge",
      sdT("slides-diagrams-local-only", "Local only — not saved to the server"),
      "slides-diagrams-local-only");
  }
};
