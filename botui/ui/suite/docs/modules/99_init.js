"use strict";
/* docs init — bootstrap on DOMContentLoaded */

// Issue #720: when opened from the Drive app (?file=...&bucket=...), load the
// document content from the drive and render it into the editor.
function loadDriveFileIntoEditor() {
  // Top-level navigation carries the args in the URL; a window opened by the
  // desktop shell (Drive "Open", deep link) receives them only via
  // window.__gbAppParams__ (the shell URL has no query string of its own).
  var params = new URLSearchParams(window.location.search);
  var gp = window.__gbAppParams__ || {};
  var file = params.get('file') || gp.file;
  var bucket = params.get('bucket') || gp.bucket;
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

function docsInit() {
  // Always re-bind: the desktop shell replaces the window body on every
  // open, so the previous DOM (and its handlers) is gone. Only the
  // page-level deep-link listener below is deduplicated.
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
  // Re-targeting an already-open docs window (Drive "Open" a second file)
  // never re-runs DOMContentLoaded — the shell dispatches gb:deep-link.
  if (!window.__gbDocsDeepLinkBound) {
    window.__gbDocsDeepLinkBound = true;
    document.addEventListener("gb:deep-link", function (event) {
      var appId = (event.detail || {}).appId || "";
      if (appId === "docs" || appId.indexOf("docs-") === 0) {
        loadDriveFileIntoEditor();
      }
    });
  }
}

// The desktop shell injects this fragment after the page has loaded, so the
// DOMContentLoaded event has already fired and its listener would never run.
if (document.readyState === "loading") {
  window.addEventListener("DOMContentLoaded", docsInit);
} else {
  docsInit();
}
