/* WindowManager — true fullscreen: platform bridge, chrome sync and keys (8/9).
 *
 * #2104 — maximize fills the workspace but keeps the taskbar/desktop chrome
 * visible, and the shell offered no fullscreen control or F11 at all. Two
 * surfaces are involved:
 *
 *   web (Chrome, any browser): the Fullscreen API on the window element, so
 *     the element goes to the top layer and everything else (taskbar, rail)
 *     drops behind it.
 *   desktop (botapp/Tauri WebView): requestFullscreen does not drive the OS
 *     window under WebKitGTK, so the OS window itself is toggled through the
 *     Tauri window API and the page hides its own chrome with a CSS class.
 *
 * Geometry is never rewritten: the fullscreen layer (top layer, or the
 * `.window-fullscreen-fallback` class) sits above the window, so leaving
 * fullscreen restores the previous inline size/position — and a window that
 * was maximized stays maximized — without a snapshot to replay.
 */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;

  var FALLBACK_CLASS = "window-fullscreen-fallback";
  var ACTIVE_CLASS = "window-fullscreen-active";
  var DOT_SELECTOR = ".window-dot-fullscreen";
  // A keystroke that lands in a text field or an embedded editor belongs to
  // that field: F11 there would steal a key the app itself may bind.
  var TYPING_SELECTOR = ".monaco-editor, .xterm, .ace_editor, [contenteditable='true']";
  // setFullscreen() resizes the OS window; ignore those resizes while the
  // transition is still settling or the probe would undo it.
  var ENTER_GRACE_MS = 700;

  function isMac() {
    var nav = window.navigator || {};
    return /Mac|iPhone|iPad/.test(String(nav.platform || nav.userAgent || ""));
  }

  // Tauri desktop edition only. The web build has no window to drive.
  function desktopWindow() {
    var tauri = window.__TAURI__;
    if (tauri && tauri.window && typeof tauri.window.getCurrentWindow === "function") {
      return tauri.window.getCurrentWindow();
    }
    return null;
  }

  function isTypingTarget(target) {
    if (!target || target.nodeType !== 1) return false;
    var tag = target.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    if (target.isContentEditable === true) return true;
    return typeof target.closest === "function" && !!target.closest(TYPING_SELECTOR);
  }

  function idOf(el) {
    return el && el.id ? String(el.id).replace(/^window-/, "") : null;
  }

  function reportFailure(what, err) {
    if (window.console && typeof window.console.warn === "function") {
      window.console.warn(`[window-manager] ${what} failed`, err);
    }
  }

  WM.defineMethods(WindowManager.prototype, {

  isFullscreen(id) {
    const el = document.getElementById(`window-${id}`);
    if (!el) return false;
    return document.fullscreenElement === el || el.classList.contains(FALLBACK_CLASS);
  },

  // The window the platform currently holds fullscreen, or null. The API owns
  // the truth when it is in play; the class covers the desktop shell and the
  // API-less fallback. Elements outside the manager (the projector/video
  // player has its own fullscreen) are deliberately ignored.
  fullscreenWindowId() {
    const apiId = idOf(document.fullscreenElement);
    if (apiId && this.openWindows.some((w) => w.id === apiId)) return apiId;
    return idOf(document.querySelector(`.${FALLBACK_CLASS}`));
  },

  toggleFullscreen(id) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (!obj || obj.noMaximize) return;
    const el = document.getElementById(`window-${id}`);
    // Fixed-size modal dialogs (Settings/Admin) carry no fullscreen control,
    // so the key must not open a mode their layout does not support.
    if (!el || el.classList.contains("window-settings-modal")) return;
    // obj.isFullscreen covers the moment between the request and the platform
    // answering it, so a second F11 always exits instead of re-entering.
    if (obj.isFullscreen || this.isFullscreen(id)) this.exitFullscreen(id);
    else this.enterFullscreen(id);
  },

  enterFullscreen(id) {
    const el = document.getElementById(`window-${id}`);
    if (!el) return;
    // One fullscreen window at a time: a programmatic focus change must not
    // leave a second window stuck in the layer.
    this.openWindows.forEach((w) => {
      if (w.id !== id && this.isFullscreen(w.id)) this.exitFullscreen(w.id);
    });
    this._fullscreenEnteredAt = Date.now();
    this._syncFullscreenChrome(id, true);
    this.focus(id);
    const desktop = desktopWindow();
    if (desktop && typeof desktop.setFullscreen === "function") {
      // Desktop shell: the OS window goes fullscreen and the page hides the
      // desktop chrome itself, otherwise the taskbar sits on top of it.
      el.classList.add(FALLBACK_CLASS);
      desktop.setFullscreen(true).catch((err) => reportFailure("window fullscreen", err));
      return;
    }
    if (typeof el.requestFullscreen === "function") {
      el.requestFullscreen({ navigationUI: "hide" }).catch(() => {
        // WebViews without the API (or a denied request) keep the window
        // covering the viewport, so the desktop chrome is hidden anyway.
        el.classList.add(FALLBACK_CLASS);
      });
      return;
    }
    el.classList.add(FALLBACK_CLASS);
  },

  exitFullscreen(id) {
    const el = document.getElementById(`window-${id}`);
    if (!el) return;
    el.classList.remove(FALLBACK_CLASS);
    this._syncFullscreenChrome(id, false);
    const desktop = desktopWindow();
    if (desktop && typeof desktop.setFullscreen === "function") {
      desktop.setFullscreen(false).catch((err) => reportFailure("window exit fullscreen", err));
    }
    if (document.fullscreenElement === el && typeof document.exitFullscreen === "function") {
      const pending = document.exitFullscreen();
      if (pending && typeof pending.catch === "function") {
        pending.catch((err) => reportFailure("document exit fullscreen", err));
      }
    }
    this.focus(id);
  },

  // Single source of truth for the control: icon, tooltip, aria and the
  // per-window flag all follow what the platform actually did.
  _syncFullscreenChrome(id, active) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (obj) obj.isFullscreen = !!active;
    const el = document.getElementById(`window-${id}`);
    if (!el) return;
    el.classList.toggle(ACTIVE_CLASS, !!active);
    const dot = el.querySelector(DOT_SELECTOR);
    if (!dot) return;
    const key = active ? "window-exit-fullscreen" : "window-fullscreen";
    const text = active ? "Exit fullscreen" : "Fullscreen";
    dot.setAttribute("data-i18n-title", key);
    dot.setAttribute("data-i18n-aria-label", key);
    dot.setAttribute("aria-pressed", active ? "true" : "false");
    const label = window.gbT ? window.gbT(key, text) : text;
    dot.setAttribute("title", label);
    dot.setAttribute("aria-label", label);
  },

  });

  // Escape leaves fullscreen before anything else consumes the key (menus,
  // modals). The Fullscreen API handles Escape natively; the CSS fallback and
  // the desktop shell have no native handler, so capture it here.
  document.addEventListener("keydown", function (e) {
    if (e.key !== "Escape") return;
    const wm = window.WindowManager;
    if (!wm || typeof wm.fullscreenWindowId !== "function") return;
    const id = wm.fullscreenWindowId();
    if (!id) return;
    e.preventDefault();
    e.stopPropagation();
    wm.exitFullscreen(id);
  }, true);

  // F11 everywhere. macOS keyboards have no F11 key, so the shell accepts the
  // system chord (Ctrl+Cmd+F) there as well. Both are ignored while the user
  // is typing in a field or an embedded editor.
  document.addEventListener("keydown", function (e) {
    const isF11 = e.key === "F11";
    const isMacChord = isMac() && e.ctrlKey && e.metaKey && String(e.key).toLowerCase() === "f";
    if (!isF11 && !isMacChord) return;
    if (isTypingTarget(e.target)) return;
    const wm = window.WindowManager;
    if (!wm || typeof wm.toggleFullscreen !== "function") return;
    const id = wm.fullscreenWindowId() || wm.activeWindowId;
    if (!id) return;
    e.preventDefault();
    wm.toggleFullscreen(id);
  });

  // The browser can leave fullscreen on its own (Esc, gesture, UA-handled
  // F11): reconcile the icon and the fallback class with the real state.
  document.addEventListener("fullscreenchange", function () {
    // The desktop shell has no element fullscreen — the OS window is the
    // authority there and the resize watcher below reconciles it.
    if (desktopWindow()) return;
    const wm = window.WindowManager;
    if (!wm || !Array.isArray(wm.openWindows)) return;
    const apiId = idOf(document.fullscreenElement);
    wm.openWindows.forEach((w) => {
      if (!w.isFullscreen || w.id === apiId) return;
      const el = document.getElementById(`window-${w.id}`);
      if (el) el.classList.remove(FALLBACK_CLASS);
      wm._syncFullscreenChrome(w.id, false);
    });
  });

  // Desktop shell: the OS window can also leave fullscreen without telling the
  // page (native window control). Reconcile on resize so the taskbar and the
  // icon never stay in the fullscreen state after the OS dropped it.
  function watchDesktopFullscreen() {
    const desktop = desktopWindow();
    if (!desktop || typeof desktop.onResized !== "function" ||
        typeof desktop.isFullscreen !== "function") return;
    desktop.onResized(function () {
      const wm = window.WindowManager;
      if (!wm || !Array.isArray(wm.openWindows)) return;
      if (Date.now() - (wm._fullscreenEnteredAt || 0) < ENTER_GRACE_MS) return;
      const flagged = wm.openWindows.filter((w) => w.isFullscreen);
      if (!flagged.length) return;
      desktop.isFullscreen().then(function (osFullscreen) {
        if (osFullscreen) return;
        flagged.forEach(function (w) {
          if (w.isFullscreen) wm.exitFullscreen(w.id);
        });
      }).catch(function (err) { reportFailure("window fullscreen probe", err); });
    });
  }
  watchDesktopFullscreen();
})(window);
