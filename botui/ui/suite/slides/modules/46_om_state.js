"use strict";
/* Slides canvas object model — local-only state, selection, geometry and undo
 * (#2430). Follows the diagrams/outline pattern: deck-scoped localStorage, one
 * full-state checkpoint per mutation, nothing written to the server. This file
 * is DOM-free so the logic can be exercised by an offline harness. */

(function () {
  var STORE = "slides:object-model:v1:";
  var LIMIT = 80;
  var PRESETS = [
    { id: "16:9", w: 960, h: 540 },
    { id: "4:3", w: 720, h: 540 },
    { id: "a4", w: 794, h: 1123 }
  ];
  var DEFAULT_SETTINGS = {
    rulers: true, grid: false, snap: true, smartGuides: true, gridSize: 10
  };

  function t(key, fallback, params) {
    var out = typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    if (params) {
      Object.keys(params).forEach(function (name) {
        out = String(out).split("{" + name + "}").join(params[name])
          .split("{ $" + name + " }").join(params[name]);
      });
    }
    return out;
  }
  function clone(value) { return JSON.parse(JSON.stringify(value)); }
  function uid(prefix) {
    return prefix + "_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 7);
  }
  function num(value, fallback) {
    var n = parseFloat(value);
    return isFinite(n) ? n : fallback;
  }

  var OM = {
    state: null,
    selection: [],
    undoStack: [],
    redoStack: [],
    listeners: {},
    _lastLabel: "",

    presets: function () { return clone(PRESETS); },
    preset: function (id) {
      var found = null;
      PRESETS.forEach(function (p) { if (p.id === id) found = p; });
      return found ? clone(found) : null;
    },
    deckId: function () {
      return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) ||
        window.__SLIDES_PRESENTATION_ID || "current";
    },
    slideIndex: function () {
      var canvas = (window.SlideCanvas && window.SlideCanvas.canvas) ||
        (typeof document !== "undefined" && document.getElementById("slide-canvas"));
      if (!canvas) return 0;
      var raw = canvas.dataset ? canvas.dataset.slideId : canvas.getAttribute("data-slide-id");
      var n = parseInt(raw, 10);
      return isFinite(n) && n >= 0 ? n : 0;
    },
    fresh: function () {
      return {
        version: 1,
        settings: clone(DEFAULT_SETTINGS),
        slides: {}
      };
    },
    load: function () {
      var raw = null;
      try { raw = window.localStorage.getItem(STORE + this.deckId()); } catch (e) { raw = null; }
      if (!raw) return this.fresh();
      try {
        var parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== "object" || !parsed.slides || typeof parsed.slides !== "object") {
          return this.fresh();
        }
        if (!parsed.settings) parsed.settings = clone(DEFAULT_SETTINGS);
        return parsed;
      } catch (e) { return this.fresh(); }
    },
    init: function () {
      if (!this.state) this.state = this.load();
      return this.state;
    },
    save: function () {
      try {
        window.localStorage.setItem(STORE + this.deckId(), JSON.stringify(this.state));
      } catch (e) { /* storage may be disabled */ }
      this.emit("change");
    },
    on: function (name, fn) {
      this.listeners[name] = this.listeners[name] || [];
      this.listeners[name].push(fn);
    },
    emit: function (name, detail) {
      (this.listeners[name] || []).forEach(function (fn) { fn(detail); });
    },
    slideKey: function (idx) { return String(idx == null ? this.slideIndex() : idx); },
    slide: function (idx) {
      this.init();
      var key = this.slideKey(idx);
      var s = this.state.slides[key];
      if (!s) {
        var size = this.preset("16:9");
        s = { size: size, objects: [], groups: [], nextZ: 1, guides: [] };
        this.state.slides[key] = s;
      }
      if (!Array.isArray(s.objects)) s.objects = [];
      if (!Array.isArray(s.groups)) s.groups = [];
      if (!Array.isArray(s.guides)) s.guides = [];
      return s;
    },

    /* ---- checkpoints: one undo step per operation ----------------------- */
    checkpoint: function (label) {
      this.init();
      this.undoStack.push({ label: label || "", snapshot: clone(this.state) });
      if (this.undoStack.length > LIMIT) this.undoStack.shift();
      this.redoStack.length = 0;
      this._lastLabel = label || "";
    },
    mutate: function (label, fn) {
      this.checkpoint(label);
      var result = fn(this.state);
      this.save();
      return result;
    },
    replaceState: function (next, label) {
      this.checkpoint(label);
      this.state = next;
      this.save();
    },
    undo: function () {
      if (!this.undoStack.length) return false;
      var entry = this.undoStack.pop();
      this.redoStack.push({ label: entry.label, snapshot: clone(this.state) });
      this.state = entry.snapshot;
      this.selection = this.selection.filter(this._findIn(this.state));
      this.save();
      this.emit("undo", entry.label);
      return true;
    },
    redo: function () {
      if (!this.redoStack.length) return false;
      var entry = this.redoStack.pop();
      this.undoStack.push({ label: entry.label, snapshot: clone(this.state) });
      this.state = entry.snapshot;
      this.selection = this.selection.filter(this._findIn(this.state));
      this.save();
      this.emit("redo", entry.label);
      return true;
    },
    canUndo: function () { return this.undoStack.length > 0; },
    canRedo: function () { return this.redoStack.length > 0; },
    lastLabel: function () { return this._lastLabel; },
    _findIn: function (state) {
      return function (id) {
        var found = false;
        Object.keys(state.slides).forEach(function (key) {
          var s = state.slides[key];
          (s.objects || []).forEach(function (o) { if (o.id === id) found = true; });
          (s.groups || []).forEach(function (g) { if (g.id === id) found = true; });
        });
        return found;
      };
    },

    /* ---- selection ------------------------------------------------------ */
    clearSelection: function () { this.selection = []; this.emit("selection", []); },
    setSelection: function (ids) {
      this.selection = (ids || []).slice();
      this.emit("selection", this.selection.slice());
    },
    toggle: function (id) {
      if (!id) return;
      var at = this.selection.indexOf(id);
      if (at >= 0) this.selection.splice(at, 1);
      else this.selection.push(id);
      this.emit("selection", this.selection.slice());
    },
    add: function (id) {
      if (id && this.selection.indexOf(id) < 0) {
        this.selection.push(id);
        this.emit("selection", this.selection.slice());
      }
    },
    isSelected: function (id) { return this.selection.indexOf(id) >= 0; },
    selectAll: function (idx) {
      this.setSelection(this.topLevel(idx).map(function (node) { return node.id; }));
    },
    selected: function (idx) {
      var s = this.slide(idx);
      var self = this;
      return this.selection.map(function (id) {
        return s.objects.find(function (o) { return o.id === id; }) ||
          s.groups.find(function (g) { return g.id === id; }) ||
          (self._fake(id));
      }).filter(Boolean);
    },

    /* ---- hierarchy + geometry ------------------------------------------ */
    groupOf: function (obj) { return obj && obj.groupId ? obj.groupId : (obj && obj.parentId) || null; },
    childObjects: function (s, groupId) {
      return (s.objects || []).filter(function (o) {
        return groupId ? o.groupId === groupId : !o.groupId;
      });
    },
    childGroups: function (s, parentId) {
      return (s.groups || []).filter(function (g) {
        return parentId ? g.parentId === parentId : !g.parentId;
      });
    },
    descendants: function (s, id) {
      var out = [];
      var self = this;
      (s.objects || []).forEach(function (o) { if (o.groupId === id) out.push(o); });
      (s.groups || []).forEach(function (g) {
        if (g.parentId === id) {
          out.push(g);
          self.descendants(s, g.id).forEach(function (child) { out.push(child); });
        }
      });
      return out;
    },
    topLevel: function (idx) {
      var s = this.slide(idx);
      return this.childGroups(s, null).concat(this.childObjects(s, null))
        .sort(function (a, b) { return num(a.z, 0) - num(b.z, 0); });
    },
    rect: function (s, id) {
      var obj = (s.objects || []).find(function (o) { return o.id === id; });
      if (obj) return { x: num(obj.x, 0), y: num(obj.y, 0), w: num(obj.w, 0), h: num(obj.h, 0) };
      var group = (s.groups || []).find(function (g) { return g.id === id; });
      if (!group) return null;
      var parts = this.descendants(s, id).filter(function (n) { return n.x !== undefined; })
        .map(function (n) { return { x: num(n.x, 0), y: num(n.y, 0), w: num(n.w, 0), h: num(n.h, 0) }; });
      if (!parts.length) return { x: 0, y: 0, w: 0, h: 0 };
      return this.union(parts);
    },
    union: function (rects) {
      if (!rects || !rects.length) return null;
      var left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity;
      rects.forEach(function (r) {
        if (!r) return;
        left = Math.min(left, r.x); top = Math.min(top, r.y);
        right = Math.max(right, r.x + r.w); bottom = Math.max(bottom, r.y + r.h);
      });
      if (!isFinite(left)) return null;
      return { x: left, y: top, w: right - left, h: bottom - top };
    },
    bounds: function (ids, idx) {
      var s = this.slide(idx);
      var self = this;
      return this.union((ids || []).map(function (id) { return self.rect(s, id); }));
    },

    /* Move a set of top-level nodes (a group moves all of its descendants).
     * Connectors are recomputed afterwards so both endpoints follow. */
    move: function (s, ids, dx, dy) {
      var self = this;
      (ids || []).forEach(function (id) {
        var obj = (s.objects || []).find(function (o) { return o.id === id; });
        if (obj && obj.type !== "connector") {
          if (!obj.locked) { obj.x = num(obj.x, 0) + dx; obj.y = num(obj.y, 0) + dy; }
          return;
        }
        var group = (s.groups || []).find(function (g) { return g.id === id; });
        if (group) {
          self.descendants(s, id).forEach(function (node) {
            if (node.x === undefined || node.locked) return;
            node.x = num(node.x, 0) + dx;
            node.y = num(node.y, 0) + dy;
          });
        }
      });
      this.updateConnectors(s);
    },
    anchorPoint: function (s, ref) {
      if (!ref || !ref.id) return null;
      var r = this.rect(s, ref.id);
      if (!r) return null;
      var x = r.x + r.w / 2, y = r.y + r.h / 2;
      if (ref.anchor === "left") { x = r.x; }
      else if (ref.anchor === "right") { x = r.x + r.w; }
      else if (ref.anchor === "top") { y = r.y; }
      else if (ref.anchor === "bottom") { y = r.y + r.h; }
      return { x: x, y: y };
    },
    updateConnectors: function (s) {
      var self = this;
      (s.objects || []).forEach(function (o) {
        if (o.type !== "connector" || !o.from || !o.to) return;
        var a = self.anchorPoint(s, o.from);
        var b = self.anchorPoint(s, o.to);
        if (!a || !b) return;
        o.x = Math.min(a.x, b.x); o.y = Math.min(a.y, b.y);
        o.w = Math.abs(b.x - a.x); o.h = Math.abs(b.y - a.y);
        o._a = a; o._b = b;
      });
    },

    /* ---- slide size ----------------------------------------------------- */
    rescale: function (s, from, to) {
      var sx = num(to.w, 960) / Math.max(1, num(from.w, 960));
      var sy = num(to.h, 540) / Math.max(1, num(from.h, 540));
      var self = this;
      (s.objects || []).forEach(function (o) {
        o.x = num(o.x, 0) * sx; o.y = num(o.y, 0) * sy;
        o.w = num(o.w, 0) * sx; o.h = num(o.h, 0) * sy;
        if (o.crop && o.crop.original) {
          var oc = o.crop.original; oc.x *= sx; oc.y *= sy; oc.w *= sx; oc.h *= sy;
        }
        if (Array.isArray(o.points)) {
          o.points.forEach(function (p) { p.x *= sx; p.y *= sy; });
        }
      });
      (s.guides || []).forEach(function (g) {
        if (g.axis === "x") g.at *= sx; else g.at *= sy;
      });
      s.size = clone(to);
      this.updateConnectors(s);
      return { sx: sx, sy: sy };
    },

    _fake: function () { return null; },
    t: t,
    clone: clone,
    uid: uid,
    num: num,
    settings: function () { this.init(); return this.state.settings; }
  };

  window.GBSlidesOM = OM;
}());
