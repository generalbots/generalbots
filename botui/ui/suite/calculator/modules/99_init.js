"use strict";

// Calculator app bootstrap (suite/calculator). The window manager re-runs
// this script on every open, so init is idempotent per DOM.

(function (app) {
  function init() {
    var box = document.getElementById("gb-calc-root") ||
      (document.currentScript ? document.currentScript.closest(".gb-calc") : null);
    if (!box || box.dataset.calcInit === "1") return;
    box.dataset.calcInit = "1";

    app.loadState();
    app.renderPads();
    app.renderTabs();
    app.renderAngle();
    app.renderMemory();
    app.renderHistory();
    app.renderScreen();

    var exprInput = box.querySelector("#gb-calc-expr");
    if (exprInput) exprInput.focus();

    app.bindEvents();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})(window.GBCalc);
