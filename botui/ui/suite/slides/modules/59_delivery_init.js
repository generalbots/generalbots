"use strict";
/* Slides delivery mode (#2432) — toolbar wiring. Two buttons are added next to
 * the existing Presenter View control: a local presenter console and a menu of
 * local delivery tools. The existing HTMX presenter button is left untouched. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var bound = false;

  function tools() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-tools-title",
      titleFallback: "Delivery tools",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        var entries = [
          ["slides-delivery-section-shows", "Custom shows", function () { D.customShows.open(); }],
          ["slides-delivery-section-sections", "Sections", function () { D.sections.open(); }],
          ["slides-delivery-section-captions", "Captions", function () { D.captions.open(); }],
          ["slides-delivery-section-rehearsal", "Rehearsal", function () { D.rehearsal.open(); }],
          ["slides-delivery-section-handouts", "Handouts", function () { D.handouts.open(); }],
          ["slides-delivery-section-recording", "Recording", function () { D.recording.open(); }]
        ];
        entries.forEach(function (entry) {
          api.body.appendChild(D.button(entry[0], entry[1], function () {
            dialog.close();
            entry[2]();
          }, "menu"));
        });
        api.addButton("slides-delivery-close", "Close", function () { dialog.close(); });
      }
    });
    return dialog;
  }

  function bind() {
    if (bound) return;
    var consoleButton = document.getElementById("slDeliveryConsoleBtn");
    var toolsButton = document.getElementById("slDeliveryToolsBtn");
    var presenterButton = document.getElementById("presenterBtn");
    if (!consoleButton && !toolsButton && !presenterButton) return;
    bound = true;
    if (consoleButton) {
      consoleButton.addEventListener("click", function () { D.console.open(); });
    }
    if (toolsButton) {
      toolsButton.addEventListener("click", function () { tools(); });
    }
    if (presenterButton) {
      presenterButton.addEventListener("click", function () {
        D.console.setPresentMode(true);
      }, true);
    }
  }

  D.tools = tools;
  D.init = function () {
    bind();
    D.captions.apply();
    D.recording.resume();
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", D.init);
  } else {
    D.init();
  }
})(window);
