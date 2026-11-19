"use strict";
/* Slides masters (#2429) — layer 2: the three-level cascade
 * (master theme -> layout -> slide override), apply-to-selection/all with a
 * before/after diff, brand-lock checks and versioned template library.
 * Pure functions take `state` explicitly so they are harness-testable. */
(function (window) {
  var M = (window.GBSlidesMasters = window.GBSlidesMasters || {});
  var has = function (obj, key) { return !!obj && Object.prototype.hasOwnProperty.call(obj, key); };
  var eq = function (a, b) { return a === b; };

  M.themeFor = function (state, slide) {
    var id = (slide && slide.themeId) || state.master.defaultThemeId;
    return state.themes[id] || null;
  };
  M.layoutFor = function (state, slide) {
    var id = slide && slide.layoutId;
    var found = null;
    state.layouts.forEach(function (l) { if (l.id === id) found = l; });
    return found;
  };

  /* Resolve one token for one slide. Returns the value and the level it came
   * from. `explicit` is true only for a per-slide override. */
  M.resolve = function (state, slideKey, key) {
    var slide = state.slides[slideKey] || {};
    if (has(slide.overrides, key)) {
      return { value: slide.overrides[key], source: "slide", explicit: true };
    }
    var layout = M.layoutFor(state, slide);
    if (layout && has(layout.themeOverrides, key)) {
      return { value: layout.themeOverrides[key], source: "layout", explicit: false };
    }
    var theme = M.themeFor(state, slide);
    if (theme && has(theme.flat, key)) {
      return { value: theme.flat[key], source: "master", explicit: false };
    }
    return { value: null, source: "none", explicit: false };
  };

  M.effectiveStyle = function (state, slideKey) {
    var out = {};
    M.TOKEN_KEYS.forEach(function (key) { out[key] = M.resolve(state, slideKey, key); });
    return out;
  };

  M.setOverride = function (slideKey, key, value) {
    return M.mutate(function (state) {
      var s = M.slide(slideKey);
      s.overrides[key] = value;
      return s;
    });
  };
  M.clearOverride = function (slideKey, key) {
    return M.mutate(function (state) {
      var s = M.slide(slideKey);
      delete s.overrides[key];
      return s;
    });
  };

  /* Preview which slides and tokens would change, without mutating state. */
  M.diffApply = function (state, options) {
    options = options || {};
    var keys = options.slideKeys && options.slideKeys.length
      ? options.slideKeys.map(String)
      : Object.keys(state.slides);
    var probe = M.clone(state);
    keys.forEach(function (key) {
      var s = probe.slides[key] || (probe.slides[key] = { themeId: null, layoutId: null, overrides: {} });
      if (options.themeId) s.themeId = options.themeId;
      if (options.layoutId) s.layoutId = options.layoutId;
    });
    var rows = [];
    keys.forEach(function (key) {
      var changes = [];
      M.TOKEN_KEYS.forEach(function (token) {
        var before = M.resolve(state, key, token);
        var after = M.resolve(probe, key, token);
        if (!eq(before.value, after.value)) {
          changes.push({ key: token, before: before.value, after: after.value, source: after.explicit ? "slide" : after.source });
        }
      });
      if (options.layoutId) {
        var beforeLayout = M.layoutFor(state, state.slides[key]);
        var afterLayout = M.layoutFor(probe, probe.slides[key]);
        var bName = beforeLayout ? beforeLayout.name : "";
        var aName = afterLayout ? afterLayout.name : "";
        if (bName !== aName) changes.push({ key: "layout", before: bName, after: aName, source: "layout" });
      }
      if (changes.length) rows.push({ slideKey: key, changes: changes });
    });
    return rows;
  };

  /* Applying a theme switches the slide's theme reference. Overrides are a
   * higher cascade tier, so explicitly coloured elements are never touched. */
  M.applyTheme = function (options) {
    var diff = M.diffApply(M.init(), { slideKeys: options.slideKeys, themeId: options.themeId });
    M.mutate(function (state) {
      var keys = options.slideKeys && options.slideKeys.length
        ? options.slideKeys : Object.keys(state.slides);
      keys.forEach(function (key) { M.slide(key).themeId = options.themeId; });
    });
    return diff;
  };
  M.applyLayout = function (options) {
    var diff = M.diffApply(M.init(), { slideKeys: options.slideKeys, layoutId: options.layoutId });
    M.mutate(function (state) {
      var keys = options.slideKeys && options.slideKeys.length
        ? options.slideKeys : Object.keys(state.slides);
      keys.forEach(function (key) { M.slide(key).layoutId = options.layoutId; });
    });
    return diff;
  };

  /* ---- layout editor --------------------------------------------------- */
  M.addLayout = function (name) {
    return M.mutate(function (state) {
      var item = {
        id: M.uid("layout"), name: name, background: "#0f172a", footer: "",
        number: false, themeOverrides: {}, placeholders: {}
      };
      state.layouts.push(item);
      return item;
    });
  };
  M.duplicateLayout = function (id) {
    return M.mutate(function (state) {
      var source = null;
      state.layouts.forEach(function (l) { if (l.id === id) source = l; });
      if (!source) return null;
      var copy = M.clone(source);
      copy.id = M.uid("layout");
      copy.name = source.name + " copy";
      state.layouts.push(copy);
      return copy;
    });
  };
  M.renameLayout = function (id, name) {
    return M.mutate(function (state) {
      state.layouts.forEach(function (l) { if (l.id === id) l.name = name; });
    });
  };
  M.deleteLayout = function (id) {
    return M.mutate(function (state) {
      if (state.layouts.length <= 1) return false;
      state.layouts = state.layouts.filter(function (l) { return l.id !== id; });
      Object.keys(state.slides).forEach(function (key) {
        if (state.slides[key].layoutId === id) state.slides[key].layoutId = null;
      });
      return true;
    });
  };
  M.setLayoutField = function (id, field, value) {
    return M.mutate(function (state) {
      state.layouts.forEach(function (l) { if (l.id === id) l[field] = value; });
    });
  };
  M.setPlaceholder = function (id, role, geometry) {
    return M.mutate(function (state) {
      state.layouts.forEach(function (l) {
        if (l.id === id) l.placeholders[role] = M.clone(geometry);
      });
    });
  };

  /* ---- brand kit ------------------------------------------------------- */
  M.fontAllowed = function (brand, font) {
    if (!font) return true;
    return (brand.fonts || []).some(function (f) {
      return String(f).toLowerCase() === String(font).toLowerCase();
    });
  };
  M.colorAllowed = function (brand, color) {
    if (!color) return true;
    var needle = String(color).toLowerCase();
    return (brand.palette || []).some(function (c) { return String(c).toLowerCase() === needle; });
  };
  M.checkInsert = function (state, input) {
    input = input || {};
    var brand = state.brand || {};
    if (!brand.locked) return { allowed: true, mode: "none", reasons: [] };
    var reasons = [];
    if (input.font && !M.fontAllowed(brand, input.font)) {
      reasons.push({ kind: "font", value: input.font });
    }
    if (input.color && !M.colorAllowed(brand, input.color)) {
      reasons.push({ kind: "color", value: input.color });
    }
    return { allowed: reasons.length === 0 || brand.mode !== "block", mode: brand.mode, reasons: reasons };
  };
  M.saveBrand = function (brand) { return M.mutate(function (state) { state.brand = M.clone(brand); }); };

  /* ---- versioned template library ------------------------------------- */
  M.saveTemplate = function (name) {
    return M.mutate(function (state) {
      var savedAt = new Date().toISOString();
      var entry = {
        name: name, savedAt: savedAt, versionId: name + "@" + savedAt,
        snapshot: M.clone({
          master: state.master, themes: state.themes, layouts: state.layouts, brand: state.brand
        })
      };
      state.templates.push(entry);
      return entry;
    });
  };
  M.deleteTemplate = function (versionId) {
    return M.mutate(function (state) {
      state.templates = state.templates.filter(function (t) { return t.versionId !== versionId; });
    });
  };
  M.restoreTemplate = function (versionId) {
    return M.mutate(function (state) {
      var found = null;
      state.templates.forEach(function (t) { if (t.versionId === versionId) found = t; });
      if (!found) return false;
      var snap = M.clone(found.snapshot);
      state.master = snap.master; state.themes = snap.themes;
      state.layouts = snap.layouts; state.brand = snap.brand;
      return true;
    });
  };
})(typeof window !== "undefined" ? window : globalThis);
