"use strict";

/**
 * Module 14: Transitions runtime for Slides.
 * Applies slide transition animations (fade, slide-left/right/up/down,
 * zoom-in/out, flip, cube) when navigating between slides in
 * presenter mode. Hooks into the existing renderPresenterSlide path
 * via DOM mutation observer on the #presenterSlide element.
 *
 * Each transition type is a CSS animation that the outgoing slide
 * plays forward, then the incoming slide plays reverse/fade-in.
 *
 * Public API: window.Transitions = { applyTransition, getTransitionCSS }.
 */

(function () {
  const TRANSITIONS = {
    none: { in: "", out: "", duration: 0 },
    fade: {
      in: "@keyframes tFadeIn { from { opacity: 0 } to { opacity: 1 } }",
      out: "@keyframes tFadeOut { from { opacity: 1 } to { opacity: 0 } }",
      inClass: "transition-fade-in",
      outClass: "transition-fade-out",
      duration: 400,
    },
    slide_left: {
      in: "@keyframes tSlideLIn { from { transform: translateX(100%) } to { transform: translateX(0) } }",
      out: "@keyframes tSlideLOut { from { transform: translateX(0) } to { transform: translateX(-100%) } }",
      inClass: "transition-slide-left-in",
      outClass: "transition-slide-left-out",
      duration: 450,
    },
    slide_right: {
      in: "@keyframes tSlideRIn { from { transform: translateX(-100%) } to { transform: translateX(0) } }",
      out: "@keyframes tSlideROut { from { transform: translateX(0) } to { transform: translateX(100%) } }",
      inClass: "transition-slide-right-in",
      outClass: "transition-slide-right-out",
      duration: 450,
    },
    slide_up: {
      in: "@keyframes tSlideUIn { from { transform: translateY(100%) } to { transform: translateY(0) } }",
      out: "@keyframes tSlideUOut { from { transform: translateY(0) } to { transform: translateY(-100%) } }",
      inClass: "transition-slide-up-in",
      outClass: "transition-slide-up-out",
      duration: 450,
    },
    slide_down: {
      in: "@keyframes tSlideDIn { from { transform: translateY(-100%) } to { transform: translateY(0) } }",
      out: "@keyframes tSlideDOut { from { transform: translateY(0) } to { transform: translateY(100%) } }",
      inClass: "transition-slide-down-in",
      outClass: "transition-slide-down-out",
      duration: 450,
    },
    zoom_in: {
      in: "@keyframes tZoomIn { from { transform: scale(0.5); opacity: 0 } to { transform: scale(1); opacity: 1 } }",
      out: "@keyframes tZoomOut { from { transform: scale(1); opacity: 1 } to { transform: scale(1.5); opacity: 0 } }",
      inClass: "transition-zoom-in",
      outClass: "transition-zoom-out",
      duration: 500,
    },
    zoom_out: {
      in: "@keyframes tZoomOutIn { from { transform: scale(1.5); opacity: 0 } to { transform: scale(1); opacity: 1 } }",
      out: "@keyframes tZoomInOut { from { transform: scale(1); opacity: 1 } to { transform: scale(0.5); opacity: 0 } }",
      inClass: "transition-zoom-out-in",
      outClass: "transition-zoom-in-out",
      duration: 500,
    },
    flip: {
      in: "@keyframes tFlipIn { from { transform: perspective(800px) rotateY(-90deg) } to { transform: perspective(800px) rotateY(0) } }",
      out: "@keyframes tFlipOut { from { transform: perspective(800px) rotateY(0) } to { transform: perspective(800px) rotateY(90deg) } }",
      inClass: "transition-flip-in",
      outClass: "transition-flip-out",
      duration: 600,
    },
    cube: {
      in: "@keyframes tCubeIn { from { transform: perspective(800px) rotateY(90deg) } to { transform: perspective(800px) rotateY(0) } }",
      out: "@keyframes tCubeOut { from { transform: perspective(800px) rotateY(0) } to { transform: perspective(800px) rotateY(-90deg) } }",
      inClass: "transition-cube-in",
      outClass: "transition-cube-out",
      duration: 700,
    },
  };

  let styleEl = null;
  function ensureStylesheet() {
    if (styleEl) return;
    styleEl = document.createElement("style");
    styleEl.id = "transitions-runtime-styles";
    let css = "";
    for (const [name, t] of Object.entries(TRANSITIONS)) {
      if (name === "none") continue;
      css += t.in + t.out;
      css += `.${t.inClass} { animation: t${name.charAt(0).toUpperCase() + name.slice(1).replace(/_([a-z])/g, (_, c) => c.toUpperCase())}In ${t.duration}ms ease-out; }`;
      css += `.${t.outClass} { animation: t${name.charAt(0).toUpperCase() + name.slice(1).replace(/_([a-z])/g, (_, c) => c.toUpperCase())}Out ${t.duration}ms ease-in; }`;
    }
    css +=
      "[data-transitioning] { will-change: transform, opacity; }\n" +
      "#presenterSlide, .presenter-slide-content { transform-origin: center center; }\n";
    styleEl.textContent = css;
    document.head.appendChild(styleEl);
  }

  function getTransitionForSlide(slide) {
    if (!slide || !slide.transition_type) return "none";
    const t = slide.transition_type;
    if (TRANSITIONS[t]) return t;
    return "none";
  }

  function applyTransition(presenterEl, slide) {
    if (!presenterEl) return Promise.resolve();
    ensureStylesheet();
    const t = getTransitionForSlide(slide);
    const def = TRANSITIONS[t];
    if (!def || t === "none") return Promise.resolve();
    return new Promise((resolve) => {
      presenterEl.setAttribute("data-transitioning", t);
      presenterEl.classList.add(def.inClass);
      setTimeout(() => {
        presenterEl.classList.remove(def.inClass);
        presenterEl.removeAttribute("data-transitioning");
        resolve();
      }, def.duration);
    });
  }

  function animateOutgoing(presenterEl, slide) {
    if (!presenterEl) return Promise.resolve();
    ensureStylesheet();
    const t = getTransitionForSlide(slide);
    const def = TRANSITIONS[t];
    if (!def || t === "none") return Promise.resolve();
    return new Promise((resolve) => {
      presenterEl.setAttribute("data-transitioning", t);
      presenterEl.classList.add(def.outClass);
      setTimeout(() => {
        presenterEl.classList.remove(def.outClass);
        presenterEl.removeAttribute("data-transitioning");
        resolve();
      }, def.duration);
    });
  }

  let lastSlideIndex = -1;
  function setupObserver() {
    const target = document.getElementById("presenterSlide") || document.querySelector(".presenter-modal");
    if (!target) return;
    const obs = new MutationObserver(() => {
      const counter = document.getElementById("presenterSlideNumber");
      if (!counter) return;
      const m = counter.textContent.match(/^\s*(\d+)/);
      if (!m) return;
      const idx = parseInt(m[1], 10) - 1;
      if (idx === lastSlideIndex) return;
      lastSlideIndex = idx;
      const slide = (window.state && window.state.slides && window.state.slides[idx]) || null;
      if (slide) applyTransition(target, slide);
    });
    obs.observe(target, { childList: true, subtree: true, attributes: true });
    const counter = document.getElementById("presenterSlideNumber");
    if (counter) {
      const cm = new MutationObserver(() => {
        const m = counter.textContent.match(/^\s*(\d+)/);
        if (!m) return;
        const idx = parseInt(m[1], 10) - 1;
        if (idx === lastSlideIndex) return;
        const slide = (window.state && window.state.slides && window.state.slides[idx]) || null;
        if (slide) applyTransition(target, slide);
        lastSlideIndex = idx;
      });
      cm.observe(counter, { childList: true, characterData: true, subtree: true });
    }
  }

  function getTransitionCSS() {
    return styleEl ? styleEl.textContent : "";
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
      ensureStylesheet();
      setupObserver();
    });
  } else {
    ensureStylesheet();
    setupObserver();
  }
  setTimeout(() => {
    ensureStylesheet();
    setupObserver();
  }, 500);
  setTimeout(() => {
    ensureStylesheet();
    setupObserver();
  }, 2000);

  window.Transitions = { applyTransition, animateOutgoing, getTransitionCSS, TRANSITIONS };
})();
