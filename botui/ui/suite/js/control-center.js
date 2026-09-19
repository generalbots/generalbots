"use strict";
/* Control Center (#1158, #1433): quick-settings panel from the taskbar tray.
   Only functional controls ship: theme toggle, agent permissions, lock and
   power. Dead controls (DND with no listener, brightness/volume sliders that
   drive nothing in a browser tab) were removed per the #1433 audit. */

const ControlCenter = (() => {
  let initialized = false;

  function init() {
    if (initialized) return;
    initialized = true;
    document.addEventListener("keydown", (e) => {
      if (e.ctrlKey && e.key === "l") {
        e.preventDefault();
        toggle();
      }
    });
  }

  function isOpen() {
    return document.getElementById("gb-control-center") !== null;
  }

  function toggle() {
    if (isOpen()) close(); else open();
  }

  function open() {
    if (isOpen()) return;
    const panel = document.createElement("div");
    panel.id = "gb-control-center";
    panel.className = "gb-control-center";
    panel.innerHTML = `
      <div class="gb-cc-header">Control Center</div>
      <div class="gb-cc-toggles">
        <button class="gb-cc-toggle" id="ccTheme">${isDark() ? "☀️ Light" : "🌙 Dark"}</button>
        <button class="gb-cc-toggle" id="ccAgent">🤖 Agent perms</button>
        <button class="gb-cc-toggle" id="ccLock">🔒 Lock</button>
      </div>
      <div class="gb-cc-power" id="ccPower">⏻ Power</div>
    `;
    document.body.appendChild(panel);
    bind(panel);
    const dismiss = (e) => {
      if (!panel.contains(e.target)) close();
    };
    setTimeout(() => document.addEventListener("click", dismiss, { once: true }), 0);
  }

  function close() {
    const panel = document.getElementById("gb-control-center");
    if (panel) panel.remove();
  }

  function bind(panel) {
    const theme = panel.querySelector("#ccTheme");
    if (theme) {
      theme.addEventListener("click", () => {
        const root = document.documentElement;
        const next = root.getAttribute("data-theme") === "dark" ? "light" : "dark";
        root.setAttribute("data-theme", next);
        try { localStorage.setItem("gb-theme", next); } catch (e) {}
        theme.textContent = next === "dark" ? "☀️ Light" : "🌙 Dark";
      });
    }
    const lock = panel.querySelector("#ccLock");
    if (lock) {
      lock.addEventListener("click", () => {
        close();
        if (window.LockScreen) window.LockScreen.lock();
      });
    }
    const agent = panel.querySelector("#ccAgent");
    if (agent) {
      agent.addEventListener("click", () => {
        close();
        if (window.AgentControl && window.AgentControl.showPanel) window.AgentControl.showPanel();
      });
    }
    const power = panel.querySelector("#ccPower");
    if (power) {
      power.addEventListener("click", () => {
        window.location.href = window.GB_LOGIN_URL || "/login";
      });
    }
  }

  function isDark() {
    return document.documentElement.getAttribute("data-theme") === "dark";
  }

  return { init, open, close, toggle, isOpen };
})();

// #1433 — the tray button checks window.ControlCenter; the bare const above
// is module-scoped and would leave the handler silently dead.
window.ControlCenter = ControlCenter;

window.ControlCenter = ControlCenter;