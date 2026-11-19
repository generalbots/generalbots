"use strict";
/* Docs equation editor (#2483) — wiring and public surface.
 *
 * Loaded after the authoring modules so the legacy #equationBtn handler is
 * redirected to the accessible editor without touching committed modules. */
(function (window) {
  var NS = (window.DocsEquations = window.DocsEquations || {});

  function closestEquation(node) {
    while (node && node.nodeType === 1) {
      if (node.className && String(node.className).indexOf("docs-eq") >= 0 &&
          String(node.className).indexOf("docs-eq-ref") < 0) return node;
      node = node.parentNode;
    }
    return null;
  }
  function onActivate(event) {
    var target = event.target;
    var node = closestEquation(target);
    if (!node) return;
    event.preventDefault();
    event.stopPropagation();
    NS.openEditor(node);
  }

  NS.init = function () {
    var authoring = window.DocsAuthoring;
    if (authoring) authoring.openEquationModal = function () { NS.openEditor(); };
    if (authoring) authoring.insertEquation = function (tex) { NS.apply({ src: tex, kind: "inline" }, null); };

    document.addEventListener("dblclick", function (event) {
      if (closestEquation(event.target)) onActivate(event);
    }, true);
    document.addEventListener("keydown", function (event) {
      if (event.key !== "Enter" && event.key !== "F2") return;
      if (closestEquation(event.target)) onActivate(event);
    }, true);

    var refBtn = document.getElementById("insertEqRefBtn");
    if (refBtn) refBtn.addEventListener("click", function () { NS.openReferences(); });
    var listBtn = document.getElementById("equationListBtn");
    if (listBtn) listBtn.addEventListener("click", function () { NS.openList(); });

    if (window.addEventListener) {
      window.addEventListener("localeChanged", function () {
        if (typeof NS.renumber === "function") NS.renumber();
      });
    }
    return NS;
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", NS.init, { once: true });
  } else {
    NS.init();
  }
})(window);
