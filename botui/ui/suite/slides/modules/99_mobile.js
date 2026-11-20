"use strict";
/* #2515 — Slides touch model. Desktop untouched: drawer, fit-to-width canvas
   and gestures install on coarse pointers after the htmx canvas-host swap. */

(function () {
  var app = document.getElementById("slides-app");
  if (!app) return;
  var main = app.querySelector(".slides-main");
  var toolbarLeft = app.querySelector(".slides-toolbar-row-main .toolbar-left");
  var tabs = main && main.querySelector(".slides-sidebar .sidebar-tabs");

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function icon(paths) {
    return '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">' + paths + "</svg>";
  }

  var toggle = document.createElement("button");
  toggle.type = "button";
  toggle.id = "slidesSidebarToggle";
  toggle.className = "btn-icon gb-mobile-sidebar-toggle";
  toggle.setAttribute("aria-controls", "slides-sidebar");
  toggle.setAttribute("aria-expanded", "false");
  toggle.setAttribute("data-i18n-aria-label", "paper-toggle-sidebar");
  toggle.setAttribute("aria-label", t("paper-toggle-sidebar", "Toggle sidebar"));
  toggle.innerHTML = icon('<line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>');

  var backdrop = document.createElement("div");
  backdrop.className = "slides-backdrop";
  backdrop.id = "slidesSidebarBackdrop";
  backdrop.setAttribute("aria-hidden", "true");

  var close = document.createElement("button");
  close.type = "button";
  close.id = "slidesSidebarClose";
  close.className = "btn-icon gb-mobile-sidebar-toggle gb-mobile-sidebar-close";
  close.setAttribute("data-i18n-aria-label", "action-close");
  close.setAttribute("aria-label", t("action-close", "Close"));
  close.innerHTML = icon('<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>');

  if (toolbarLeft) toolbarLeft.insertBefore(toggle, toolbarLeft.firstChild);
  if (main) main.insertBefore(backdrop, main.firstChild);
  if (tabs) tabs.insertBefore(close, tabs.firstChild);

  function setDrawer(open) {
    if (!main) return;
    main.classList.toggle("sidebar-open", open);
    toggle.setAttribute("aria-expanded", open ? "true" : "false");
  }
  toggle.addEventListener("click", function () { setDrawer(!main.classList.contains("sidebar-open")); });
  close.addEventListener("click", function () { setDrawer(false); });
  backdrop.addEventListener("click", function () { setDrawer(false); });
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && main && main.classList.contains("sidebar-open")) setDrawer(false);
  });

  var isTouch = (function () {
    try {
      return !!(window.matchMedia && (window.matchMedia("(pointer: coarse)").matches ||
        window.matchMedia("(max-width: 760px)").matches));
    } catch (e) { return false; }
  })();
  if (!isTouch) return;

  var scroll = null;
  var userZoom = 1;

  // Fit the fixed 960x540 slide to the phone by setting `zoom` on the canvas.
  // Zoom keeps hit-testing aligned with the scale-aware drag fix in 03_canvas.js.
  function fit() {
    scroll = main.querySelector(".sl-canvas-scroll") || main.querySelector(".sl-canvas-container");
    if (!scroll) { setTimeout(fit, 200); return; }
    var canvas = scroll.querySelector(".sl-canvas");
    if (!canvas) { setTimeout(fit, 200); return; }
    var available = Math.max(240, scroll.clientWidth - 24);
    var maxScale = available / 960;
    var s = Math.max(0.25, Math.min(maxScale * userZoom, 2));
    if (!isFinite(s) || s <= 0) s = 1;
    canvas.style.zoom = String(s);
    if (window.SlideCanvas) window.SlideCanvas.scale = s;
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { setTimeout(fit, 300); });
  } else {
    setTimeout(fit, 300);
  }
  window.addEventListener("resize", fit);
  window.addEventListener("orientationchange", function () { setTimeout(fit, 300); });

  // Two fingers pan the canvas and pinch-adjust zoom. A second pointer also
  // ends any object drag started by the first finger, so a pan can never be
  // mistaken for a move (acceptance: two-finger pan does not select).
  var pointers = {};
  var multi = false;
  var lastMid = null;
  var lastDist = 0;

  function points() {
    var ids = Object.keys(pointers);
    return ids.slice(0, 2).map(function (id) { return pointers[id]; });
  }
  function midpoint(pts) {
    return { x: (pts[0].x + pts[1].x) / 2, y: (pts[0].y + pts[1].y) / 2 };
  }
  function distance(pts) {
    return Math.sqrt(Math.pow(pts[0].x - pts[1].x, 2) + Math.pow(pts[0].y - pts[1].y, 2));
  }

  if (main) {
    main.addEventListener("pointerdown", function (e) {
      if (e.pointerType !== "touch" || !e.target.closest || !e.target.closest(".sl-canvas")) return;
      pointers[e.pointerId] = { x: e.clientX, y: e.clientY };
      if (Object.keys(pointers).length === 2) {
        multi = true;
        window.dispatchEvent(new PointerEvent("pointerup", { bubbles: true }));
        var pts = points();
        lastMid = midpoint(pts);
        lastDist = distance(pts);
        e.stopPropagation();
      } else if (multi) {
        e.stopPropagation();
      }
    }, true);

    main.addEventListener("pointermove", function (e) {
      if (e.pointerType !== "touch" || !pointers[e.pointerId]) return;
      pointers[e.pointerId] = { x: e.clientX, y: e.clientY };
      var pts = points();
      scroll = main.querySelector(".sl-canvas-scroll") || main.querySelector(".sl-canvas-container");
      if (!multi || pts.length < 2 || !scroll) return;
      e.preventDefault();
      var mid = midpoint(pts);
      var dist = distance(pts);
      scroll.scrollLeft -= mid.x - lastMid.x;
      scroll.scrollTop -= mid.y - lastMid.y;
      if (lastDist > 0) {
        userZoom = Math.max(0.5, Math.min(3, userZoom * (dist / lastDist)));
        fit();
      }
      lastMid = mid;
      lastDist = dist;
    }, { passive: false, capture: true });

    function endPointer(e) {
      delete pointers[e.pointerId];
      if (Object.keys(pointers).length < 2) {
        multi = false;
        lastMid = null;
        lastDist = 0;
      }
    }
    main.addEventListener("pointerup", endPointer, true);
    main.addEventListener("pointercancel", endPointer, true);
  }
})();
