"use strict";
/* GBCollabA11y — shared accessibility helpers for collaboration apps.
 *
 * Two concerns live here so Docs/Slides/Drive don't each reimplement them:
 *   1. A screen-reader live region with an `announce()` helper (polite by
 *      default, assertive when a collaborator joins/edits/comments).
 *   2. A global Escape handler that closes any open collab panel (comments,
 *      activity, version history, follow) and returns keyboard control.
 *
 * Public API (window.GBCollabA11y):
 *   announce(text, assertive?)  — push text to the live region
 *   ensureLive()                — idempotently create the region
 */
(function (window) {
  var LIVE_ID = "gb-aria-live";
  var live = null;

  function ensureLive() {
    if (live && live.parentNode) return live;
    live = document.getElementById(LIVE_ID);
    if (!live) {
      live = document.createElement("div");
      live.id = LIVE_ID;
      live.setAttribute("role", "status");
      live.setAttribute("aria-live", "polite");
      live.setAttribute("aria-atomic", "true");
      live.style.cssText =
        "position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);" +
        "clip-path:inset(50%);white-space:nowrap;";
      document.body.appendChild(live);
    }
    return live;
  }

  function announce(text, assertive) {
    var region = ensureLive();
    region.setAttribute("aria-live", assertive ? "assertive" : "polite");
    region.textContent = "";
    window.setTimeout(function () { region.textContent = String(text); }, 30);
  }

  // Escape closes whichever collab panel is open; each close() is idempotent.
  // #2439 — only consume Escape when a panel was ACTUALLY open. The old code
  // set handled=true whenever the globals existed, so it swallowed every
  // Escape at document-bubble and the window-manager's close-focused-window
  // shortcut never saw it. Detect openness from the DOM before closing.
  document.addEventListener("keydown", function (e) {
    if (e.key !== "Escape" || e.defaultPrevented) return;
    var wasOpen = !!(document.querySelector(".gbc-open, .gba-open, .gbv-open, .gbs-open") ||
      (window.GBCollabFollow && document.querySelector("#gb-follow-panel") &&
       document.querySelector("#gb-follow-panel").style.display === "block"));
    if (!wasOpen) return;
    if (window.GBCollabComments) window.GBCollabComments.close();
    if (window.GBCollabActivity) window.GBCollabActivity.close();
    if (window.GBCollabVersions) window.GBCollabVersions.close();
    if (window.GBCollabFollow) window.GBCollabFollow.hide();
    e.stopPropagation();
  });

  window.GBCollabA11y = { announce: announce, ensureLive: ensureLive };
})(window);
