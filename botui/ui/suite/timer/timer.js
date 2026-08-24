"use strict";
/* Timer / Pomodoro (#1154): countdown timer with session tracking and
   desktop notifications on completion. */

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
        window.GBToasts.show("Timer", "Time's up! Take a break. 🎉", "success");
      } else {
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

  document.addEventListener("DOMContentLoaded", function () {
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
  });

  window.GBTimer = { start: start, pause: pause, reset: reset, setMode: setMode };
})();