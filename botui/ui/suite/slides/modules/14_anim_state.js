"use strict";
/* Slides animation/transition timeline — state, catalogs and local store (#2431).
 * Plain script, loaded after 01_state.js. All visible strings use saT().
 * Authored data lives in localStorage and is explicitly labelled local-only:
 * no server endpoint is claimed until the backend compile gate lands. */

function saT(key, fallback, params) {
  return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
}

(function (window) {
  var STORE_PREFIX = "slides:anim:v1:";
  var listeners = {};

  function deckId() {
    return (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current";
  }
  function slideIndex() {
    var cv = window.SlideCanvas && window.SlideCanvas.canvas;
    if (!cv) return 0;
    var n = parseInt(cv.dataset.slideId, 10);
    return isFinite(n) && n >= 0 ? n : 0;
  }
  function canvas() {
    return window.SlideCanvas && window.SlideCanvas.canvas ? window.SlideCanvas.canvas : null;
  }
  function elements() {
    var cv = canvas();
    return cv ? Array.prototype.slice.call(cv.querySelectorAll(".sl-element")) : [];
  }

  /* Effect library — descriptors keep id/category/fallback keyed for i18n. */
  var EFFECTS = [
    { id: "fade", cat: "entrance", key: "slides-anim-fx-fade", fallback: "Fade" },
    { id: "fly", cat: "entrance", key: "slides-anim-fx-fly", fallback: "Fly in" },
    { id: "wipe", cat: "entrance", key: "slides-anim-fx-wipe", fallback: "Wipe" },
    { id: "zoom", cat: "entrance", key: "slides-anim-fx-zoom", fallback: "Zoom" },
    { id: "pulse", cat: "emphasis", key: "slides-anim-fx-pulse", fallback: "Pulse" },
    { id: "spin", cat: "emphasis", key: "slides-anim-fx-spin", fallback: "Spin" },
    { id: "grow", cat: "emphasis", key: "slides-anim-fx-grow", fallback: "Grow and shrink" },
    { id: "fade-out", cat: "exit", key: "slides-anim-fx-fade", fallback: "Fade" },
    { id: "fly-out", cat: "exit", key: "slides-anim-fx-fly", fallback: "Fly in" },
    { id: "line", cat: "motion", key: "slides-anim-fx-line", fallback: "Line path" },
    { id: "curve", cat: "motion", key: "slides-anim-fx-curve", fallback: "Curve path" },
    { id: "custom", cat: "motion", key: "slides-anim-fx-custom", fallback: "Custom path" },
    { id: "paragraph", cat: "build", key: "slides-anim-fx-paragraph", fallback: "Build paragraphs" }
  ];
  var CATEGORIES = [
    { id: "entrance", key: "slides-anim-cat-entrance", fallback: "Entrance" },
    { id: "emphasis", key: "slides-anim-cat-emphasis", fallback: "Emphasis" },
    { id: "exit", key: "slides-anim-cat-exit", fallback: "Exit" },
    { id: "motion", key: "slides-anim-cat-motion", fallback: "Motion path" },
    { id: "build", key: "slides-anim-cat-build", fallback: "Paragraph build" }
  ];
  var TRANSITIONS = [
    { id: "none", key: "slides-trans-none", fallback: "None" },
    { id: "fade", key: "slides-trans-fade", fallback: "Fade" },
    { id: "wipe", key: "slides-trans-wipe", fallback: "Wipe" },
    { id: "push", key: "slides-trans-push", fallback: "Push" },
    { id: "cover", key: "slides-trans-cover", fallback: "Cover" },
    { id: "dissolve", key: "slides-trans-dissolve", fallback: "Dissolve" }
  ];

  function freshDeck() {
    return { reduceMotion: false, slides: {}, defaultTransition: defaultTransition() };
  }
  function defaultTransition() {
    return { type: "fade", duration: 500, direction: "left", easing: "ease", advance: "click", seconds: 5 };
  }
  function freshEffect(id, kind) {
    var def = effectDef(id);
    return {
      id: "fx_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 6),
      elementId: kind && kind.elementId ? kind.elementId : null,
      effect: id,
      category: def ? def.cat : "entrance",
      duration: def && def.cat === "motion" ? 800 : 500,
      delay: 0,
      easing: "ease",
      start: "click",
      direction: "left",
      path: { x1: 0, y1: 0, cx: 0.5, cy: 0, x2: 1, y2: 0 },
      levels: []
    };
  }
  function effectDef(id) {
    for (var i = 0; i < EFFECTS.length; i++) if (EFFECTS[i].id === id) return EFFECTS[i];
    return null;
  }
  function effectName(fx) {
    var def = effectDef(fx && fx.effect);
    return def ? saT(def.key, def.fallback) : (fx && fx.effect) || "";
  }
  function transitionName(id) {
    for (var i = 0; i < TRANSITIONS.length; i++) if (TRANSITIONS[i].id === id) return saT(TRANSITIONS[i].key, TRANSITIONS[i].fallback);
    return id || "";
  }

  function load() {
    var raw = null;
    try { raw = window.localStorage.getItem(STORE_PREFIX + deckId()); } catch (e) { raw = null; }
    if (!raw) return freshDeck();
    try {
      var parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== "object") return freshDeck();
      if (!parsed.slides || typeof parsed.slides !== "object") parsed.slides = {};
      if (!parsed.defaultTransition || typeof parsed.defaultTransition !== "object") parsed.defaultTransition = defaultTransition();
      parsed.reduceMotion = !!parsed.reduceMotion;
      return parsed;
    } catch (e) {
      return freshDeck();
    }
  }

  var state = load();

  function save() {
    try { window.localStorage.setItem(STORE_PREFIX + deckId(), JSON.stringify(state)); } catch (e) { /* storage may be disabled */ }
    emit("saved");
  }
  function slideData(idx, create) {
    var key = String(idx == null ? slideIndex() : idx);
    if (!state.slides[key] || typeof state.slides[key] !== "object") {
      if (!create) return { effects: [], transition: null };
      state.slides[key] = { effects: [], transition: null };
    }
    if (!Array.isArray(state.slides[key].effects)) state.slides[key].effects = [];
    return state.slides[key];
  }
  function effects(idx) { return slideData(idx, false).effects; }
  function setEffects(list, idx) { slideData(idx, true).effects = Array.isArray(list) ? list : []; save(); emit("effects"); }
  function addEffect(id, elementId, idx) {
    var fx = freshEffect(id, { elementId: elementId });
    var data = slideData(idx, true);
    data.effects.push(fx);
    save(); emit("effects");
    return fx;
  }
  function updateEffect(effectId, patch, idx) {
    var list = effects(idx);
    for (var i = 0; i < list.length; i++) {
      if (list[i].id === effectId) {
        for (var k in patch) if (Object.prototype.hasOwnProperty.call(patch, k)) list[i][k] = patch[k];
        save(); emit("effects");
        return list[i];
      }
    }
    return null;
  }
  function removeEffect(effectId, idx) {
    var list = effects(idx);
    var out = [];
    for (var i = 0; i < list.length; i++) if (list[i].id !== effectId) out.push(list[i]);
    setEffects(out, idx);
  }
  function moveEffect(effectId, delta, idx) {
    var list = effects(idx).slice();
    var from = -1;
    for (var i = 0; i < list.length; i++) if (list[i].id === effectId) { from = i; break; }
    if (from < 0) return false;
    var to = Math.max(0, Math.min(list.length - 1, from + delta));
    if (to === from) return false;
    var moved = list.splice(from, 1)[0];
    list.splice(to, 0, moved);
    setEffects(list, idx);
    return true;
  }
  function transition(idx, inherit) {
    var own = slideData(idx, false).transition;
    if (own && typeof own === "object") return own;
    return inherit === false ? null : state.defaultTransition;
  }
  function setTransition(cfg, idx, all) {
    var copy = {};
    for (var k in cfg) if (Object.prototype.hasOwnProperty.call(cfg, k)) copy[k] = cfg[k];
    if (all) {
      state.defaultTransition = copy;
      state.slides = {};
    } else {
      slideData(idx, true).transition = copy;
    }
    save(); emit("transition");
  }
  function setReduceMotion(value) { state.reduceMotion = !!value; save(); emit("reduce"); }
  function reduced() {
    try {
      var mq = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)");
      return !!state.reduceMotion || !!(mq && mq.matches);
    } catch (e) { return !!state.reduceMotion; }
  }

  function on(evt, fn) {
    if (!listeners[evt]) listeners[evt] = [];
    listeners[evt].push(fn);
  }
  function emit(evt) {
    var args = Array.prototype.slice.call(arguments, 1);
    var fns = listeners[evt] || [];
    for (var i = 0; i < fns.length; i++) {
      try { fns[i].apply(null, args); } catch (e) { console.warn("[gb-slides-anim] listener failed", e); }
    }
  }

  window.SlidesAnimation = {
    saT: saT,
    deckId: deckId,
    slideIndex: slideIndex,
    canvas: canvas,
    elements: elements,
    EFFECTS: EFFECTS,
    CATEGORIES: CATEGORIES,
    TRANSITIONS: TRANSITIONS,
    effectDef: effectDef,
    effectName: effectName,
    transitionName: transitionName,
    freshEffect: freshEffect,
    defaultTransition: defaultTransition,
    load: load,
    save: save,
    slideData: slideData,
    effects: effects,
    setEffects: setEffects,
    addEffect: addEffect,
    updateEffect: updateEffect,
    removeEffect: removeEffect,
    moveEffect: moveEffect,
    transition: transition,
    setTransition: setTransition,
    setReduceMotion: setReduceMotion,
    reduced: reduced,
    on: on,
    emit: emit,
    state: state
  };
})(window);
