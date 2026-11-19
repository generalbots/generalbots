"use strict";
/* Slides native-style chart authoring (#2486) — layer 1: deck-scoped local state.
 * Pure model + CRUD. DOM-free so an offline harness can exercise every
 * acceptance criterion. Nothing here touches the server. The genuine native
 * PPTX chart part, embedded workbook, vector PDF, PowerPoint round-trip and
 * real Sheet fetch are out of scope for this local layer. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var STORE = "gb-slides-charts:";

  C.TYPES = ["column", "bar", "column-stacked", "line", "area", "pie", "donut",
    "scatter", "bubble", "combo", "waterfall"];
  C.NUMBER_FORMATS = ["0", "0.0", "#,##0", "0%", "currency"];
  C.AXES = ["primary", "secondary"];

  C.tr = function (key, fallback, params) {
    var out = typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    if (params) {
      Object.keys(params).forEach(function (name) {
        out = String(out).split("{" + name + "}").join(String(params[name]))
          .split("{ $" + name + " }").join(String(params[name]));
      });
    }
    return out;
  };
  C.clone = function (value) { return JSON.parse(JSON.stringify(value)); };
  C.uid = function (prefix) {
    return (prefix || "chart") + "_" + Date.now().toString(36) + "_" +
      Math.random().toString(36).slice(2, 7);
  };
  C.deckId = function () {
    return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) ||
      window.__SLIDES_PRESENTATION_ID || "current";
  };
  C.storeKey = function () { return STORE + C.deckId(); };

  function fresh() { return { version: 1, charts: {}, order: [] }; }
  C.fresh = fresh;

  function storageOf(explicit) {
    if (explicit) return explicit;
    try { return window.localStorage || null; } catch (error) { return null; }
  }

  C.load = function (explicit) {
    var store = storageOf(explicit);
    if (!store) return fresh();
    try {
      var raw = store.getItem(C.storeKey());
      if (!raw) return fresh();
      var parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== "object" || !parsed.charts || typeof parsed.charts !== "object") {
        return fresh();
      }
      var base = fresh();
      base.charts = parsed.charts;
      base.order = Array.isArray(parsed.order) ? parsed.order.filter(function (id) {
        return Object.prototype.hasOwnProperty.call(base.charts, id);
      }) : Object.keys(parsed.charts);
      Object.keys(base.charts).forEach(function (id) {
        base.charts[id] = C.normalize(base.charts[id]);
        if (base.order.indexOf(id) < 0) base.order.push(id);
      });
      return base;
    } catch (error) {
      return fresh();
    }
  };

  C.save = function (state, explicit) {
    var store = storageOf(explicit);
    if (!store) return false;
    try {
      store.setItem(C.storeKey(), JSON.stringify(state || fresh()));
      return true;
    } catch (error) {
      return false;
    }
  };

  C.init = function (explicit) {
    if (!C.state) C.state = C.load(explicit);
    return C.state;
  };
  C.reset = function (explicit) { C.state = fresh(); C.save(C.state, explicit); return C.state; };

  C.patch = function (mutator, explicit) {
    var state = C.init(explicit);
    var result = mutator(state);
    C.save(state, explicit);
    return result;
  };

  function num(value, fallback) {
    var n = Number(value);
    return isFinite(n) ? n : fallback;
  }
  C.num = num;

  function strArray(value) {
    if (!Array.isArray(value)) return [];
    return value.map(function (item) { return item === null || item === undefined ? "" : String(item); });
  }

  /* The single normalization boundary. Every read, write and round-trip goes
   * through here so a re-opened definition compares equal to the original. */
  C.normalize = function (input) {
    var raw = input && typeof input === "object" ? input : {};
    var categories = strArray(raw.categories);
    var rawSeries = Array.isArray(raw.series) ? raw.series : [];
    var series = rawSeries.map(function (item, index) {
      var source = item && typeof item === "object" ? item : {};
      var values = Array.isArray(source.values) ? source.values.map(function (v) {
        return num(v, 0);
      }) : [];
      while (values.length < categories.length) values.push(0);
      return {
        id: source.id ? String(source.id) : "s" + (index + 1),
        name: source.name === undefined || source.name === null ? "" : String(source.name),
        values: values.slice(0, Math.max(1, categories.length)),
        color: source.color ? String(source.color) : "",
        axis: C.AXES.indexOf(source.axis) >= 0 ? source.axis : "primary"
      };
    });
    if (!series.length) {
      series = [{ id: "s1", name: "Series 1", values: categories.map(function () { return 0; }), color: "", axis: "primary" }];
    }
    var options = raw.options && typeof raw.options === "object" ? raw.options : {};
    var a11y = raw.a11y && typeof raw.a11y === "object" ? raw.a11y : {};
    var link = raw.link && typeof raw.link === "object" ? raw.link : {};
    var axisTitles = raw.axisTitles && typeof raw.axisTitles === "object" ? raw.axisTitles : {};
    return {
      id: raw.id ? String(raw.id) : "",
      slideIndex: num(raw.slideIndex, 0),
      type: C.TYPES.indexOf(raw.type) >= 0 ? raw.type : "column",
      categories: categories,
      series: series,
      title: raw.title === undefined || raw.title === null ? "" : String(raw.title),
      legend: raw.legend !== false,
      dataLabels: !!raw.dataLabels,
      gridlines: raw.gridlines !== false,
      numberFormat: C.NUMBER_FORMATS.indexOf(raw.numberFormat) >= 0 ? raw.numberFormat : "0",
      axisTitles: {
        x: axisTitles.x === undefined ? "" : String(axisTitles.x),
        y: axisTitles.y === undefined ? "" : String(axisTitles.y),
        y2: axisTitles.y2 === undefined ? "" : String(axisTitles.y2)
      },
      options: {
        stacked: !!options.stacked,
        holeSize: Math.min(0.85, Math.max(0.15, num(options.holeSize, 0.55))),
        trendline: !!options.trendline,
        errorBars: !!options.errorBars,
        comboSecondary: options.comboSecondary !== false,
        bubbleIndex: num(options.bubbleIndex, 2)
      },
      a11y: {
        altText: a11y.altText === undefined ? "" : String(a11y.altText),
        showDataTable: !!a11y.showDataTable,
        colorBlindSafe: !!a11y.colorBlindSafe,
        contrastCheck: a11y.contrastCheck !== false
      },
      link: {
        range: link.range === undefined ? "" : String(link.range),
        refreshOnOpen: !!link.refreshOnOpen,
        lastRefreshed: num(link.lastRefreshed, 0),
        stale: !!link.stale
      },
      createdAt: num(raw.createdAt, 0),
      updatedAt: num(raw.updatedAt, 0)
    };
  };

  C.list = function (state) {
    state = state || C.init();
    return state.order.map(function (id) { return state.charts[id]; }).filter(Boolean);
  };
  C.forSlide = function (index, state) {
    return C.list(state).filter(function (chart) { return chart.slideIndex === num(index, 0); });
  };
  C.get = function (id, state) {
    state = state || C.init();
    var found = state.charts[String(id)];
    return found ? C.normalize(found) : null;
  };

  C.create = function (patch, explicit) {
    var now = Date.now();
    var def = C.normalize(patch || {});
    def.id = def.id || C.uid("chart");
    def.createdAt = def.createdAt || now;
    def.updatedAt = now;
    return C.patch(function (state) {
      state.charts[def.id] = def;
      if (state.order.indexOf(def.id) < 0) state.order.push(def.id);
      return C.clone(def);
    }, explicit);
  };

  C.update = function (id, patch, explicit) {
    var existing = C.get(id);
    if (!existing) return null;
    var merged = C.clone(existing);
    Object.keys(patch || {}).forEach(function (key) {
      if (key === "id" || key === "createdAt") return;
      merged[key] = patch[key];
    });
    merged.id = existing.id;
    merged.createdAt = existing.createdAt;
    merged.updatedAt = Date.now();
    var def = C.normalize(merged);
    return C.patch(function (state) {
      state.charts[def.id] = def;
      return C.clone(def);
    }, explicit);
  };

  C.duplicate = function (id, explicit) {
    var copy = C.get(id);
    if (!copy) return null;
    copy.id = "";
    copy.title = copy.title;
    copy.createdAt = 0;
    copy.updatedAt = 0;
    return C.create(copy, explicit);
  };

  C.remove = function (id, explicit) {
    return C.patch(function (state) {
      if (!Object.prototype.hasOwnProperty.call(state.charts, id)) return false;
      delete state.charts[id];
      state.order = state.order.filter(function (item) { return item !== id; });
      return true;
    }, explicit);
  };

  /* Explicit definition round-trip: the same normalization used by load/save. */
  C.serialize = function (def) { return JSON.stringify(C.normalize(def)); };
  C.deserialize = function (json) {
    try { return C.normalize(JSON.parse(json)); } catch (error) { return null; }
  };

  /* Self-check requested by the issue: a combo chart with a secondary axis
   * must preserve its type and per-series axis mapping after a local re-open. */
  C.selfCheck = function () {
    var source = C.normalize({
      type: "combo",
      categories: ["Q1", "Q2", "Q3", "Q4"],
      series: [
        { id: "rev", name: "Revenue", values: [10, 20, 15, 30], axis: "primary" },
        { id: "margin", name: "Margin", values: [2, 4, 3, 6], axis: "secondary" }
      ],
      options: { comboSecondary: true }
    });
    var reopened = C.deserialize(C.serialize(source));
    return C.verifyRoundTrip(source, reopened);
  };

  C.verifyRoundTrip = function (source, reopened) {
    var checks = [];
    function check(name, ok) { checks.push({ name: name, ok: !!ok }); }
    check("type", !!reopened && reopened.type === source.type);
    check("series-count", !!reopened && reopened.series.length === source.series.length);
    if (reopened) {
      source.series.forEach(function (series, index) {
        var other = reopened.series[index];
        check("axis-" + series.id, !!other && other.axis === series.axis);
        check("values-" + series.id, !!other && JSON.stringify(other.values) === JSON.stringify(series.values));
        check("name-" + series.id, !!other && other.name === series.name);
      });
      check("categories", JSON.stringify(reopened.categories) === JSON.stringify(source.categories));
    }
    return {
      pass: checks.every(function (item) { return item.ok; }),
      checks: checks,
      failed: checks.filter(function (item) { return !item.ok; }).map(function (item) { return item.name; })
    };
  };
})(typeof window !== "undefined" ? window : globalThis);
