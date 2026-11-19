"use strict";
/* Taskbar window previews + jump lists for the desktop (#1155 multitasking).
   Hover shows a thumbnail on mouse pointers; on touch the same information is
   reached by tapping the dock item, with a long-press shortcut for the jump
   list. The right-click jump list stays identical on desktop (#2419). */

const TaskbarPreviews = (() => {
  function label(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }

  function attach() {
    const taskbar = document.getElementById("taskbar") || document.querySelector(".taskbar");
    if (!taskbar) return;
    // #1305 — previews are HOVER tooltips on the taskbar dock item. They must
    // NOT trigger on gb-window-focus: that event fires on every window
    // focus/caption click, and the old code appended the preview into the
    // window's own status bar ([data-window-id] is the footer bar),
    // rendering a black blob at the top-left of the window.
    let hoverTimer = null;
    let hoverWinId = null;
    const dockSelector = ".taskbar-dock-item";

    function dockWinId(item) {
      const did = item && item.id && String(item.id).replace("dock-item-", "");
      return did || null;
    }

    document.addEventListener("mouseover", (e) => {
      const item = e.target.closest && e.target.closest(dockSelector);
      if (!item) return;
      const id = dockWinId(item);
      if (!id) return;
      if (hoverWinId === id) return; // already showing for this item
      clearTimeout(hoverTimer);
      hoverWinId = id;
      hoverTimer = setTimeout(() => {
        const wm = window.WindowManager;
        const win = wm && wm.getWindow && wm.getWindow(id);
        if (win) showPreview(win, item);
      }, 450);
    });
    document.addEventListener("mouseout", (e) => {
      const item = e.target.closest && e.target.closest(dockSelector);
      if (!item) return;
      const id = dockWinId(item);
      if (id && hoverWinId === id) {
        clearTimeout(hoverTimer);
        hoverWinId = null;
        hidePreview();
      }
    });
    document.addEventListener("gb-window-close", () => hidePreview());

    const gestures = window.gbGestures;
    if (!gestures || !gestures.isCoarsePointer()) return;

    function winFor(item) {
      const id = dockWinId(item);
      const wm = window.WindowManager;
      return id && wm && wm.getWindow ? wm.getWindow(id) : null;
    }

    function showTouchActions(win, item) {
      hidePreview();
      showPreview(win, item);
      const tip = item.querySelector(".taskbar-preview");
      const menu = buildJumpList(win);
      if (tip && menu) {
        menu.style.position = "static";
        tip.appendChild(menu);
      }
    }

    // Tap is the discoverable path: the preview now carries the jump-list
    // buttons, so no action exists behind hover or right-click only.
    gestures.onTap(document, (e) => {
      const item = e.target.closest && e.target.closest(dockSelector);
      if (!item) return;
      const win = winFor(item);
      if (win) showTouchActions(win, item);
    });

    gestures.onLongPress(document, (e) => {
      const item = e.target.closest && e.target.closest(dockSelector + ", [data-window-id]");
      if (!item) return;
      let win = winFor(item);
      if (!win) {
        const wm = window.WindowManager;
        const id = item.dataset && item.dataset.windowId;
        win = id && wm && wm.getWindow ? wm.getWindow(id) : null;
      }
      if (!win) return;
      const menu = buildJumpList(win);
      if (!menu) return;
      hidePreview();
      menu.style.position = "fixed";
      document.body.appendChild(menu);
      gestures.clampToViewport(menu, e.clientX, e.clientY);
      const dismiss = (ev) => { if (!menu.contains(ev.target)) menu.remove(); };
      setTimeout(() => document.addEventListener("click", dismiss, { once: true }), 0);
    });
  }

  function showPreview(win, anchorEl) {
    if (!win) return;
    hidePreview();
    // Anchor INSIDE the taskbar dock item (position:relative there), never
    // inside the window chrome — a preview in the window footer is the #1305
    // black-blur regression.
    const btn = anchorEl || document.querySelector(`.taskbar-dock-item#dock-item-${CSS.escape(String(win.id))}`);
    if (!btn) return;
    const tip = document.createElement("div");
    tip.className = "taskbar-preview";
    tip.innerHTML = `
      <div class="tp-title">${String(win.title || win.id).substring(0, 60)}</div>
      <div class="tp-body">${win.preview || "<div class='tp-placeholder'>…</div>"}</div>
    `;
    btn.appendChild(tip);
  }

  function hidePreview() {
    Array.from(document.querySelectorAll(".taskbar-preview")).forEach((el) => el.remove());
  }

  function buildJumpList(win) {
    const wm = window.WindowManager;
    if (!wm) return null;
    const actions = [
      { label: label("window-minimize", "Minimize"), fn: () => wm.minimizeWindow && wm.minimizeWindow(win.id) },
      { label: label("window-maximize", "Maximize"), fn: () => wm.maximizeWindow && wm.maximizeWindow(win.id) },
      { label: label("action-close", "Close"), fn: () => wm.closeWindow && wm.closeWindow(win.id) },
    ];
    const menu = document.createElement("div");
    menu.className = "taskbar-jumplist";
    actions.forEach((a) => {
      const item = document.createElement("button");
      item.textContent = a.label;
      item.addEventListener("click", () => { a.fn(); menu.remove(); });
      menu.appendChild(item);
    });
    return menu;
  }

  document.addEventListener("DOMContentLoaded", () => {
    attach();
    document.addEventListener("contextmenu", (e) => {
      const btn = e.target.closest && e.target.closest("[data-window-id]");
      if (!btn) return;
      e.preventDefault();
      const wm = window.WindowManager;
      const win = wm && wm.getWindow && wm.getWindow(btn.dataset.windowId);
      if (!win) return;
      const menu = buildJumpList(win);
      if (!menu) return;
      menu.style.position = "fixed";
      menu.style.left = `${e.clientX}px`;
      menu.style.top = `${e.clientY}px`;
      document.body.appendChild(menu);
      if (window.gbGestures) window.gbGestures.clampToViewport(menu, e.clientX, e.clientY);
      const dismiss = (ev) => { if (!menu.contains(ev.target)) menu.remove(); };
      document.addEventListener("click", dismiss, { once: true });
    });
  });

  return { attach, showPreview, hidePreview, buildJumpList };
})();

window.TaskbarPreviews = TaskbarPreviews;
