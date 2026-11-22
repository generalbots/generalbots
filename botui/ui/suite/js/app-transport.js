"use strict";
/* #2337 — transport classification of every app in the active .product.
 *
 * htmx-first        server returns HTML fragments; no app-owned request loop.
 * hybrid            HTMX for the primary surface, JS for a stateful subsystem.
 * stateful-exception WebSocket, canvas or a local runtime owns the surface;
 *                    errors and loading are reported through gbStates/GBToasts.
 *
 * This is machine-readable project state, not UI copy: it has no user-facing
 * strings and therefore needs no locale catalog keys.
 */
(function (window) {
  var classifications = {
    "chat": "stateful-exception",
    "drive": "stateful-exception",
    "browser": "stateful-exception",
    "clock-widget": "stateful-exception",
    "jukebox": "stateful-exception",
    "player": "stateful-exception",
    "calculator": "stateful-exception",
    "notes": "stateful-exception",
    "notepad": "stateful-exception",
    "photos": "stateful-exception",
    "timer": "stateful-exception",
    "weather": "stateful-exception",
    "integrations": "stateful-exception",
    "preview": "stateful-exception",
    "docs": "hybrid",
    "sheet": "hybrid",
    "slides": "hybrid",
    "tasks": "hybrid",
    "calendar": "hybrid",
    "paper": "htmx-first",
    "crm": "htmx-first",
    "billing": "htmx-first",
    "products": "htmx-first",
    "mail": "htmx-first"
  };
  window.GB_APP_TRANSPORT = Object.freeze(classifications);
})(window);
