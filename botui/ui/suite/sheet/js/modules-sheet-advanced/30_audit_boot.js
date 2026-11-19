"use strict";
/* Sheet advanced module: 30_audit_boot — registers the local-only formula
   auditor toolbar entry and keeps overlays/watch in sync with sheet events. */

(function () {
  var wired = false;

  function onReady(callback) {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", callback, { once: true });
    } else {
      callback();
    }
  }

  function wireButton() {
    var button = document.getElementById("sheetAuditBtn");
    if (!button || button.dataset.auditWired === "true") return;
    button.dataset.auditWired = "true";
    button.addEventListener("click", function (event) {
      if (window.SheetAuditUI) window.SheetAuditUI.toggle(event.currentTarget);
    });
  }

  function wireEvents() {
    if (wired) return;
    wired = true;
    document.addEventListener("gb-sheet-selection", function () {
      if (window.SheetAuditUI) window.SheetAuditUI.rerender();
    });
    document.addEventListener("gb-sheet-tab", function () {
      if (window.SheetAuditOverlay) window.SheetAuditOverlay.chainCellChanges();
      if (window.SheetAuditUI) window.SheetAuditUI.rerender();
    });
    document.addEventListener("gb-sheet-audit-open", function (event) {
      if (window.SheetAuditUI) window.SheetAuditUI.open(event.target || null);
    });
    window.addEventListener("gb-sheet-locale", function () {
      if (window.SheetI18n && typeof window.SheetI18n.localize === "function") {
        window.SheetI18n.localize(document);
      }
    });
  }

  function boot() {
    wireButton();
    wireEvents();
    if (window.SheetAuditOverlay) window.SheetAuditOverlay.boot();
  }

  window.SheetAdvancedAudit = {
    open: function (trigger) {
      return window.SheetAuditUI ? window.SheetAuditUI.open(trigger) : null;
    },
    close: function () {
      if (window.SheetAuditUI) window.SheetAuditUI.close();
    },
    toggle: function (trigger) {
      if (window.SheetAuditUI) window.SheetAuditUI.toggle(trigger);
    },
    refreshGraph: function () {
      return window.SheetAuditUI ? window.SheetAuditUI.refreshGraph() : null;
    },
    updateWatch: function () {
      if (window.SheetAuditSections) window.SheetAuditSections.updateWatch();
    },
    toggleShowFormulas: function () {
      return window.SheetAuditOverlay ? window.SheetAuditOverlay.toggleShowFormulas() : false;
    },
  };

  onReady(boot);
})();
