/* WindowManager — focus, close, sizing and drag/resize (4/9). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;

  WM.defineMethods(WindowManager.prototype, {
  focus(id) {
    this.activeWindowId = id;
    const el = document.getElementById(`window-${id}`);
    if (el) {
      el.style.zIndex = this.zIndexCounter++;
      el.classList.add("window-focused");
    }
    this.openWindows.forEach((w) => {
      if (w.id !== id) {
        const other = document.getElementById(`window-${w.id}`);
        if (other) other.classList.remove("window-focused");
      }
    });

    const obj = this.openWindows.find((w) => w.id === id);
    if (obj) document.title = `${obj.title} - General Bots`;
    this._updateDockActive();
    document.dispatchEvent(new CustomEvent("gb-window-focus", { detail: obj || { id } }));
  },

  _updateDockActive() {
    const center = this.getTaskbarCenter();
    if (!center) return;
    center.querySelectorAll(".taskbar-dock-item").forEach((item) => item.classList.remove("active"));
    if (this.activeWindowId) {
      const active = document.getElementById(`dock-item-${this.activeWindowId}`);
      if (active) active.classList.add("active");
    }
  },

  close(id) {
    // Remember an explicit close of the Vibe workbench (localStorage) so
    // the auto-open on load (desktop.html ?app=vibe / ?vibe=) does not
    // fight the user: close the bar once, reload keeps it closed; opening
    // it again clears the flag so the next reload restores it.
    if (id === "vibe" && this.getWindow(id)) {
      try {
        localStorage.setItem("gb.vibe.closed", "1");
      } catch (e) {
        /* storage may be disabled — the close still works for the session */
      }
    }
    // Close owned tool windows first. This gives an IDE workbench a real
    // parent/child lifecycle: closing Vibe cannot leave a Browser, dialog,
    // terminal or run dock orphaned behind it.
    const children = this.openWindows
      .filter((w) => w.ownerId === id)
      .map((w) => w.id);
    children.forEach((childId) => this.close(childId));

    const el = document.getElementById(`window-${id}`);
    // #2104 — a closed window must not leave the desktop (or the OS window,
    // in the desktop shell) stuck in fullscreen.
    if (el && typeof this.exitFullscreen === "function" && this.isFullscreen(id)) {
      this.exitFullscreen(id);
    }
    if (el) {
      if (this.useGlassWindows) {
        el.classList.add("closing");
        setTimeout(() => el.remove(), 200);
      } else {
        el.remove();
      }
    }
    const dockEl = document.getElementById(`dock-item-${id}`);
    if (dockEl) dockEl.remove();
    this.openWindows = this.openWindows.filter((w) => w.id !== id);
    if (this.activeWindowId === id) {
      this.activeWindowId = null;
      this._updateDockActive();
      if (this.openWindows.length === 0) {
        document.title = window.gbT
          ? window.gbT("desktop-document-title", "General Bots Desktop")
          : "General Bots Desktop";
      }
    }
    document.dispatchEvent(new CustomEvent("gb-window-close", { detail: { id } }));
    document.dispatchEvent(new CustomEvent("gb-window-changed", { detail: { action: "close", id } }));
  },

  toggleMinimize(id) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (!obj) return;
    const el = document.getElementById(`window-${id}`);
    if (!el) return;
    // #2104 — minimizing leaves fullscreen first: a hidden window that still
    // owns the screen would be unreachable.
    if (typeof this.isFullscreen === "function" && this.isFullscreen(id)) {
      this.exitFullscreen(id);
    }
    if (obj.isMinimized) {
      el.style.display = "flex";
      obj.isMinimized = false;
      this.focus(id);
    } else {
      el.style.display = "none";
      obj.isMinimized = true;
      if (this.activeWindowId === id) {
        this.activeWindowId = null;
        this._updateDockActive();
      }
    }
  },

  toggleMaximize(id) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (!obj || obj.noMaximize) return;
    const el = document.getElementById(`window-${id}`);
    if (!el) return;
    if (obj.isMaximized) {
      el.style.width = obj.previousState.width;
      el.style.height = obj.previousState.height;
      el.style.top = obj.previousState.top;
      el.style.left = obj.previousState.left;
      el.style.borderRadius = "";
      el.classList.remove("window-maximized-target");
      obj.isMaximized = false;
      const anyMaximized = this.openWindows.some((w) => w.isMaximized);
      if (!anyMaximized) document.body.classList.remove("window-maximized");
    } else {
      obj.previousState = { width: el.style.width, height: el.style.height, top: el.style.top, left: el.style.left };
      el.style.width = "100%";
      el.style.height = "100%";
      el.style.top = "0";
      el.style.left = "0";
      el.style.borderRadius = "0";
      el.classList.add("window-maximized-target");
      obj.isMaximized = true;
      document.body.classList.add("window-maximized");
    }
    this._saveWindowPreference(id, obj.isMaximized);
    this.focus(id);
  },

  /// Remember the maximized state per app so a reopened window keeps the
  /// user's choice across reloads. Chat defaults to maximized on first use.
  _baseAppId(id) {
    return String(id || "").split("-")[0];
  },

  _windowPreferences() {
    try {
      return gbStoreGet("gb.window.prefs.v1", {}) || {};
    } catch (e) {
      return {};
    }
  },

  _saveWindowPreference(id, maximized) {
    const base = this._baseAppId(id);
    try {
      const prefs = this._windowPreferences();
      prefs[base] = Object.assign({}, prefs[base], { maximized: !!maximized });
      localStorage.setItem("gb.window.prefs.v1", JSON.stringify(prefs));
    } catch (e) {
      /* storage may be disabled; the toggle still works for the session */
    }
  },

  _applyWindowPreference(id) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (!obj || obj.isMaximized) return;
    const base = this._baseAppId(id);
    const prefs = this._windowPreferences();
    const known = prefs[base] && typeof prefs[base].maximized === "boolean";
    // First-ever open: the conversation surface fills the workspace.
    const wantMax = known ? prefs[base].maximized : base === "chat";
    if (wantMax) this.toggleMaximize(id);
  },

  _makeDraggable(el) {
    const header = el.querySelector(".window-header-glass") || el.querySelector(".window-header");
    if (!header) return;
    let isDragging = false, startX, startY, initialLeft, initialTop, activePointerId = null, moveRaf = 0;

    const isCompactSurface = () => {
      // #2410: phone/compact surfaces keep sheet geometry pinned.
      return window.innerWidth < 768;
    };

    const canFreeDrag = () => {
      // #2410: compact phone surfaces must not free-drag; tablet/coarse-pointer
      // surfaces (>=768px) still allow pointer-driven drag per acceptance.
      return !isCompactSurface();
    };

    const endDrag = () => {
      if (!isDragging) return;
      isDragging = false;
      if (moveRaf) { cancelAnimationFrame(moveRaf); moveRaf = 0; }
      document.body.style.userSelect = "";
      if (activePointerId != null) {
        try { header.releasePointerCapture(activePointerId); } catch (_e) {}
      }
      activePointerId = null;
      header.removeEventListener("pointermove", onMove);
      header.removeEventListener("pointerup", onUp);
      header.removeEventListener("pointercancel", onUp);
      document.removeEventListener("pointermove", onMove);
      document.removeEventListener("pointerup", onUp);
      document.removeEventListener("pointercancel", onUp);
    };

    const onDown = (e) => {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      if (e.target.closest(".window-dot") || e.target.closest("button")) return;
      const id = el.id.replace("window-", "");
      this.focus(id);
      if (!canFreeDrag()) return;
      // #2104 — a fullscreen window owns the screen: no drag, no restore.
      if (this.isFullscreen && this.isFullscreen(id)) return;
      const wd = this.openWindows.find((w) => w.id === id);
      if (wd && wd.isMaximized && !wd.noMaximize) this.toggleMaximize(id);
      // Re-read after possible restore.
      isDragging = true;
      activePointerId = e.pointerId;
      startX = e.clientX; startY = e.clientY;
      initialLeft = parseInt(el.style.left || 0, 10);
      initialTop = parseInt(el.style.top || 0, 10);
      document.body.style.userSelect = "none";
      try { header.setPointerCapture(e.pointerId); } catch (_e) {}
      // Listen on header and document for robustness (synthetic events may not honor capture).
      header.addEventListener("pointermove", onMove);
      header.addEventListener("pointerup", onUp);
      header.addEventListener("pointercancel", onUp);
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
      document.addEventListener("pointercancel", onUp);
    };

    const onMove = (e) => {
      if (!isDragging) return;
      if (e.pointerId !== activePointerId) return;
      if (!canFreeDrag()) { endDrag(); return; }
      const nx = Math.max(-200, initialLeft + e.clientX - startX);
      const ny = Math.max(0, initialTop + e.clientY - startY);
      // Apply immediately for deterministic behavior; rAF keeps last value consistent.
      el.style.left = `${nx}px`;
      el.style.top = `${ny}px`;
      if (moveRaf) cancelAnimationFrame(moveRaf);
      moveRaf = requestAnimationFrame(() => { moveRaf = 0; });
    };

    const onUp = (e) => {
      if (e && activePointerId != null && e.pointerId !== activePointerId) return;
      endDrag();
    };

    header.style.touchAction = "none";
    header.addEventListener("pointerdown", onDown);
    header.addEventListener("dblclick", (e) => {
      if (e.target.closest(".window-dot") || e.target.closest("button")) return;
      if (!canFreeDrag()) return;
      const id = el.id.replace("window-", "");
      // #2104 — double-click must not maximize (and so shrink) a fullscreen
      // window; Escape/F11 leave fullscreen instead.
      if (this.isFullscreen && this.isFullscreen(id)) return;
      const wd = this.openWindows.find((w) => w.id === id);
      if (wd && !wd.noMaximize) this.toggleMaximize(id);
    });
    // Focus on pointer interaction without preventing app click handlers.
    el.addEventListener("pointerdown", (e) => {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      this.focus(el.id.replace("window-", ""));
    }, true);
  },

  _makeResizable(el) {
    // Fixed popup windows (e.g. Vibe New Project / Members) are NOT
    // resizable (product spec): no resize handle and no window scrollbar —
    // all fields are visible at once. Skip the CSS resize + auto overflow
    // so a popup never grows a corner grip or clips its content.
    if (el.classList.contains("window-popup")) {
      el.style.resize = "none";
      el.style.overflow = "hidden";
      return;
    }
    const coarse = window.matchMedia && window.matchMedia("(pointer: coarse)").matches;
    const compact = window.innerWidth < 768;
    // #2410: avoid browser-native resizer on compact/coarse-touch surfaces;
    // fine pointer desktop keeps native behavior.
    el.style.resize = (compact || coarse) ? "none" : "both";
    // Only the inner .window-body scrolls. Making the window element itself
    // `overflow: auto` produced a second, outer scrollbar (title bar and
    // body scrolling together) whenever content exceeded the frame height.
    el.style.overflow = "hidden";
  },

  });
})(window);
