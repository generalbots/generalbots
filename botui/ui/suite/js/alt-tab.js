"use strict";
/* Alt-Tab window switcher for the desktop (#1155 multitasking).
   Cycles through open windows with a preview overlay. */

const AltTab = (() => {
  let active = false;
  let index = 0;
  let windows = [];

  function isOpen() { return active; }

  function open() {
    const wm = window.WindowManager;
    if (!wm || !wm.listWindows) return;
    windows = (wm.listWindows() || []).filter((w) => w.title || w.id);
    if (!windows.length) return;
    active = true;
    index = 0;
    render();
  }

  function close() {
    active = false;
    const overlay = document.getElementById("alt-tab-overlay");
    if (overlay) overlay.remove();
    const wm = window.WindowManager;
    if (wm && wm.focusWindow && windows[index]) wm.focusWindow(windows[index].id);
  }

  function next() {
    if (!windows.length) return;
    index = (index + 1) % windows.length;
    render();
  }

  function prev() {
    if (!windows.length) return;
    index = (index - 1 + windows.length) % windows.length;
    render();
  }

  function render() {
    let overlay = document.getElementById("alt-tab-overlay");
    if (!overlay) {
      overlay = document.createElement("div");
      overlay.id = "alt-tab-overlay";
      overlay.className = "alt-tab-overlay";
      document.body.appendChild(overlay);
    }
    overlay.innerHTML = windows.map((w, i) => `
      <div class="alt-tab-item ${i === index ? "active" : ""}" data-i="${i}">
        <div class="alt-tab-icon">${w.icon || "📄"}</div>
        <div class="alt-tab-title">${String(w.title || w.id).substring(0, 40)}</div>
      </div>
    `).join("");
    Array.from(overlay.querySelectorAll(".alt-tab-item")).forEach((el) => {
      el.addEventListener("click", () => {
        index = parseInt(el.dataset.i, 10);
        close();
      });
    });
  }

  document.addEventListener("DOMContentLoaded", () => {
    document.addEventListener("keydown", (e) => {
      if (e.altKey && e.key === "Tab") {
        e.preventDefault();
        if (!active) open(); else next();
      } else if (e.altKey && e.key === "Shift" && active) {
        e.preventDefault();
        prev();
      } else if (active && (e.key === "Escape" || e.key === "Enter")) {
        e.preventDefault();
        close();
      }
    });
  });

  return { open, close, next, prev, isOpen: () => active };
})();

window.AltTab = AltTab;