"use strict";
/* Shell execution mode contract (#2422, offline groundwork slice).
 *
 * Three measured modes, one codebase:
 *   - desktop: viewport >= 1025px (any pointer) — window manager, byte-identical
 *   - tablet:  768-1024px, or coarse pointer with room (>=768 and <1025)
 *   - compact: < 768px — one pane, drawer/back-stack oriented shell
 *
 * Hard rule: no fork. Desktop-local and web share this file. At >=1025px this
 * module must never execute a compact code path; it only publishes the detected
 * mode as data attributes/classes and a CustomEvent for consumers that gate
 * their own behavior.
 */
const GBShellMode = (() => {
  const BREAKPOINTS = Object.freeze({
    compactMax: 767,
    tabletMin: 768,
    tabletMax: 1024,
    desktopMin: 1025,
  });

  let currentMode = "desktop";
  let started = false;

  function isCoarsePointer() {
    try {
      return !!(window.matchMedia && window.matchMedia("(pointer: coarse)").matches);
    } catch (_e) {
      return false;
    }
  }

  function viewportWidth() {
    const vv = window.visualViewport;
    if (vv && typeof vv.width === "number" && vv.width > 0) {
      return vv.width;
    }
    return window.innerWidth || (document.documentElement && document.documentElement.clientWidth) || 0;
  }

  function detectMode(widthPx) {
    const width = typeof widthPx === "number" ? widthPx : viewportWidth();
    if (width >= BREAKPOINTS.desktopMin) {
      return "desktop";
    }
    if (width < BREAKPOINTS.tabletMin) {
      return "compact";
    }
    // 768-1024px: tablet by viewport. Coarse pointer with room (<1025, >=768)
    // also resolves to tablet; coarse pointers below tabletMin remain compact.
    if (width <= BREAKPOINTS.tabletMax || isCoarsePointer()) {
      return "tablet";
    }
    return "desktop";
  }

  function applyMode(mode, opts) {
    const options = opts || {};
    const prev = currentMode;
    currentMode = mode;

    const root = document.documentElement;
    const body = document.body;
    if (root) {
      root.dataset.shellMode = mode;
      root.classList.toggle("shell-mode-desktop", mode === "desktop");
      root.classList.toggle("shell-mode-tablet", mode === "tablet");
      root.classList.toggle("shell-mode-compact", mode === "compact");
    }
    if (body) {
      body.dataset.shellMode = mode;
      body.classList.toggle("shell-mode-desktop", mode === "desktop");
      body.classList.toggle("shell-mode-tablet", mode === "tablet");
      body.classList.toggle("shell-mode-compact", mode === "compact");
    }

    if (!options.silent && prev !== mode) {
      try {
        window.dispatchEvent(
          new CustomEvent("gb:shell-mode-change", {
            detail: { mode: mode, prev: prev, width: viewportWidth() },
          })
        );
      } catch (_e) {
        // Non-fatal: event dispatch must not break shell bootstrap.
      }
    }
    return mode;
  }

  function refresh(opts) {
    return applyMode(detectMode(), opts);
  }

  function getMode() {
    return currentMode;
  }

  function isDesktop() {
    return currentMode === "desktop";
  }

  function isTablet() {
    return currentMode === "tablet";
  }

  function isCompact() {
    return currentMode === "compact";
  }

  function onViewportChange() {
    refresh();
  }

  function start() {
    if (started) {
      return refresh({ silent: true });
    }
    started = true;
    refresh({ silent: true });

    window.addEventListener("resize", onViewportChange, { passive: true });
    window.addEventListener("orientationchange", onViewportChange, { passive: true });

    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", onViewportChange, { passive: true });
    }

    try {
      const coarseMql = window.matchMedia("(pointer: coarse)");
      if (coarseMql && coarseMql.addEventListener) {
        coarseMql.addEventListener("change", onViewportChange);
      } else if (coarseMql && coarseMql.addListener) {
        coarseMql.addListener(onViewportChange);
      }
    } catch (_e) {
      // matchMedia listener registration is best-effort.
    }

    return currentMode;
  }

  const api = Object.freeze({
    BREAKPOINTS: BREAKPOINTS,
    detectMode: detectMode,
    getMode: getMode,
    isDesktop: isDesktop,
    isTablet: isTablet,
    isCompact: isCompact,
    refresh: refresh,
    start: start,
  });

  window.GBShellMode = api;
  start();
  return api;
})();
