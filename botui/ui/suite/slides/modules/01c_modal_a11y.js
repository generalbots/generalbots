// botui/ui/suite/slides/modules/01c_modal_a11y.js
// Generic modal accessibility: focus trap + Escape to close.
// Works on any element with role="dialog" and aria-modal="true".
// Hooks into the existing showModal/hideModal pattern by listening
// to DOM changes and keydown events globally.
//
// This module is included by all three suites (sheet, docs, slides)
// via a small inline <script> tag in each HTML. It is self-contained
// and idempotent — re-running init is safe.
"use strict";

(function () {
  const FOCUSABLE =
    'a[href],area[href],input:not([disabled]):not([type="hidden"]),select:not([disabled]),textarea:not([disabled]),button:not([disabled]),iframe,object,embed,[tabindex]:not([tabindex="-1"]),[contenteditable]';

  let activeModal = null;
  let previouslyFocused = null;

  function getFocusable(modal) {
    if (!modal) return [];
    return Array.prototype.slice
      .call(modal.querySelectorAll(FOCUSABLE))
      .filter(function (el) {
        return !el.hasAttribute("disabled") && el.offsetParent !== null;
      });
  }

  function trap(e) {
    if (!activeModal) return;
    if (e.key !== "Tab") return;
    const focusable = getFocusable(activeModal);
    if (focusable.length === 0) {
      e.preventDefault();
      activeModal.focus();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function onKeyDown(e) {
    if (!activeModal) return;
    if (e.key === "Escape" || e.key === "Esc") {
      e.preventDefault();
      e.stopPropagation();
      // Use the suite's hideModal if available; otherwise just hide
      if (
        window.slidesApp &&
        typeof window.slidesApp.hideModal === "function"
      ) {
        window.slidesApp.hideModal(activeModal.id);
      } else if (
        window.docsApp &&
        typeof window.docsApp.hideModal === "function"
      ) {
        window.docsApp.hideModal(activeModal.id);
      } else if (
        window.sheetApp &&
        typeof window.sheetApp.hideModal === "function"
      ) {
        window.sheetApp.hideModal(activeModal.id);
      } else {
        activeModal.classList.add("hidden");
      }
    }
  }

  function onModalShown(modal) {
    if (activeModal === modal) return;
    if (activeModal) {
      // Switching from one modal to another — release previous
      activeModal = null;
    }
    activeModal = modal;
    previouslyFocused = document.activeElement;
    const focusable = getFocusable(modal);
    if (focusable.length > 0) {
      // Prefer the close button or first focusable
      const closeBtn = modal.querySelector(".btn-close");
      const target = closeBtn || focusable[0];
      setTimeout(function () {
        target.focus();
      }, 0);
    } else {
      setTimeout(function () {
        modal.focus();
      }, 0);
    }
  }

  function onModalHidden(modal) {
    if (activeModal !== modal) return;
    activeModal = null;
    if (previouslyFocused && previouslyFocused.focus) {
      try {
        previouslyFocused.focus();
      } catch (e) {
        // Element may have been removed — ignore
      }
    }
    previouslyFocused = null;
  }

  function checkModals() {
    const modals = document.querySelectorAll('[role="dialog"]');
    modals.forEach(function (modal) {
      const isHidden = modal.classList.contains("hidden");
      if (!isHidden && activeModal !== modal) {
        onModalShown(modal);
      } else if (isHidden && activeModal === modal) {
        onModalHidden(modal);
      }
    });
  }

  function init() {
    // Initial check
    checkModals();

    // Use a throttled mutation observer
    let pending = false;
    const observer = new MutationObserver(function () {
      if (pending) return;
      pending = true;
      requestAnimationFrame(function () {
        pending = false;
        checkModals();
      });
    });
    observer.observe(document.body, {
      attributes: true,
      attributeFilter: ["class"],
      subtree: true,
    });

    // Global keydown handler for Tab trap and Escape
    document.addEventListener("keydown", function (e) {
      if (e.key === "Tab") trap(e);
      else if (e.key === "Escape" || e.key === "Esc") onKeyDown(e);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
