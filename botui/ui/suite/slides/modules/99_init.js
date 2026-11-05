"use strict";
/* slides init — bootstrap on DOMContentLoaded */

// Issue #720 parity: when opened from Drive (or a deep link) the app imports
// the presentation file through the existing /api/slides/import endpoint and
// switches the canvas to the imported deck. The shell URL carries no query
// string, so the args come from window.__gbAppParams__ as well.
function loadDrivePresentationIntoApp() {
  var params = new URLSearchParams(window.location.search);
  var gp = window.__gbAppParams__ || {};
  var file = params.get('file') || gp.file;
  var bucket = params.get('bucket') || gp.bucket;
  if (!file) return;

  // Several office windows (docs/sheet/slides) can be open at once and each
  // carries a #saveStatus — scope the lookup to this app's window body.
  var scope = document.querySelector('[id^="window-body-slides"]') || document;
  var setStatus = function (text) {
    var el = scope.querySelector('#saveStatus');
    if (el) el.textContent = text;
  };
  setStatus('Opening from Drive…');

  fetch('/api/files/download-inline', {
    method: 'POST',
    headers: Object.assign(
      { 'Content-Type': 'application/json' },
      (window.GBSidebarConvos && window.GBSidebarConvos.authHeaders) ? window.GBSidebarConvos.authHeaders() : {}
    ),
    body: JSON.stringify({ bucket: bucket || '', path: file, scope: 'bot' })
  })
    .then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.blob();
    })
    .then(function (blob) {
      var form = new FormData();
      form.append('file', blob, file.split('/').pop() || 'import.pptx');
      return fetch('/api/slides/import', { method: 'POST', body: form });
    })
    .then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    })
    .then(function (presentation) {
      var id = presentation && presentation.id;
      if (!id) throw new Error('import returned no id');
      window.__SLIDES_PRESENTATION_ID = id;
      var titleEl = scope.querySelector('#presTitle');
      if (titleEl) {
        titleEl.value = presentation.name || file.split('/').pop();
        // hx-vals is static JSON; point the rename at the imported deck so
        // it does not create a separate presentation with the id "current".
        titleEl.setAttribute('hx-vals', JSON.stringify({ id: id, action: 'rename' }));
        if (window.htmx) htmx.process(titleEl);
      }
      if (window.htmx) {
        htmx.ajax('GET', '/suite/slides/fragments/presentation-view?id=' + encodeURIComponent(id),
          { target: '#slides-content', swap: 'innerHTML' });
        htmx.ajax('GET', '/suite/slides/fragments/slide-thumbnails?id=' + encodeURIComponent(id),
          { target: '#sidebar-thumbs', swap: 'innerHTML' });
      }
      setStatus('Opened from Drive');
    })
    .catch(function (err) {
      setStatus('Failed to open from Drive: ' + err.message);
    });
}

function slidesInit() {
  // Always re-bind: the desktop shell replaces the window body on every open
  // (fresh DOM). Only the page-level deep-link listener is deduplicated.
  injectCanvasStyles();
  initSidebar();
  initAuth();
  initCollab();
  window.SlidesCanvas = SlideCanvas;
  if (window.SlidesPresenter && window.SlidesPresenter.refreshPresenterStatus) {
    window.SlidesPresenter.refreshPresenterStatus();
  }
  loadDrivePresentationIntoApp();
  if (!window.__gbSlidesDeepLinkBound) {
    window.__gbSlidesDeepLinkBound = true;
    document.addEventListener('gb:deep-link', function (event) {
      var appId = (event.detail || {}).appId || '';
      if (appId === 'slides' || appId.indexOf('slides-') === 0) {
        loadDrivePresentationIntoApp();
      }
    });
  }
}

// The shell injects this fragment after the page has already loaded, so a
// plain DOMContentLoaded listener would never run in window mode.
if (document.readyState === 'loading') {
  window.addEventListener('DOMContentLoaded', slidesInit);
} else {
  slidesInit();
}
