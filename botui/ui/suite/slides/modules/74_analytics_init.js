"use strict";
/* Slides analytics (#2489) — layer 8, dialog entry points and toolbar wiring.
 * Two local-only buttons are added next to the delivery controls; the existing
 * share and presenter controls are left untouched. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var bound = false;

  A.open = A.open || {};

  A.open.console = function () {
    var state = A.init();
    var dialog = A.openDialog({
      titleKey: "slides-analytics-console-title",
      titleFallback: "Audience console (local preview)",
      onBuild: function (api) { A.panels.console(api, state); }
    });
    return dialog;
  };

  A.open.report = function () { return openPanel("slides-analytics-report-title",
    "Presentation analytics (local report)", A.panels.report); };
  A.open.qa = function () { return openPanel("slides-analytics-qa-title",
    "Audience Q&A (local)", A.panels.qa); };
  A.open.polls = function () { return openPanel("slides-analytics-polls-title",
    "Polls (local)", A.panels.polls); };
  A.open.follow = function () { return openPanel("slides-analytics-follow-title",
    "Follow-along (simulated)", A.panels.follow); };
  A.open.export = function () { return openPanel("slides-analytics-export-title",
    "Export and retention", A.panels.export); };

  function openPanel(titleKey, titleFallback, render) {
    var state = A.init();
    return A.openDialog({
      titleKey: titleKey,
      titleFallback: titleFallback,
      onBuild: function (api) { render(api, state); }
    });
  }

  A.open.tools = function () {
    var dialog = A.openDialog({
      titleKey: "slides-analytics-tools",
      titleFallback: "Audience tools",
      onBuild: function (api) {
        api.body.appendChild(A.localNote());
        [["slides-analytics-section-report", "Analytics report", "report"],
          ["slides-analytics-section-qa", "Q&A", "qa"],
          ["slides-analytics-section-polls", "Polls", "polls"],
          ["slides-analytics-section-follow", "Follow-along", "follow"],
          ["slides-analytics-section-export", "Export", "export"]].forEach(function (entry) {
          api.body.appendChild(A.button(entry[0], entry[1], function () {
            dialog.close();
            A.open[entry[2]]();
          }, "menu"));
        });
        api.addButton("slides-analytics-close", "Close", function () { dialog.close(); });
      }
    });
    return dialog;
  };

  function seedDevices(state) {
    if (!state.follow || !Array.isArray(state.follow.devices) || state.follow.devices.length) return;
    A.follow.addDevice(state, { id: "device-local-1", ts: Date.now() });
    A.follow.addDevice(state, { id: "device-local-2", ts: Date.now() });
  }

  function bind() {
    if (bound) return;
    var consoleButton = document.getElementById("slAnalyticsConsoleBtn");
    var toolsButton = document.getElementById("slAnalyticsToolsBtn");
    if (!consoleButton && !toolsButton) return;
    bound = true;
    if (consoleButton) {
      consoleButton.addEventListener("click", function () { A.open.console(); });
    }
    if (toolsButton) {
      toolsButton.addEventListener("click", function () { A.open.tools(); });
    }
  }

  A.initUI = function () {
    var state = A.init();
    if (state.console && state.console.startedAt === A.BASE_TS) {
      state.console.startedAt = Date.now();
    }
    seedDevices(state);
    A.save(state);
    bind();
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", A.initUI);
  } else {
    A.initUI();
  }
})(typeof window !== "undefined" ? window : globalThis);
