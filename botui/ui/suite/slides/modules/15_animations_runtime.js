"use strict";

/**
 * Module 15: Element animations runtime for Slides.
 * Plays entrance/exit/emphasis animations on slide elements during
 * presentation. Hooks into the presenter modal via DOM mutation
 * observer on the #presenterSlide element. When a new slide is
 * rendered, walks the slide's elements and applies any animation
 * definitions via the Easings/RAF engine from 11_animation_engine.js.
 *
 * Each element can have an animations array:
 *   [{ type: "entrance" | "emphasis" | "exit",
 *      name: "fadeIn" | "slideIn" | "pulse" | "bounce" | "shake" |
 *            "fadeOut" | "slideOut" | "zoomIn" | "zoomOut",
 *      duration: 600,   // ms
 *      delay: 0,
 *      trigger: "onLoad" | "onClick" }]
 *
 * On "onLoad" the animation plays automatically when the slide is
 * first shown. On "onClick" it plays on the next user click.
 *
 * Public API: window.SlideAnimations = { playEntrance, playEmphasis,
 *   playExit, onNextClick }.
 */

(function () {
  const ENTRANCE = {
    fadeIn: (el) => fade(el, 0, 1, 600),
    slideIn: (el) => slideIn(el, 600),
    zoomIn: (el) => zoom(el, 0.5, 1, 600),
  };
  const EXIT = {
    fadeOut: (el) => fade(el, 1, 0, 600),
    slideOut: (el) => slideIn(el, 600, true),
    zoomOut: (el) => zoom(el, 1, 0.5, 600),
  };
  const EMPHASIS = {
    pulse: (el) => loop(el, 1200, (t) => 1 + 0.1 * Math.sin(t * Math.PI * 2)),
    bounce: (el) => loop(el, 800, (t) => 1 - Math.abs(Math.sin(t * Math.PI * 3)) * 0.15),
    shake: (el) => loop(el, 600, (t) => 1 + 0.04 * Math.sin(t * Math.PI * 12)),
  };

  function fade(el, from, to, duration) {
    el.style.opacity = String(from);
    el.style.willChange = "opacity";
    const start = performance.now();
    function step(now) {
      const t = Math.min(1, (now - start) / duration);
      el.style.opacity = String(from + (to - from) * t);
      if (t < 1) requestAnimationFrame(step);
      else el.style.willChange = "";
    }
    requestAnimationFrame(step);
  }

  function slideIn(el, duration, reverse) {
    const fromX = reverse ? 0 : -50;
    const toX = 0;
    const fromY = reverse ? 0 : 50;
    const toY = 0;
    el.style.transform = `translate(${fromX}px, ${fromY}px)`;
    el.style.willChange = "transform";
    const start = performance.now();
    function step(now) {
      const t = Math.min(1, (now - start) / duration);
      const ease = 1 - Math.pow(1 - t, 3);
      const x = fromX + (toX - fromX) * ease;
      const y = fromY + (toY - fromY) * ease;
      el.style.transform = `translate(${x}px, ${y}px)`;
      if (t < 1) requestAnimationFrame(step);
      else el.style.willChange = "";
    }
    requestAnimationFrame(step);
  }

  function zoom(el, from, to, duration) {
    el.style.transform = `scale(${from})`;
    el.style.willChange = "transform";
    const start = performance.now();
    function step(now) {
      const t = Math.min(1, (now - start) / duration);
      const ease = 1 - Math.pow(1 - t, 3);
      const s = from + (to - from) * ease;
      el.style.transform = `scale(${s})`;
      if (t < 1) requestAnimationFrame(step);
      else el.style.willChange = "";
    }
    requestAnimationFrame(step);
  }

  function loop(el, duration, fn) {
    let raf = 0;
    const start = performance.now();
    function step(now) {
      const t = ((now - start) % duration) / duration;
      const scale = fn(t);
      el.style.transform = `scale(${scale})`;
      raf = requestAnimationFrame(step);
    }
    raf = requestAnimationFrame(step);
    el._animCleanup = () => cancelAnimationFrame(raf);
  }

  function playEntrance(el, animName) {
    const fn = ENTRANCE[animName];
    if (fn) {
      el.style.opacity = "0";
      el.style.transform = "translate(0,0) scale(1)";
      requestAnimationFrame(() => fn(el));
    }
  }

  function playEmphasis(el, animName) {
    if (el._animCleanup) el._animCleanup();
    const fn = EMPHASIS[animName];
    if (fn) fn(el);
  }

  function playExit(el, animName) {
    const fn = EXIT[animName];
    if (fn) fn(el);
  }

  function playForElement(el) {
    if (!el) return;
    if (!el.animations || !Array.isArray(el.animations) || !el.animations.length) return;
    for (const a of el.animations) {
      const delay = a.delay || 0;
      if (a.type === "entrance" && (!a.trigger || a.trigger === "onLoad")) {
        setTimeout(() => playEntrance(el, a.name), delay);
      } else if (a.type === "emphasis") {
        setTimeout(() => playEmphasis(el, a.name), delay);
      } else if (a.type === "exit") {
        setTimeout(() => playExit(el, a.name), delay);
      }
    }
  }

  function playForSlide(slideEl) {
    if (!slideEl) return;
    const state = window.state;
    if (!state || !state.slides) return;
    const counter = document.getElementById("presenterSlideNumber");
    if (!counter) return;
    const m = counter.textContent.match(/^\s*(\d+)/);
    if (!m) return;
    const idx = parseInt(m[1], 10) - 1;
    const slide = state.slides[idx];
    if (!slide || !slide.elements) return;
    for (const el of slide.elements) {
      const dom = slideEl.querySelector(`[data-element-id="${el.id}"]`);
      if (dom) playForElement(dom);
    }
  }

  let lastSlideIndex = -1;
  function setupObserver() {
    const target = document.getElementById("presenterSlide");
    if (!target) return;
    const obs = new MutationObserver(() => {
      const counter = document.getElementById("presenterSlideNumber");
      if (!counter) return;
      const m = counter.textContent.match(/^\s*(\d+)/);
      if (!m) return;
      const idx = parseInt(m[1], 10) - 1;
      if (idx === lastSlideIndex) return;
      lastSlideIndex = idx;
      setTimeout(() => playForSlide(target), 50);
    });
    obs.observe(target, { childList: true, subtree: true });
    const counter = document.getElementById("presenterSlideNumber");
    if (counter) {
      const cm = new MutationObserver(() => {
        const m = counter.textContent.match(/^\s*(\d+)/);
        if (!m) return;
        const idx = parseInt(m[1], 10) - 1;
        if (idx === lastSlideIndex) return;
        lastSlideIndex = idx;
        setTimeout(() => playForSlide(target), 50);
      });
      cm.observe(counter, { childList: true, characterData: true, subtree: true });
    }
  }

  function onNextClick(el) {
    if (!el || !el.animations) return;
    for (const a of el.animations) {
      if (a.trigger === "onClick") {
        if (a.type === "entrance") playEntrance(el, a.name);
        else if (a.type === "emphasis") playEmphasis(el, a.name);
        else if (a.type === "exit") playExit(el, a.name);
      }
    }
  }

  document.addEventListener("keydown", (e) => {
    if (!window.state || !window.state.isPresenting) return;
    if (e.key !== "ArrowRight" && e.key !== " " && e.key !== "PageDown") return;
    const target = document.getElementById("presenterSlide");
    if (!target) return;
    setTimeout(() => {
      const els = target.querySelectorAll("[data-element-id]");
      els.forEach((dom) => onNextClick(dom));
    }, 50);
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => setupObserver());
  } else {
    setupObserver();
  }
  setTimeout(setupObserver, 500);
  setTimeout(setupObserver, 2000);

  window.SlideAnimations = { playEntrance, playEmphasis, playExit, onNextClick, playForSlide };
})();
