/**
 * #2954 — offline regression guard for the uncaught
 * "Cannot read properties of null (reading 'classList')" seen when a suite app
 * window is opened/closed and a stale listener or late callback runs after the
 * app's DOM was removed.
 *
 * #2939 guarded the Attendant outside-click listener only. The same pattern
 * remained in three app-owned paths that are reachable after close:
 *   - tickets.js      — document Escape listener -> closeTicketsModal()
 *   - project-state_a.js — toggleDetailsPanel / loadProjectData / render detail
 *   - integrations/02_render.js — renderCatalogHealth()
 * Each must null-check the lookup before touching .classList, and must fail
 * closed (no exception, no state change) when the element is absent.
 *
 * No server, browser, cargo or i18n change is involved; the driver owns live
 * UAT, issue closure and deployment.
 */
"use strict";

var h = require("./00_harness");

var TICKETS = "botui/ui/suite/tickets/tickets.js";
var PROJECT = "botui/ui/suite/project/project-state_a.js";
var INTEGRATIONS = "botui/ui/suite/integrations/modules/02_render.js";

function src(rel) { return h.readRepo(rel); }
function count(text, needle) { return text.split(needle).length - 1; }

h.staticTest("#2954 tickets modal helpers null-check before classList", function () {
  var text = src(TICKETS);
  h.assert(count(text, 'document.getElementById("tickets-modal").classList') === 0,
    "no direct getElementById(...).classList dereference remains");
  h.assert(count(text, 'var modal = document.getElementById("tickets-modal")') === 2,
    "both helpers look the modal up first");
  h.assert(count(text, "if (!modal) return;") === 2,
    "both helpers fail closed when the modal is absent");
  h.assert(count(text, 'modal.classList.add("open")') === 1 &&
    count(text, 'modal.classList.remove("open")') === 1,
    "existing open/close behaviour is preserved when the modal exists");
});

h.staticTest("#2954 project panel/root lookups null-check before classList", function () {
  var text = src(PROJECT);
  h.assert(count(text, "document.getElementById('details-panel').classList") === 0,
    "no direct details-panel classList dereference remains");
  h.assert(count(text, "document.querySelector('.project-app').classList") === 0,
    "no direct .project-app classList dereference remains");
  h.assert(count(text, "if (!panel) return;") >= 1,
    "toggleDetailsPanel fails closed");
  h.assert(count(text, "if (appRoot) appRoot.classList.add('has-project')") === 1,
    "loadProjectData guards the app root");
  h.assert(count(text, "if (detailsPanel) detailsPanel.classList.remove('collapsed')") === 1,
    "the detail reveal guards the panel");
});

h.staticTest("#2954 integrations catalog health null-checks before classList", function () {
  var text = src(INTEGRATIONS);
  h.assert(count(text, "health.classList.remove(\"is-ready\", \"is-error\")") === 1,
    "the normal render path is intact");
  var start = text.indexOf("namespace.renderCatalogHealth = function (root) {");
  h.assert(start !== -1, "renderCatalogHealth is defined");
  var body = text.slice(start, text.indexOf("function matchesProvider", start));
  h.assert(body.indexOf("if (!health || !healthText) return;") !== -1,
    "the renderer fails closed when either health slot is missing");
  h.assert(body.indexOf("if (!health || !healthText) return;") <
    body.indexOf('health.classList.remove("is-ready", "is-error")'),
    "the guard runs before any classList access");
});

// Runtime proof: the real tickets.js in a stubbed vm, with #tickets-modal
// absent (the post-close state). The stale Escape listener must not throw.
h.test("post-close Escape with no tickets modal throws no classList TypeError", function () {
  var docListeners = {};
  var modalCalls = 0;
  var modal = {
    classList: {
      add: function (name) { if (name === "open") modalCalls++; },
      remove: function (name) { if (name === "open") modalCalls++; },
    },
  };
  var present = false;
  var sandbox = {
    console: { log: function () {}, error: function () {} },
    API_BASE: "",
    fetch: function () { return { then: function () { return this; }, catch: function () { return this; } }; },
    htmx: { ajax: function () {} },
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    document: {
      readyState: "loading",
      addEventListener: function (type, fn) { (docListeners[type] = docListeners[type] || []).push(fn); },
      getElementById: function (id) {
        // #tickets-modal is gone (window closed); the "new" button still has
        // to exist so the module's boot-time binding can complete.
        return id === "tickets-new-btn" ? { addEventListener: function () {} } : null;
      },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
    },
  };
  sandbox.window = sandbox;
  h.vm.createContext(sandbox);
  h.vm.runInContext(src(TICKETS), sandbox, { filename: TICKETS });

  var keys = docListeners.keydown || [];
  h.assert(keys.length === 1, "the tickets Escape listener is installed");
  // This call threw with the pre-fix direct dereference on a missing element.
  keys.forEach(function (fn) { fn({ key: "Escape" }); });
  h.assertEqual(modalCalls, 0, "no classList call while the modal is absent");
  sandbox.closeTicketsModal();
  sandbox.openTicketsModal();
  h.assertEqual(modalCalls, 0, "both helpers fail closed after close");

  // With the element present the original behaviour is unchanged.
  present = true;
  sandbox.document.getElementById = function (id) { return present && id === "tickets-modal" ? modal : null; };
  sandbox.closeTicketsModal();
  sandbox.openTicketsModal();
  h.assertEqual(modalCalls, 2, "open/close still work while the window exists");
});
