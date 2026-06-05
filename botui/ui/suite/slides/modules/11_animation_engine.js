"use strict";

/**
 * Module 11: Animation engine for Slides.
 * Provides: object animations, keyframes, easing, timing, replay, onComplete callbacks.
 */

function animationId() {
  return "anim-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8);
}

const Easings = {
  linear: (t) => t,
  easeIn: (t) => t * t,
  easeOut: (t) => t * (2 - t),
  easeInOut: (t) => (t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t),
  easeInCubic: (t) => t * t * t,
  easeOutCubic: (t) => --t * t * t + 1,
  easeInOutCubic: (t) => (t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1),
  easeOutBounce: (t) => {
    if (t < 1 / 2.75) return 7.5625 * t * t;
    if (t < 2 / 2.75) return 7.5625 * (t -= 1.5 / 2.75) * t + 0.75;
    if (t < 2.5 / 2.75) return 7.5625 * (t -= 2.25 / 2.75) * t + 0.9375;
    return 7.5625 * (t -= 2.625 / 2.75) * t + 0.984375;
  },
  easeOutElastic: (t) => {
    if (t === 0 || t === 1) return t;
    const p = 0.3;
    return Math.pow(2, -10 * t) * Math.sin(((t - p / 4) * (2 * Math.PI)) / p) + 1;
  },
};

function createAnimation(element, options) {
  if (!element) return null;
  const opts = Object.assign(
    {
      duration: 1000,
      delay: 0,
      easing: "easeInOut",
      from: {},
      to: {},
      onStart: null,
      onUpdate: null,
      onComplete: null,
    },
    options || {}
  );
  const easingFn = Easings[opts.easing] || Easings.linear;
  const id = animationId();
  const animation = {
    id,
    element,
    startTime: null,
    rafHandle: null,
    completed: false,
    cancelled: false,
  };
  function step(timestamp) {
    if (animation.cancelled) return;
    if (animation.startTime == null) animation.startTime = timestamp;
    const elapsed = timestamp - animation.startTime;
    if (elapsed < opts.delay) {
      animation.rafHandle = requestAnimationFrame(step);
      return;
    }
    if (elapsed === opts.delay && opts.onStart) opts.onStart(element);
    const t = Math.min(1, (elapsed - opts.delay) / opts.duration);
    const eased = easingFn(t);
    const style = interpolateStyle(opts.from, opts.to, eased);
    applyStyle(element, style);
    if (opts.onUpdate) opts.onUpdate(element, t);
    if (t < 1) {
      animation.rafHandle = requestAnimationFrame(step);
    } else {
      animation.completed = true;
      if (opts.onComplete) opts.onComplete(element);
    }
  }
  animation.play = function play() {
    animation.cancelled = false;
    animation.completed = false;
    animation.startTime = null;
    animation.rafHandle = requestAnimationFrame(step);
    return animation;
  };
  animation.cancel = function cancel() {
    animation.cancelled = true;
    if (animation.rafHandle) cancelAnimationFrame(animation.rafHandle);
  };
  return animation;
}

function interpolateStyle(from, to, t) {
  const out = {};
  const keys = new Set([...Object.keys(from || {}), ...Object.keys(to || {})]);
  for (const k of keys) {
    const a = from && from[k] != null ? from[k] : 0;
    const b = to && to[k] != null ? to[k] : 0;
    if (typeof a === "number" && typeof b === "number") {
      out[k] = a + (b - a) * t;
    } else {
      out[k] = t < 0.5 ? a : b;
    }
  }
  return out;
}

function applyStyle(element, style) {
  if (!element || !style) return;
  if (style.opacity != null) element.style.opacity = String(style.opacity);
  if (style.x != null) element.style.transform = (element.style.transform || "") + ` translateX(${style.x}px)`;
  if (style.y != null) element.style.transform = (element.style.transform || "") + ` translateY(${style.y}px)`;
  if (style.scale != null) element.style.transform = (element.style.transform || "") + ` scale(${style.scale})`;
  if (style.rotate != null) element.style.transform = (element.style.transform || "") + ` rotate(${style.rotate}deg)`;
  if (style.width != null) element.style.width = `${style.width}px`;
  if (style.height != null) element.style.height = `${style.height}px`;
  if (style.color) element.style.color = style.color;
  if (style.background) element.style.background = style.background;
}

function fadeIn(element, duration) {
  return createAnimation(element, {
    duration: duration || 600,
    from: { opacity: 0 },
    to: { opacity: 1 },
  });
}

function fadeOut(element, duration) {
  return createAnimation(element, {
    duration: duration || 600,
    from: { opacity: 1 },
    to: { opacity: 0 },
  });
}

function slideIn(element, direction, duration) {
  const dx = direction === "left" ? -100 : direction === "right" ? 100 : 0;
  const dy = direction === "up" ? -100 : direction === "down" ? 100 : 0;
  return createAnimation(element, {
    duration: duration || 600,
    from: { x: dx, y: dy, opacity: 0 },
    to: { x: 0, y: 0, opacity: 1 },
  });
}

function pulse(element, scale, duration) {
  return createAnimation(element, {
    duration: duration || 800,
    easing: "easeInOut",
    from: { scale: 1 },
    to: { scale: scale || 1.2 },
  });
}

function playSequence(animations, onAllComplete) {
  let i = 0;
  function next() {
    if (i >= animations.length) {
      if (onAllComplete) onAllComplete();
      return;
    }
    const a = animations[i++];
    if (!a) {
      next();
      return;
    }
    const original = a.onComplete;
    a.onComplete = function () {
      if (original) original(a.element);
      next();
    };
    a.play();
  }
  next();
}

window.SlidesAnimation = {
  Easings,
  createAnimation,
  fadeIn,
  fadeOut,
  slideIn,
  pulse,
  playSequence,
  interpolateStyle,
  applyStyle,
};
