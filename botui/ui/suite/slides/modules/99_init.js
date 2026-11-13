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

// True for a Drive path of an importable deck: a folder path or an office
// extension. Anything else in `file` is a deck id, not a file.
function isDriveOfficePath(value) {
  if (!value) return false;
  if (value.indexOf('/') !== -1) return true;
  return /\.(pptx|ppt|odp|key|pdf)$/i.test(value);
}

// Bind the app to a stored deck: title, rename target and every toolbar form
// that sends the literal "current"/"new".
function pointDeckAt(id) {
  if (!id || id === 'current' || id === 'new') {
    id = (window.getSlidesPresentationId && window.getSlidesPresentationId()) || id;
  }
  window.__SLIDES_PRESENTATION_ID = id;
  var titleEl = slidesScope().querySelector('#presTitle');
  if (titleEl) {
    titleEl.setAttribute('hx-vals', JSON.stringify({ id: id, action: 'rename' }));
    if (window.htmx) htmx.process(titleEl);
  }
}

function loadDeckById(id) {
  if (!id || id === 'current' || id === 'new') return;
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
      if (!r.ok) throw new Error(r.status === 404 ? 'Deck not found' : 'HTTP ' + r.status);
      return r.json();
    })
    .then(function (presentation) {
      var titleEl = scope.querySelector('#presTitle');
      if (titleEl && presentation && presentation.name) titleEl.value = presentation.name;
      setStatus('Opened deck');
      if (window.htmx) {
        htmx.ajax('GET', '/suite/slides/fragments/presentation-view?id=' + encodeURIComponent(id),
          { target: '#slides-content', swap: 'innerHTML' });
        htmx.ajax('GET', '/suite/slides/fragments/slide-thumbnails?id=' + encodeURIComponent(id),
          { target: '#sidebar-thumbs', swap: 'innerHTML' });
      }
    })
    .catch(function (err) {
      var msg = err && err.message ? err.message : 'Deck not found';
      setStatus(msg);
      var titleEl = scope.querySelector('#presTitle');
      if (titleEl) titleEl.value = 'Deck not found';
      if (window.htmx) {
        htmx.ajax('GET', '/suite/slides/fragments/presentation-view?id=' + encodeURIComponent(id),
          { target: '#slides-content', swap: 'innerHTML' });
        htmx.ajax('GET', '/suite/slides/fragments/slide-thumbnails?id=' + encodeURIComponent(id),
          { target: '#sidebar-thumbs', swap: 'innerHTML' });
      }
    });
}

// Start a brand-new blank deck: mint a unique id and render a transient
// unsaved deck (persisted only on first save) so no shared "current" object
// is ever written.
function loadNewBlankDeck() {
  var id = (window.getSlidesPresentationId && window.getSlidesPresentationId()) ||
    ('deck-' + Date.now().toString(36));
  pointDeckAt(id);
  if (!window.htmx) return;
  var qs = 'id=' + encodeURIComponent(id) + '&new=1';
  htmx.ajax('GET', '/suite/slides/fragments/presentation-view?' + qs,
    { target: '#slides-content', swap: 'innerHTML' });
  htmx.ajax('GET', '/suite/slides/fragments/slide-thumbnails?' + qs,
    { target: '#sidebar-thumbs', swap: 'innerHTML' });
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
  // The curated verbs emit `app://slides?file=<deck id>`, so `file` is a deck
  // reference unless it names a real office file. Treating that id as a Drive
  // path made the download 404 and left the blank deck on screen.
  var fileIsPath = isDriveOfficePath(file);
  var deckRef = deckRefFromParams(params, gp) || (fileIsPath ? '' : file || '');
  if (deckRef && !fileIsPath) {
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
  // not fall back to the literal "current"/"new" deck.
  var initialView = document.querySelector(".sl-view[data-presentation-id]");
  if (initialView && initialView.dataset.presentationId &&
      initialView.dataset.presentationId !== "current" &&
      initialView.dataset.presentationId !== "new") {
    pointDeckAt(initialView.dataset.presentationId);
  }
  // Resolve what to open: a deep-link deck id, a Drive office file, or a
  // fresh blank deck with a unique id (never "current").
  var bootRef = deckRefFromParams(
    new URLSearchParams(window.location.search),
    window.__gbAppParams__ || {}
  );
  var gpBoot = window.__gbAppParams__ || {};
  var bootFile = (new URLSearchParams(window.location.search)).get('file') || gpBoot.file;
  var bootFileIsPath = isDriveOfficePath(bootFile);
  var requestedDeck = bootRef || (bootFileIsPath ? '' : bootFile || '');
  if (requestedDeck) {
    loadDeckById(requestedDeck);
  } else if (bootFileIsPath) {
    loadDrivePresentationIntoApp();
  } else {
    // No specific deck requested: start a fresh blank deck with a unique id.
    loadNewBlankDeck();
  }
  if (!window.__gbSlidesDeepLinkBound) {
    window.__gbSlidesDeepLinkBound = true;
    document.addEventListener('gb:deep-link', function (event) {
      var detail = event.detail || {};
      var appId = detail.appId || '';
      if (appId === 'slides' || appId.indexOf('slides-') === 0) {
        var lp = detail.params || null;
        var p = new URLSearchParams(window.location.search);
        var ref = lp ? deckRefFromParams(new URLSearchParams(), lp) : deckRefFromParams(p, window.__gbAppParams__ || {});
        var f = (lp && (lp.file || lp.path)) || p.get('file') || (window.__gbAppParams__ || {}).file;
        if (ref && !isDriveOfficePath(ref)) {
          loadDeckById(ref);
        } else if (f && isDriveOfficePath(f)) {
          loadDrivePresentationIntoApp(lp);
        } else if (ref) {
          loadDeckById(ref);
        }
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
