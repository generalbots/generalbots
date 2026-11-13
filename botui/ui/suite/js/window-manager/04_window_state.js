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
    let isDragging = false, startX, startY, initialLeft, initialTop;

    const onDown = (e) => {
      if (e.target.closest(".window-dot") || e.target.closest("button")) return;
      // Dragging a maximized window is nonsensical (it fills the screen),
      // so restore it to its previous size/position first — standard
      // desktop behavior.
      const id = el.id.replace("window-", "");
      // #2104 — a fullscreen window owns the screen: no drag, no restore.
      if (this.isFullscreen && this.isFullscreen(id)) return;
      const wd = this.openWindows.find((w) => w.id === id);
      if (wd && wd.isMaximized && !wd.noMaximize) this.toggleMaximize(id);
      isDragging = true;
      startX = e.clientX; startY = e.clientY;
      initialLeft = parseInt(el.style.left || 0, 10);
      initialTop = parseInt(el.style.top || 0, 10);
      this.focus(id);
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    };
    const onMove = (e) => {
      if (!isDragging) return;
      requestAnimationFrame(() => {
        el.style.left = `${Math.max(-200, initialLeft + e.clientX - startX)}px`;
        el.style.top = `${Math.max(0, initialTop + e.clientY - startY)}px`;
      });
    };
    const onUp = () => {
      isDragging = false;
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
    };
    header.addEventListener("mousedown", onDown);
    header.addEventListener("dblclick", (e) => {
      if (e.target.closest(".window-dot") || e.target.closest("button")) return;
      const id = el.id.replace("window-", "");
      // #2104 — double-click must not maximize (and so shrink) a fullscreen
      // window; Escape/F11 leave fullscreen instead.
      if (this.isFullscreen && this.isFullscreen(id)) return;
      const wd = this.openWindows.find((w) => w.id === id);
      if (wd && !wd.noMaximize) this.toggleMaximize(id);
    });
    el.addEventListener("mousedown", () => this.focus(el.id.replace("window-", "")));
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
    el.style.resize = "both";
    // Only the inner .window-body scrolls. Making the window element itself
    // `overflow: auto` produced a second, outer scrollbar (title bar and
    // body scrolling together) whenever content exceeded the frame height.
    el.style.overflow = "hidden";
  },

  });
})(window);
