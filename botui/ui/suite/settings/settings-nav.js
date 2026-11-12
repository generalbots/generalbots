"use strict";
/* Settings section navigation (#2219).
 *
 * The sidebar is a set of plain `#section` anchors. Inside the desktop shell
 * the document URL belongs to the whole suite, so the anchor's default action
 * hash-navigated the TOP page (e.g. /default -> /default/#profile); the suite
 * router then swapped #main-content and the Settings window — including any
 * field a UI-hint plan was filling — was torn down.
 *
 * This module owns the navigation instead: it activates the section locally and
 * mirrors the fragment with history.replaceState (no hashchange, no page-level
 * navigation). It is loaded by index.html directly and does not depend on
 * settings-actions.js, which only ships with the admin partial.
 */
(function () {
  function activate(section, anchor) {
    if (!section) return;
    var sections = document.querySelectorAll(".settings-section");
    for (var i = 0; i < sections.length; i++) {
      sections[i].classList.remove("active");
    }
    var target = document.getElementById(section + "-section");
    if (target) target.classList.add("active");
    var items = document.querySelectorAll(".settings-nav .nav-item");
    for (var j = 0; j < items.length; j++) {
      items[j].classList.remove("active");
    }
    if (anchor) {
      anchor.classList.add("active");
    } else {
      var match = document.querySelector('.settings-nav a[href="#' + section + '"]');
      if (match) match.classList.add("active");
    }
    // Only the standalone Settings page owns its URL. Inside the desktop
    // shell the address belongs to the suite: writing a fragment there makes
    // the suite router treat "#security" as an app route on the next load.
    if (/\/settings(\/|$)/.test(window.location.pathname)) {
      try {
        history.replaceState(null, "", "#" + section);
      } catch (e) {
        /* file:// or sandboxed documents */
      }
    }
    // Panels that key their own loading off the hash (OS, shared links, AI
    // accounts) no longer receive a hashchange event, so they are poked here.
    if (section === "os" && typeof window.loadOsPanel === "function") {
      try { window.loadOsPanel(); } catch (e) { /* panel optional */ }
    }
    if (typeof window.GBSettingsOnSection === "function") {
      try { window.GBSettingsOnSection(section); } catch (e) { /* optional */ }
    }
  }

  function onClick(event) {
    var anchor = event.target && event.target.closest
      ? event.target.closest(".settings-nav a[href^='#']")
      : null;
    if (!anchor) return;
    var section = (anchor.getAttribute("href") || "").slice(1);
    if (!section) return;
    event.preventDefault();
    event.stopPropagation();
    activate(section, anchor);
  }

  document.addEventListener("click", onClick, true);

  function initFromHash() {
    if (!/\/settings(\/|$)/.test(window.location.pathname)) return;
    var section = (window.location.hash || "").slice(1);
    if (!section) return;
    if (!document.getElementById(section + "-section")) return;
    activate(section, null);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initFromHash);
  } else {
    initFromHash();
  }

  window.GBSettingsActivateSection = activate;
})();
