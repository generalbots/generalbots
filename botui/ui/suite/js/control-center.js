"use strict";
/* Control Center (#1158): quick-settings panel from the taskbar tray —
   theme toggle, do-not-disturb, volume/brightness sliders, lock & power. */

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
      <div class="gb-cc-header">Quick Settings</div>
      <div class="gb-cc-toggles">
        <button class="gb-cc-toggle" id="ccTheme">${isDark() ? "☀️ Light" : "🌙 Dark"}</button>
        <button class="gb-cc-toggle" id="ccDnd">🔕 Do Not Disturb</button>
        <button class="gb-cc-toggle" id="ccAgent">🤖 Agent perms</button>
        <button class="gb-cc-toggle" id="ccLock">🔒 Lock</button>
      </div>
      <div class="gb-cc-slider-row">
        <span>🔆</span>
        <input type="range" id="ccBrightness" min="20" max="100" value="100" />
      </div>
      <div class="gb-cc-slider-row">
        <span>🔊</span>
        <input type="range" id="ccVolume" min="0" max="100" value="100" />
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
    const dnd = panel.querySelector("#ccDnd");
    if (dnd) {
      dnd.addEventListener("click", () => {
        const active = dnd.classList.toggle("active");
        dnd.textContent = active ? "🔕 DND On" : "🔕 Do Not Disturb";
        window.dispatchEvent(new CustomEvent("gb-dnd-changed", { detail: { enabled: active } }));
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

window.ControlCenter = ControlCenter;