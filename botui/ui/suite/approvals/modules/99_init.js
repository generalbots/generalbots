"use strict";
(function () {
  var A = window.GBApprovals;

  function init() {
    A.applyI18n(document.querySelector('[data-gb-app="approvals"]') || document);
    A.events.bind();
    A.render.renderAll();
    A.events.refresh();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
