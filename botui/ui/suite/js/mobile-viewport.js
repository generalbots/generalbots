"use strict";
/* Mobile viewport bridge (#2516): publishes visualViewport as --gb-vv-height,
   --gb-vv-bottom (keyboard inset) and --gb-vv-top; consumers stay declarative. */
const MobileViewport = (() => {
  let started = false, frame = 0, keyIdle = 0, keyState = false;

  function deferIdle(fn) {
    if (typeof window.requestIdleCallback === "function") return window.requestIdleCallback(fn, { timeout: 800 });
    return window.setTimeout(fn, 1);
  }
  // #2933 — coalesce the cosmetic keyboard flag to one idle toggle.
  function queueKeyboard(root, open) {
    keyState = open;
    if (keyIdle) return;
    keyIdle = 1;
    deferIdle(function () { keyIdle = 0; root.classList.toggle("gb-keyboard-open", keyState); });
  }

  function apply() {
    const vv = window.visualViewport;
    const root = document.documentElement;
    if (!root) return;
    if (!vv) {
      root.style.setProperty("--gb-vv-height", window.innerHeight + "px");
      root.style.setProperty("--gb-vv-bottom", "0px");
      root.style.setProperty("--gb-vv-top", "0px");
      return;
    }
    const bottom = Math.max(
      0,
      window.innerHeight - (vv.height + vv.offsetTop)
    );
    root.style.setProperty("--gb-vv-height", Math.round(vv.height) + "px");
    root.style.setProperty("--gb-vv-bottom", Math.round(bottom) + "px");
    root.style.setProperty("--gb-vv-top", Math.round(vv.offsetTop) + "px");
    // Cosmetic keyboard flag.
    queueKeyboard(root, bottom > 120);
  }

  // #2933 — one apply per frame; start() still paints synchronously.
  function schedule() {
    if (frame) return;
    frame = window.requestAnimationFrame(function () { frame = 0; apply(); });
  }

  function start() {
    if (started || typeof window === "undefined") return;
    started = true;
    apply();
    const vv = window.visualViewport;
    if (vv) {
      vv.addEventListener("resize", schedule);
      vv.addEventListener("scroll", schedule);
    }
    window.addEventListener("orientationchange", function () {
      // The viewport settles after the rotation animation.
      setTimeout(schedule, 250);
    });
    window.addEventListener("resize", schedule);
  }

  if (typeof window !== "undefined") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", start);
    } else {
      start();
    }
  }

  return { start: start, refresh: apply };
})();

window.MobileViewport = MobileViewport;
