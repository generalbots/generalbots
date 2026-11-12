"use strict";
/* Timer / Pomodoro (#1154): countdown timer with session tracking and
   desktop notifications on completion. #2261 — a completed session is also
   handed to the bot through the chat send path (`window.sendMessage`). */

(function () {
  if (window.GBTimer) return;

  let totalSeconds = 25 * 60;
  let remaining = totalSeconds;
  let running = false;
  let interval = null;
  let sessions = 0;

  function setMode(min, label) {
    totalSeconds = min * 60;
    remaining = totalSeconds;
    running = false;
    if (interval) { clearInterval(interval); interval = null; }
    updateDisplay();
    const labelEl = document.getElementById("timerLabel");
    if (labelEl) labelEl.textContent = label;
    const startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = "Start";
      startBtn.classList.remove("running");
    }
    document.querySelectorAll(".timer-mode").forEach(function (m) {
      m.classList.toggle("active", parseInt(m.dataset.min, 10) === min);
    });
  }

  function updateDisplay() {
    const el = document.getElementById("timerDisplay");
    if (!el) return;
    const m = Math.floor(remaining / 60);
    const s = remaining % 60;
    el.textContent = String(m).padStart(2, "0") + ":" + String(s).padStart(2, "0");
    document.title = el.textContent + " - Timer";
  }

  function tick() {
    remaining -= 1;
    if (remaining <= 0) {
      remaining = 0;
      updateDisplay();
      stop(true);
      return;
    }
    updateDisplay();
  }

  function start() {
    if (running) return;
    // #2261 — starting again after a completed session must count down afresh;
    // otherwise the first tick sees a spent countdown and fires another "time's
    // up" (and now another bot notification) immediately.
    if (remaining <= 0) remaining = totalSeconds;
    running = true;
    interval = setInterval(tick, 1000);
    const startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = "Pause";
      startBtn.classList.add("running");
    }
  }

  function pause() {
    running = false;
    if (interval) { clearInterval(interval); interval = null; }
    const startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = "Start";
      startBtn.classList.remove("running");
    }
  }

  // #2261 (TIMER-01) — a finished timer must reach the bot, not only the local
  // UI: the toast/browser Notification of `stop()` were the end of the line, so
  // the assistant never learned the session ended and could not confirm it,
  // log it or suggest the next step.
  function timerMessage(label, minutes) {
    return "[Timer] " + label + " finished (" + minutes + " min). " +
      "Please confirm the session is over and suggest the next step.";
  }

  function chatComposerReady() {
    return typeof window.sendMessage === "function" && !!document.getElementById("messageInput");
  }

  function notifyBot(text) {
    var delivered = false;
    if (chatComposerReady()) {
      try {
        window.sendMessage(text);
        delivered = true;
      } catch (e) {
        delivered = false;
      }
    }
    if (!delivered) {
      // The chat surface is not mounted (timer opened on its own, or the desktop
      // never opened Chat): open it and send as soon as its composer exists.
      // Bounded — a stale timer must not poll forever.
      try {
        if (typeof window.openDeepLink === "function") window.openDeepLink("chat", {});
      } catch (e2) { /* no desktop shell: the event below is all we can do */ }
      var attempts = 0;
      var retry = setInterval(function () {
        attempts += 1;
        if (chatComposerReady()) {
          clearInterval(retry);
          try { window.sendMessage(text); } catch (e3) { /* give up quietly */ }
          return;
        }
        if (attempts >= 20) clearInterval(retry);
      }, 250);
    }
    // Other surfaces (and tests) can observe the completion without the chat.
    try {
      window.dispatchEvent(new CustomEvent("gb-timer-complete", {
        detail: { text: text, delivered: delivered },
      }));
    } catch (e4) { /* non-DOM context */ }
    return delivered;
  }

  function stop(completed) {
    running = false;
    if (interval) { clearInterval(interval); interval = null; }
    const startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = "Start";
      startBtn.classList.remove("running");
    }
    if (completed) {
      sessions += 1;
      const sessionsEl = document.getElementById("timerSessions");
      if (sessionsEl) sessionsEl.textContent = sessions + " session" + (sessions === 1 ? "" : "s") + " completed";
      if (window.GBToasts) {
        window.GBToasts.show("Timer", "Time's up! Take a break.", "success");
      }
      // #2261 — hand the completion to the bot through the chat/tool path.
      const labelEl = document.getElementById("timerLabel");
      const label = (labelEl && labelEl.textContent) || "Timer";
      notifyBot(timerMessage(label, Math.round(totalSeconds / 60)));
      // #1816: honour the desktop-notifications preference before firing a
      // native Notification.
      var notifAllowed = typeof window.gbDesktopNotificationsEnabled === "function"
        ? window.gbDesktopNotificationsEnabled()
        : localStorage.getItem("gb-desktop-notifications") !== "false";
      if (notifAllowed) {
        try { new Notification("Timer", { body: "Time's up!" }); } catch (e) {}
      }
    }
  }

  function reset() {
    remaining = totalSeconds;
    running = false;
    if (interval) { clearInterval(interval); interval = null; }
    updateDisplay();
    const startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = "Start";
      startBtn.classList.remove("running");
    }
  }

  // #1981 — the fragment is injected into an already-loaded desktop window,
  // where DOMContentLoaded has long fired; wiring only to that event left the
  // timer with dead buttons. Boot now if the document is past parsing.
  function boot() {
    document.querySelectorAll(".timer-mode").forEach(function (m) {
      m.addEventListener("click", function () {
        setMode(parseInt(m.dataset.min, 10), m.dataset.label);
      });
    });
    const startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.addEventListener("click", function () {
        if (running) pause(); else start();
      });
    }
    const resetBtn = document.getElementById("timerReset");
    if (resetBtn) resetBtn.addEventListener("click", reset);
    setMode(25, "Focus");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }

  window.GBTimer = { start: start, pause: pause, reset: reset, setMode: setMode, notify: notifyBot };
})();
