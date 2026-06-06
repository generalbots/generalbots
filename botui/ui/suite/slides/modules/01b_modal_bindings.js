// slides/modules/01b_modal_bindings.js
"use strict";

(function () {
  function bind(id, modalId) {
    const el = document.getElementById(id);
    if (el && window.slidesApp && typeof window.slidesApp.hideModal === "function") {
      el.addEventListener("click", () => window.slidesApp.hideModal(modalId));
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      bind("closeShareModal", "shareModal");
      bind("closeImageModal", "imageModal");
      bind("closeShapeModal", "shapeModal");
      bind("closeNotesModal", "notesModal");
      bind("closeBackgroundModal", "backgroundModal");
      bind("cancelImageBtn", "imageModal");
      bind("cancelNotesBtn", "notesModal");
      bind("cancelBackgroundBtn", "backgroundModal");
    });
  } else {
    bind("closeShareModal", "shareModal");
    bind("closeImageModal", "imageModal");
    bind("closeShapeModal", "shapeModal");
    bind("closeNotesModal", "notesModal");
    bind("closeBackgroundModal", "backgroundModal");
    bind("cancelImageBtn", "imageModal");
    bind("cancelNotesBtn", "notesModal");
    bind("cancelBackgroundBtn", "backgroundModal");
  }
})();
