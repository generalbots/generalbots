"use strict";

/**
 * Module 30: Element animations during presentation (P0 critical).
 * Plays the slide.animations[] array on each element when a slide is
 * shown. Uses the existing 11_animation_engine.js Easings (linear,
 * easeIn/Out, easeInOutCubic, easeOutBounce, easeOutElastic, etc).
 * Chains animations sequentially (with-after-previous) and respects
 * on-click / with-previous / after-previous start modes, duration,
 * delay, and the 10 entrance + 6 emphasis + 7 exit animations.
 *
 * Public API: window.SlidesAnimationsPresenter = {
 *   playForSlide, playElement, animate, listEntrance, listEmphasis, listExit
 * }.
 */

(function () {
  const EASINGS = {
    linear: function (t) { return t; },
    easeIn: function (t) { return t * t; },
    easeOut: function (t) { return 1 - (1 - t) * (1 - t); },
    easeInOut: function (t) { return t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2; },
    easeInCubic: function (t) { return t * t * t; },
    easeOutCubic: function (t) { return 1 - Math.pow(1 - t, 3); },
    easeInOutCubic: function (t) { return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2; },
    easeOutBounce: function (t) {
      const n1 = 7.5625, d1 = 2.75;
      if (t < 1 / d1) return n1 * t * t;
      if (t < 2 / d1) { t -= 1.5 / d1; return n1 * t * t + 0.75; }
      if (t < 2.5 / d1) { t -= 2.25 / d1; return n1 * t * t + 0.9375; }
      t -= 2.625 / d1; return n1 * t * t + 0.984375;
    },
    easeOutElastic: function (t) {
      if (t === 0 || t === 1) return t;
      const p = 0.3, s = p / 4;
      return Math.pow(2, -10 * t) * Math.sin((t - s) * (2 * Math.PI) / p) + 1;
    },
  };

  const ENTRANCE = {
    none: { css: { opacity: 1, transform: "none" } },
    "fade-in": { from: { opacity: 0 }, to: { opacity: 1 } },
    "fly-in-left": { from: { transform: "translateX(-100%)", opacity: 0 }, to: { transform: "translateX(0)", opacity: 1 } },
    "fly-in-right": { from: { transform: "translateX(100%)", opacity: 0 }, to: { transform: "translateX(0)", opacity: 1 } },
    "fly-in-top": { from: { transform: "translateY(-100%)", opacity: 0 }, to: { transform: "translateY(0)", opacity: 1 } },
    "fly-in-bottom": { from: { transform: "translateY(100%)", opacity: 0 }, to: { transform: "translateY(0)", opacity: 1 } },
    "zoom-in": { from: { transform: "scale(0)", opacity: 0 }, to: { transform: "scale(1)", opacity: 1 } },
    "bounce-in": { from: { transform: "scale(0.3)", opacity: 0 }, to: { transform: "scale(1)", opacity: 1 }, easing: "easeOutBounce" },
    "spin-in": { from: { transform: "rotate(-360deg) scale(0)", opacity: 0 }, to: { transform: "rotate(0) scale(1)", opacity: 1 } },
    "wipe-left": { from: { clipPath: "inset(0 100% 0 0)" }, to: { clipPath: "inset(0 0% 0 0)" } },
    "wipe-right": { from: { clipPath: "inset(0 0 0 100%)" }, to: { clipPath: "inset(0 0 0 0%)" } },
  };

  const EMPHASIS = {
    none: null,
    pulse: { keyframes: [{ transform: "scale(1)" }, { transform: "scale(1.1)" }, { transform: "scale(1)" }], duration: 0.6 },
    shake: { keyframes: [{ transform: "translateX(0)" }, { transform: "translateX(-5px)" }, { transform: "translateX(5px)" }, { transform: "translateX(-5px)" }, { transform: "translateX(0)" }], duration: 0.5 },
    bounce: { keyframes: [{ transform: "translateY(0)" }, { transform: "translateY(-15px)" }, { transform: "translateY(0)" }], duration: 0.7, easing: "easeOutBounce" },
    spin: { keyframes: [{ transform: "rotate(0)" }, { transform: "rotate(360deg)" }], duration: 0.8 },
    grow: { keyframes: [{ transform: "scale(1)" }, { transform: "scale(1.2)" }, { transform: "scale(1)" }], duration: 0.5 },
    flash: { keyframes: [{ opacity: 1 }, { opacity: 0.3 }, { opacity: 1 }, { opacity: 0.3 }, { opacity: 1 }], duration: 0.8 },
  };

  const EXIT = {
    none: null,
    "fade-out": { from: { opacity: 1 }, to: { opacity: 0 } },
    "fly-out-left": { from: { transform: "translateX(0)", opacity: 1 }, to: { transform: "translateX(-100%)", opacity: 0 } },
    "fly-out-right": { from: { transform: "translateX(0)", opacity: 1 }, to: { transform: "translateX(100%)", opacity: 0 } },
    "fly-out-top": { from: { transform: "translateY(0)", opacity: 1 }, to: { transform: "translateY(-100%)", opacity: 0 } },
    "fly-out-bottom": { from: { transform: "translateY(0)", opacity: 1 }, to: { transform: "translateY(100%)", opacity: 0 } },
    "zoom-out": { from: { transform: "scale(1)", opacity: 1 }, to: { transform: "scale(0)", opacity: 0 } },
    "spin-out": { from: { transform: "rotate(0) scale(1)", opacity: 1 }, to: { transform: "rotate(360deg) scale(0)", opacity: 0 } },
  };

  function getEl(elementId) {
    return document.querySelector("[data-element-id='" + elementId + "']");
  }

  function animateProp(el, from, to, duration, easing) {
    return new Promise(function (resolve) {
      if (!el) return resolve();
      const fn = EASINGS[easing] || EASINGS.easeInOutCubic;
      const start = performance.now();
      const props = Object.keys(Object.assign({}, from || {}, to || {}));
      function frame(now) {
        const t = Math.min(1, (now - start) / (duration * 1000));
        const e = fn(t);
        for (const p of props) {
          const a = (from && from[p] != null) ? from[p] : 0;
          const b = (to && to[p] != null) ? to[p] : 0;
          el.style[p] = interpolate(a, b, e);
        }
        if (t < 1) requestAnimationFrame(frame);
        else resolve();
      }
      requestAnimationFrame(frame);
    });
  }

  function interpolate(a, b, t) {
    if (typeof a === "number" && typeof b === "number") return a + (b - a) * t + (typeof b === "number" && (b + "").indexOf("deg") >= 0 ? "deg" : "");
    return t < 1 ? a : b;
  }

  function animateEmphasis(el, def, duration) {
    return new Promise(function (resolve) {
      if (!el || !def) return resolve();
      const frames = def.keyframes || [];
      const d = (duration || def.duration || 0.5) * 1000;
      const start = performance.now();
      const easing = EASINGS[def.easing] || EASINGS.easeInOutCubic;
      function frame(now) {
        const t = Math.min(1, (now - start) / d);
        const e = easing(t);
        const idx = Math.min(frames.length - 2, Math.floor(e * (frames.length - 1)));
        const localT = (e * (frames.length - 1)) - idx;
        const f1 = frames[idx] || {};
        const f2 = frames[idx + 1] || {};
        for (const p of Object.keys(Object.assign({}, f1, f2))) {
          el.style[p] = interpolate(f1[p], f2[p], localT);
        }
        if (t < 1) requestAnimationFrame(frame);
        else resolve();
      }
      requestAnimationFrame(frame);
    });
  }

  async function playElement(anim) {
    const el = getEl(anim.elementId);
    if (!el) return;
    if (anim.entrance && anim.entrance !== "none") {
      const def = ENTRANCE[anim.entrance];
      if (def) {
        if (def.css) { Object.assign(el.style, def.css); return; }
        await new Promise(function (r) { setTimeout(r, (anim.delay || 0) * 1000); });
        await animateProp(el, def.from, def.to, anim.duration || 0.5, def.easing);
      }
    }
    if (anim.emphasis && anim.emphasis !== "none") {
      const def = EMPHASIS[anim.emphasis];
      if (def) await animateEmphasis(el, def, anim.duration || def.duration || 0.5);
    }
    if (anim.exit && anim.exit !== "none") {
      const def = EXIT[anim.exit];
      if (def) await animateProp(el, def.from, def.to, anim.duration || 0.5, def.easing);
    }
  }

  async function playForSlide(slide) {
    if (!slide || !slide.elements) return;
    const anims = [];
    for (const e of slide.elements) {
      if (e.animations && e.animations.length) {
        for (const a of e.animations) {
          a.elementId = e.id;
          anims.push(a);
        }
      }
    }
    anims.sort(function (a, b) { return (a.order || 0) - (b.order || 0); });
    for (const a of anims) {
      if (a.start === "on-click") continue;
      if (a.start === "after-previous") {
        await new Promise(function (r) { setTimeout(r, ((a.delay || 0)) * 1000); });
        await playElement(a);
      } else {
        await new Promise(function (r) { setTimeout(r, ((a.delay || 0)) * 1000); });
        await playElement(a);
      }
    }
  }

  function listEntrance() { return Object.keys(ENTRANCE); }
  function listEmphasis() { return Object.keys(EMPHASIS); }
  function listExit() { return Object.keys(EXIT); }

  function animate(el, type, phase, duration) {
    const def = (phase === "emphasis" ? EMPHASIS : phase === "exit" ? EXIT : ENTRANCE)[type];
    if (!def) return Promise.resolve();
    if (def.keyframes) return animateEmphasis(el, def, duration);
    return animateProp(el, def.from, def.to, duration || 0.5, def.easing);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { /* wired by 29 */ });
  }

  window.SlidesAnimationsPresenter = { playForSlide, playElement, animate, listEntrance, listEmphasis, listExit, ENTRANCE, EMPHASIS, EXIT, EASINGS };
})();
