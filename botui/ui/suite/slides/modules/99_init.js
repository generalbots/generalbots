"use strict";
/* slides init — bootstrap on DOMContentLoaded */

// Deck reference carried by a chat/command deep link. `app://slides?deck_id=…`
// (and the `file_id` variant the curated verbs emit) names a stored deck, but
// this boot only understood bucket+path: every id link fell through and the
// shell stayed on the blank deck addressed as "current", so inserting a slide
// changed a deck nobody was looking at (UAT SLIDE-02). Same class of bug the
// sheet boot had (UAT SHEET-01).
function deckRefFromParams(params, gp) {
  return params.get('deck_id') || params.get('file_id') || params.get('id') ||
    gp.deck_id || gp.file_id || gp.id || '';
}

function slidesScope() {
  return document.querySelector('[id^="window-body-slides"]') || document;
}

// Bind the app to a stored deck: title, rename target and every toolbar form
// that sends the literal "current".
function pointDeckAt(id) {
  window.__SLIDES_PRESENTATION_ID = id;
  var titleEl = slidesScope().querySelector('#presTitle');
  if (titleEl) {
    titleEl.setAttribute('hx-vals', JSON.stringify({ id: id, action: 'rename' }));
    if (window.htmx) htmx.process(titleEl);
  }
}

function loadDeckById(id) {
  pointDeckAt(id);
  var scope = slidesScope();
  var headers = (window.GBSidebarConvos && window.GBSidebarConvos.authHeaders)
    ? window.GBSidebarConvos.authHeaders()
    : {};
  var setStatus = function (text) {
    var el = scope.querySelector('#saveStatus');
    if (el) el.textContent = text;
  };
  setStatus('Opening deck…');
  fetch('/api/slides/' + encodeURIComponent(id), { headers: headers })
    .then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    })
    .then(function (presentation) {
      var titleEl = scope.querySelector('#presTitle');
      if (titleEl && presentation && presentation.name) titleEl.value = presentation.name;
      setStatus('Opened deck');
    })
    .catch(function (err) {
      // Keep the id: new slides and edits must still target the requested deck.
      setStatus('Deck not loaded: ' + err.message);
    });
  if (window.htmx) {
    htmx.ajax('GET', '/suite/slides/fragments/presentation-view?id=' + encodeURIComponent(id),
      { target: '#slides-content', swap: 'innerHTML' });
    htmx.ajax('GET', '/suite/slides/fragments/slide-thumbnails?id=' + encodeURIComponent(id),
      { target: '#sidebar-thumbs', swap: 'innerHTML' });
  }
}

// Issue #720 parity: when opened from Drive (or a deep link) the app imports
// the presentation file through the existing /api/slides/import endpoint and
// switches the canvas to the imported deck. The shell URL carries no query
// string, so the args come from window.__gbAppParams__ as well.
function loadDrivePresentationIntoApp(linkParams) {
  var params = new URLSearchParams(window.location.search);
  // A re-target of an already-open window carries the link's own params;
  // `__gbAppParams__` accumulates every app's keys, so prefer the event's.
  var gp = linkParams || window.__gbAppParams__ || {};
  var file = params.get('file') || gp.file;
  var bucket = params.get('bucket') || gp.bucket;
  var deckRef = deckRefFromParams(params, gp);
  if (!file && deckRef) {
    loadDeckById(deckRef);
    return;
  }
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
      // Source identity: repeated opens of the same Drive file resolve to the
      // same deck instead of piling up duplicates (#1710).
      form.append('bucket', bucket || '');
      form.append('path', file);
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
  // A deck the shell rendered directly (already-imported Drive file) has no
  // deep link to run: adopt its id so thumbnails, rename and canvas edits do
  // not fall back to the literal "current" deck.
  var initialView = document.querySelector(".sl-view[data-presentation-id]");
  if (initialView && initialView.dataset.presentationId &&
      initialView.dataset.presentationId !== "current") {
    pointDeckAt(initialView.dataset.presentationId);
  }
  // The two hx-trigger="load" fragments still ask for id="new" (the blank
  // deck). This script runs while the window body is being injected, before
  // htmx processes it, so retargeting here makes the very first load fetch
  // the deck the deep link asked for.
  var bootRef = deckRefFromParams(
    new URLSearchParams(window.location.search),
    window.__gbAppParams__ || {}
  );
  if (bootRef) {
    pointDeckAt(bootRef);
    ["#slides-content", "#sidebar-thumbs"].forEach(function (selector) {
      var container = document.querySelector(selector);
      if (container) container.setAttribute("hx-vals", JSON.stringify({ id: bootRef }));
    });
  }
  loadDrivePresentationIntoApp();
  if (!window.__gbSlidesDeepLinkBound) {
    window.__gbSlidesDeepLinkBound = true;
    document.addEventListener('gb:deep-link', function (event) {
      var appId = (event.detail || {}).appId || '';
      if (appId === 'slides' || appId.indexOf('slides-') === 0) {
        loadDrivePresentationIntoApp((event.detail || {}).params || null);
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
