"use strict";
/* Sheets structured data (#2436) — public entry points and boot wiring. */

(function () {
  const S = window.SheetStructured;

  function open(kind) {
    switch (kind) {
      case "slicer":
        if (window.SheetStructuredSlicer) window.SheetStructuredSlicer.open();
        break;
      case "goalseek":
        if (window.SheetStructuredGoalSeek) window.SheetStructuredGoalSeek.open();
        break;
      case "scenarios":
        if (window.SheetStructuredScenarios) window.SheetStructuredScenarios.open();
        break;
      case "sparkline":
      default:
        if (window.SheetStructuredSparklines) window.SheetStructuredSparklines.open();
        break;
    }
  }

  function relocalize() {
    const nodes = document.querySelectorAll(".ss-structured-dialog [data-i18n]");
    Array.prototype.slice.call(nodes).forEach(function (node) {
      const key = node.getAttribute("data-i18n");
      if (!key) return;
      const value = window.gbT ? window.gbT(key, node.textContent) : node.textContent;
      node.textContent = value;
    });
    if (window.SheetStructuredSparklines) window.SheetStructuredSparklines.repaint();
  }

  function boot() {
    if (window.SheetStructuredSparklines) window.SheetStructuredSparklines.repaint();
  }

  S.open = open;
  S.openPanel = open;
  S.relocalize = relocalize;

  window.addEventListener("gb-sheet-locale", relocalize);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
