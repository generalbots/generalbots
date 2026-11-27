"use strict";
/* Assistant parity: the full free-model notice in Settings > Modelo. */
(function () {
  function init() {
    var D = window.GBFreeDisclosure;
    if (!D) return;
    D.renderSettings();
    document.addEventListener("click", function (event) {
      var el = event.target;
      if (!el || !el.hasAttribute || !el.hasAttribute("data-open-model-settings")) return;
      event.preventDefault();
      if (el.scrollIntoView) el.scrollIntoView({ block: "center" });
    }, true);
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})();
