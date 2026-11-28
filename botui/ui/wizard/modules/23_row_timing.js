/* First-run wizard — honest per-row durations on the install step.
   A row's clock starts the first time the provisioner reports it in
   progress and freezes the first time it ends (ready, skipped or failed);
   repeats of the same state never move it again. One 1-second tick
   refreshes only the rows still in progress and stops when none is left. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var rows = {};
  var tick = null;

  /* "1,2s" on a Portuguese page, "1.2s" in English. */
  function seconds(value) {
    var locale = window.WizardI18n ? window.WizardI18n.getLocale() : undefined;
    try { return new Intl.NumberFormat(locale, { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(value); }
    catch (e) { return value.toFixed(1); }
  }

  function wholeTime(ms) {
    var total = Math.max(0, Math.floor(ms / 1000));
    return total < 60 ? total + "s" : Math.floor(total / 60) + "m " + (total % 60) + "s";
  }

  function finishedTime(ms) {
    if (ms == null || ms <= 0) { return ""; }
    if (ms < 1000) { return Math.round(ms) + "ms"; }
    if (ms < 60000) { return seconds(ms / 1000) + "s"; }
    return wholeTime(ms);
  }

  function inProgress(id) {
    var row = rows[id];
    return !!(row && row.start && row.duration === null);
  }

  function text(id) {
    var row = rows[id];
    if (!row) { return ""; }
    if (row.duration !== null) { return finishedTime(row.duration); }
    return row.start ? wholeTime(Date.now() - row.start) : "";
  }

  function durationNode(id) {
    var key = window.CSS && window.CSS.escape ? window.CSS.escape(id) : id;
    return document.querySelector('[data-comp="' + key + '"] .install-step-dur');
  }

  function stopTick() {
    if (tick) { clearInterval(tick); tick = null; }
  }

  function refresh() {
    var open = Object.keys(rows).filter(inProgress);
    open.forEach(function (id) {
      var node = durationNode(id);
      if (node) { node.textContent = text(id); }
    });
    if (!open.length) { stopTick(); }
  }

  function ensureTick() {
    if (!tick) { tick = setInterval(refresh, 1000); }
  }

  /* Records one progress event; only the first start and the first end
     of a row count. */
  function note(id, stateName, elapsedMs) {
    var row = rows[id];
    if (!row) { row = rows[id] = { start: 0, duration: null }; }
    if (row.duration !== null) { return; }
    if (stateName === "starting") {
      if (!row.start) { row.start = Date.now(); ensureTick(); }
    } else if (stateName === "ready" || stateName === "failed") {
      row.duration = row.start ? Date.now() - row.start : (elapsedMs || 0);
      if (!Object.keys(rows).some(inProgress)) { stopTick(); }
    }
  }

  /* Stops every clock still running (the run failed or was cancelled). */
  function freezeOpen() {
    Object.keys(rows).filter(inProgress).forEach(function (id) {
      rows[id].duration = Date.now() - rows[id].start;
      var node = durationNode(id);
      if (node) { node.textContent = text(id); }
    });
    stopTick();
  }

  function reset() {
    rows = {};
    stopTick();
  }

  function running() { return !!tick; }

  App.rowTimer = { note: note, text: text, inProgress: inProgress, freezeOpen: freezeOpen, reset: reset, running: running };
})();
