"use strict";
/* Taskbar window previews + jump lists for the desktop (#1155 multitasking).
   Hovering a taskbar item shows a thumbnail preview of the window;
   right-click shows a jump list with window actions. */

const TaskbarPreviews = (() => {
  function attach() {
    const taskbar = document.getElementById("taskbar") || document.querySelector(".taskbar");
    if (!taskbar) return;
    // Live previews: hook into window focus changes via WindowManager events
    // (dispatched on document by window-manager.js as gb-window-focus).
    document.addEventListener("gb-window-focus", (e) => showPreview(e.detail));
    document.addEventListener("gb-window-close", () => hidePreview());
  }

  function showPreview(win) {
    if (!win) return;
    const btn = document.querySelector(`[data-window-id="${win.id}"]`);
    if (!btn) return;
    const tip = document.createElement("div");
    tip.className = "taskbar-preview";
    tip.innerHTML = `
      <div class="tp-title">${String(win.title || win.id).substring(0, 60)}</div>
      <div class="tp-body">${win.preview || "<div class='tp-placeholder'>…</div>"}</div>
    `;
    btn.appendChild(tip);
    setTimeout(() => tip.remove(), 2500);
  }

  function hidePreview() {
    Array.from(document.querySelectorAll(".taskbar-preview")).forEach((el) => el.remove());
  }

  function buildJumpList(win) {
    const wm = window.WindowManager;
    if (!wm) return null;
    const actions = [
      { label: "Minimize", fn: () => wm.minimizeWindow && wm.minimizeWindow(win.id) },
      { label: "Maximize", fn: () => wm.maximizeWindow && wm.maximizeWindow(win.id) },
      { label: "Close", fn: () => wm.closeWindow && wm.closeWindow(win.id) },
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
      const dismiss = (ev) => { if (!menu.contains(ev.target)) menu.remove(); };
      document.addEventListener("click", dismiss, { once: true });
    });
  });

  return { attach, showPreview, hidePreview, buildJumpList };
})();

window.TaskbarPreviews = TaskbarPreviews;