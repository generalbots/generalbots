/* #2099 — "Expand" on an inline chat preview must escape the message bubble.
 *
 * The transcript caps a bot turn at 80% of the pane (chat-layout.css), so an
 * "expanded" preview that only grew its height stayed narrow and clipped. The
 * preview node is never cloned or re-rendered here: it is portaled into a modal
 * layer mounted on the chat root, so the artifact renderer keeps writing into
 * the very same element and the bubble's max-width can no longer clamp it.
 * Collapsing puts the node back at its exact spot in the transcript.
 *
 * Contract:
 *   GBArtifactOverlay.toggle(box, button, { label })  // Expand ⇄ Collapse
 *   GBArtifactOverlay.open(box, { label, focusEl, onClose })
 *   GBArtifactOverlay.close(box)                      // omit box: close any
 *   GBArtifactOverlay.isOpen(box)
 */
"use strict";

window.GBArtifactOverlay = window.GBArtifactOverlay || (function () {
  var FOCUSABLE = "a[href], button:not([disabled]), iframe, input:not([disabled])," +
    " select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";
  var state = null;

  // Mounted on the chat root (not document.body) so the artifact keeps every
  // `#chat-app …` style; it is still outside `.message`, which owns the cap.
  function host() {
    return document.getElementById("chat-app") || document.body;
  }

  function focusables(root) {
    var found = root.querySelectorAll(FOCUSABLE);
    var list = [];
    for (var i = 0; i < found.length; i += 1) {
      if (!found[i].hidden && found[i].offsetParent !== null) list.push(found[i]);
    }
    return list;
  }

  function focusEl(el) {
    if (!el || !el.isConnected) return;
    try { el.focus({ preventScroll: true }); } catch (e) { el.focus(); }
  }

  // Escape closes; Tab cycles inside the overlay instead of walking the
  // transcript behind it. Key presses typed into a sandboxed (opaque origin)
  // artifact iframe belong to that document, so they never reach us — the
  // always-visible Collapse button and the backdrop click cover that case.
  function onKeyDown(event) {
    if (!state) return;
    if (event.key === "Escape" || event.key === "Esc") {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    if (event.key !== "Tab") return;
    var items = focusables(state.box);
    if (!items.length) {
      event.preventDefault();
      return;
    }
    var first = items[0];
    var last = items[items.length - 1];
    var active = document.activeElement;
    var inside = state.box.contains(active);
    if (event.shiftKey && (active === first || !inside)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && (active === last || !inside)) {
      event.preventDefault();
      first.focus();
    }
  }

  function open(box, options) {
    if (!box) return null;
    options = options || {};
    if (state && state.box === box) return state.overlay;
    if (state) close();

    var placeholder = document.createComment("gb-artifact-anchor");
    if (box.parentNode) box.parentNode.insertBefore(placeholder, box);

    var overlay = document.createElement("div");
    overlay.className = "gb-artifact-overlay";
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    overlay.setAttribute("aria-label", options.label || "Expanded preview");
    overlay.tabIndex = -1;
    var frame = document.createElement("div");
    frame.className = "gb-artifact-overlay-frame";
    frame.appendChild(box);
    overlay.appendChild(frame);
    overlay.addEventListener("mousedown", function (event) {
      if (event.target === overlay) close();
    });

    var messages = document.getElementById("messages");
    state = {
      box: box,
      overlay: overlay,
      placeholder: placeholder,
      scrollTop: messages ? messages.scrollTop : null,
      bodyOverflow: document.body.style.overflow,
      onClose: typeof options.onClose === "function" ? options.onClose : null,
      focusEl: options.focusEl || null,
    };
    document.body.style.overflow = "hidden";
    box.classList.add("gb-artifact-expanded");
    host().appendChild(overlay);
    document.addEventListener("keydown", onKeyDown, true);
    if (state.focusEl && state.focusEl.isConnected) {
      focusEl(state.focusEl);
    } else {
      var items = focusables(box);
      focusEl(items.length ? items[0] : box);
    }
    return overlay;
  }

  function close(box) {
    if (!state) return;
    if (box && state.box !== box) return;
    var done = state;
    state = null;
    document.removeEventListener("keydown", onKeyDown, true);
    document.body.style.overflow = done.bodyOverflow;
    done.box.classList.remove("gb-artifact-expanded");
    // The transcript may have been re-rendered while the overlay was open;
    // then the anchor is detached and the preview goes with it.
    if (done.placeholder.parentNode) {
      done.placeholder.parentNode.replaceChild(done.box, done.placeholder);
    } else {
      done.box.remove();
    }
    if (done.overlay.parentNode) done.overlay.remove();
    var messages = document.getElementById("messages");
    if (messages && done.scrollTop !== null) messages.scrollTop = done.scrollTop;
    focusEl(done.focusEl);
    if (done.onClose) done.onClose();
  }

  function isOpen(box) {
    return !!state && (!box || state.box === box);
  }

  function collapse(box, button) {
    if (!box) return;
    close(box);
    box.classList.remove("expanded");
    if (button) {
      button.textContent = "Expand";
      button.setAttribute("aria-expanded", "false");
    }
  }

  // One toggle for every inline preview: the label, the `.expanded` class
  // (which still drives the height rules) and the overlay stay in sync, and
  // Escape ends exactly like a second click.
  function toggle(box, button, options) {
    if (!box) return;
    if (box.classList.contains("expanded")) {
      collapse(box, button);
      return;
    }
    options = options || {};
    box.classList.add("expanded");
    if (button) {
      button.textContent = "Collapse";
      button.setAttribute("aria-expanded", "true");
    }
    open(box, {
      label: options.label,
      focusEl: button || null,
      onClose: function () {
        box.classList.remove("expanded");
        if (button) {
          button.textContent = "Expand";
          button.setAttribute("aria-expanded", "false");
        }
      },
    });
  }

  return { open: open, close: close, toggle: toggle, isOpen: isOpen };
})();
