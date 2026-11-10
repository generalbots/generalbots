/* WindowManager — class core: window creation and chrome entry points (2/9). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var APPS_REGISTRY = WM.APPS_REGISTRY;
  var CATEGORY_LABELS = WM.CATEGORY_LABELS;
  var isCoreAppId = WM.isCoreAppId;
  var isPreviewId = WM.isPreviewId;

class WindowManager {
  constructor() {
    this.openWindows = [];
    this.activeWindowId = null;
    this.zIndexCounter = 100;
    this.workspace = null;
    this.taskbarCenter = null;
    this.startMenuOpen = false;
    this.useGlassWindows = true;
      this._startMenuKeyHandler = (e) => {
        if (e.key === "Escape") this.closeStartMenu();
      };
  }

  getWorkspace() {
    if (!this.workspace) {
      this.workspace = document.getElementById("desktop-content") || document.body;
    }
    return this.workspace;
  }

  getTaskbarCenter() {
    if (!this.taskbarCenter) {
      this.taskbarCenter = document.getElementById("taskbar-center");
    }
    return this.taskbarCenter;
  }

  getIconSvg(id) {
    const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
    if (app) return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${app.icon}</svg>`;
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/></svg>';
  }

  // Tool windows (VB6/Delphi-style floating palettes) pass opts.tool:
  // they get a slim title bar, NO status bar (no "client portion") and a
  // smaller default footprint. opts.popup makes an even smaller, compact
  // popup-sized tool window (e.g. the Vibe Members list).
  open(id, title, htmlContent, opts) {
    opts = opts || {};
    // #1614/#1615 — an app the product profile hides must not open through
    // any route (launcher, palette, deep link, direct hash). Block it with
    // an explicit message instead of rendering an unfinished surface.
    if (typeof window.isAppHidden === "function" && window.isAppHidden(id)) {
      if (window.GBToasts && typeof window.GBToasts.show === "function") {
        window.GBToasts.show("Not available", "This application is not part of this edition.", "info");
      }
      return;
    }
    const existingWindow = this.openWindows.find((w) => w.id === id);
    if (existingWindow) {
      if (opts.ownerId) existingWindow.ownerId = opts.ownerId;
      this.focus(id);
      return;
    }
    // The user explicitly opened the Vibe workbench again (launcher,
    // command palette, deep link): forget the remembered closed state so
    // the next page load restores the bar instead of skipping it.
    if (id === "vibe") {
      try {
        localStorage.removeItem("gb.vibe.closed");
      } catch (e) {
        /* storage may be disabled — the open still works */
      }
    }

    const windowData = {
      id,
      title,
      ownerId: opts.ownerId || null,
      noMaximize: opts.noMaximize === true,
      isMinimized: false,
      isMaximized: false,
      isFullscreen: false,
      previousState: null,
    };
    this.openWindows.push(windowData);

    const workspace = this.getWorkspace();
    // #1288 — windows open with a ~10% top margin instead of hugging the
    // title bar (the old 60px base stacked popups right under the chrome).
    // They still cascade BELOW/RIGHT of the compact Vibe bar (VB4
    // workbench) so a freshly opened Terminal/Browser never lands on top
    // of it and blocks its toolbar buttons.
    const wRect = workspace.getBoundingClientRect();
    const tenPct = Math.max(60, Math.round(wRect.height * 0.10));
    let topBase = tenPct;
    let leftBase = 180;
    const vibeEl = document.getElementById("window-vibe");
    if (vibeEl) {
      const vRect = vibeEl.getBoundingClientRect();
      topBase = Math.max(tenPct, vRect.bottom - wRect.top + 14);
      leftBase = Math.max(180, vRect.left - wRect.left + 12);
    }
    const offset = (this.openWindows.length * 28) % 140;
    // #1632 item 7 — Chat is the primary surface: give it a stable
    // top-left footprint inside the workspace instead of the cascade
    // offset, which pushed the first window under the taskbar/sidebar on
    // narrow screens. Explicit opts still win.
    const APP_DEFAULT_POSITION = { chat: { top: 24, left: 24 } };
    const defPos = APP_DEFAULT_POSITION[id];
    const top = typeof opts.top === "number"
      ? opts.top
      : defPos ? defPos.top : topBase + offset;
    const left = typeof opts.left === "number"
      ? opts.left
      : defPos ? defPos.left : leftBase + offset;

    const windowEl = document.createElement("div");
    windowEl.id = `window-${id}`;
    windowEl.style.top = `${top}px`;
    windowEl.style.left = `${left}px`;
    windowEl.style.zIndex = this.zIndexCounter++;

    if (this.useGlassWindows) {
      windowEl.className = "window-element-glass";
      windowEl.innerHTML = this._glassHeader(id, title, opts) + this._glassBody(id);
    } else {
      windowEl.className = "window-element";
      windowEl.innerHTML = this._legacyHeader(id, title, opts) + this._legacyBody(id);
    }

    if (opts.tool) {
      windowEl.classList.add("window-tool");
      if (opts.popup) windowEl.classList.add("window-popup");
    } else if (!opts.noStatusBar) {
      // Issue #725: every app window gets an inverted 3D bevel status bar.
      // The Vibe workbench opts out: VB-style design, no status bar.
      windowEl.insertAdjacentHTML("beforeend", this._statusBar(id));
    }

    workspace.appendChild(windowEl);
    // Chrome (title-bar controls, status bar) is built by the window manager,
    // not by the app fragment, so translate it explicitly at open time.
    if (window.i18n && typeof window.i18n.translatePage === "function") {
      window.i18n.translatePage(windowEl);
    }
    // #1315 — per-app default window size. Timer (and other small
    // utilities) open compact; users can still resize/maximize freely.
    const APP_DEFAULT_SIZE = {
      // The primary surface gets the canvas, not a postage stamp.
      chat: { width: 1080, height: 800 },
      timer: { width: 360, height: 300 },
      timeclock: { width: 400, height: 320 },
      calculator: { width: 320, height: 420 },
      clock: { width: 360, height: 300 },
      notepad: { width: 420, height: 320 },
      // #1304 — Settings is a fixed-size dialog, not a resizable workspace.
      settings: { width: 940, height: 680 },
      // Preview is a reading surface: wide enough for a page, not full screen.
      preview: { width: 780, height: 640 },
      admin: { width: 940, height: 680 },
    };
    const defSize = APP_DEFAULT_SIZE[id];
    if (defSize) {
      // Clamp to the viewport so the fixed footprint always fits.
      windowEl.style.width = `min(${defSize.width}px, calc(100vw - 40px))`;
      windowEl.style.height = `min(${defSize.height}px, calc(100vh - 96px))`;
    }
    // #1304 — Settings/Admin are modal dialogs: fixed footprint, no resize
    // grip and no maximize button (the layout is designed for one size).
    if (id === "settings" || id === "admin") {
      windowEl.classList.add("window-settings-modal");
    }
    // #1849 — install the per-app status listener before the fragment scripts
    // run so an app that publishes `gb:status:<id>` during boot is captured.
    if (typeof this._statusBarInit === "function") this._statusBarInit(id);
    this._injectBodyContent(id, htmlContent);
    // #1849 — hand the freshly-created footer to the shared status renderer.
    // Apps may already have published a status while their fragment scripts
    // executed; this replays the stored payload (or the app registry default)
    // into the live slots.
    if (typeof this._statusBarInit === "function") this._statusBarInit(id);
    this._addTaskbarDockItem(id);
    this._makeDraggable(windowEl);
    this._makeResizable(windowEl);
    this.focus(id);
    if (!opts.tool) this._trackRecent(id, title);
    // The cascade position above is computed while the desktop may still
    // be laying out; an app auto-opened on load (e.g. ?app=vibe) can land
    // with a stale/negative offset that pushes it off-screen. Clamp the
    // window into the visible workspace so it can never be unreachable.
    this._clampWindowIntoView(id);
    this._applyWindowPreference(id);
    // #1432/#1439/#1434 — the open event carries the deep-link params
    // snapshot so the session restore can reopen windows to WHERE they
    // were (conversation id, person, project).
    document.dispatchEvent(new CustomEvent("gb-window-changed", {
      detail: { action: "open", id, params: Object.assign({}, window.__gbAppParams__ || {}) },
    }));
    if (window.htmx) htmx.process(windowEl);
  }
  }

  WM.WindowManager = WindowManager;

  // Keep the class methods non-enumerable, exactly like native class syntax.
  WM.defineMethods = function (proto, methods) {
    Object.keys(methods).forEach(function (name) {
      Object.defineProperty(proto, name, {
        value: methods[name],
        writable: true,
        configurable: true,
        enumerable: false,
      });
    });
  };
})(window);
