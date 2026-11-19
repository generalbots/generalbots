"use strict";
/* Slides delivery mode (#2432) — recording indicator. A local state with an
 * elapsed clock and an unmistakable "Recording preview — no media is captured"
 * label. No screen, audio, annotation or Drive capture is performed. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var startedAt = 0;
  var lastDuration = 0;
  var timeNode = null;
  var ticker = null;

  function state() { return D.load().recording; }
  function isActive() { return !!state().active; }
  function elapsed() {
    return isActive() ? Date.now() - startedAt : lastDuration;
  }
  function paintIndicator() {
    var badge = document.getElementById("sd-recording-indicator");
    if (!isActive()) {
      if (ticker !== null) { window.clearInterval(ticker); ticker = null; }
      if (badge && badge.parentNode) badge.parentNode.removeChild(badge);
      return;
    }
    if (ticker === null) ticker = window.setInterval(paintIndicator, 1000);
    if (!badge) {
      badge = document.createElement("div");
      badge.id = "sd-recording-indicator";
      badge.className = "sd-recording-indicator";
      badge.setAttribute("role", "status");
      badge.setAttribute("aria-live", "polite");
      var dot = document.createElement("span");
      dot.className = "sd-recording-dot";
      badge.appendChild(dot);
      var label = D.el("span", "sd-recording-label", "slides-delivery-recording-label",
        "Recording preview — no media is captured");
      badge.appendChild(label);
      timeNode = D.el("strong", "sd-recording-time");
      badge.appendChild(timeNode);
      document.body.appendChild(badge);
    }
    if (timeNode) timeNode.textContent = D.formatTime(elapsed());
  }
  function start() {
    if (isActive()) return false;
    startedAt = Date.now();
    D.patch(function (data) {
      data.recording.active = true;
      data.recording.startedAt = startedAt;
    });
    paintIndicator();
    D.status("slides-delivery-recording-started", "Local recording state started");
    return true;
  }
  function resume() {
    var saved = state();
    if (!saved.active) return false;
    startedAt = Number(saved.startedAt) || Date.now();
    lastDuration = Number(saved.duration) || 0;
    paintIndicator();
    return true;
  }
  function stop() {
    if (!isActive()) return false;
    lastDuration = Date.now() - startedAt;
    D.patch(function (data) {
      data.recording.active = false;
      data.recording.duration = lastDuration;
    });
    paintIndicator();
    timeNode = null;
    D.status("slides-delivery-recording-stopped", "Local recording state stopped");
    return true;
  }

  function open() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-recording-title",
      titleFallback: "Recording preview",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        var warning = D.el("p", "sd-recording-warning", "slides-delivery-recording-label",
          "Recording preview — no media is captured");
        warning.setAttribute("role", "status");
        warning.setAttribute("aria-live", "polite");
        api.body.appendChild(warning);
        api.body.appendChild(D.el("p", "sd-hint", "slides-delivery-recording-note",
          "Capture, narration and playback are not available in this local preview."));

        var meter = D.el("div", "sd-recording-meter");
        meter.appendChild(D.el("span", "sd-timer-label", "slides-delivery-recording-elapsed", "Recording time"));
        var clock = D.el("strong", "sd-timer-value");
        clock.textContent = D.formatTime(elapsed());
        meter.appendChild(clock);
        api.body.appendChild(meter);

        var toggle = D.button(isActive()
          ? "slides-delivery-recording-stop" : "slides-delivery-recording-start",
          isActive() ? "Stop local recording" : "Start local recording", function () {
            if (isActive()) stop(); else start();
            var active = isActive();
            D.labelled(toggle, "data-i18n", active
              ? "slides-delivery-recording-stop" : "slides-delivery-recording-start",
              active ? "Stop local recording" : "Start local recording");
            clock.textContent = D.formatTime(elapsed());
            api.setStatus(active ? "slides-delivery-recording-started" : "slides-delivery-recording-stopped",
              active ? "Local recording state started" : "Local recording state stopped");
          }, "primary");
        api.body.appendChild(toggle);
        api.addButton("slides-delivery-close", "Close", function () { api.close(); });
      }
    });
    return dialog;
  }

  D.recording = {
    state: state,
    isActive: isActive,
    elapsed: elapsed,
    start: start,
    stop: stop,
    resume: resume,
    open: open
  };
})(window);
