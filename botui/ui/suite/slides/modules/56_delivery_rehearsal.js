"use strict";
/* Slides delivery mode (#2432) — rehearsal targets, a practice timer and an
 * over/under summary. Timings persist locally; applying them to a server-side
 * PPTX timing model is explicitly out of scope for this local layer. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var accumulated = 0;
  var startedAt = 0;
  var running = false;
  var lastSlide = null;
  var lastLapAt = 0;
  var timeNode = null;

  function rehearsal() { return D.load().rehearsal; }
  function targets() { return rehearsal().targets || {}; }
  function actual() { return rehearsal().actual || {}; }

  function setTarget(index, seconds) {
    var value = Math.max(0, Math.floor(Number(seconds) || 0));
    D.patch(function (data) { data.rehearsal.targets[index] = value; });
    return value;
  }
  function elapsed() {
    return accumulated + (running ? Date.now() - startedAt : 0);
  }
  function start() {
    if (running) return;
    running = true;
    startedAt = Date.now();
    if (lastSlide === null) { lastSlide = D.currentSlide(); lastLapAt = Date.now(); }
  }
  function pause() {
    if (!running) return;
    accumulated = elapsed();
    running = false;
    startedAt = 0;
  }
  function reset() {
    accumulated = 0;
    startedAt = running ? Date.now() : 0;
    lastSlide = D.currentSlide();
    lastLapAt = Date.now();
    paint();
  }
  function lap() {
    var now = Date.now();
    var current = D.currentSlide();
    if (lastSlide !== null && lastLapAt > 0) {
      var seconds = Math.max(0, Math.round((now - lastLapAt) / 1000));
      D.patch(function (data) {
        var key = String(lastSlide);
        data.rehearsal.actual[key] = (Number(data.rehearsal.actual[key]) || 0) + seconds;
      });
    }
    lastSlide = current;
    lastLapAt = now;
    paint();
    return now;
  }

  function rows() {
    var targetMap = targets();
    var actualMap = actual();
    var keys = Object.keys(targetMap).concat(Object.keys(actualMap));
    var unique = keys.filter(function (key, position) { return keys.indexOf(key) === position; });
    unique.sort(function (a, b) { return (Number(a) || 0) - (Number(b) || 0); });
    return unique.map(function (key) {
      var target = Number(targetMap[key]);
      var actualValue = Number(actualMap[key]);
      var hasTarget = isFinite(target);
      var hasActual = isFinite(actualValue);
      var delta = hasTarget && hasActual ? actualValue - target : 0;
      return {
        index: Number(key) || 0,
        target: hasTarget ? target : null,
        actual: hasActual ? actualValue : null,
        delta: delta,
        status: !hasTarget || !hasActual ? "missing" : (delta > 0 ? "over" : (delta < 0 ? "under" : "on"))
      };
    });
  }
  function apply() {
    D.patch(function (data) { data.rehearsal.appliedAt = Date.now(); });
    D.status("slides-delivery-rehearsal-applied",
      "Timings stored locally — server timing export is not available");
    return true;
  }
  function paint() {
    if (timeNode) timeNode.textContent = D.formatTime(elapsed());
  }

  function open() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-rehearsal-title",
      titleFallback: "Rehearsal",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        var current = D.currentSlide();

        var targetField = D.field("slides-delivery-rehearsal-target", "Target (s)",
          targets()[current] === undefined ? "" : targets()[current]);
        targetField.control.type = "number";
        targetField.control.min = "0";
        D.labelled(targetField.control, "data-i18n-placeholder",
          "slides-delivery-rehearsal-target-placeholder", "60");
        api.body.appendChild(targetField);
        api.addButton("slides-delivery-rehearsal-save", "Save target", function () {
          setTarget(current, targetField.control.value);
          api.setStatus("slides-delivery-saved", "Saved locally");
          renderRows();
        }, "primary");

        var timer = D.el("div", "sd-rehearsal-timer");
        timer.appendChild(D.el("span", "sd-timer-label", "slides-delivery-rehearsal-timer", "Practice timer"));
        timeNode = D.el("strong", "sd-timer-value");
        paint();
        timer.appendChild(timeNode);
        timer.appendChild(D.button("slides-delivery-rehearsal-start", "Start", start));
        timer.appendChild(D.button("slides-delivery-rehearsal-pause", "Pause", pause));
        timer.appendChild(D.button("slides-delivery-rehearsal-lap", "Record slide timing", function () {
          lap();
          api.setStatus("slides-delivery-rehearsal-lap-done", "Slide timing recorded locally");
          renderRows();
        }));
        timer.appendChild(D.button("slides-delivery-console-reset", "Reset timer", reset));
        api.body.appendChild(timer);

        api.body.appendChild(D.el("h3", "sd-subhead", "slides-delivery-rehearsal-summary", "Timing summary"));
        var list = D.el("div", "sd-rehearsal-list");
        list.id = "sd-rehearsal-list";
        api.body.appendChild(list);

        function statusText(row) {
          if (row.status === "over") {
            return D.tr("slides-delivery-rehearsal-over", "Over target by { $n } s", { n: row.delta });
          }
          if (row.status === "under") {
            return D.tr("slides-delivery-rehearsal-under", "Under target by { $n } s", { n: Math.abs(row.delta) });
          }
          if (row.status === "on") {
            return D.tr("slides-delivery-rehearsal-on-target", "On target");
          }
          return D.tr("slides-delivery-rehearsal-missing", "Not rehearsed yet");
        }
        function renderRows() {
          list.textContent = "";
          var data = rows();
          if (!data.length) {
            list.appendChild(D.el("p", "sd-hint", "slides-delivery-rehearsal-none", "No timings recorded yet."));
            return;
          }
          data.forEach(function (row) {
            var line = D.el("div", "sd-rehearsal-row sd-rehearsal-" + row.status);
            var label = D.el("span", "sd-rehearsal-slide");
            label.textContent = D.tr("slides-delivery-slide", "Slide { $n }", { n: row.index + 1 });
            line.appendChild(label);
            var detail = D.el("span", "sd-rehearsal-detail");
            detail.textContent = (row.target === null ? "—" : row.target + "s") + " / " +
              (row.actual === null ? "—" : row.actual + "s") + " · " + statusText(row);
            line.appendChild(detail);
            list.appendChild(line);
          });
        }
        renderRows();
        api.addButton("slides-delivery-rehearsal-apply", "Apply timings (local)", function () {
          apply();
          api.setStatus("slides-delivery-rehearsal-applied",
            "Timings stored locally — server timing export is not available");
        }, "primary");
        api.addButton("slides-delivery-close", "Close", function () {
          if (timeNode) timeNode = null;
          api.close();
        });
      }
    });
    return dialog;
  }

  D.rehearsal = {
    targets: targets,
    actual: actual,
    setTarget: setTarget,
    elapsed: elapsed,
    start: start,
    pause: pause,
    reset: reset,
    lap: lap,
    rows: rows,
    apply: apply,
    open: open
  };
})(window);
