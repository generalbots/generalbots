"use strict";
/* Timer / Pomodoro (#1154): countdown timer with session tracking and
   desktop notifications on completion. Persists state in localStorage. */

(function () {
  if (window.GBTimer) return;

  var STORAGE_KEY = "gb-timer-state-v1";
  var totalSeconds = 25 * 60;
  var remaining = totalSeconds;
  var running = false;
  var interval = null;
  var sessions = 0;
  var endsAt = null; // epoch ms when running
  var currentModeMin = 25;

  function loadState() {
    try {
      var raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return null;
      return JSON.parse(raw);
    } catch (e) { return null; }
  }
  function saveState() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        totalSeconds: totalSeconds,
        remaining: running && endsAt ? Math.max(0, Math.ceil((endsAt - Date.now()) / 1000)) : remaining,
        running: running,
        endsAt: endsAt,
        sessions: sessions,
        modeMin: currentModeMin
      }));
    } catch (e) { /* ignore */ }
  }

  function getLabelForMin(min) {
    var btn = document.querySelector('.timer-mode[data-min="' + String(min) + '"]');
    if (btn) {
      var lbl = (btn.textContent || '').trim();
      if (lbl) return lbl;
      if (btn.getAttribute('data-label')) return btn.getAttribute('data-label');
    }
    if (min === 5) return (window.gbT ? window.gbT('timer-mode-short-break', 'Short Break') : 'Short Break');
    if (min === 15) return (window.gbT ? window.gbT('timer-mode-long-break', 'Long Break') : 'Long Break');
    return (window.gbT ? window.gbT('timer-mode-focus', 'Focus') : 'Focus');
  }

  function setMode(min, labelIgnored) {
    var mins = parseInt(min, 10) || 25;
    currentModeMin = mins;
    totalSeconds = mins * 60;
    remaining = totalSeconds;
    running = false;
    endsAt = null;
    if (interval) { clearInterval(interval); interval = null; }
    updateDisplay();
    var labelEl = document.getElementById("timerLabel");
    if (labelEl) labelEl.textContent = getLabelForMin(mins);
    var startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = (window.gbT ? window.gbT('action-start', 'Start') : 'Start');
      startBtn.classList.remove("running");
    }
    document.querySelectorAll(".timer-mode").forEach(function (m) {
      m.classList.toggle("active", parseInt(m.dataset.min, 10) === mins);
    });
    saveState();
  }

  function updateDisplay() {
    var el = document.getElementById("timerDisplay");
    if (!el) return;
    var r = remaining;
    if (r < 0) r = 0;
    var m = Math.floor(r / 60);
    var s = r % 60;
    el.textContent = String(m).padStart(2, "0") + ":" + String(s).padStart(2, "0");
    if (window.top === window) {
      try { document.title = el.textContent + " - Timer"; } catch (e) { window.gbLogError ? gbLogError(e, "updateDisplay") : console.error("updateDisplay", e); }
    }
  }

  function tick() {
    if (running && endsAt) {
      remaining = Math.max(0, Math.ceil((endsAt - Date.now()) / 1000));
    } else {
      remaining -= 1;
    }
    if (remaining <= 0) {
      remaining = 0;
      updateDisplay();
      saveState();
      stop(true);
      return;
    }
    updateDisplay();
    saveState();
  }

  function start() {
    if (running) return;
    if (remaining <= 0) remaining = totalSeconds;
    running = true;
    endsAt = Date.now() + remaining * 1000;
    if (interval) clearInterval(interval);
    interval = setInterval(tick, 1000);
    var startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = (window.gbT ? window.gbT('action-pause', 'Pause') : 'Pause');
      startBtn.classList.add("running");
    }
    saveState();
  }

  function pause() {
    running = false;
    if (endsAt && remaining > 0) {
      remaining = Math.max(0, Math.ceil((endsAt - Date.now()) / 1000));
    }
    endsAt = null;
    if (interval) { clearInterval(interval); interval = null; }
    var startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = (window.gbT ? window.gbT('action-start', 'Start') : 'Start');
      startBtn.classList.remove("running");
    }
    saveState();
  }

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
      } catch (e) { delivered = false; }
    }
    if (!delivered) {
      try {
        if (typeof window.openDeepLink === "function") window.openDeepLink("chat", {});
      } catch (e2) { window.gbLogError ? gbLogError(e2, "notifyBot") : console.error("notifyBot", e2); }
      var attempts = 0;
      var retry = setInterval(function () {
        attempts += 1;
        if (chatComposerReady()) {
          clearInterval(retry);
          try { window.sendMessage(text); } catch (e3) { window.gbLogError ? gbLogError(e3, "notifyBot") : console.error("notifyBot", e3); }
          return;
        }
        if (attempts >= 20) clearInterval(retry);
      }, 250);
    }
    try {
      window.dispatchEvent(new CustomEvent("gb-timer-complete", {
        detail: { text: text, delivered: delivered },
      }));
    } catch (e4) { window.gbLogError ? gbLogError(e4, "notifyBot") : console.error("notifyBot", e4); }
    return delivered;
  }

  function stop(completed) {
    running = false;
    endsAt = null;
    if (interval) { clearInterval(interval); interval = null; }
    var startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = (window.gbT ? window.gbT('action-start', 'Start') : 'Start');
      startBtn.classList.remove("running");
    }
    if (completed) {
      sessions += 1;
      var sessionsEl = document.getElementById("timerSessions");
      if (sessionsEl) sessionsEl.textContent = sessions + " session" + (sessions === 1 ? "" : "s") + " completed";
      if (window.GBToasts) {
        window.GBToasts.show("Timer", (window.gbT ? window.gbT('timer-time-up', "Time's up! Take a break.") : "Time's up! Take a break."), "success");
      }
      var labelEl = document.getElementById("timerLabel");
      var label = (labelEl && labelEl.textContent) || getLabelForMin(currentModeMin);
      notifyBot(timerMessage(label, Math.round(totalSeconds / 60)));
      var notifAllowed = typeof window.gbDesktopNotificationsEnabled === "function"
        ? window.gbDesktopNotificationsEnabled()
        : localStorage.getItem("gb-desktop-notifications") !== "false";
      if (notifAllowed) {
        try { new Notification("Timer", { body: "Time's up!" }); } catch (e) { window.gbLogError ? gbLogError(e, "stop") : console.error("stop", e); }
      }
    }
    saveState();
  }

  function reset() {
    remaining = totalSeconds;
    running = false;
    endsAt = null;
    if (interval) { clearInterval(interval); interval = null; }
    updateDisplay();
    var startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.textContent = (window.gbT ? window.gbT('action-start', 'Start') : 'Start');
      startBtn.classList.remove("running");
    }
    saveState();
  }

  function restoreState() {
    var st = loadState();
    if (!st || typeof st !== 'object') {
      setMode(25, null);
      return;
    }
    currentModeMin = parseInt(st.modeMin, 10) || 25;
    totalSeconds = parseInt(st.totalSeconds, 10) || (currentModeMin * 60);
    sessions = parseInt(st.sessions, 10) || 0;
    var sessionsEl = document.getElementById("timerSessions");
    if (sessionsEl && sessions > 0) sessionsEl.textContent = sessions + " session" + (sessions === 1 ? "" : "s") + " completed";
    // set active mode UI
    document.querySelectorAll(".timer-mode").forEach(function (m) {
      m.classList.toggle("active", parseInt(m.dataset.min, 10) === currentModeMin);
    });
    var labelEl = document.getElementById("timerLabel");
    if (labelEl) labelEl.textContent = getLabelForMin(currentModeMin);
    if (st.running && st.endsAt) {
      var now = Date.now();
      var rem = Math.ceil((parseInt(st.endsAt, 10) - now) / 1000);
      if (rem > 0) {
        remaining = rem;
        running = true;
        endsAt = parseInt(st.endsAt, 10);
        updateDisplay();
        if (interval) clearInterval(interval);
        interval = setInterval(tick, 1000);
        var startBtn = document.getElementById("timerStart");
        if (startBtn) {
          startBtn.textContent = (window.gbT ? window.gbT('action-pause', 'Pause') : 'Pause');
          startBtn.classList.add("running");
        }
        return;
      } else {
        // timer expired while away
        remaining = 0;
        running = false;
        endsAt = null;
        updateDisplay();
        stop(true);
        return;
      }
    } else {
      remaining = parseInt(st.remaining, 10);
      if (isNaN(remaining) || remaining < 0) remaining = totalSeconds;
      running = false;
      endsAt = null;
      updateDisplay();
      var startBtn2 = document.getElementById("timerStart");
      if (startBtn2) {
        startBtn2.textContent = (window.gbT ? window.gbT('action-start', 'Start') : 'Start');
        startBtn2.classList.remove("running");
      }
    }
  }

  function boot() {
    document.querySelectorAll(".timer-mode").forEach(function (m) {
      m.addEventListener("click", function () {
        setMode(parseInt(m.dataset.min, 10), m.dataset.label);
      });
    });
    var startBtn = document.getElementById("timerStart");
    if (startBtn) {
      startBtn.addEventListener("click", function () {
        if (running) pause(); else start();
      });
    }
    var resetBtn = document.getElementById("timerReset");
    if (resetBtn) resetBtn.addEventListener("click", reset);
    restoreState();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }

  window.GBTimer = { start: start, pause: pause, reset: reset, setMode: setMode, notify: notifyBot };
})();
