"use strict";
/* Slides delivery mode (#2432) — local presenter console. Current/next slide
 * previews, elapsed timer with pause/reset, speaker notes and an honest
 * audience-view indicator. It is a local preview window: no second display,
 * no audience connection and no server state are created. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var elapsedMs = 0;
  var startedAt = 0;
  var running = false;
  var ticker = null;
  var dialog = null;
  var timeNode = null;
  var audienceNode = null;
  var presentMode = false;

  function elapsed() {
    return running ? elapsedMs + (Date.now() - startedAt) : elapsedMs;
  }
  function start() {
    if (running) return;
    running = true;
    startedAt = Date.now();
    if (ticker === null) ticker = window.setInterval(paintTime, 1000);
  }
  function pause() {
    if (!running) return;
    elapsedMs = elapsed();
    running = false;
    startedAt = 0;
    if (ticker !== null) { window.clearInterval(ticker); ticker = null; }
  }
  function reset() {
    elapsedMs = 0;
    startedAt = running ? Date.now() : 0;
    paintTime();
  }
  function paintTime() {
    if (timeNode) timeNode.textContent = D.formatTime(elapsed());
  }

  function paintPreview(host, index) {
    if (!host) return;
    host.textContent = "";
    var thumbs = document.querySelectorAll(".sl-thumb");
    var thumb = thumbs[index];
    if (!thumb) return;
    Array.prototype.slice.call(thumb.children || []).forEach(function (child) {
      if (child && typeof child.cloneNode === "function") host.appendChild(child.cloneNode(true));
    });
  }

  function existingNotes(index) {
    var saved = D.load().notes[index];
    if (typeof saved === "string") return saved;
    var thumbs = document.querySelectorAll(".sl-thumb");
    var host = thumbs[index] || document.getElementById("slide-canvas");
    if (!host || typeof host.querySelector !== "function") return "";
    var note = host.querySelector(".sl-notes, .sl-speaker-notes, [data-notes]");
    if (!note) return "";
    return String(note.getAttribute("data-notes") || note.textContent || "").trim();
  }

  function nextIndex() {
    var current = D.currentSlide();
    if (D.customShows && typeof D.customShows.isActive === "function" && D.customShows.isActive()) {
      return D.customShows.nextIndex(current);
    }
    return Math.min(D.slideCount() - 1, current + 1);
  }
  function previousIndex() {
    var current = D.currentSlide();
    if (D.customShows && typeof D.customShows.isActive === "function" && D.customShows.isActive()) {
      return D.customShows.prevIndex(current);
    }
    return Math.max(0, current - 1);
  }
  function go(index) {
    if (!D.jump(index)) return;
    if (D.rehearsal && typeof D.rehearsal.lap === "function") D.rehearsal.lap();
    render();
  }

  function paintAudience() {
    if (!audienceNode) return;
    var key = presentMode
      ? "slides-delivery-console-audience-presenter"
      : "slides-delivery-console-audience-local";
    var fallback = presentMode
      ? "Presenter view opened — audience sync unavailable locally"
      : "Local preview — no audience window connected";
    D.labelled(audienceNode, "data-i18n", key, fallback);
    audienceNode.setAttribute("role", "status");
    audienceNode.setAttribute("aria-live", "polite");
  }

  function render() {
    if (!dialog) return;
    var current = D.currentSlide();
    paintPreview(document.getElementById("sd-preview-current"), current);
    paintPreview(document.getElementById("sd-preview-next"), nextIndex());
    var currentLabel = document.getElementById("sd-console-current-label");
    if (currentLabel) {
      D.labelled(currentLabel, "data-i18n", "slides-delivery-console-current", "Current slide");
      currentLabel.setAttribute("data-i18n-params", JSON.stringify({ n: current + 1 }));
    }
    var nextLabel = document.getElementById("sd-console-next-label");
    if (nextLabel) {
      D.labelled(nextLabel, "data-i18n", "slides-delivery-console-next", "Next slide");
      nextLabel.setAttribute("data-i18n-params", JSON.stringify({ n: nextIndex() + 1 }));
    }
    var notes = document.getElementById("sd-console-notes");
    if (notes && document.activeElement !== notes) {
      var text = existingNotes(current);
      notes.value = text;
      var empty = document.getElementById("sd-console-notes-empty");
      if (empty) empty.style.display = text ? "none" : "";
    }
    var counter = document.getElementById("sd-console-counter");
    if (counter) {
      counter.textContent = D.tr("slides-delivery-slide", "Slide { $n }", { n: current + 1 }) +
        " / " + D.slideCount();
    }
    paintAudience();
    paintTime();
  }

  function build(api) {
    api.body.appendChild(D.localNote());

    var audience = D.el("div", "sd-audience");
    audience.id = "sd-console-audience";
    audienceNode = D.el("span", "sd-audience-text");
    audience.appendChild(audienceNode);
    api.body.appendChild(audience);

    var previews = D.el("div", "sd-previews");
    [["sd-preview-current", "sd-console-current-label", "slides-delivery-console-current", "Current slide"],
     ["sd-preview-next", "sd-console-next-label", "slides-delivery-console-next", "Next slide"]]
      .forEach(function (spec) {
        var box = D.el("div", "sd-preview");
        var label = D.el("h3", "sd-preview-label", spec[2], spec[3]);
        label.id = spec[1];
        box.appendChild(label);
        var frame = document.createElement("div");
        frame.className = "sd-preview-frame";
        frame.id = spec[0];
        box.appendChild(frame);
        previews.appendChild(box);
      });
    api.body.appendChild(previews);

    var timer = D.el("div", "sd-timer");
    var timerLabel = D.el("span", "sd-timer-label", "slides-delivery-console-timer", "Elapsed");
    timer.appendChild(timerLabel);
    timeNode = D.el("strong", "sd-timer-value");
    timeNode.textContent = D.formatTime(elapsed());
    timer.appendChild(timeNode);
    timer.appendChild(D.button("slides-delivery-console-pause", "Pause timer", function () {
      if (running) {
        pause();
        D.status("slides-delivery-console-paused", "Timer paused");
      } else {
        start();
        D.status("slides-delivery-console-resumed", "Timer resumed");
      }
      paintPauseLabel();
    }));
    timer.appendChild(D.button("slides-delivery-console-reset", "Reset timer", function () {
      reset();
      D.status("slides-delivery-console-reset-done", "Timer reset");
    }));
    api.body.appendChild(timer);

    var notesHead = D.el("div", "sd-notes-head");
    notesHead.appendChild(D.el("h3", "sd-notes-title", "slides-delivery-console-notes", "Speaker notes"));
    var zoom = document.createElement("select");
    D.labelled(zoom, "data-i18n-aria-label", "slides-delivery-console-notes-zoom", "Notes zoom");
    ["90%", "100%", "115%", "130%"].forEach(function (value) {
      var option = document.createElement("option");
      option.value = value;
      option.textContent = value;
      zoom.appendChild(option);
    });
    zoom.value = "100%";
    zoom.addEventListener("change", function () {
      var notesArea = document.getElementById("sd-console-notes");
      if (notesArea) notesArea.style.fontSize = zoom.value;
    });
    notesHead.appendChild(zoom);
    api.body.appendChild(notesHead);
    var notesArea = document.createElement("textarea");
    notesArea.id = "sd-console-notes";
    notesArea.rows = 4;
    D.labelled(notesArea, "data-i18n-placeholder", "slides-delivery-console-notes-placeholder",
      "Write local speaker notes…");
    D.labelled(notesArea, "data-i18n-aria-label", "slides-delivery-console-notes", "Speaker notes");
    api.body.appendChild(notesArea);
    var empty = D.el("p", "sd-hint", "slides-delivery-console-notes-empty",
      "No speaker notes for this slide.");
    empty.id = "sd-console-notes-empty";
    api.body.appendChild(empty);
    api.body.appendChild(D.button("slides-delivery-console-save-notes", "Save notes locally", function () {
      var index = D.currentSlide();
      D.patch(function (data) { data.notes[index] = notesArea.value; });
      D.status("slides-delivery-console-notes-saved", "Speaker notes saved locally");
    }, "primary"));

    var nav = D.el("div", "sd-console-nav");
    nav.appendChild(D.button("slides-delivery-previous", "Previous", function () { go(previousIndex()); }));
    nav.appendChild(D.button("slides-delivery-next", "Next", function () {
      if (D.customShows && typeof D.customShows.advance === "function" && D.customShows.isActive()) {
        D.customShows.advance();
        render();
        return;
      }
      go(nextIndex());
    }, "primary"));
    var counter = D.el("span", "sd-console-counter");
    counter.id = "sd-console-counter";
    nav.appendChild(counter);
    api.body.appendChild(nav);

    api.addButton("slides-delivery-close", "Close", function () { api.close(); });
    function paintPauseLabel() {
      var buttons = api.body.querySelectorAll(".sd-timer .sd-btn");
      if (buttons[0]) {
        D.labelled(buttons[0], "data-i18n", running
          ? "slides-delivery-console-pause" : "slides-delivery-console-resume",
          running ? "Pause timer" : "Resume timer");
      }
    }
    paintPauseLabel();
    render();
  }

  function open() {
    if (dialog) return dialog;
    start();
    dialog = D.openDialog({
      titleKey: "slides-delivery-console-title",
      titleFallback: "Presenter console (local preview)",
      onBuild: build
    });
    var originalClose = dialog.close;
    dialog.close = function () {
      pause();
      if (ticker !== null) { window.clearInterval(ticker); ticker = null; }
      timeNode = null;
      audienceNode = null;
      dialog = null;
      originalClose();
    };
    D.status("slides-delivery-console-started", "Presentation timer started");
    return dialog;
  }

  D.console = {
    open: open,
    close: function () { if (dialog) dialog.close(); },
    isOpen: function () { return !!dialog; },
    elapsed: elapsed,
    pause: pause,
    reset: reset,
    start: start,
    render: render,
    nextIndex: nextIndex,
    previousIndex: previousIndex,
    setPresentMode: function (active) { presentMode = !!active; paintAudience(); }
  };
})(window);
