"use strict";
/* Assistant parity: the free-model reminder above the chat composer. */
(function () {
  function init() {
    var D = window.GBFreeDisclosure;
    if (!D) return;
    D.renderComposerBar();
    document.addEventListener("change", function (event) {
      if (event.target && event.target.id === "modelPicker") D.renderComposerBar();
    }, true);
    document.addEventListener("click", function (event) {
      var el = event.target;
      if (!el || !el.hasAttribute || !el.hasAttribute("data-open-model-settings")) return;
      event.preventDefault();
      if (window.openApp) window.openApp("settings");
      else window.location.hash = "#settings";
    }, true);
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})();
