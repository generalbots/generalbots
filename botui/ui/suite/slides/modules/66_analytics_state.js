"use strict";
/* Slides presentation analytics and audience interaction (#2489) — layer 1.
 * DOM-free, deck-scoped localStorage state plus deterministic simulated event
 * fixtures. Every record is local: no server telemetry, live transport or
 * real multi-device sync is claimed anywhere in this layer. */
(function (window) {
  var A = (window.GBSlidesAnalytics = window.GBSlidesAnalytics || {});
  var STORE = "gb-slides-analytics:";
  var SLIDES = 8;
  var BASE_TS = 1768473600000;

  A.SLIDE_COUNT = SLIDES;
  A.BASE_TS = BASE_TS;
  A.DEVICE_BUCKETS = ["desktop", "mobile", "tablet"];
  A.GEO_BUCKETS = ["Africa", "Asia", "Europe", "North America", "Oceania", "South America"];
  A.RESYNC_BUDGET_MS = 2000;
  A.POLL_RESPONSE_BUDGET_MS = 2000;
  A.MAX_SLIDE_DELTA = 1;

  A.tr = function (key, fallback, params) {
    var out = typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    if (params) {
      Object.keys(params).forEach(function (name) {
        out = String(out).split("{" + name + "}").join(String(params[name]))
          .split("{ $" + name + " }").join(String(params[name]));
      });
    }
    return out;
  };
  A.clone = function (value) { return JSON.parse(JSON.stringify(value)); };
  A.num = function (value, fallback) {
    var n = Number(value);
    return isFinite(n) ? n : fallback;
  };
  A.uid = function (prefix) {
    return (prefix || "sa") + "_" + Date.now().toString(36) + "_" +
      Math.random().toString(36).slice(2, 7);
  };
  A.pseudonym = function (id) {
    var text = String(id || "viewer");
    var hash = 0;
    for (var i = 0; i < text.length; i += 1) hash = (hash * 31 + text.charCodeAt(i)) % 1000003;
    return "anon-" + hash.toString(36);
  };
  A.deckId = function () {
    return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) ||
      window.__SLIDES_PRESENTATION_ID || "current";
  };
  A.storeKey = function () { return STORE + A.deckId(); };

  function fresh() {
    return {
      version: 1,
      slideCount: SLIDES,
      options: { anonymise: true, retentionDays: 90 },
      viewers: [],
      events: [],
      questions: [],
      polls: [],
      follow: { presenterSlide: 0, devices: [] },
      console: { startedAt: BASE_TS }
    };
  }
  A.fresh = fresh;

  function storageOf(explicit) {
    if (explicit) return explicit;
    try { return window.localStorage || null; } catch (error) { return null; }
  }

  A.load = function (explicit) {
    var store = storageOf(explicit);
    if (!store) return fresh();
    try {
      var raw = store.getItem(A.storeKey());
      if (!raw) return fresh();
      var parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== "object") return fresh();
      var base = fresh();
      ["viewers", "events", "questions", "polls"].forEach(function (field) {
        if (Array.isArray(parsed[field])) base[field] = parsed[field];
      });
      if (parsed.options && typeof parsed.options === "object") {
        base.options.anonymise = !!parsed.options.anonymise;
        base.options.retentionDays = A.num(parsed.options.retentionDays, 90);
      }
      if (parsed.follow && typeof parsed.follow === "object") {
        base.follow.presenterSlide = A.num(parsed.follow.presenterSlide, 0);
        if (Array.isArray(parsed.follow.devices)) base.follow.devices = parsed.follow.devices;
      }
      if (parsed.console && typeof parsed.console === "object") {
        base.console.startedAt = A.num(parsed.console.startedAt, BASE_TS);
      }
      base.slideCount = A.num(parsed.slideCount, SLIDES);
      return base;
    } catch (error) {
      return fresh();
    }
  };

  A.save = function (state, explicit) {
    var store = storageOf(explicit);
    if (!store) return false;
    try {
      store.setItem(A.storeKey(), JSON.stringify(state || fresh()));
      return true;
    } catch (error) {
      return false;
    }
  };

  A.init = function (explicit) {
    if (!A.state) A.state = A.load(explicit);
    return A.state;
  };

  A.patch = function (mutator, explicit) {
    var state = A.init(explicit);
    var result = mutator(state);
    A.save(state, explicit);
    return result;
  };

  A.bucketDevice = function (raw) {
    var text = String(raw || "").toLowerCase();
    if (text.indexOf("ipad") >= 0 || text.indexOf("tablet") >= 0) return "tablet";
    if (text.indexOf("iphone") >= 0 || text.indexOf("pixel") >= 0 ||
      text.indexOf("samsung") >= 0 || text.indexOf("android") >= 0) return "mobile";
    return "desktop";
  };

  A.bucketGeo = function (raw) {
    var text = String(raw || "");
    if (text.indexOf("Brazil") >= 0) return "South America";
    if (text.indexOf("Mexico") >= 0 || text.indexOf("New York") >= 0) return "North America";
    if (text.indexOf("Berlin") >= 0 || text.indexOf("Lisbon") >= 0) return "Europe";
    if (text.indexOf("Tokyo") >= 0) return "Asia";
    if (text.indexOf("Sydney") >= 0) return "Oceania";
    return "Africa";
  };

  /* Deterministic fixture: 20 viewers, 23 opens (three revisits), per-slide
   * enter/dwell/leave events and a fixed completion/drop-off distribution. */
  A.seed = function (state, count) {
    var total = Math.max(1, A.num(count, 20));
    var viewers = [];
    var events = [];
    var devices = ["iPhone 15 Pro", "Pixel 9", "MacBook Pro 16", "iPad Air",
      "Dell Latitude", "Samsung S25"];
    var places = ["São Paulo, Brazil", "Mexico City, Mexico", "Berlin, Germany",
      "Lisbon, Portugal", "Tokyo, Japan", "New York, USA"];
    for (var i = 0; i < total; i += 1) {
      var reached = i % 4 === 0 ? 4 : (i % 5 === 0 ? 6 : SLIDES);
      var viewer = {
        id: "v-" + (i + 1),
        email: "viewer" + (i + 1) + "@example.test",
        name: "Attendee " + (i + 1),
        deviceRaw: devices[i % devices.length],
        geoRaw: places[i % places.length],
        slidesSeen: reached,
        dwellMs: 0,
        completed: reached === SLIDES,
        firstView: 0,
        lastView: 0
      };
      var openedAt = BASE_TS + i * 60000;
      events.push({ type: "open", viewerId: viewer.id, ts: openedAt, slide: 0, duration: 0 });
      viewer.firstView = viewer.lastView = openedAt;
      for (var j = 0; j < reached; j += 1) {
        var enteredAt = openedAt + 5000 + j * 45000;
        var dwell = 15000 + ((i + j) * 7 % 20) * 1000;
        events.push({ type: "enter", viewerId: viewer.id, ts: enteredAt, slide: j, duration: 0 });
        events.push({ type: "dwell", viewerId: viewer.id, ts: enteredAt + dwell,
          slide: j, duration: dwell });
        if (j === reached - 1) {
          events.push({ type: "leave", viewerId: viewer.id, ts: enteredAt + dwell,
            slide: j, duration: 0 });
        }
        viewer.dwellMs += dwell;
        viewer.lastView = enteredAt + dwell;
      }
      if (i < 3) {
        var revisit = BASE_TS + (total + i) * 60000;
        events.push({ type: "open", viewerId: viewer.id, ts: revisit, slide: 0, duration: 0 });
        viewer.lastView = revisit;
      }
      viewers.push(viewer);
    }
    state.viewers = viewers;
    state.events = events;
    state.slideCount = SLIDES;
    return state;
  };

  A.formatTime = function (ms) {
    var seconds = Math.max(0, Math.floor(A.num(ms, 0) / 1000));
    var minutes = Math.floor(seconds / 60);
    var rest = seconds % 60;
    return minutes + ":" + (rest < 10 ? "0" : "") + rest;
  };

  A.dateLabel = function (ts) {
    var date = new Date(A.num(ts, 0));
    if (isNaN(date.getTime())) return "";
    return date.toISOString().replace("T", " ").slice(0, 19) + " UTC";
  };
})(typeof window !== "undefined" ? window : globalThis);
