"use strict";
/* Mobile navigation model (#2416).

   Under 768px the shell used to delete the left rail with `display:none
   !important`, so conversation history, the Apps rail and "New chat" were
   unreachable — the toggle could not undo `!important`. This module turns the
   rail into an off-canvas drawer with one persistent header button, a
   backdrop, Escape/swipe-free dismissal, and a Back button that closes the
   drawer instead of leaving the app.

   The desktop window manager is untouched: above the breakpoint the button
   and the backdrop are removed and the rail goes back to a normal column. */

const MobileNav = (() => {
  const BREAKPOINT = 768;
  const BACKDROP_ID = "gbRailBackdrop";
  const BUTTON_ID = "gbRailButton";
  let initialized = false;

  function compact() {
    return window.matchMedia
      ? window.matchMedia("(max-width: " + BREAKPOINT + "px)").matches
      : window.innerWidth <= BREAKPOINT;
  }

  function label() {
    return window.gbT ? window.gbT("sidebar-toggle", "Toggle sidebar") : "Toggle sidebar";
  }

  function rail() {
    return document.getElementById("chatSidebar");
  }

  function isOpen() {
    return document.body.classList.contains("gb-rail-open");
  }

  function open() {
    const sidebar = rail();
    if (!sidebar) return;
    document.body.classList.add("gb-rail-open");
    if (!document.getElementById(BACKDROP_ID)) {
      const backdrop = document.createElement("div");
      backdrop.id = BACKDROP_ID;
      backdrop.className = "gb-rail-backdrop";
      backdrop.addEventListener("click", close);
      document.body.appendChild(backdrop);
    }
    try {
      history.pushState({ gbRail: "open" }, "");
    } catch (e) {
      console.warn("mobile nav: could not push history state", e);
    }
    const first = sidebar.querySelector("a, button, [tabindex]");
    if (first) first.focus();
  }

  function close() {
    document.body.classList.remove("gb-rail-open");
    const backdrop = document.getElementById(BACKDROP_ID);
    if (backdrop) backdrop.remove();
    const button = document.getElementById(BUTTON_ID);
    if (button) button.focus();
  }

  function toggle() {
    if (isOpen()) close();
    else open();
  }

  function ensureButton() {
    if (!compact()) {
      removeButton();
      return;
    }
    if (document.getElementById(BUTTON_ID) || !rail()) return;
    const button = document.createElement("button");
    button.id = BUTTON_ID;
    button.type = "button";
    button.className = "gb-rail-button";
    button.setAttribute("aria-controls", "chatSidebar");
    button.setAttribute("aria-expanded", "false");
    button.setAttribute("aria-label", label());
    button.setAttribute("data-i18n-aria-label", "sidebar-toggle");
    button.innerHTML =
      '<svg viewBox="0 0 24 24" width="22" height="22" fill="none" ' +
      'stroke="currentColor" stroke-width="2" stroke-linecap="round" ' +
      'aria-hidden="true"><line x1="3" y1="6" x2="21" y2="6"/>' +
      '<line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>';
    button.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      toggle();
    });
    document.body.appendChild(button);
  }

  function removeButton() {
    const button = document.getElementById(BUTTON_ID);
    if (button) button.remove();
    if (!compact()) close();
  }

  function syncExpanded() {
    const button = document.getElementById(BUTTON_ID);
    if (button) button.setAttribute("aria-expanded", isOpen() ? "true" : "false");
  }

  function init() {
    if (initialized || typeof document === "undefined") return;
    initialized = true;
    ensureButton();

    window.addEventListener("resize", function () {
      ensureButton();
      syncExpanded();
    });

    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape" && isOpen()) {
        // Let the popstate entry go away with the drawer.
        history.back();
      }
    });

    window.addEventListener("popstate", function () {
      if (isOpen()) {
        document.body.classList.remove("gb-rail-open");
        const backdrop = document.getElementById(BACKDROP_ID);
        if (backdrop) backdrop.remove();
      }
      syncExpanded();
    });

    const observer = new MutationObserver(syncExpanded);
    observer.observe(document.body, { attributes: true, attributeFilter: ["class"] });
  }

  if (typeof document !== "undefined") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  }

  return { init: init, open: open, close: close, toggle: toggle, isOpen: isOpen };
})();

window.MobileNav = MobileNav;
