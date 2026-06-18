"use strict";
/* docs init — bootstrap on DOMContentLoaded */

window.addEventListener("DOMContentLoaded", function () {
  injectEditorStyles();
  initSidebar();
  initAuth();
  initCollab();
  window.DocsEditor = {
    setSaveStatus: setSaveStatus,
    scheduleSave: scheduleSave,
    AI: DocumentAIDriver,
    insertPageBreak: insertPageBreak,
    openHeaderFooterModal: openHeaderFooterModal,
    insertHeaderFooterZone: insertHeaderFooterZone,
    updatePageCount: updatePageCount
  };
});
