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
    // #2410 — maximize/fullscreen toggles end in focus(); keep the resize
    // grip visibility in step with the new state.
    if (el) this._syncResizeGrip(el);
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

  // #2410 — a drag/resize that actually moved must not also fire the app's
  // own click handlers on release (e.g. opening a file after moving it).
  _suppressGestureClick() {
    const block = (e) => {
      e.preventDefault();
      e.stopPropagation();
    };
    const remove = () => {
      clearTimeout(timer);
      document.removeEventListener("click", block, true);
      document.removeEventListener("dblclick", block, true);
    };
    const timer = setTimeout(remove, 250);
    document.addEventListener("click", block, true);
    document.addEventListener("dblclick", block, true);
  },

  // Show the real grip only where resizing is allowed. Called on every clamp
  // so a rotate/keyboard change updates the affordance with the geometry.
  _syncResizeGrip(el) {
    const grip = el.querySelector(":scope > .window-resize-grip");
    if (!grip) return;
    const id = el.id.replace("window-", "");
    const wd = this.openWindows.find((w) => w.id === id);
    const show = !this._isSheetMode()
      && !el.classList.contains("window-settings-modal")
      && !el.classList.contains("window-popup")
      && !(this.isFullscreen && this.isFullscreen(id))
      && !(wd && wd.isMaximized && !wd.noMaximize);
    grip.style.display = show ? "" : "none";
    grip.setAttribute("aria-hidden", show ? "false" : "true");
    grip.setAttribute("tabindex", show ? "0" : "-1");
  },

  _makeDraggable(el) {
    const header = el.querySelector(".window-header-glass") || el.querySelector(".window-header");
    if (!header) return;
    let dragging = false, pointerId = null, startX = 0, startY = 0;
    let initialLeft = 0, initialTop = 0, moved = false, previousUserSelect = "";

    const endDrag = () => {
      if (!dragging) return;
      dragging = false;
      document.body.style.userSelect = previousUserSelect;
      if (pointerId != null) {
        try { header.releasePointerCapture(pointerId); } catch (_e) {}
      }
      pointerId = null;
      document.removeEventListener("pointermove", onMove);
      document.removeEventListener("pointerup", onUp);
      document.removeEventListener("pointercancel", onUp);
      if (moved) this._suppressGestureClick();
      moved = false;
    };

    const onDown = (e) => {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      if (e.target.closest(".window-dot") || e.target.closest("button") || e.target.closest(".window-resize-grip")) return;
      const id = el.id.replace("window-", "");
      this.focus(id);
      // #2410 — coarse phones are pinned sheets, never free-dragged.
      if (this._isSheetMode()) return;
      // #2104 — a fullscreen window owns the screen: no drag, no restore.
      if (this.isFullscreen && this.isFullscreen(id)) return;
      const wd = this.openWindows.find((w) => w.id === id);
      if (wd && wd.isMaximized && !wd.noMaximize) this.toggleMaximize(id);
      dragging = true;
      moved = false;
      pointerId = e.pointerId;
      startX = e.clientX;
      startY = e.clientY;
      initialLeft = parseInt(el.style.left || "0", 10) || 0;
      initialTop = parseInt(el.style.top || "0", 10) || 0;
      previousUserSelect = document.body.style.userSelect;
      document.body.style.userSelect = "none";
      try { header.setPointerCapture(e.pointerId); } catch (_e) {}
      // Registered on document: captured pointer events target the header but
      // still bubble here, so synthetic streams without capture also work.
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
      document.addEventListener("pointercancel", onUp);
    };

    const onMove = (e) => {
      if (!dragging || e.pointerId !== pointerId) return;
      if (this._isSheetMode()) { endDrag(); return; }
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) moved = true;
      el.style.left = `${Math.max(-200, initialLeft + dx)}px`;
      el.style.top = `${Math.max(0, initialTop + dy)}px`;
    };

    const onUp = (e) => {
      if (e && pointerId != null && e.pointerId !== pointerId) return;
      endDrag();
    };

    // touch-action:none belongs to this drag handle ONLY; the window body
    // keeps its own scrolling on touch devices.
    header.style.touchAction = "none";
    header.addEventListener("pointerdown", onDown);
    header.addEventListener("dblclick", (e) => {
      if (e.target.closest(".window-dot") || e.target.closest("button")) return;
      if (this._isSheetMode()) return;
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
    // all fields are visible at once.
    if (el.classList.contains("window-popup")) {
      el.style.resize = "none";
      el.style.overflow = "hidden";
      return;
    }
    // #2410 — the browser-native CSS resizer is unreachable on touch and has
    // no accessible grip. A real >=44px bottom-right grip owns resize for
    // both pointer types; only the inner .window-body scrolls (no outer bar).
    el.style.resize = "none";
    el.style.overflow = "hidden";
    let grip = el.querySelector(":scope > .window-resize-grip");
    if (!grip) {
      grip = document.createElement("div");
      grip.className = "window-resize-grip";
      grip.setAttribute("role", "button");
      grip.setAttribute("tabindex", "0");
      grip.setAttribute("data-i18n-aria-label", "window-resize-grip");
      grip.setAttribute("aria-label", window.gbT
        ? window.gbT("window-resize-grip", "Resize window")
        : "Resize window");
      el.appendChild(grip);
    }
    this._syncResizeGrip(el);

    let resizing = false, pointerId = null, startX = 0, startY = 0;
    let startW = 0, startH = 0, moved = false, previousUserSelect = "";
    const id = el.id.replace("window-", "");

    const canResize = () => {
      if (this._isSheetMode()) return false;
      if (el.classList.contains("window-settings-modal")) return false;
      if (this.isFullscreen && this.isFullscreen(id)) return false;
      const wd = this.openWindows.find((w) => w.id === id);
      return !(wd && wd.isMaximized && !wd.noMaximize);
    };

    const endResize = () => {
      if (!resizing) return;
      resizing = false;
      document.body.style.userSelect = previousUserSelect;
      if (pointerId != null) {
        try { grip.releasePointerCapture(pointerId); } catch (_e) {}
      }
      pointerId = null;
      document.removeEventListener("pointermove", onResizeMove);
      document.removeEventListener("pointerup", onResizeUp);
      document.removeEventListener("pointercancel", onResizeUp);
      if (moved) this._suppressGestureClick();
      moved = false;
      this._clampWindowIntoView(id);
    };

    const onResizeMove = (e) => {
      if (!resizing || e.pointerId !== pointerId) return;
      if (!canResize()) { endResize(); return; }
      const b = this._viewportBounds();
      const left = parseInt(el.style.left || "0", 10) || 0;
      const top = parseInt(el.style.top || "0", 10) || 0;
      const maxW = Math.max(320, b.width - b.margin * 2 - left);
      const maxH = Math.max(200, b.bottom - top - b.margin);
      const w = Math.min(Math.max(320, startW + e.clientX - startX), maxW);
      const h = Math.min(Math.max(200, startH + e.clientY - startY), maxH);
      if (Math.abs(w - startW) > 3 || Math.abs(h - startH) > 3) moved = true;
      el.style.width = `${Math.floor(w)}px`;
      el.style.height = `${Math.floor(h)}px`;
    };

    const onResizeUp = (e) => {
      if (e && pointerId != null && e.pointerId !== pointerId) return;
      endResize();
    };

    grip.addEventListener("pointerdown", (e) => {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      if (!canResize()) return;
      e.preventDefault();
      e.stopPropagation();
      this.focus(id);
      resizing = true;
      moved = false;
      pointerId = e.pointerId;
      startX = e.clientX;
      startY = e.clientY;
      const rect = el.getBoundingClientRect();
      startW = rect.width;
      startH = rect.height;
      previousUserSelect = document.body.style.userSelect;
      document.body.style.userSelect = "none";
      try { grip.setPointerCapture(e.pointerId); } catch (_e) {}
      document.addEventListener("pointermove", onResizeMove);
      document.addEventListener("pointerup", onResizeUp);
      document.addEventListener("pointercancel", onResizeUp);
    });

    // Keyboard equivalent: arrows resize in 16px steps (Shift = 48px).
    grip.addEventListener("keydown", (e) => {
      if (!canResize()) return;
      const step = e.shiftKey ? 48 : 16;
      const delta = { ArrowRight: [step, 0], ArrowLeft: [-step, 0], ArrowDown: [0, step], ArrowUp: [0, -step] }[e.key];
      if (!delta) return;
      e.preventDefault();
      const rect = el.getBoundingClientRect();
      const b = this._viewportBounds();
      const left = parseInt(el.style.left || "0", 10) || 0;
      const top = parseInt(el.style.top || "0", 10) || 0;
      const maxW = Math.max(320, b.width - b.margin * 2 - left);
      const maxH = Math.max(200, b.bottom - top - b.margin);
      const w = Math.min(Math.max(320, rect.width + delta[0]), maxW);
      const h = Math.min(Math.max(200, rect.height + delta[1]), maxH);
      el.style.width = `${Math.floor(w)}px`;
      el.style.height = `${Math.floor(h)}px`;
      this._clampWindowIntoView(id);
    });
  },

  });
})(window);
