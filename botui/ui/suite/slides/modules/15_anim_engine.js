"use strict";
/* Slides animation/transition playback engine (#2431).
 * Frame application is deterministic (inline styles) so final states are
 * always reachable; Web Animations is an optional enhancement only. Every
 * effect is wrapped in try/catch — one broken animation never blocks the deck. */

(function (window) {
  var A = window.SlidesAnimation;
  if (!A) return;
  var baseStyles = new WeakMap();
  var raf = null;
  var playing = false;
  var currentList = [];
  var currentTotal = 0;
  var lastTime = 0;
  var failedIds = [];

  function status(message, key) {
    var node = document.getElementById("sl-anim-status") || document.getElementById("sl-trans-status");
    if (node) node.textContent = message;
    A.emit("status", { message: message, key: key || "" });
  }
  function t(key, fallback, params) { return A.saT(key, fallback, params); }

  /* Resolve relative start modes into absolute times. Click starts a new
   * group; `with` shares the previous start; `after` follows the end. */
  function timeline(list) {
    var out = [];
    var groupEnd = 0;
    var prevStart = 0;
    var prevEnd = 0;
    for (var i = 0; i < list.length; i++) {
      var fx = list[i] || {};
      var dur = Math.max(0, Number(fx.duration) || 0);
      var delay = Math.max(0, Number(fx.delay) || 0);
      var start;
      if (fx.start === "with") start = prevStart + delay;
      else if (fx.start === "after") start = prevEnd + delay;
      else start = groupEnd + delay;
      var end = start + dur;
      out.push({ effect: fx, start: start, end: end, duration: dur });
      prevStart = start;
      prevEnd = end;
      if (fx.start === "click" || i === 0) groupEnd = end;
      else groupEnd = Math.max(groupEnd, end);
    }
    return out;
  }
  function totalOf(list) {
    var items = timeline(list);
    var max = 0;
    for (var i = 0; i < items.length; i++) max = Math.max(max, items[i].end);
    return max;
  }
  function elementFor(fx) {
    if (!fx || !fx.elementId) return null;
    var cv = A.canvas();
    if (!cv) return null;
    var found = cv.querySelector('[data-id="' + String(fx.elementId).replace(/"/g, "") + '"]');
    return found || null;
  }
  function remember(el) {
    if (!baseStyles.has(el)) baseStyles.set(el, el.getAttribute("style") || "");
  }
  function resetEl(el) {
    if (!el) return;
    var base = baseStyles.get(el);
    if (base === undefined) base = "";
    el.setAttribute("style", base);
  }
  function ease(p, name) {
    if (name === "linear") return p;
    if (name === "ease-in") return p * p;
    if (name === "ease-out") return 1 - (1 - p) * (1 - p);
    if (name === "ease-in-out") return p < 0.5 ? 2 * p * p : 1 - Math.pow(-2 * p + 2, 2) / 2;
    return p * p * (3 - 2 * p);
  }
  function vec(fx, p) {
    var dir = fx.direction || "left";
    var d = (fx.effect === "fly-out" || fx.effect === "fly") ? 60 : 40;
    var sx = dir === "left" ? -d : dir === "right" ? d : 0;
    var sy = dir === "up" ? -d : dir === "down" ? d : 0;
    return { x: sx * (1 - p), y: sy * (1 - p) };
  }
  function pathPoint(fx, p) {
    var path = fx.path || {};
    var x1 = Number(path.x1) || 0, y1 = Number(path.y1) || 0;
    var x2 = Number(path.x2) || 0, y2 = Number(path.y2) || 0;
    var cx = Number(path.cx) || 0, cy = Number(path.cy) || 0;
    var q = 1 - p;
    var x = q * q * x1 + 2 * q * p * cx + p * p * x2;
    var y = q * q * y1 + 2 * q * p * cy + p * p * y2;
    return { x: x * 100, y: y * 100 };
  }

  /* Returns the inline-style patch for one effect at progress p (0..1).
   * `final` forces the settled state; `initial` forces the pre-animation state. */
  function patchFor(fx, p, mode) {
    var cat = fx.category || "entrance";
    var id = fx.effect || "";
    if (mode === "initial") {
      if (cat === "entrance") return { opacity: "0" };
      if (cat === "motion") { var s = pathPoint(fx, 0); return { transform: "translate(" + s.x + "px," + s.y + "px)" }; }
      return {};
    }
    if (mode === "final" || p >= 1) {
      if (cat === "exit") return { opacity: "0" };
      if (cat === "motion") { var e = pathPoint(fx, 1); return { transform: "translate(" + e.x + "px," + e.y + "px)" }; }
      return {};
    }
    var eased = ease(p, fx.easing);
    if (cat === "entrance") {
      if (id === "fly" || id === "wipe") { var v = vec(fx, eased); return { opacity: String(eased), transform: "translate(" + v.x + "px," + v.y + "px)" }; }
      if (id === "zoom") return { opacity: String(eased), transform: "scale(" + (0.4 + 0.6 * eased) + ")" };
      return { opacity: String(eased) };
    }
    if (cat === "exit") {
      if (id === "fly-out" || id === "wipe") { var w = vec(fx, eased); return { opacity: String(1 - eased), transform: "translate(" + (-w.x) + "px," + (-w.y) + "px)" }; }
      return { opacity: String(1 - eased) };
    }
    if (cat === "emphasis") {
      var amount = Math.sin(eased * Math.PI);
      if (id === "spin") return { transform: "rotate(" + (amount * 360) + "deg)" };
      if (id === "grow") return { transform: "scale(" + (1 + 0.25 * amount) + ")" };
      return { transform: "scale(" + (1 + 0.08 * amount) + ")" };
    }
    if (cat === "motion") { var m = pathPoint(fx, eased); return { transform: "translate(" + m.x + "px," + m.y + "px)" }; }
    return {};
  }
  function applyPatch(el, patch) {
    for (var k in patch) if (Object.prototype.hasOwnProperty.call(patch, k)) el.style[k] = patch[k];
  }
  function paragraphs(el, fx, p) {
    if (!el || (fx.category !== "build")) return;
    var kids = Array.prototype.slice.call(el.querySelectorAll("p,li,.sl-para"));
    if (!kids.length) return;
    var reveal = Math.floor(p * (kids.length + 0.0001));
    for (var i = 0; i < kids.length; i++) kids[i].style.opacity = i < reveal ? "1" : "0";
  }

  function applyFrame(list, time) {
    failedIds = [];
    var items = timeline(list);
    var touched = [];
    for (var i = 0; i < items.length; i++) {
      var fx = items[i].effect;
      var el = elementFor(fx);
      if (!el) continue;
      remember(el);
      if (touched.indexOf(el) < 0) { resetEl(el); touched.push(el); }
    }
    for (var j = 0; j < items.length; j++) {
      var it = items[j];
      var element = elementFor(it.effect);
      if (!element) continue;
      try {
        var mode = time < it.start ? "initial" : time >= it.end ? "final" : "progress";
        var progress = it.duration > 0 ? (time - it.start) / it.duration : 1;
        applyPatch(element, patchFor(it.effect, Math.max(0, Math.min(1, progress)), mode));
        paragraphs(element, it.effect, mode === "final" ? 1 : mode === "initial" ? 0 : progress);
      } catch (e) {
        failedIds.push(it.effect.id);
        console.warn("[gb-slides-anim] effect failed", it.effect.id, e);
        try { resetEl(element); } catch (inner) { /* element may be detached */ }
      }
    }
    return failedIds;
  }

  function revealAll(list) {
    var items = timeline(list || currentList);
    var seen = [];
    for (var i = 0; i < items.length; i++) {
      var el = elementFor(items[i].effect);
      if (!el || seen.indexOf(el) >= 0) continue;
      seen.push(el);
      try {
        remember(el);
        resetEl(el);
        applyPatch(el, patchFor(items[i].effect, 1, "final"));
      } catch (e) { resetEl(el); }
      try {
        var kids = el.querySelectorAll("p,li,.sl-para");
        for (var k = 0; k < kids.length; k++) kids[k].style.opacity = "1";
      } catch (e) { /* detached */ }
    }
  }
  function stop() {
    playing = false;
    // Best-effort: cancel is a no-op if the frame already fired.
    if (raf !== null && window.cancelAnimationFrame) { try { window.cancelAnimationFrame(raf); } catch (e) {} raf = null; }
  }
  function seek(time, list) {
    currentList = list || currentList;
    currentTotal = totalOf(currentList);
    lastTime = Math.max(0, Math.min(currentTotal, Number(time) || 0));
    var failed = applyFrame(currentList, lastTime);
    if (failed.length) status(t("slides-anim-status-error", "An animation failed — content is shown in its final state"), "slides-anim-status-error");
    return lastTime;
  }
  function play(list, opts) {
    stop();
    currentList = list || [];
    currentTotal = totalOf(currentList);
    var cfg = opts || {};
    if (cfg.reduced || A.reduced()) {
      revealAll(currentList);
      status(t("slides-anim-status-reduced", "Reduced motion — showing final states immediately"), "slides-anim-status-reduced");
      return Promise.resolve();
    }
    status(t("slides-anim-status-preview", "Previewing build"), "slides-anim-status-preview");
    return new Promise(function (resolve) {
      var begin = null;
      function tick(now) {
        if (!playing) { resolve(); return; }
        if (begin === null) begin = now;
        var elapsed = now - begin;
        seek(elapsed, currentList);
        if (elapsed >= currentTotal || !currentTotal) {
          revealAll(currentList);
          playing = false; raf = null; resolve(); return;
        }
        raf = window.requestAnimationFrame ? window.requestAnimationFrame(tick) : null;
        if (raf === null) { revealAll(currentList); playing = false; resolve(); }
      }
      playing = true;
      raf = window.requestAnimationFrame ? window.requestAnimationFrame(tick) : null;
      if (raf === null) { revealAll(currentList); playing = false; resolve(); }
    });
  }

  /* Slide transition — enhancement only; failure is a silent no-op so it can
   * never stop a slide change. Non-blocking (does not preventDefault). */
  function playTransition(target, cfg) {
    if (!target || A.reduced()) return;
    var conf = cfg || A.defaultTransition();
    if (!conf.type || conf.type === "none") return;
    var duration = Math.max(0, Number(conf.duration) || 0);
    try {
      if (!target.animate) return;
      var kind = conf.type;
      var from = kind === "fade" ? { opacity: 0.15 }
        : kind === "dissolve" ? { opacity: 0.3, filter: "blur(4px)" }
        : kind === "push" || kind === "cover" ? { transform: "translateX(" + (conf.direction === "right" ? "-40px" : "40px") + ")" }
        : { clipPath: "inset(0 100% 0 0)" };
      target.animate([from, { opacity: 1, filter: "blur(0)", transform: "translateX(0)", clipPath: "inset(0 0 0 0)" }],
        { duration: duration, easing: conf.easing || "ease" });
    } catch (e) {
      console.warn("[gb-slides-anim] transition skipped", e);
    }
  }
  function hookSlideChanges() {
    if (document.__saSlideHook) return;
    document.__saSlideHook = true;
    document.addEventListener("click", function (e) {
      var thumb = e.target && e.target.closest ? e.target.closest(".sl-thumb") : null;
      if (!thumb) return;
      window.setTimeout(function () {
        var cv = A.canvas();
        if (cv) playTransition(cv, A.transition(A.slideIndex()));
      }, 0);
    }, true);
  }

  window.SlidesAnimationEngine = {
    status: status,
    timeline: timeline,
    totalOf: totalOf,
    applyFrame: applyFrame,
    revealAll: revealAll,
    stop: stop,
    seek: seek,
    play: play,
    playTransition: playTransition,
    hookSlideChanges: hookSlideChanges,
    get total() { return currentTotal; },
    get time() { return lastTime; },
    get playing() { return playing; },
    get failed() { return failedIds.slice(); }
  };
})(window);
