"use strict";
/* Slides object model — boot wiring (#2430). One toolbar entry opens the local
 * editor; keyboard shortcuts are scoped to the editor so they never steal the
 * existing SlidesUndo / outline shortcuts used elsewhere in the app. */

(function () {
  var OM = window.GBSlidesOM;
  var Dialog = window.GBSlidesOMDialog;
  var Canvas = window.GBSlidesOMCanvas;
  var Ops = window.GBSlidesOMOps;

  function bindToolbar() {
    var button = document.getElementById("slObjectModelBtn");
    if (!button || button.__omBound) return;
    button.__omBound = true;
    button.addEventListener("click", function (event) {
      event.preventDefault();
      OM.init();
      if (typeof Dialog.open === "function") Dialog.open();
    });
  }

  function onCaptureKey(event) {
    if (!Dialog.isOpen()) return;
    var key = (event.key || "").toLowerCase();
    var target = event.target;
    if (target && target.closest && target.closest("input,textarea,select,[contenteditable='true']")) return;
    if ((event.ctrlKey || event.metaKey) && !event.altKey && key === "z") {
      event.preventDefault();
      event.stopImmediatePropagation();
      if (event.shiftKey) OM.redo(); else OM.undo();
      return;
    }
    if ((event.ctrlKey || event.metaKey) && !event.altKey && key === "y") {
      event.preventDefault();
      event.stopImmediatePropagation();
      OM.redo();
      return;
    }
    if ((event.ctrlKey || event.metaKey) && key === "a") {
      event.preventDefault();
      event.stopImmediatePropagation();
      OM.selectAll();
      return;
    }
    if (key === "escape" && OM.selection.length) {
      OM.clearSelection();
    }
  }

  function bind() {
    OM.init();
    if (Canvas && typeof Canvas.bind === "function") Canvas.bind();
    bindToolbar();
    if (!window.__omInitKeyBound) {
      window.__omInitKeyBound = true;
      document.addEventListener("keydown", onCaptureKey, true);
    }
    if (Canvas && typeof Canvas.render === "function") Canvas.render();
  }

  if (typeof document !== "undefined") {
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind);
    else bind();
    document.body.addEventListener("htmx:afterSwap", function () {
      setTimeout(bindToolbar, 0);
    });
  }

  window.GBSlidesObjectModel = {
    state: OM,
    ops: Ops,
    canvas: Canvas,
    dialog: Dialog
  };
}());
