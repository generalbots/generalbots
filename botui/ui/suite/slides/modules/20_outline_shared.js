"use strict";
/* Slides review tools — shared local-only state, DOM deck model, accessible UI (#2433).
 * No server writes: every datum these tools own lives in localStorage, keyed by
 * deck id, and is labelled local-only in the UI. Slide text is read from the
 * rendered thumbnails/canvas (read-only) and replaced in place with one-step undo.
 */

var SO_STORE_PREFIX = "slides-outline:";
var soDialogSeq = 0;
var soUndoStack = [];
var soRedoStack = [];
var SO_UNDO_LIMIT = 100;

function soT(key, fallback, params) {
  return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
}

function soScope() {
  return document.querySelector('[id^="window-body-slides"]') ||
    document.getElementById("slides-app") || document;
}
function so$(selector, root) { return (root || soScope()).querySelector(selector); }
function so$$(selector, root) {
  return Array.prototype.slice.call((root || soScope()).querySelectorAll(selector));
}
function soEl(tag, cls, text, key) {
  var node = document.createElement(tag);
  if (cls) node.className = cls;
  if (key) node.setAttribute("data-i18n", key);
  if (text !== undefined && text !== null) node.textContent = text;
  return node;
}
function soEscape(value) {
  return String(value == null ? "" : value)
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

function soDeckId() {
  return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) ||
    window.__SLIDES_PRESENTATION_ID || "current";
}
function soKey(name) { return SO_STORE_PREFIX + name + ":" + soDeckId(); }
function soLoad(name, fallback) {
  try {
    var raw = window.localStorage.getItem(soKey(name));
    return raw ? JSON.parse(raw) : fallback;
  } catch (error) { return fallback; }
}
function soSave(name, value) {
  try {
    window.localStorage.setItem(soKey(name), JSON.stringify(value));
    return true;
  } catch (error) { return false; }
}

/* The deck model is derived from the DOM: each thumbnail embeds a rendered
 * preview of one slide with all of its elements. The live canvas duplicates the
 * current slide, so both copies are tracked for read/write of text. */
function soSlides() {
  var host = so$("#sidebar-thumbs");
  var thumbs = host ? so$$(".sl-thumb", host) : [];
  var list = [];
  thumbs.forEach(function (thumb, position) {
    var raw = thumb.getAttribute("data-index") || thumb.getAttribute("data-slide");
    var index = parseInt(raw, 10);
    if (!isFinite(index)) index = position;
    list.push({ index: index, thumb: thumb, elements: so$$(".sl-element", thumb), canvasElements: [] });
  });
  if (!list.length) {
    var fallbackCanvas = so$("#slide-canvas") || document.getElementById("slide-canvas");
    if (fallbackCanvas) {
      var fallbackIndex = parseInt(fallbackCanvas.getAttribute("data-slide-id") || "0", 10) || 0;
      list.push({ index: fallbackIndex, thumb: null, elements: so$$(".sl-element", fallbackCanvas), canvasElements: [] });
    }
  }
  var canvas = so$("#slide-canvas") || document.getElementById("slide-canvas");
  if (canvas) {
    var current = parseInt(canvas.getAttribute("data-slide-id") || "0", 10) || 0;
    list.forEach(function (slide) {
      if (slide.index === current) slide.canvasElements = so$$(".sl-element", canvas);
    });
  }
  return list;
}

function soTextFields(element) {
  var type = element.getAttribute("data-type") || "";
  var fields = [];
  if (type === "table") {
    so$$("th, td", element).forEach(function (cell, position) {
      fields.push({ field: "cell:" + position, node: cell, text: cell.textContent || "" });
    });
  } else if (type === "chart") {
    var title = element.querySelector('div[style*="font-weight:600"]') || element.firstElementChild || element;
    fields.push({ field: "chart-title", node: title, text: title.textContent || "" });
  } else if (type === "title" || type === "text") {
    var textNode = element.querySelector("p,h1,h2,h3") || element;
    fields.push({ field: "text", node: textNode, text: textNode.textContent || "" });
  }
  return fields;
}

function soElementRefs() {
  var refs = [];
  soSlides().forEach(function (slide) {
    slide.elements.concat(slide.canvasElements).forEach(function (element, position) {
      soTextFields(element).forEach(function (field) {
        refs.push({
          kind: "element", slideIndex: slide.index,
          elementId: element.getAttribute("data-id") || "", elementIndex: position,
          elementType: element.getAttribute("data-type") || "", field: field.field,
          node: field.node, text: field.text
        });
      });
    });
  });
  return refs;
}

function soNoteRefs() {
  var notes = soLoad("notes", {});
  var refs = [];
  Object.keys(notes).forEach(function (slideKey) {
    (notes[slideKey] || []).forEach(function (note) {
      refs.push({
        kind: "note", slideIndex: parseInt(slideKey, 10) || 0, noteId: note.id,
        field: "text", text: note.text || ""
      });
    });
  });
  return refs;
}

function soWriteText(ref, value) {
  if (ref.kind === "note") {
    var notes = soLoad("notes", {});
    var list = notes[String(ref.slideIndex)] || [];
    list.forEach(function (note) { if (note.id === ref.noteId) note.text = value; });
    notes[String(ref.slideIndex)] = list;
    soSave("notes", notes);
    return;
  }
  soSlides().forEach(function (slide) {
    if (slide.index !== ref.slideIndex) return;
    slide.elements.concat(slide.canvasElements).forEach(function (element, position) {
      if (ref.elementId && element.getAttribute("data-id") !== ref.elementId) return;
      if (!ref.elementId && position !== ref.elementIndex) return;
      soTextFields(element).forEach(function (field) {
        if (field.field === ref.field && field.node.textContent === ref.before) {
          field.node.textContent = value;
        }
      });
    });
  });
}

function soStatus(message) {
  var host = soScope();
  var toolbar = so$(".slides-toolbar", host) || host;
  var region = so$("#slides-tools-status", host);
  if (!region) {
    region = soEl("div", "slides-tools-status", "", "slides-outline-status-empty");
    region.id = "slides-tools-status";
    region.setAttribute("role", "status");
    region.setAttribute("aria-live", "polite");
    toolbar.insertBefore(region, toolbar.firstChild);
  }
  region.textContent = message || "";
}

function soLocalBadge() {
  return soEl("span", "slides-tools-local-badge",
    soT("slides-outline-local-only", "Local only — not saved to the server"),
    "slides-outline-local-only");
}

function soElFoci(box) {
  var selector = 'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),' +
    'textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';
  return so$$(selector, box).filter(function (node) {
    return !node.hasAttribute("hidden") && node.getAttribute("aria-hidden") !== "true";
  });
}

function soDialog(options) {
  var config = options || {};
  var previous = document.activeElement;
  var overlay = soEl("div", "slides-tools-overlay");
  var box = soEl("div", "slides-tools-dialog");
  box.setAttribute("role", "dialog");
  box.setAttribute("aria-modal", "true");
  box.tabIndex = -1;
  var titleId = "so-dialog-title-" + (++soDialogSeq);
  box.setAttribute("aria-labelledby", titleId);
  var head = soEl("div", "slides-tools-dialog-head");
  var heading = soEl("h2", "", soT(config.titleKey, config.title || ""), config.titleKey);
  heading.id = titleId;
  var close = soEl("button", "slides-tools-close", soT("slides-outline-close", "Close"), "slides-outline-close");
  close.type = "button";
  close.setAttribute("data-i18n-aria-label", "slides-outline-close");
  close.setAttribute("aria-label", soT("slides-outline-close", "Close"));
  head.appendChild(heading);
  head.appendChild(close);
  var body = soEl("div", "slides-tools-dialog-body");
  var foot = soEl("div", "slides-tools-dialog-foot");
  box.appendChild(head);
  box.appendChild(body);
  box.appendChild(foot);
  overlay.appendChild(box);
  (soScope() || document.body).appendChild(overlay);

  function dismiss(result) {
    document.removeEventListener("keydown", onKey, true);
    if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
    if (previous && typeof previous.focus === "function") previous.focus();
    if (typeof config.onClose === "function") config.onClose(result);
  }
  function onKey(event) {
    if (event.key === "Escape") { event.preventDefault(); dismiss(false); return; }
    if (event.key !== "Tab") return;
    var foci = soElFoci(box);
    if (!foci.length) return;
    var first = foci[0], last = foci[foci.length - 1];
    if (event.shiftKey && (document.activeElement === first || document.activeElement === box)) {
      event.preventDefault(); last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault(); first.focus();
    }
  }
  document.addEventListener("keydown", onKey, true);
  close.addEventListener("click", function () { dismiss(false); });
  overlay.addEventListener("pointerdown", function (event) {
    if (event.target === overlay) dismiss(false);
  });
  window.setTimeout(function () {
    var foci = soElFoci(box);
    (foci[0] || box).focus();
  }, 0);
  return { overlay: overlay, box: box, body: body, foot: foot, dismiss: dismiss };
}

function soConfirm(options) {
  var config = options || {};
  return new Promise(function (resolve) {
    var dialog = soDialog({
      titleKey: config.titleKey, title: config.title,
      onClose: function (result) { resolve(!!result); }
    });
    var message = soEl("p", "slides-tools-message", soT(config.messageKey, config.message || ""), config.messageKey);
    dialog.body.appendChild(message);
    if (config.detail) {
      var detail = soEl("ul", "slides-tools-confirm-detail");
      config.detail.forEach(function (text) { detail.appendChild(soEl("li", "", text)); });
      dialog.body.appendChild(detail);
    }
    dialog.body.appendChild(soLocalBadge());
    var cancel = soEl("button", "slides-tools-btn", soT("slides-outline-cancel", "Cancel"), "slides-outline-cancel");
    var confirm = soEl("button", "slides-tools-btn primary",
      soT(config.confirmKey || "slides-outline-confirm", config.confirm || "Confirm"),
      config.confirmKey || "slides-outline-confirm");
    cancel.type = "button"; confirm.type = "button";
    cancel.addEventListener("click", function () { dialog.dismiss(false); });
    confirm.addEventListener("click", function () { dialog.dismiss(true); });
    dialog.foot.appendChild(cancel);
    dialog.foot.appendChild(confirm);
  });
}

function soUndo() {
  var action = soUndoStack.pop();
  if (!action) { soStatus(soT("slides-outline-nothing-to-undo", "Nothing to undo")); return; }
  try {
    action.undo();
    soRedoStack.push(action);
    soStatus(soT("slides-outline-undone", "Undone: {action}", { action: action.label }));
  } catch (error) {
    soStatus(soT("slides-outline-undo-failed", "Undo failed"));
  }
  soUpdateUndoUI();
}
function soRedo() {
  var action = soRedoStack.pop();
  if (!action) { soStatus(soT("slides-outline-nothing-to-redo", "Nothing to redo")); return; }
  try {
    action.redo();
    soUndoStack.push(action);
    soStatus(soT("slides-outline-redone", "Redone: {action}", { action: action.label }));
  } catch (error) {
    soStatus(soT("slides-outline-redo-failed", "Redo failed"));
  }
  soUpdateUndoUI();
}
function soPushUndo(action) {
  if (!action) return;
  soUndoStack.push(action);
  if (soUndoStack.length > SO_UNDO_LIMIT) soUndoStack.shift();
  soRedoStack.length = 0;
  soUpdateUndoUI();
}
function soUpdateUndoUI() {
  so$$(".slides-tools-undo").forEach(function (button) { button.disabled = soUndoStack.length === 0; });
  so$$(".slides-tools-redo").forEach(function (button) { button.disabled = soRedoStack.length === 0; });
}

function soJumpToSlide(index) {
  var slide = null;
  soSlides().forEach(function (item) { if (item.index === index) slide = item; });
  if (slide && slide.thumb && typeof slide.thumb.scrollIntoView === "function") {
    slide.thumb.scrollIntoView({ block: "nearest" });
  }
  if (typeof window.refreshSlideView === "function") {
    window.refreshSlideView(index);
  } else if (window.htmx) {
    window.htmx.ajax("GET", "/suite/slides/fragments/presentation-view?id=" +
      encodeURIComponent(soDeckId()) + "&slide=" + encodeURIComponent(index),
      { target: "#slides-content", swap: "innerHTML" });
  }
  soStatus(soT("slides-outline-jumped", "Jumped to slide {n}", { n: index + 1 }));
}

function soPanelHost() {
  var host = so$("#slides-tools-panel-host");
  if (!host) {
    host = soEl("div", "slides-tools-panel-host");
    host.id = "slides-tools-panel-host";
    (so$("#slides-app") || soScope()).appendChild(host);
  }
  return host;
}
function soOpenPanel(id, builder) {
  var host = soPanelHost();
  host.textContent = "";
  var panel = builder();
  panel.id = id;
  host.appendChild(panel);
  return panel;
}
function soClosePanels() {
  var host = so$("#slides-tools-panel-host");
  if (host) host.textContent = "";
}

window.GBSlidesOutline = {
  t: soT, scope: soScope, $: so$, $$: so$$, el: soEl, escape: soEscape,
  deckId: soDeckId, load: soLoad, save: soSave,
  slides: soSlides, textFields: soTextFields, elementRefs: soElementRefs,
  noteRefs: soNoteRefs, writeText: soWriteText,
  status: soStatus, localBadge: soLocalBadge, dialog: soDialog, confirm: soConfirm,
  undo: soUndo, redo: soRedo, pushUndo: soPushUndo, updateUndoUI: soUpdateUndoUI,
  jump: soJumpToSlide, panelHost: soPanelHost, openPanel: soOpenPanel, closePanels: soClosePanels
};

if (!window.__soKeysBound) {
  window.__soKeysBound = true;
  document.addEventListener("keydown", function (event) {
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
    var key = (event.key || "").toLowerCase();
    if (key !== "z" && key !== "y") return;
    var target = event.target;
    if (target && target.closest && !target.closest(".slides-tools-panel-host, .slides-tools-overlay")) return;
    event.preventDefault();
    if (key === "y" || event.shiftKey) soRedo(); else soUndo();
  });
}
