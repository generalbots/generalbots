/* #2439 SHELL — laptop keyboard window management (9x, loaded last).
 *
 * Additive module: it never replaces a WindowManager method and never calls
 * window.close() — the browser tab is not ours to close. It binds:
 *   Alt + 1..9            focus the Nth open window (stable open order)
 *   Ctrl + Tab            cycle forward through open windows
 *   Ctrl + Shift + Tab    cycle backward
 *   Escape                close ONLY the focused, non-locked app window
 *   ?                     toggle this help overlay
 * Shortcuts are ignored while the user is typing in a field/editor.
 */
(function (win, doc) {
  "use strict";

  // Re-executing the file must never add a second keydown listener. When the
  // WindowManager is not there yet we leave the guard unset so a later load
  // can still install; without a manager the module is a complete no-op.
  if (win.__GB_WINDOW_KEYBOARD_SHORTCUTS__) return;
  var wm = win.WindowManager;
  if (!wm || (!Array.isArray(wm.openWindows) && typeof wm.listWindows !== "function")) return;
  win.__GB_WINDOW_KEYBOARD_SHORTCUTS__ = true;

  var HELP_ID = "gb-window-shortcuts-help";
  // Chat is the primary surface of the shell; system dialogs are locked
  // surfaces. Individual windows may also opt out with locked / noClose /
  // closable:false on their window-data object.
  var LOCKED_IDS = { chat: true, about: true, auth: true, settings: true };
  var TYPING_SELECTOR = ".monaco-editor, .xterm, .ace_editor, [contenteditable]:not([contenteditable='false'])";

  function t(key, fallback) {
    return win.gbT && typeof win.gbT === "function" ? win.gbT(key, fallback) : fallback;
  }

  function windowsOf(manager) {
    if (Array.isArray(manager.openWindows)) return manager.openWindows.slice();
    if (typeof manager.listWindows === "function") return manager.listWindows() || [];
    return [];
  }

  function findWindow(manager, id) {
    var list = windowsOf(manager);
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].id === id) return list[i];
    }
    return null;
  }

  function isTyping(target) {
    if (!target || target.nodeType !== 1) return false;
    var tag = String(target.tagName || "").toUpperCase();
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    if (target.isContentEditable === true) return true;
    if (typeof target.closest === "function" && target.closest(TYPING_SELECTOR)) return true;
    if (typeof target.getAttribute === "function") {
      var editable = target.getAttribute("contenteditable");
      if (editable !== null && editable !== "false") return true;
    }
    return false;
  }

  function activate(manager, item) {
    if (!item || !item.id) return false;
    if (item.isMinimized && typeof manager.toggleMinimize === "function") {
      manager.toggleMinimize(item.id);
      return true;
    }
    if (typeof manager.focus === "function") {
      manager.focus(item.id);
      return true;
    }
    if (typeof manager.focusWindow === "function") {
      manager.focusWindow(item.id);
      return true;
    }
    return false;
  }

  function focusByNumber(manager, n) {
    var list = windowsOf(manager);
    if (n < 1 || n > list.length) return false;
    return activate(manager, list[n - 1]);
  }

  function cycle(manager, direction) {
    var list = windowsOf(manager);
    if (!list.length) return false;
    var active = manager.activeWindowId;
    var index = -1;
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].id === active) {
        index = i;
        break;
      }
    }
    var next = index === -1
      ? (direction > 0 ? 0 : list.length - 1)
      : (index + direction + list.length) % list.length;
    return activate(manager, list[next]);
  }

  function isLocked(manager, item) {
    if (!item) return true;
    if (item.locked === true || item.noClose === true || item.closable === false) return true;
    var id = String(item.id || "");
    var base = id.split("-")[0];
    if (LOCKED_IDS[id] || LOCKED_IDS[base]) return true;
    if (typeof manager.isCoreAppId === "function" && manager.isCoreAppId(base)) return true;
    return false;
  }

  function closeFocused(manager) {
    // Escape leaves fullscreen first — 08_fullscreen.js owns that key in the
    // capture phase and stops propagation, so normally we never see it here.
    if (typeof manager.fullscreenWindowId === "function" && manager.fullscreenWindowId()) return false;
    var id = manager.activeWindowId;
    if (!id) return false;
    var item = findWindow(manager, id);
    if (isLocked(manager, item)) return false;
    if (typeof manager.close !== "function") return false;
    manager.close(id);
    return true;
  }

  // ── Help overlay ──────────────────────────────────────────────────────

  var helpEl = null;
  var helpCloseBtn = null;
  var lastFocus = null;

  function applyStyles(el, rules) {
    Object.keys(rules).forEach(function (prop) {
      el.style[prop] = rules[prop];
    });
  }

  function makeRow(keys, key, fallback) {
    var row = doc.createElement("div");
    applyStyles(row, { display: "flex", alignItems: "center", gap: "12px", marginBottom: "6px" });
    var cap = doc.createElement("span");
    cap.textContent = keys;
    applyStyles(cap, {
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
      fontSize: "11px",
      whiteSpace: "nowrap",
      padding: "2px 6px",
      borderRadius: "4px",
      background: "rgba(255,255,255,0.12)",
      color: "inherit"
    });
    var desc = doc.createElement("span");
    desc.textContent = t(key, fallback);
    applyStyles(desc, { opacity: "0.85" });
    row.appendChild(cap);
    row.appendChild(desc);
    return row;
  }

  function helpElement() {
    var existing = doc.getElementById ? doc.getElementById(HELP_ID) : null;
    if (existing) return existing;
    if (!doc.createElement || !doc.body || !doc.body.appendChild) return null;

    var el = doc.createElement("div");
    helpEl = el;
    el.id = HELP_ID;
    el.className = "gb-window-shortcuts-help";
    el.setAttribute("role", "dialog");
    el.setAttribute("aria-modal", "false");
    el.setAttribute("aria-label", t("a11y-keyboard-shortcuts", "Keyboard Shortcuts"));
    el.hidden = true;
    applyStyles(el, {
      position: "fixed",
      right: "16px",
      bottom: "48px",
      zIndex: "2147483000",
      width: "360px",
      maxWidth: "calc(100vw - 32px)",
      boxSizing: "border-box",
      padding: "14px 16px",
      borderRadius: "10px",
      border: "1px solid rgba(255,255,255,0.18)",
      background: "rgba(18,20,24,0.96)",
      color: "#f5f5f5",
      font: "13px/1.5 system-ui, sans-serif",
      boxShadow: "0 12px 40px rgba(0,0,0,0.45)",
      display: "none"
    });

    var head = doc.createElement("div");
    applyStyles(head, { display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" });
    var title = doc.createElement("strong");
    title.textContent = t("a11y-keyboard-shortcuts", "Keyboard Shortcuts");
    var closeBtn = doc.createElement("button");
    helpCloseBtn = closeBtn;
    closeBtn.type = "button";
    closeBtn.textContent = "×";
    var closeLabel = t("action-close", "Close");
    closeBtn.setAttribute("aria-label", closeLabel);
    closeBtn.setAttribute("title", closeLabel);
    applyStyles(closeBtn, {
      background: "transparent",
      border: "0",
      color: "inherit",
      cursor: "pointer",
      fontSize: "18px",
      lineHeight: "1",
      padding: "0 4px"
    });
    closeBtn.addEventListener("click", hideHelp);
    head.appendChild(title);
    head.appendChild(closeBtn);

    // Only existing catalog keys are used; the English strings are gbT's
    // fallbacks. Keycap labels are key names, not translatable prose.
    var rows = [
      makeRow("Alt + 1 … 9", "a11y-shortcut-navigate", "Navigate between elements"),
      makeRow("Ctrl + Tab / Ctrl + Shift + Tab", "a11y-shortcut-navigate", "Navigate between elements"),
      makeRow("Esc", "window-close", "Close"),
      makeRow("?", "a11y-shortcut-show-help", "Show this help")
    ];
    el.appendChild(head);
    rows.forEach(function (row) { el.appendChild(row); });
    doc.body.appendChild(el);
    return el;
  }

  function helpOpen() {
    var el = helpEl || (doc.getElementById ? doc.getElementById(HELP_ID) : null);
    return !!(el && !el.hidden);
  }

  function hideHelp() {
    var el = helpEl || (doc.getElementById ? doc.getElementById(HELP_ID) : null);
    if (!el || el.hidden) return false;
    el.hidden = true;
    el.style.display = "none";
    if (lastFocus && typeof lastFocus.focus === "function") lastFocus.focus();
    lastFocus = null;
    return true;
  }

  function showHelp() {
    var el = helpElement();
    if (!el) return false;
    if (!el.hidden) {
      hideHelp();
      return true;
    }
    lastFocus = doc.activeElement || null;
    el.hidden = false;
    el.style.display = "block";
    if (helpCloseBtn && typeof helpCloseBtn.focus === "function") helpCloseBtn.focus();
    return true;
  }

  // ── Keyboard routing ──────────────────────────────────────────────────

  function onKeyDown(e) {
    if (!e || e.defaultPrevented) return;
    if (isTyping(e.target)) return;
    var key = String(e.key || "");
    var digit = /^[1-9]$/.exec(key);

    if (key === "?" || (key === "/" && e.shiftKey) || (e.code === "Slash" && e.shiftKey)) {
      e.preventDefault();
      showHelp();
      return;
    }
    if (key === "Escape") {
      if (helpOpen()) {
        e.preventDefault();
        hideHelp();
      } else if (closeFocused(wm)) {
        e.preventDefault();
      }
      return;
    }
    if (e.altKey && !e.ctrlKey && !e.metaKey && digit) {
      if (helpOpen()) hideHelp();
      if (focusByNumber(wm, parseInt(key, 10))) e.preventDefault();
      return;
    }
    if (e.ctrlKey && !e.altKey && !e.metaKey && key === "Tab") {
      if (helpOpen()) hideHelp();
      if (cycle(wm, e.shiftKey ? -1 : 1)) e.preventDefault();
    }
  }

  // Bound on window (bubble): other document-level Escape handlers (palette,
  // menus, fullscreen) run first; if one called preventDefault or stopped
  // propagation we do not also close the focused app window.
  win.addEventListener("keydown", onKeyDown);

  win.GBWindowKeyboardShortcuts = {
    help: showHelp,
    hideHelp: hideHelp,
    isInstalled: function () { return true; }
  };
})(window, document);
