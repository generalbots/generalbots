"use strict";
/* Lock Screen (#1158): full-screen lock overlay with clock, unlock, and
   optional auto-lock after inactivity. */

const LockScreen = (() => {
  const LOCK_KEY = "gb-lock-until";
  let initialized = false;
  let idleTimer = null;

  function init() {
    if (initialized) return;
    initialized = true;
    // Auto-lock after 60 minutes of inactivity (was 10 min — too aggressive
    // for an always-open workbench; it kept covering the desktop while the
    // user was reading. The tray Lock button still locks on demand).
    const IDLE_MS = 60 * 60 * 1000;
    const resetIdle = () => {
      if (idleTimer) clearTimeout(idleTimer);
      idleTimer = setTimeout(() => {
        if (!isLocked()) lock();
      }, IDLE_MS);
    };
    ["mousemove", "keydown", "mousedown", "touchstart", "scroll"].forEach((ev) =>
      document.addEventListener(ev, resetIdle, { passive: true })
    );
    resetIdle();
  }

  function isLocked() {
    return document.getElementById("gb-lock-screen") !== null;
  }

  function lock() {
    if (isLocked()) return;
    const overlay = document.createElement("div");
    overlay.id = "gb-lock-screen";
    overlay.className = "gb-lock-screen";
    overlay.innerHTML = `
      <div class="gb-lock-widget">
        <div class="gb-lock-time" id="gbLockTime">--:--</div>
        <div class="gb-lock-date" id="gbLockDate"></div>
        <div class="gb-lock-hint">Press Enter or click to unlock</div>
      </div>
    `;
    document.body.appendChild(overlay);
    tick();
    setInterval(tick, 1000);
    const unlock = () => {
      overlay.remove();
      try { localStorage.setItem(LOCK_KEY, "0"); } catch (e) {}
    };
    overlay.addEventListener("click", unlock);
    overlay.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === "Escape") unlock();
    });
    try { localStorage.setItem(LOCK_KEY, String(Date.now())); } catch (e) {}
  }

  function tick() {
    const now = new Date();
    const timeEl = document.getElementById("gbLockTime");
    const dateEl = document.getElementById("gbLockDate");
    if (timeEl) timeEl.textContent = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    if (dateEl) dateEl.textContent = now.toLocaleDateString([], { weekday: "long", month: "long", day: "numeric" });
  }

  return { init, lock, isLocked };
})();

window.LockScreen = LockScreen;