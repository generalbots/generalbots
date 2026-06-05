"use strict";

/**
 * Module 29: Slide transitions during presentation (P0 critical).
 * Wires the existing slide.transition field (fade, slide-left/right/
 * up/down, zoom-in/out, flip, cube) to the actual transition between
 * slides in presenter mode. Uses CSS animation classes injected at
 * runtime with configurable duration. Skips the transition on the
 * first slide (no prior slide).
 *
 * Public API: window.SlidesTransitionsPresenter = {
 *   applyToSlideChange, getTransitionClass, getDuration, animate,
 *   applyGlobal, list
 * }.
 */

(function () {
  function getState() { return window.state || null; }
  function getCanvas() { return document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas"); }

  const TRANSITION_DEFS = {
    none: { className: "transition-none", duration: 0 },
    fade: { className: "transition-fade", duration: 400 },
    "slide-left": { className: "transition-slide-left", duration: 500 },
    "slide-right": { className: "transition-slide-right", duration: 500 },
    "slide-up": { className: "transition-slide-up", duration: 500 },
    "slide-down": { className: "transition-slide-down", duration: 500 },
    "zoom-in": { className: "transition-zoom-in", duration: 500 },
    "zoom-out": { className: "transition-zoom-out", duration: 500 },
    flip: { className: "transition-flip", duration: 700 },
    cube: { className: "transition-cube", duration: 800 },
  };

  function ensureStyles() {
    if (document.getElementById("slidesTransitionsStyle")) return;
    const s = document.createElement("style");
    s.id = "slidesTransitionsStyle";
    s.textContent = `
      .slide-canvas, .slides-canvas, #slideCanvas { transition: none; will-change: transform, opacity; }
      .transition-fade { animation: st-fade var(--st-dur, 0.4s) ease both; }
      .transition-slide-left { animation: st-slide-left var(--st-dur, 0.5s) ease both; }
      .transition-slide-right { animation: st-slide-right var(--st-dur, 0.5s) ease both; }
      .transition-slide-up { animation: st-slide-up var(--st-dur, 0.5s) ease both; }
      .transition-slide-down { animation: st-slide-down var(--st-dur, 0.5s) ease both; }
      .transition-zoom-in { animation: st-zoom-in var(--st-dur, 0.5s) ease both; }
      .transition-zoom-out { animation: st-zoom-out var(--st-dur, 0.5s) ease both; }
      .transition-flip { animation: st-flip var(--st-dur, 0.7s) ease both; }
      .transition-cube { animation: st-cube var(--st-dur, 0.8s) ease both; transform-style: preserve-3d; perspective: 1200px; }
      @keyframes st-fade { from { opacity: 0; } to { opacity: 1; } }
      @keyframes st-slide-left { from { transform: translateX(100%); } to { transform: translateX(0); } }
      @keyframes st-slide-right { from { transform: translateX(-100%); } to { transform: translateX(0); } }
      @keyframes st-slide-up { from { transform: translateY(100%); } to { transform: translateY(0); } }
      @keyframes st-slide-down { from { transform: translateY(-100%); } to { transform: translateY(0); } }
      @keyframes st-zoom-in { from { transform: scale(0.5); opacity: 0; } to { transform: scale(1); opacity: 1; } }
      @keyframes st-zoom-out { from { transform: scale(1.5); opacity: 0; } to { transform: scale(1); opacity: 1; } }
      @keyframes st-flip { from { transform: rotateY(-90deg); opacity: 0; } to { transform: rotateY(0); opacity: 1; } }
      @keyframes st-cube { from { transform: rotateY(90deg) translateZ(80px); opacity: 0; } to { transform: rotateY(0) translateZ(0); opacity: 1; } }
    `;
    document.head.appendChild(s);
  }

  function list() { return Object.keys(TRANSITION_DEFS); }
  function getTransitionClass(name) { return (TRANSITION_DEFS[name] || TRANSITION_DEFS.none).className; }
  function getDuration(name) { return (TRANSITION_DEFS[name] || TRANSITION_DEFS.none).duration; }

  function applyToSlideChange(targetEl, transition) {
    if (!targetEl) return Promise.resolve();
    ensureStyles();
    const def = TRANSITION_DEFS[transition] || TRANSITION_DEFS.none;
    if (def.duration === 0) return Promise.resolve();
    targetEl.style.setProperty("--st-dur", (def.duration / 1000) + "s");
    targetEl.classList.add(def.className);
    return new Promise(function (resolve) {
      const onEnd = function () { targetEl.classList.remove(def.className); targetEl.removeEventListener("animationend", onEnd); resolve(); };
      targetEl.addEventListener("animationend", onEnd);
      setTimeout(onEnd, def.duration + 100);
    });
  }

  function animate(transition, targetEl) { return applyToSlideChange(targetEl, transition); }

  function applyGlobal(name) {
    const s = getState();
    if (!s) return false;
    for (const slide of s.slides || []) {
      slide.transition = name;
    }
    return true;
  }

  function hookStartPresentation() {
    const orig = window.startPresentation || function () {};
    window.startPresentation = function () {
      const c = getCanvas();
      const s = getState();
      if (!c || !s) return orig();
      let idx = s.currentSlide || 0;
      c.style.setProperty("--st-dur", "0.4s");
      function showNext() {
        idx = (idx + 1) % (s.slides || []).length;
        s.currentSlide = idx;
        if (typeof window.SlidesNavigate === "object" && window.SlidesNavigate.goTo) {
          window.SlidesNavigate.goTo(idx);
        }
        const slide = (s.slides || [])[idx];
        const tr = (slide && slide.transition) || "none";
        applyToSlideChange(c, tr);
        if (typeof window.SlidesAnimationsPresenter === "object" && window.SlidesAnimationsPresenter.playForSlide) {
          window.SlidesAnimationsPresenter.playForSlide(slide);
        }
      }
      function showPrev() {
        idx = (idx - 1 + (s.slides || []).length) % (s.slides || []).length;
        s.currentSlide = idx;
        if (typeof window.SlidesNavigate === "object" && window.SlidesNavigate.goTo) {
          window.SlidesNavigate.goTo(idx);
        }
        const slide = (s.slides || [])[idx];
        const tr = (slide && slide.transition) || "none";
        applyToSlideChange(c, tr);
        if (typeof window.SlidesAnimationsPresenter === "object" && window.SlidesAnimationsPresenter.playForSlide) {
          window.SlidesAnimationsPresenter.playForSlide(slide);
        }
      }
      document.addEventListener("keydown", function (e) {
        if (e.key === "ArrowRight" || e.key === "PageDown" || e.key === " ") { e.preventDefault(); showNext(); }
        else if (e.key === "ArrowLeft" || e.key === "PageUp") { e.preventDefault(); showPrev(); }
        else if (e.key === "Escape") { window.exitPresentation && window.exitPresentation(); }
      });
      orig();
    };
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { ensureStyles(); hookStartPresentation(); });
  } else {
    ensureStyles();
    hookStartPresentation();
  }

  window.SlidesTransitionsPresenter = { applyToSlideChange, getTransitionClass, getDuration, animate, applyGlobal, list, TRANSITION_DEFS };
})();
