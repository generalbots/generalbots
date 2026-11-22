"use strict";
/* #2461 — touch present mode for phones/tablets.
 *
 * The desktop presenter flow only requests fullscreen and takes a WebSocket
 * presenter role: there is no on-screen deck and no touch navigation. This
 * module adds a self-contained fullscreen presentation surface that reuses the
 * local delivery API (GBSlidesDelivery) for slide count/navigation, clones the
 * live slide canvas into a fitted stage and advances on tap/swipe.
 *
 * It is loaded only by the Slides app and stays inert on desktop pointers
 * until the Presenter View "Iniciar" control or F5 is activated.
 */
(function (window, document) {
  var D = window.GBSlidesDelivery;
  if (!D) return;

  var ID = "gb-touch-present";
  var overlay = null;
  var stage = null;
  var counter = null;
  var index = 0;
  var RERENDER_MS = 260;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function button(labelKey, fallback, onClick, bg) {
    var b = document.createElement("button");
    b.type = "button";
    b.textContent = t(labelKey, fallback);
    b.setAttribute("aria-label", t(labelKey, fallback));
    b.style.cssText = "min-width:44px;min-height:44px;padding:0 18px;border:0;border-radius:8px;" +
      "background:" + bg + ";color:#f8fafc;font-size:15px;font-weight:600;cursor:pointer;touch-action:manipulation;";
    b.addEventListener("click", function (e) { e.stopPropagation(); onClick(); });
    return b;
  }

  function render() {
    if (!overlay) return;
    var count = Math.max(1, D.slideCount());
    index = Math.max(0, Math.min(count - 1, index));
    var canvas = document.getElementById("slide-canvas");
    stage.textContent = "";
    if (canvas) {
      var clone = canvas.cloneNode(true);
      clone.removeAttribute("id");
      clone.style.width = "960px";
      clone.style.height = "540px";
      clone.style.transformOrigin = "top left";
      clone.style.pointerEvents = "none";
      // Fit the fixed 960x540 slide into the available stage box.
      var box = stage.getBoundingClientRect();
      var scale = box.width > 0 && box.height > 0
        ? Math.min(box.width / 960, box.height / 540) : 1;
      if (!isFinite(scale) || scale <= 0) scale = 1;
      clone.style.transform = "scale(" + scale + ")";
      stage.appendChild(clone);
    }
    if (counter) counter.textContent = (index + 1) + " / " + count;
  }

  function go(delta) {
    var count = Math.max(1, D.slideCount());
    var next = Math.max(0, Math.min(count - 1, index + delta));
    if (next === index) return;
    index = next;
    D.jump(index);
    // Navigation swaps the underlying canvas through HTMX; re-clone afterwards.
    window.setTimeout(render, RERENDER_MS);
  }

  function build() {
    overlay = document.createElement("div");
    overlay.id = ID;
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-label", t("slides-delivery-console-current", "Presentation"));
    overlay.style.cssText = "position:fixed;inset:0;z-index:var(--z-blocker,9999);background:#050914;" +
      "display:none;flex-direction:column;color:#f8fafc;touch-action:none;";

    var bar = document.createElement("div");
    bar.style.cssText = "display:flex;align-items:center;justify-content:space-between;" +
      "gap:12px;padding:8px 12px;min-height:60px;background:#0f172a;box-sizing:border-box;";
    counter = document.createElement("span");
    counter.setAttribute("role", "status");
    counter.setAttribute("aria-live", "polite");
    counter.style.cssText = "font-size:15px;font-weight:600;color:#c7d2fe;";
    bar.appendChild(counter);
    bar.appendChild(button("action-close", "Close", close, "#7f1d1d"));
    overlay.appendChild(bar);

    stage = document.createElement("div");
    stage.style.cssText = "flex:1;min-height:0;display:flex;align-items:center;justify-content:center;" +
      "overflow:hidden;padding:8px;box-sizing:border-box;";
    overlay.appendChild(stage);

    var nav = document.createElement("div");
    nav.style.cssText = "display:flex;align-items:center;justify-content:space-between;" +
      "gap:12px;padding:8px 12px;min-height:60px;background:#0f172a;box-sizing:border-box;";
    nav.appendChild(button("action-back", "Back", function () { go(-1); }, "#334155"));
    nav.appendChild(button("action-next", "Next", function () { go(1); }, "#3b82f6"));
    overlay.appendChild(nav);

    // Tap the left third to go back, the right third to advance. A horizontal
    // swipe of >=48px advances/back regardless of where it starts. Taps on the
    // nav buttons are stopped by their own handlers.
    var sx = 0, sy = 0, moved = false;
    overlay.addEventListener("pointerdown", function (e) {
      if (e.target.closest && e.target.closest("button")) return;
      sx = e.clientX; sy = e.clientY; moved = false;
    });
    overlay.addEventListener("pointermove", function (e) {
      if (Math.abs(e.clientX - sx) > 12 || Math.abs(e.clientY - sy) > 12) moved = true;
    });
    overlay.addEventListener("pointerup", function (e) {
      if (e.target.closest && e.target.closest("button")) return;
      var dx = e.clientX - sx;
      var dy = e.clientY - sy;
      if (Math.abs(dx) >= 48 && Math.abs(dx) > Math.abs(dy)) {
        go(dx < 0 ? 1 : -1);
      } else if (!moved) {
        go(e.clientX > window.innerWidth / 2 ? 1 : -1);
      }
    });

    document.body.appendChild(overlay);
  }

  function open(startIndex) {
    if (!overlay) build();
    index = typeof startIndex === "number" ? startIndex : D.currentSlide();
    overlay.style.display = "flex";
    render();
    if (document.documentElement.requestFullscreen && !document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(function () { /* best effort */ });
    }
  }

  function close() {
    if (overlay) overlay.style.display = "none";
    if (document.fullscreenElement && document.exitFullscreen) {
      document.exitFullscreen().catch(function () { /* best effort */ });
    }
  }

  document.addEventListener("click", function (e) {
    var b = e.target && e.target.closest && e.target.closest("#presenter-panel button");
    if (b && /Iniciar/i.test(b.textContent || "")) {
      window.setTimeout(function () { open(D.currentSlide()); }, 300);
    }
  }, true);

  document.addEventListener("keydown", function (e) {
    if (e.key === "F5") { e.preventDefault(); open(D.currentSlide()); }
    else if (e.key === "Escape" && overlay && overlay.style.display === "flex") close();
  });

  window.GBTouchPresent = { open: open, close: close, go: go };
})(window, document);
