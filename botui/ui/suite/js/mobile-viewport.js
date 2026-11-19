"use strict";
/* Mobile viewport bridge (#2516).
   On iOS Safari (and Chrome on Android with the keyboard open) the layout
   viewport does not shrink when the on-screen keyboard appears: `100vh` keeps
   the height of the page *without* the keyboard, so a bottom-anchored composer
   ends up underneath it and the user types blind.

   `window.visualViewport` is the only reliable source for what the user can
   actually see. This module publishes it as CSS custom properties, updated on
   resize/scroll and on orientation change:

     --gb-vv-height   visible height in px
     --gb-vv-bottom   inset from the bottom of the layout viewport to the
                      visible bottom (the keyboard height)
     --gb-vv-top      inset from the top (URL bar collapse)

   Every consumer stays declarative; no layout maths in the apps. */
const MobileViewport = (() => {
  let started = false;

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
    // Keyboard-open flag lets the CSS drop non-essential chrome while typing.
    const keyboardOpen = bottom > 120;
    root.classList.toggle("gb-keyboard-open", keyboardOpen);
  }

  function start() {
    if (started || typeof window === "undefined") return;
    started = true;
    apply();
    const vv = window.visualViewport;
    if (vv) {
      vv.addEventListener("resize", apply);
      vv.addEventListener("scroll", apply);
    }
    window.addEventListener("orientationchange", function () {
      // The viewport settles after the rotation animation.
      setTimeout(apply, 250);
    });
    window.addEventListener("resize", apply);
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
