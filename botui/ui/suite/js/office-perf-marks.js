/* office-perf-marks.js — #2459 first-party performance marks for opening,
 * first content, saving and exporting office apps. Internal instrumentation
 * only: marks carry NO PII — no document content, no user/tenant/file ids.
 * Only a normalized app label (and export format) plus timing numbers are
 * recorded. Budgets are read from window.GBOfficePerf.report(). */
(function (global) {
  "use strict";

  if (!global || global.__GBOfficePerfLoaded === true) return;
  global.__GBOfficePerfLoaded = true;

  var MAX_AGE_MS = 5 * 60 * 1000;
  var NO_LABEL = "unknown";

  var perf = (typeof global.performance !== "undefined" && global.performance)
    ? global.performance : null;
  var canMark = !!(perf && typeof perf.mark === "function");
  var canMeasure = !!(perf && typeof perf.measure === "function");
  var canEntries = !!(perf && typeof perf.getEntriesByName === "function");
  var canClear = !!(perf && typeof perf.clearMeasures === "function");

  /* appId -> { updatedAt, marks: { stage: markName } }. Keys are normalized
   * app labels, never raw ids, so a per-document id can never leak into a mark. */
  var cycles = Object.create(null);
  var measureSeq = 0;

  function hasOwn(object, key) {
    return Object.prototype.hasOwnProperty.call(object, key);
  }

  /* Reduce an arbitrary id to a short, aggregate app label. Instance/record
   * suffixes (-/:) are dropped; only [a-z0-9] survives. */
  function label(value, maxLength) {
    if (typeof value !== "string" && typeof value !== "number") return NO_LABEL;
    var raw = String(value).split(/[-:]/, 1)[0].toLowerCase().replace(/[^a-z0-9]/g, "");
    if (!raw) return NO_LABEL;
    return raw.slice(0, maxLength || 32);
  }

  function sweep(now) {
    for (var key in cycles) {
      if (hasOwn(cycles, key) && now - cycles[key].updatedAt > MAX_AGE_MS) {
        delete cycles[key];
      }
    }
  }

  function cycleFor(appId, now) {
    var key = label(appId, 32);
    sweep(now);
    var cycle = cycles[key];
    if (!cycle) {
      cycle = { updatedAt: now, marks: Object.create(null) };
      cycles[key] = cycle;
    }
    cycle.updatedAt = now;
    return { key: key, cycle: cycle };
  }

  function addMark(appId, stage) {
    if (!canMark) return null;
    var now = Date.now();
    var entry = cycleFor(appId, now);
    if (stage === "open") entry.cycle.marks = Object.create(null);
    var name = "gb.office." + entry.key + "." + stage;
    try {
      perf.mark(name);
    } catch (e) {
      return null;
    }
    entry.cycle.marks[stage] = name;
    if (stage.indexOf("export.") === 0) entry.cycle.marks.export = name;
    return name;
  }

  /* Duration between two marks, via a transient performance.measure() whose
   * entry is read back and cleared. Returns null when the host lacks the API. */
  function duration(startName, endName) {
    if (!startName || !endName || !canMeasure || !canEntries) return null;
    var measureName = startName + "~" + endName + "~" + (measureSeq++);
    try {
      perf.measure(measureName, startName, endName);
      var entries = perf.getEntriesByName(measureName, "measure");
      if (!entries || !entries.length) return null;
      var value = entries[entries.length - 1].duration;
      return typeof value === "number" && isFinite(value) ? value : null;
    } catch (e) {
      return null;
    } finally {
      if (canClear) {
        try { perf.clearMeasures(measureName); } catch (ignored) { /* no-op */ }
      }
    }
  }

  function report() {
    var result = { noPii: true, apps: {} };
    if (!canMeasure || !canEntries) return result;
    sweep(Date.now());
    for (var app in cycles) {
      if (!hasOwn(cycles, app)) continue;
      var marks = cycles[app].marks;
      var open = marks.open || null;
      var first = marks["first-content"] || null;
      var save = marks.save || null;
      var exported = marks.export || null;
      result.apps[app] = {
        openToFirstContent: duration(open, first),
        firstContentToSave: duration(first, save),
        saveToExport: duration(save, exported),
        openToSave: duration(open, save),
        openToExport: duration(open, exported)
      };
    }
    return result;
  }

  var api = {
    markOpen: function (appId) { return addMark(appId, "open"); },
    markFirstContent: function (appId) { return addMark(appId, "first-content"); },
    markSave: function (appId) { return addMark(appId, "save"); },
    markExport: function (appId, format) {
      return addMark(appId, "export." + label(format, 16));
    },
    report: report
  };

  /* Optional auto-hook: these events are optional, so registering listeners is
   * harmless when nothing dispatches them. Listeners live on document, where
   * the suite dispatches gb:deep-link and app save/export events. */
  if (global.document && typeof global.document.addEventListener === "function") {
    global.document.addEventListener("gb:deep-link", function (event) {
      var detail = event && event.detail;
      if (detail && detail.appId) api.markOpen(detail.appId);
    });
    global.document.addEventListener("gb:office-saved", function (event) {
      var detail = event && event.detail;
      if (detail && detail.appId) api.markSave(detail.appId);
    });
    global.document.addEventListener("gb:office-exported", function (event) {
      var detail = event && event.detail;
      if (detail && detail.appId) api.markExport(detail.appId, detail.format);
    });
  }

  global.GBOfficePerf = api;
})(typeof window !== "undefined" ? window : null);
