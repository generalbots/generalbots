"use strict";

// GB Snap Assist (#1155): Windows-11 style window snapping for the desktop
// shell. Provides:
//   1. Hover-maximize layout picker (halves, quarters, thirds, full).
//   2. Drag-to-edge ghost overlay with release snapping.
//   3. Keyboard snapping: Alt+Left/Right (half), Alt+Up/Down (cycle),
//      Shift = third.
//   4. Snap side-by-side memory: snapping one window left leaves the other
//      side open for the next window (classic Windows 7 behavior).
//   5. Restore: dragging a snapped window back returns its previous geometry.
//
// All geometry is applied through WindowManager.snapWindow(id, layout) so the
// window data model stays the single source of truth.

window.GBSnapAssist = window.GBSnapAssist || {};

(function (mod) {
  var ghost = null;
  var dragState = null;
  var EDGE = 12; // pixels from the screen edge that trigger snapping

  var LAYOUTS = {
    left: { x: 0, y: 0, w: 0.5, h: 1 },
    right: { x: 0.5, y: 0, w: 0.5, h: 1 },
    top: { x: 0, y: 0, w: 1, h: 0.5 },
    bottom: { x: 0, y: 0.5, w: 1, h: 0.5 },
    "top-left": { x: 0, y: 0, w: 0.5, h: 0.5 },
    "top-right": { x: 0.5, y: 0, w: 0.5, h: 0.5 },
    "bottom-left": { x: 0, y: 0.5, w: 0.5, h: 0.5 },
    "bottom-right": { x: 0.5, y: 0.5, w: 0.5, h: 0.5 },
    "third-left": { x: 0, y: 0, w: 1 / 3, h: 1 },
    "third-right": { x: 2 / 3, y: 0, w: 1 / 3, h: 1 },
    "two-thirds": { x: 1 / 3, y: 0, w: 2 / 3, h: 1 },
    full: { x: 0, y: 0, w: 1, h: 1 },
  };

  function wm() {
    return window.WindowManager || null;
  }

  // ── Layout picker on the maximize dot ────────────────────────

  var picker = null;

  function openPicker(id, anchorEl) {
    closePicker();
    var app = (window.APPS_REGISTRY || []).find(function (a) { return a.id === id; });
    var title = app ? app.title : id;
    picker = document.createElement("div");
    picker.className = "gb-snap-picker";
    picker.setAttribute("role", "menu");
    picker.innerHTML =
      '<div class="gb-snap-picker-title">Snap "' + title + '"</div>' +
      '<div class="gb-snap-picker-grid">' +
      '<button data-layout="top-left" title="Top left"></button>' +
      '<button data-layout="top" title="Top half"></button>' +
      '<button data-layout="top-right" title="Top right"></button>' +
      '<button data-layout="left" title="Left half"></button>' +
      '<button data-layout="full" title="Maximize"></button>' +
      '<button data-layout="right" title="Right half"></button>' +
      '<button data-layout="bottom-left" title="Bottom left"></button>' +
      '<button data-layout="bottom" title="Bottom half"></button>' +
      '<button data-layout="bottom-right" title="Bottom right"></button>' +
      '<button data-layout="third-left" title="Left third"></button>' +
      '<button data-layout="two-thirds" title="Two thirds"></button>' +
      '<button data-layout="third-right" title="Right third"></button>' +
      "</div>";
    var rect = anchorEl.getBoundingClientRect();
    picker.style.left = Math.min(rect.left, window.innerWidth - 220) + "px";
    picker.style.top = rect.bottom + 6 + "px";

    picker.addEventListener("click", function (e) {
      var btn = e.target.closest("button[data-layout]");
      if (!btn) return;
      applyLayout(id, btn.getAttribute("data-layout"));
      closePicker();
    });
    document.addEventListener("click", closePicker, { once: true });
    document.body.appendChild(picker);
  }

  function closePicker() {
    if (picker) { picker.remove(); picker = null; }
  }

  function applyLayout(id, layout) {
    var manager = wm();
    if (!manager || !LAYOUTS[layout]) return;
    var obj = manager.openWindows.find(function (w) { return w.id === id; });
    if (!obj) return;
    // Maximize dot toggles fullscreen; picker "full" reuses the same path.
    var el = document.getElementById("window-" + id);
    if (!el) return;
    if (layout === "full") {
      if (!obj.isMaximized) manager.toggleMaximize(id);
      return;
    }
    manager.snapWindow(id, layout === "two-thirds" ? "third-right" : layout);
    // Side-by-side memory: after a half snap, remember the open side so the
    // next window snapped there fills it (Windows 7 behavior).
    if (layout === "left" || layout === "right") {
      mod.pendingSide = layout === "left" ? "right" : "left";
    }
  }

  // ── Ghost overlay + edge drag ────────────────────────────────

  function showGhost(layout) {
    hideGhost();
    ghost = document.createElement("div");
    ghost.className = "gb-snap-ghost";
    ghost.setAttribute("data-layout", layout);
    applyGhostGeometry(layout);
    document.body.appendChild(ghost);
  }

  function applyGhostGeometry(layout) {
    if (!ghost) return;
    var l = LAYOUTS[layout];
    if (!l) return;
    var pad = 4;
    ghost.style.left = (l.x * window.innerWidth + pad) + "px";
    ghost.style.top = (l.y * window.innerHeight + pad) + "px";
    ghost.style.width = (l.w * window.innerWidth - pad * 2) + "px";
    ghost.style.height = (l.h * window.innerHeight - pad * 2) + "px";
  }

  function hideGhost() {
    if (ghost) { ghost.remove(); ghost = null; }
  }

  function layoutForPointer(x, y) {
    var w = window.innerWidth, h = window.innerHeight;
    var res = [];
    if (x < EDGE) res.push("left");
    if (x > w - EDGE) res.push("right");
    if (y < EDGE) res.push("top");
    if (y > h - EDGE) res.push("bottom");
    if (res.length === 0) return null;
    var primary = res[0];
    // Corner quarters combine edge hits.
    if (res.length === 2) {
      var a = res[0], b = res[1];
      var map = {
        "left,top": "top-left", "top,left": "top-left",
        "right,top": "top-right", "top,right": "top-right",
        "left,bottom": "bottom-left", "bottom,left": "bottom-left",
        "right,bottom": "bottom-right", "bottom,right": "bottom-right",
      };
      return map[primary + "," + res[1]] || primary;
    }
    return primary;
  }

  function beginDrag(id, el) {
    // Restore a snapped window before dragging so it can be dropped anywhere.
    var manager = wm();
    var obj = manager.openWindows.find(function (w) { return w.id === id; });
    if (obj && obj.snapLayout) {
      manager.restoreWindow(id);
    }
    dragState = { id: id, x: 0, y: 0 };
    document.addEventListener("mousemove", onDragMove);
    document.addEventListener("mouseup", onDragUp);
  }

  function onDragMove(e) {
    if (!dragState) return;
    dragState.x = e.clientX;
    dragState.y = e.clientY;
    var layout = layoutForPointer(e.clientX, e.clientY);
    if (layout) {
      showGhost(layout);
    } else {
      hideGhost();
    }
  }

  function onDragUp() {
    if (!dragState) return;
    var layout = layoutForPointer(dragState.x, dragState.y);
    if (layout) {
      var manager = wm();
      var el = document.getElementById("window-" + dragState.id);
      if (el && manager) {
        var l = LAYOUTS[layout];
        var pad = 4;
        el.style.left = (l.x * window.innerWidth + pad) + "px";
        el.style.top = (l.y * window.innerHeight + pad) + "px";
        el.style.width = (l.w * window.innerWidth - pad * 2) + "px";
        el.style.height = (l.h * window.innerHeight - pad * 2) + "px";
        el.style.borderRadius = "0";
        var obj = manager.openWindows.find(function (w) { return w.id === dragState.id; });
        if (obj) {
          obj.isMaximized = false;
          obj.snapLayout = layout;
          obj.previousState = { width: el.style.width, height: el.style.height, top: el.style.top, left: el.style.left };
        }
        if (layout === "left" || layout === "right") {
          mod.pendingSide = layout === "left" ? "right" : "left";
        }
        window.dispatchEvent(new CustomEvent("gb-snap-applied", { detail: { id: dragState.id, layout: layout } }));
      }
    }
    hideGhost();
    dragState = null;
    document.removeEventListener("mousemove", onDragMove);
    document.removeEventListener("mouseup", onDragUp);
  }

  // ── Keyboard snapping ────────────────────────────────────────

  function onKeyDown(e) {
    if (!e.altKey || e.ctrlKey || e.metaKey) return;
    var manager = wm();
    if (!manager || !manager.activeWindowId) return;
    var key = e.key;
    var shift = e.shiftKey;
    var layout = null;

    if (key === "ArrowLeft") layout = shift ? "third-left" : "left";
    else if (key === "ArrowRight") layout = shift ? "third-right" : "right";
    else if (key === "ArrowUp") layout = shift ? "top" : "full";
    else if (key === "ArrowDown") layout = "bottom";

    if (!layout) return;
    e.preventDefault();
    applyLayout(manager.activeWindowId, layout);
  }

  // ── Init ─────────────────────────────────────────────────────

  mod.init = function () {
    if (mod.initialized) return;
    mod.initialized = true;

    // Wire the maximize dot: single click toggles max (existing behavior),
    // hover opens the layout picker.
    document.addEventListener("mouseover", function (e) {
      var dot = e.target.closest(".window-dot-maximize");
      if (!dot) return;
      var header = dot.closest(".window-header-glass");
      if (!header) return;
      var el = header.closest("[id^=window-]");
      if (!el) return;
      var id = el.id.replace("window-", "");
      // Delay so a quick click still toggles maximize instead of opening the
      // picker; a sustained hover (>=350ms) opens the picker.
      clearTimeout(mod.__hoverTimer);
      mod.__hoverTimer = setTimeout(function () { openPicker(id, dot); }, 350);
    });
    document.addEventListener("mouseout", function (e) {
      if (e.target.closest(".window-dot-maximize")) {
        clearTimeout(mod.__hoverTimer);
      }
    });
    document.addEventListener("click", function (e) {
      var dot = e.target.closest(".window-dot-maximize");
      if (!dot) return;
      clearTimeout(mod.__hoverTimer);
      closePicker();
    });

    // Hook titlebar drag to enable edge snapping.
    document.addEventListener("mousedown", function (e) {
      var header = e.target.closest(".window-header-glass") || e.target.closest(".window-header");
      if (!header) return;
      if (e.target.closest(".window-dot") || e.target.closest("button")) return;
      var el = header.closest("[id^=window-]");
      if (!el) return;
      var id = el.id.replace("window-", "");
      // Only engage for normal (non-maximized) windows; maximized windows
      // restore via WindowManager's own drag handler.
      var obj = wm().openWindows.find(function (w) { return w.id === id; });
      if (obj && !obj.isMaximized) {
        beginDrag(id, el);
      }
    });

    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", function () {
      if (ghost) applyGhostGeometry(ghost.getAttribute("data-layout"));
    });
  };
})(window.GBSnapAssist);
