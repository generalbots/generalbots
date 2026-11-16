"use strict";
/* Slides undo/redo — style formatting and element delete (#2289).
 * Stack stores actions that can reapply and persist inverse operations so the
 * server copy stays consistent with the visible state.
 */
(function () {
  var undoStack = [];
  var redoStack = [];
  var MAX = 100;
  var applying = false;

  function getDeckId() {
    return (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current";
  }
  function getSlideIndex() {
    var canvas = document.getElementById("slide-canvas");
    return canvas ? parseInt(canvas.dataset.slideId, 10) || 0 : 0;
  }
  function findElement(id) {
    if (!id) return null;
    return document.querySelector('#slide-canvas [data-id="' + CSS.escape(String(id)) + '"]') ||
      document.querySelector('#slide-canvas [data-id="' + String(id).replace(/"/g, '\\"') + '"]');
  }
  function updateButtons() {
    var u = document.getElementById("slidesUndoBtn");
    var r = document.getElementById("slidesRedoBtn");
    if (u) {
      u.disabled = undoStack.length === 0 || applying;
      u.setAttribute("aria-disabled", u.disabled ? "true" : "false");
    }
    if (r) {
      r.disabled = redoStack.length === 0 || applying;
      r.setAttribute("aria-disabled", r.disabled ? "true" : "false");
    }
  }
  function push(action) {
    if (applying || !action) return;
    undoStack.push(action);
    if (undoStack.length > MAX) undoStack.shift();
    redoStack.length = 0;
    updateButtons();
  }

  function applyLocalStyle(el, style) {
    if (!el || !style) return;
    if (Object.prototype.hasOwnProperty.call(style, "font_weight")) el.style.fontWeight = style.font_weight || "";
    if (Object.prototype.hasOwnProperty.call(style, "font_style")) el.style.fontStyle = style.font_style || "";
    if (Object.prototype.hasOwnProperty.call(style, "text_align")) el.style.textAlign = style.text_align || "";
    if (Object.prototype.hasOwnProperty.call(style, "color")) el.style.color = style.color || "";
    if (Object.prototype.hasOwnProperty.call(style, "fill")) el.style.background = style.fill || "";
    if (Object.prototype.hasOwnProperty.call(style, "font_size")) {
      el.style.fontSize = (style.font_size === null || style.font_size === undefined || style.font_size === "") ? "" : (style.font_size + "px");
    }
    if (Object.prototype.hasOwnProperty.call(style, "font_family")) {
      el.style.fontFamily = style.font_family || "";
    }
  }
  function readStyle(el, keys) {
    var out = {};
    keys.forEach(function (k) {
      if (k === "font_weight") out.font_weight = el.style.fontWeight || "normal";
      else if (k === "font_style") out.font_style = el.style.fontStyle || "normal";
      else if (k === "text_align") out.text_align = el.style.textAlign || "left";
      else if (k === "color") out.color = el.style.color || "";
      else if (k === "fill") out.fill = el.style.background || "";
      else if (k === "font_size") {
        var v = parseFloat(el.style.fontSize);
        out.font_size = isFinite(v) ? v : null;
      } else if (k === "font_family") {
        out.font_family = el.style.fontFamily || "";
      }
    });
    return out;
  }
  function persistStyle(el, style) {
    if (!el) return Promise.resolve();
    return fetch("/api/slides/element/update", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        presentation_id: getDeckId(),
        slide_index: getSlideIndex(),
        element_id: el.dataset.id,
        style: style
      })
    }).catch(function (err) {
      console.warn("[slides-undo] failed to persist style update", err);
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show(
          typeof gbT === "function" ? gbT("slides-title", "Slides") : "Slides",
          typeof gbT === "function" ? gbT("slides-undo-save-style-failed", "Could not save style change") : "Could not save style change",
          "warning"
        );
      }
    });
  }

  function rgbToHex(rgb) {
    if (!rgb) return "";
    var m = String(rgb).match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
    if (!m) return String(rgb);
    function h(n){ n = Math.max(0, Math.min(255, parseInt(n,10)||0)); return ("0"+n.toString(16)).slice(-2); }
    return "#" + h(m[1]) + h(m[2]) + h(m[3]);
  }
  function styleFromElement(el) {
    var style = {};
    if (el.style.fontWeight) style.font_weight = el.style.fontWeight;
    if (el.style.fontStyle) style.font_style = el.style.fontStyle;
    if (el.style.textAlign) style.text_align = el.style.textAlign;
    if (el.style.color) style.color = rgbToHex(el.style.color) || el.style.color;
    if (el.style.background || el.style.backgroundColor) style.fill = rgbToHex(el.style.backgroundColor || el.style.background) || (el.style.backgroundColor || el.style.background);
    var fs = parseFloat(el.style.fontSize);
    if (isFinite(fs)) style.font_size = fs;
    var ff = el.style.fontFamily;
    if (ff) style.font_family = ff.replace(/["']/g, "");
    return style;
  }
  function contentFromElement(el) {
    var type = el.dataset.type || "text";
    var content = {};
    if (type === "text" || type === "title") {
      var t = el.querySelector("p, h1, h2, h3");
      content.text = (t ? t.textContent : el.textContent) || "";
      if (t) content.html = t.innerHTML;
    } else if (type === "shape") {
      content.shape_type = (el.dataset.shapeType || el.getAttribute("data-shape-type") || "rectangle");
    } else if (type === "image") {
      var img = el.querySelector("img");
      if (img) content.src = img.getAttribute("src") || "";
    }
    return content;
  }
  function buildAddPayload(el, slideIndex) {
    var type = el.dataset.type || "shape";
    var payload = {
      presentation_id: getDeckId(),
      slide_index: slideIndex,
      element: {
        id: el.dataset.id,
        element_type: type,
        x: parseFloat(el.style.left) || 0,
        y: parseFloat(el.style.top) || 0,
        width: parseFloat(el.style.width) || el.offsetWidth || 120,
        height: parseFloat(el.style.height) || el.offsetHeight || 80,
        rotation: parseFloat(el.dataset.rotation || "0") || 0,
        content: contentFromElement(el),
        style: styleFromElement(el),
        animations: [],
        z_index: parseInt(el.style.zIndex, 10) || 0,
        locked: false
      }
    };
    // flat fallback fields some handlers accept
    payload.element_id = el.dataset.id;
    payload.element_type = type;
    payload.x = payload.element.x; payload.y = payload.element.y;
    payload.width = payload.element.width; payload.height = payload.element.height;
    payload.rotation = payload.element.rotation;
    if (payload.element.content.text) payload.text = payload.element.content.text;
    if (payload.element.content.shape_type) payload.shape_type = payload.element.content.shape_type;
    if (payload.element.content.src) payload.src = payload.element.content.src;
    return payload;
  }

  async function applyStyleAction(action) {
    var el = findElement(action.elementId);
    if (!el) return;
    applyLocalStyle(el, action.style);
    await persistStyle(el, action.style);
  }
  async function applyDeleteAction(action) {
    var el = findElement(action.elementId);
    if (!el) return;
    // remove locally without recording another undo entry
    applying = true;
    try {
      if (window.SlideCanvas && window.SlideCanvas.selectedId === action.elementId) {
        window.SlideCanvas.deselectAll();
      }
      el.remove();
    } finally { applying = false; }
    await fetch("/api/slides/element/delete", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        presentation_id: getDeckId(),
        slide_index: action.slideIndex,
        element_id: action.elementId
      })
    }).catch(function (err) {
      console.warn("[slides-undo] failed to persist delete", err);
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show(
          typeof gbT === "function" ? gbT("slides-title", "Slides") : "Slides",
          typeof gbT === "function" ? gbT("slides-undo-delete-failed", "Could not save deletion") : "Could not save deletion",
          "warning"
        );
      }
    });
  }
  async function applyRestoreAction(action) {
    // Ask server to re-add element, then refresh canvas from server state by reloading deck view.
    await fetch("/api/slides/element/add", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(action.payload)
    }).catch(function (err) {
      console.warn("[slides-undo] failed to restore element", err);
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show(
          typeof gbT === "function" ? gbT("slides-title", "Slides") : "Slides",
          typeof gbT === "function" ? gbT("slides-undo-restore-failed", "Could not restore element") : "Could not restore element",
          "warning"
        );
      }
    });
    // Re-render current slide to get canonical element DOM/bindings.
    if (window.htmx) {
      var slide = action.slideIndex;
      htmx.ajax('GET', '/suite/slides/fragments/presentation-view?id=' + encodeURIComponent(getDeckId()) + '&slide=' + encodeURIComponent(slide),
        { target: '#slides-content', swap: 'innerHTML' });
    }
  }

  async function doUndo() {
    if (applying || undoStack.length === 0) return;
    var action = undoStack.pop();
    applying = true;
    try {
      if (action.type === "style") {
        await applyStyleAction({ elementId: action.elementId, style: action.before });
        redoStack.push(action);
      } else if (action.type === "delete") {
        await applyRestoreAction(action);
        redoStack.push(action);
      }
    } finally {
      applying = false;
      updateButtons();
    }
  }
  async function doRedo() {
    if (applying || redoStack.length === 0) return;
    var action = redoStack.pop();
    applying = true;
    try {
      if (action.type === "style") {
        await applyStyleAction({ elementId: action.elementId, style: action.after });
        undoStack.push(action);
      } else if (action.type === "delete") {
        await applyDeleteAction(action);
        undoStack.push(action);
      }
    } finally {
      applying = false;
      updateButtons();
    }
  }

  // Public API
  window.SlidesUndo = {
    push: push,
    undo: doUndo,
    redo: doRedo,
    updateButtons: updateButtons,
    isApplying: function () { return applying; },
    recordStyleChange: function (elementId, before, after) {
      if (!elementId || !before || !after) return;
      var keys = Object.keys(after);
      if (!keys.length) return;
      var cleanBefore = {}, cleanAfter = {}, changed = false;
      keys.forEach(function (k) {
        if (JSON.stringify(before[k]) !== JSON.stringify(after[k])) {
          cleanBefore[k] = before[k];
          cleanAfter[k] = after[k];
          changed = true;
        }
      });
      if (!changed) return;
      push({ type: "style", elementId: elementId, before: cleanBefore, after: cleanAfter });
    },
    recordDelete: function (elementId, element, slideIndex) {
      if (!elementId || !element) return;
      var payload = buildAddPayload(element, slideIndex);
      push({ type: "delete", elementId: elementId, slideIndex: slideIndex, payload: payload, outerHTML: element.outerHTML });
    }
  };

  function bind() {
    var u = document.getElementById("slidesUndoBtn");
    var r = document.getElementById("slidesRedoBtn");
    if (u) u.addEventListener("click", function (e) { e.preventDefault(); doUndo(); });
    if (r) r.addEventListener("click", function (e) { e.preventDefault(); doRedo(); });
    if (!window.__slidesUndoKeysBound) {
      window.__slidesUndoKeysBound = true;
      var onKeyDown = function (e) {
        var t = e.target;
        if (t && t.nodeType === 1 && (t.isContentEditable || (t.closest && t.closest("input,textarea,select,[contenteditable='true']")))) return;
        var key = (e.key || "").toLowerCase();
        if ((e.ctrlKey || e.metaKey) && !e.altKey && key === "z") {
          e.preventDefault();
          if (e.shiftKey) doRedo(); else doUndo();
        } else if ((e.ctrlKey || e.metaKey) && !e.altKey && key === "y") {
          e.preventDefault();
          doRedo();
        }
      };
      document.addEventListener("keydown", onKeyDown);
      window.addEventListener("keydown", onKeyDown);
    }
    updateButtons();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind);
  else bind();
  document.body.addEventListener("htmx:afterSwap", function () { setTimeout(updateButtons, 0); });
})();
