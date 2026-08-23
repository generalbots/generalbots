"use strict";

// GB Clock app (suite/clock): world time, stopwatch with laps and a
// countdown timer. Stopwatch/timer use timestamp math so they remain
// accurate when the tab is throttled or backgrounded.

window.GBClockApp = window.GBClockApp || {};

(function (app) {
  var FALLBACK_ZONES = [
    "UTC", "America/Sao_Paulo", "America/New_York", "America/Los_Angeles",
    "Europe/London", "Europe/Berlin", "Africa/Cairo", "Asia/Dubai",
    "Asia/Tokyo", "Australia/Sydney",
  ];

  var state = {
    tab: "time",
    clockTimer: null,
    swRunning: false,
    swStartTs: 0,
    swAccumulated: 0,
    swTimer: null,
    swLaps: [],
    tTotalMs: 0,
    tDeadline: 0,
    tRemainingMs: 0,
    tRunning: false,
    tTimer: null,
    audioCtx: null,
  };

  function root() {
    return document.getElementById("gb-clock-root") ||
      (document.currentScript ? document.currentScript.closest(".gb-clock") : null);
  }

  function pad(n, width) {
    var s = String(Math.floor(Math.abs(n)));
    while (s.length < (width || 2)) s = "0" + s;
    return n < 0 ? "-" + s : s;
  }

  // ── World time ───────────────────────────────────────────────

  function fillTimezones(select) {
    var zones = FALLBACK_ZONES;
    try {
      if (Intl.supportedValuesOf) zones = Intl.supportedValuesOf("timeZone");
    } catch (e) { /* keep fallback list */ }
    select.innerHTML = "";
    zones.forEach(function (zone) {
      var opt = document.createElement("option");
      opt.value = zone;
      var label = zone.split("/").pop().replace(/_/g, " ");
      opt.textContent = label + " (" + zone + ")";
      select.appendChild(opt);
    });
    var guess = "UTC";
    try { guess = Intl.DateTimeFormat().resolvedOptions().timeZone; } catch (e) {}
    if (zones.indexOf(guess) !== -1) select.value = guess;
  }

  function tickClock() {
    var box = root();
    if (!box) return;
    var digital = box.querySelector("#gb-clock-digital");
    var dateEl = box.querySelector("#gb-clock-date");
    var tz = box.querySelector("#gb-clock-tz");
    if (!digital) return;
    var zone = tz ? tz.value : undefined;
    try {
      var now = new Date();
      digital.textContent = new Intl.DateTimeFormat([], {
        hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false,
        timeZone: zone,
      }).format(now);
      if (dateEl) {
        dateEl.textContent = new Intl.DateTimeFormat([], {
          weekday: "long", year: "numeric", month: "long", day: "numeric",
          timeZone: zone,
        }).format(now);
      }
    } catch (e) {
      digital.textContent = "--:--:--";
    }
  }

  function startClock() {
    if (state.clockTimer) return;
    tickClock();
    state.clockTimer = setInterval(tickClock, 1000);
    document.addEventListener("visibilitychange", function () {
      if (document.hidden) {
        clearInterval(state.clockTimer);
        state.clockTimer = null;
      } else {
        startClock();
      }
    });
  }

  // ── Stopwatch ────────────────────────────────────────────────

  function formatStopwatch(ms) {
    var minutes = Math.floor(ms / 60000);
    var seconds = Math.floor((ms % 60000) / 1000);
    var tenths = Math.floor((ms % 1000) / 100);
    return pad(minutes) + ":" + pad(seconds) + "." + tenths;
  }

  function renderStopwatch() {
    var box = root();
    if (!box) return;
    var display = box.querySelector("#gb-clock-sw-display");
    if (!display) return;
    var elapsed = state.swAccumulated +
      (state.swRunning ? Date.now() - state.swStartTs : 0);
    display.textContent = formatStopwatch(elapsed);
  }

  function toggleStopwatch() {
    var box = root();
    var btn = box ? box.querySelector("#gb-clock-sw-start") : null;
    if (state.swRunning) {
      state.swAccumulated += Date.now() - state.swStartTs;
      state.swRunning = false;
      if (btn) btn.textContent = "Start";
      clearInterval(state.swTimer);
      state.swTimer = null;
    } else {
      state.swStartTs = Date.now();
      state.swRunning = true;
      if (btn) btn.textContent = "Pause";
      state.swTimer = setInterval(renderStopwatch, 100);
    }
    renderStopwatch();
  }

  function resetStopwatch() {
    state.swRunning = false;
    state.swAccumulated = 0;
    state.swLaps = [];
    clearInterval(state.swTimer);
    state.swTimer = null;
    var box = root();
    var btn = box ? box.querySelector("#gb-clock-sw-start") : null;
    if (btn) btn.textContent = "Start";
    var laps = box ? box.querySelector("#gb-clock-sw-laps") : null;
    if (laps) laps.innerHTML = "";
    renderStopwatch();
  }

  function addLap() {
    if (!state.swRunning) return;
    var box = root();
    var lapsEl = box ? box.querySelector("#gb-clock-sw-laps") : null;
    if (!lapsEl) return;
    var total = state.swAccumulated + (Date.now() - state.swStartTs);
    var previous = state.swLaps.reduce(function (acc, lap) { return acc + lap.split; }, 0);
    var split = total - previous;
    state.swLaps.push({ split: split, total: total });
    var li = document.createElement("li");
    var name = document.createElement("span");
    name.textContent = "Lap " + state.swLaps.length;
    var value = document.createElement("span");
    value.textContent = formatStopwatch(split);
    li.appendChild(name);
    li.appendChild(value);
    lapsEl.insertBefore(li, lapsEl.firstChild);
  }

  // ── Timer ────────────────────────────────────────────────────

  function beep(times) {
    try {
      if (!state.audioCtx) {
        var Ctx = window.AudioContext || window.webkitAudioContext;
        if (!Ctx) return;
        state.audioCtx = new Ctx();
      }
      var ctx = state.audioCtx;
      for (var i = 0; i < times; i++) {
        var osc = ctx.createOscillator();
        var gain = ctx.createGain();
        osc.type = "square";
        osc.frequency.value = 880;
        gain.gain.setValueAtTime(0.08, ctx.currentTime + i * 0.22);
        osc.connect(gain).connect(ctx.destination);
        osc.start(ctx.currentTime + i * 0.22);
        osc.stop(ctx.currentTime + i * 0.22 + 0.12);
      }
    } catch (e) { /* audio unavailable: visual flash only */ }
  }

  function readTimerInputs() {
    var box = root();
    if (!box) return 0;
    var minutes = parseInt(box.querySelector("#gb-clock-t-min").value, 10) || 0;
    var seconds = parseInt(box.querySelector("#gb-clock-t-sec").value, 10) || 0;
    return Math.max(0, minutes * 60000 + seconds * 1000);
  }

  function renderTimer(remainingMs) {
    var box = root();
    if (!box) return;
    var display = box.querySelector("#gb-clock-t-display");
    var fill = box.querySelector("#gb-clock-t-fill");
    if (display) {
      var totalSeconds = Math.ceil(remainingMs / 1000);
      display.textContent = pad(totalSeconds / 60) + ":" + pad(totalSeconds % 60);
      display.classList.toggle("done", state.tRunning === false && remainingMs === 0 && state.tTotalMs > 0);
    }
    if (fill && state.tTotalMs > 0) {
      fill.style.width = Math.max(0, Math.min(100, (remainingMs / state.tTotalMs) * 100)) + "%";
    }
  }

  function timerTick() {
    var remaining = Math.max(0, state.tDeadline - Date.now());
    state.tRemainingMs = remaining;
    if (remaining === 0) {
      pauseTimer();
      beep(3);
    }
    // Render after pause so the finished state (flash) is reflected.
    renderTimer(remaining);
  }

  function startPauseTimer() {
    var box = root();
    var btn = box ? box.querySelector("#gb-clock-t-start") : null;
    if (state.tRunning) {
      pauseTimer();
      return;
    }
    if (state.tRemainingMs <= 0) {
      state.tTotalMs = readTimerInputs();
      state.tRemainingMs = state.tTotalMs;
      if (state.tTotalMs <= 0) return;
    }
    state.tDeadline = Date.now() + state.tRemainingMs;
    state.tRunning = true;
    if (btn) btn.textContent = "Pause";
    state.tTimer = setInterval(timerTick, 200);
    timerTick();
  }

  function pauseTimer() {
    state.tRunning = false;
    clearInterval(state.tTimer);
    state.tTimer = null;
    var box = root();
    var btn = box ? box.querySelector("#gb-clock-t-start") : null;
    if (btn) btn.textContent = "Start";
  }

  function resetTimer() {
    pauseTimer();
    state.tTotalMs = readTimerInputs();
    state.tRemainingMs = state.tTotalMs;
    renderTimer(state.tRemainingMs);
  }

  // ── Tabs and wiring ──────────────────────────────────────────

  function renderTabs(box) {
    box.querySelectorAll(".gb-clock-tab").forEach(function (tab) {
      tab.classList.toggle("active", tab.getAttribute("data-tab") === state.tab);
    });
    box.querySelector("#gb-clock-time").hidden = state.tab !== "time";
    box.querySelector("#gb-clock-stopwatch").hidden = state.tab !== "stopwatch";
    box.querySelector("#gb-clock-timer").hidden = state.tab !== "timer";
  }

  app.init = function () {
    var box = root();
    if (!box || box.dataset.clockInit === "1") return;
    box.dataset.clockInit = "1";

    fillTimezones(box.querySelector("#gb-clock-tz"));

    box.querySelectorAll(".gb-clock-tab").forEach(function (tab) {
      tab.addEventListener("click", function () {
        state.tab = tab.getAttribute("data-tab");
        renderTabs(box);
      });
    });

    box.querySelector("#gb-clock-tz").addEventListener("change", tickClock);

    box.querySelector("#gb-clock-sw-start").addEventListener("click", toggleStopwatch);
    box.querySelector("#gb-clock-sw-lap").addEventListener("click", addLap);
    box.querySelector("#gb-clock-sw-reset").addEventListener("click", resetStopwatch);

    box.querySelector("#gb-clock-t-start").addEventListener("click", startPauseTimer);
    box.querySelector("#gb-clock-t-reset").addEventListener("click", resetTimer);

    ["#gb-clock-t-min", "#gb-clock-t-sec"].forEach(function (sel) {
      box.querySelector(sel).addEventListener("input", function () {
        if (state.tRunning) return;
        state.tTotalMs = readTimerInputs();
        state.tRemainingMs = state.tTotalMs;
        renderTimer(state.tRemainingMs);
      });
    });

    startClock();
    renderStopwatch();
    resetTimer();
  };
})(window.GBClockApp);

(function () {
  function boot() { window.GBClockApp.init(); }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
