// botui/ui/suite/sheet/modules/00d_touch_handler.js
// Pointer Events wrapper that unifies mouse, touch, and pen input.
// Existing code listens for mousedown/mousemove/mouseup/click; this
// module synthesizes those events from pointer events so the same
// logic works on touch devices. Also provides gesture detection:
// tap, double-tap, long-press, two-finger pinch.
//
// Loaded by all three suites (sheet, docs, slides) — it auto-detects
// the main interactive container via the data-touch-target attribute
// on the <body> or, if absent, by querying common selectors.
//
// Usage from a suite HTML:
//   <body data-touch-target="#grid">
//   <script src="../modules/00d_touch_handler.js"></script>
"use strict";

(function () {
  const GESTURE_LONG_PRESS_MS = 500;
  const GESTURE_DOUBLE_TAP_MS = 300;
  const PINCH_MIN_DISTANCE = 10;

  let activePointers = new Map();
  let lastTapTime = 0;
  let lastTapTarget = null;
  let longPressTimer = null;
  let pinchStartDistance = 0;
  let pinchInitialValue = null;

  function getTarget() {
    // Try explicit data-touch-target first (set on <body> or root container)
    const explicit = document.querySelector("[data-touch-target]");
    if (explicit) {
      const sel = explicit.getAttribute("data-touch-target");
      if (sel && sel !== "self") {
        const el = document.querySelector(sel);
        if (el) return el;
      }
      return explicit;
    }
    // Fall back to common interactive container IDs
    return (
      document.querySelector("#grid") ||
      document.querySelector("#editor") ||
      document.querySelector("#canvas") ||
      document.querySelector(".sheet-app") ||
      document.querySelector(".docs-app") ||
      document.querySelector(".slides-app") ||
      document.documentElement
    );
  }

  function isInteractiveTarget(el) {
    if (!el) return false;
    const tag = el.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    if (el.isContentEditable) return true;
    if (el.closest && el.closest(".modal:not(.hidden)")) return true;
    return false;
  }

  function dispatchMouse(type, originalEvent) {
    const target = originalEvent.target;
    if (!target || isInteractiveTarget(target)) return;
    const evt = new MouseEvent(type, {
      bubbles: true,
      cancelable: true,
      view: window,
      button: 0,
      clientX: originalEvent.clientX,
      clientY: originalEvent.clientY,
      screenX: originalEvent.screenX,
      screenY: originalEvent.screenY,
      ctrlKey: originalEvent.ctrlKey,
      shiftKey: originalEvent.shiftKey,
      altKey: originalEvent.altKey,
      metaKey: originalEvent.metaKey,
    });
    target.dispatchEvent(evt);
  }

  function distance(a, b) {
    const dx = a.clientX - b.clientX;
    const dy = a.clientY - b.clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }

  function onPointerDown(e) {
    activePointers.set(e.pointerId, {
      x: e.clientX,
      y: e.clientY,
      startX: e.clientX,
      startY: e.clientY,
      startTime: Date.now(),
      target: e.target,
    });

    if (activePointers.size === 1) {
      // Single pointer — long press detection
      clearTimeout(longPressTimer);
      longPressTimer = setTimeout(function () {
        const p = activePointers.get(e.pointerId);
        if (p && Date.now() - p.startTime >= GESTURE_LONG_PRESS_MS) {
          const dx = p.x - p.startX;
          const dy = p.y - p.startY;
          if (Math.abs(dx) < 10 && Math.abs(dy) < 10) {
            const ev = new CustomEvent("longpress", {
              bubbles: true,
              cancelable: true,
              detail: { clientX: p.x, clientY: p.y, target: p.target },
            });
            p.target.dispatchEvent(ev);
          }
        }
      }, GESTURE_LONG_PRESS_MS);

      // Try to dispatch as mousedown for non-touch pointers
      if (e.pointerType === "touch") {
        dispatchMouse("mousedown", e);
      }
    } else if (activePointers.size === 2) {
      // Pinch start
      const pts = Array.from(activePointers.values());
      pinchStartDistance = distance(pts[0], pts[1]);
      pinchInitialValue = null;
    }
  }

  function onPointerMove(e) {
    const p = activePointers.get(e.pointerId);
    if (!p) return;
    p.x = e.clientX;
    p.y = e.clientY;

    if (activePointers.size === 1 && e.pointerType === "touch") {
      dispatchMouse("mousemove", e);
    } else if (activePointers.size === 2) {
      const pts = Array.from(activePointers.values());
      const d = distance(pts[0], pts[1]);
      if (pinchStartDistance > 0 && Math.abs(d - pinchStartDistance) > PINCH_MIN_DISTANCE) {
        const ev = new CustomEvent("pinch", {
          bubbles: true,
          cancelable: true,
          detail: {
            scale: d / pinchStartDistance,
            centerX: (pts[0].clientX + pts[1].clientX) / 2,
            centerY: (pts[0].clientY + pts[1].clientY) / 2,
          },
        });
        e.target.dispatchEvent(ev);
        pinchStartDistance = d;
      }
    }
  }

  function onPointerUp(e) {
    const p = activePointers.get(e.pointerId);
    activePointers.delete(e.pointerId);
    clearTimeout(longPressTimer);

    if (!p) return;

    const dt = Date.now() - p.startTime;
    const dx = p.x - p.startX;
    const dy = p.y - p.startY;
    const moved = Math.abs(dx) > 10 || Math.abs(dy) > 10;

    if (e.pointerType === "touch") {
      dispatchMouse("mouseup", e);
      if (!moved && dt < GESTURE_LONG_PRESS_MS) {
        // Tap
        const now = Date.now();
        const isDoubleTap =
          lastTapTarget === p.target &&
          now - lastTapTime < GESTURE_DOUBLE_TAP_MS;
        if (isDoubleTap) {
          const ev = new CustomEvent("doubletap", {
            bubbles: true,
            cancelable: true,
            detail: { clientX: p.x, clientY: p.y, target: p.target },
          });
          p.target.dispatchEvent(ev);
          lastTapTime = 0;
          lastTapTarget = null;
        } else {
          dispatchMouse("click", e);
          lastTapTime = now;
          lastTapTarget = p.target;
        }
      }
    }
  }

  function onPointerCancel(e) {
    activePointers.delete(e.pointerId);
    clearTimeout(longPressTimer);
  }

  function init() {
    const target = getTarget();
    if (!target) return;

    target.addEventListener("pointerdown", onPointerDown, { passive: true });
    target.addEventListener("pointermove", onPointerMove, { passive: true });
    target.addEventListener("pointerup", onPointerUp, { passive: true });
    target.addEventListener("pointercancel", onPointerCancel, { passive: true });

    // Prevent native pinch-zoom on the target (but allow on form fields)
    target.addEventListener(
      "touchmove",
      function (e) {
        if (e.touches.length > 1 && !isInteractiveTarget(e.target)) {
          e.preventDefault();
        }
      },
      { passive: false }
    );

    // Expose API
    window.TouchHandler = {
      isActive: function () {
        return activePointers.size > 0;
      },
      activeCount: function () {
        return activePointers.size;
      },
      target: function () {
        return target;
      },
    };
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
