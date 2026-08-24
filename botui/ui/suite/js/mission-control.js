"use strict";
/* Mission Control — overview of all open windows, virtual desktops, and running apps.
 * Activated via the Mission Control button in the taskbar or Ctrl+Up.
 * Part of #1155 multitasking. */

const MissionControl = (() => {
  const MODULE = "mission-control";

  function isOpen() {
    return document.getElementById("mission-control") !== null;
  }

  function open() {
    if (isOpen()) return;
    const overlay = document.createElement("div");
    overlay.id = "mission-control";
    overlay.className = "mission-control-overlay";
    overlay.innerHTML = `
      <div class="mc-header">
        <h2>Mission Control</h2>
        <div class="mc-desktops"></div>
        <button class="mc-close" title="Close (Esc)">✕</button>
      </div>
      <div class="mc-windows" id="mc-windows"></div>
    `;
    document.body.appendChild(overlay);
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) close();
    });
    render();
  }

  function close() {
    const overlay = document.getElementById("mission-control");
    if (overlay) overlay.remove();
  }

  function render() {
    renderDesktops();
    renderWindows();
  }

  function renderDesktops() {
    const container = document.querySelector(".mc-desktops");
    if (!container || typeof VirtualDesktops === "undefined") return;
    const list = VirtualDesktops.list() || [{ id: "desktop-1", name: "Desktop 1", active: true }];
    const activeId = VirtualDesktops.activeId ? VirtualDesktops.activeId() : "desktop-1";
    container.innerHTML = list.map((d) => `
      <button class="mc-desktop ${d.id === activeId ? "active" : ""}" data-id="${d.id}">
        ${d.name}
      </button>
    `).join("");
    Array.from(container.querySelectorAll(".mc-desktop")).forEach((btn) => {
      btn.addEventListener("click", () => {
        if (VirtualDesktops.switchTo) VirtualDesktops.switchTo(btn.dataset.id);
        render();
      });
    });
  }

  function renderWindows() {
    const container = document.getElementById("mc-windows");
    if (!container) return;
    const wm = window.__WM__ || window.WindowManager;
    if (!wm || !wm.listWindows) { container.innerHTML = "<p class='mc-empty'>No open windows.</p>"; return; }
    const wins = wm.listWindows();
    if (!wins.length) { container.innerHTML = "<p class='mc-empty'>No open windows.</p>"; return; }
    container.innerHTML = wins.map((w) => `
      <div class="mc-window" data-id="${w.id}">
        <div class="mc-window-preview">${w.icon || "📄"}</div>
        <div class="mc-window-title">${escapeHtml(w.title || w.id)}</div>
      </div>
    `).join("");
    Array.from(container.querySelectorAll(".mc-window")).forEach((card) => {
      card.addEventListener("click", () => {
        if (wm.focusWindow) wm.focusWindow(card.dataset.id);
        if (wm.restoreWindow) wm.restoreWindow(card.dataset.id);
        close();
      });
    });
  }

  function toggle() {
    if (isOpen()) close(); else open();
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  document.addEventListener("DOMContentLoaded", () => {
    document.addEventListener("keydown", (e) => {
      if (e.ctrlKey && e.key === "ArrowUp") { e.preventDefault(); toggle(); }
      if (e.key === "Escape" && isOpen()) close();
    });
  });

  return { open, close, toggle, isOpen, render };
})();

window.MissionControl = MissionControl;