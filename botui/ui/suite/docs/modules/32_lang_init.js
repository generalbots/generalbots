"use strict";
/* Docs language tools — wiring (#2484). One delegated listener for the dialog
 * panels, one toolbar hook and an opt-in as-you-type autoformat listener. */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  function bindToolbar() {
    var button = document.getElementById("langToolsBtn");
    if (button) {
      button.addEventListener("click", function (event) {
        if (event && typeof event.stopPropagation === "function") event.stopPropagation();
        NS.open();
      });
    }
  }

  function bindPanels() {
    var container = document.getElementById("docs-modal-container");
    if (!container || container.dataset.langBound === "true") return;
    container.dataset.langBound = "true";
    container.addEventListener("click", function (event) { NS.handlePanelClick(event); });
    container.addEventListener("change", function (event) { NS.handlePanelChange(event); });
  }

  function bindAutoformat() {
    document.addEventListener("input", function (event) {
      var target = event && event.target;
      var article = NS.getArticle();
      if (!target || !article || target !== article) return;
      if (!NS.state.autoformat) return;
      NS.maybeAutoformatAtCaret();
    }, true);
  }

  function start() {
    NS.loadState();
    bindToolbar();
    bindPanels();
    bindAutoformat();
  }

  window.DocsLangOpen = function () { NS.open(); };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
