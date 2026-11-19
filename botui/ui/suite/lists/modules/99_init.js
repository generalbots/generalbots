"use strict";
/* #2537 - bootstrap. Idempotent so repeated HTMX swaps of the lists fragment
   never stack listeners. */
(function () {
  function boot() {
    if (window.GBSegments && typeof window.GBSegments.init === "function") {
      window.GBSegments.init();
    }
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
  document.addEventListener("htmx:afterSwap", boot);
})();
