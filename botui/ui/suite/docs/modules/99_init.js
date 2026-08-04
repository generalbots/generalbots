"use strict";
/* docs init — bootstrap on DOMContentLoaded */

// Issue #720: when opened from the Drive app (?file=...&bucket=...), load the
// document content from the drive and render it into the editor.
function loadDriveFileIntoEditor() {
  var params = new URLSearchParams(window.location.search);
  var file = params.get('file');
  var bucket = params.get('bucket');
  if (!file) return;

  fetch('/api/docs/open-from-drive', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ bucket: bucket || '', path: file })
  })
    .then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    })
    .then(function (doc) {
      var titleEl = document.getElementById('docTitle');
      if (titleEl) titleEl.value = doc.title || file.split('/').pop();
      var contentEl = document.getElementById('docs-content');
      var article = contentEl && contentEl.querySelector('article[contenteditable]');
      if (!article && contentEl) {
        article = document.createElement('article');
        article.className = 'docs-doc-view';
        article.setAttribute('contenteditable', 'true');
        article.id = 'doc-content';
        contentEl.appendChild(article);
        attachEditorHandlers(contentEl);
      }
      if (article) {
        article.innerHTML = doc.content || '';
        if (article.dataset) article.dataset.docId = doc.id || file;
        if (window.updatePageCount) window.updatePageCount();
      }
      setSaveStatus('Opened from Drive', false);
    })
    .catch(function (err) {
      setSaveStatus('Failed to open from Drive: ' + err.message, true);
    });
}

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
  loadDriveFileIntoEditor();
});
