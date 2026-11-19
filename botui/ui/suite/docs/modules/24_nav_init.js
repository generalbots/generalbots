"use strict";
/* Boot wiring for the navigation/find-replace/content-library feature (#2485). */

(function () {
  var NS = window.DocsNav;
  if (!NS) return;

  NS.jumpToOffset = function (offset) {
    var article = NS.getArticle();
    if (!article) return null;
    var host = document.getElementById("docs-content") || article.parentElement;
    if (!host) return null;
    var hostTop = host.getBoundingClientRect().top;
    var articleTop = article.getBoundingClientRect().top;
    var top = host.scrollTop + (articleTop - hostTop) + offset;
    if (typeof host.scrollTo === "function") {
      host.scrollTo({ top: Math.max(0, top), behavior: "smooth" });
    } else {
      host.scrollTop = Math.max(0, top);
    }
    return article;
  };

  function bind(id, handler) {
    var el = document.getElementById(id);
    if (el) el.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      handler();
    });
  }

  function onKeydown(event) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z" && !event.shiftKey) {
      var dialog = document.querySelector(".gb-fr-dialog");
      if (dialog && NS.undoReplace()) {
        event.preventDefault();
        event.stopPropagation();
      }
    }
  }

  function boot() {
    if (NS.refreshPane) NS.refreshPane();
    bind("navPaneBtn", function () { NS.openPane(); });
    bind("findReplaceBtn", function () { NS.openFindReplace(); });
    bind("bookmarkBtn", function () { NS.openBookmarkDialog(); });
    bind("quickPartsBtn", function () { NS.openContentLibrary(); });
    if (NS.bindQuickPartKeys) NS.bindQuickPartKeys();
    document.addEventListener("keydown", onKeydown, true);
    document.addEventListener("htmx:afterSwap", function (event) {
      if (event.target && (event.target.id === "docs-content" || event.target.id === "docs-panel-body")) {
        if (NS.refreshPane) NS.refreshPane();
      }
    });
    document.addEventListener("gb:deep-link", function () {
      if (NS.refreshPane) NS.refreshPane();
    });
  }

  if (document.readyState === "loading") {
    window.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
