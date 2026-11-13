"use strict";
/* #2184 — formatting row of the two-row slides toolbar.
 *
 * The canvas renders elements from the server, so a style change has to go to
 * `/api/slides/element/update` (the handler merges the `style` object into the
 * element) AND be applied locally for instant feedback. Every control acts on
 * the element currently selected on the slide.
 * #2291 — colour pickers use GBColorPicker: live preview stays local, a single
 * commit (swatch/hex/native change) produces one update + one edit record +
 * one snapshot per interaction (never per drag pixel).
 */
(function () {
  function selectedElement() {
    var id = window.SlideCanvas && window.SlideCanvas.selectedId;
    if (!id) return null;
    return document.querySelector('#slide-canvas [data-id="' + id + '"]');
  }

  function applyLocal(el, style) {
    if (style.font_weight) el.style.fontWeight = style.font_weight;
    if (style.font_style) el.style.fontStyle = style.font_style;
    if (style.text_align) el.style.textAlign = style.text_align;
    if (style.color) el.style.color = style.color;
    if (style.fill) el.style.background = style.fill;
    if (style.font_size) el.style.fontSize = style.font_size + "px";
  }

  function applyLocalOnly(style) {
    var el = selectedElement();
    if (!el) return;
    applyLocal(el, style);
  }

  function persist(style) {
    var el = selectedElement();
    if (!el) return;
    applyLocal(el, style);
    var canvas = document.getElementById("slide-canvas");
    var slide = canvas ? parseInt(canvas.dataset.slideId, 10) || 0 : 0;
    fetch("/api/slides/element/update", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        presentation_id: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current",
        slide_index: slide,
        element_id: el.dataset.id,
        style: style,
      }),
    }).catch(function (err) {
      console.error("[gb-slides] Failed to save toolbar style:", err);
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show("Slides", "Could not save style change", "error");
      }
    });
    if (window.recordSlidesEdit) window.recordSlidesEdit();
    if (window.recordSlidesSnapshot) window.recordSlidesSnapshot();
  }

  // Toggle helper: bold/italic flip between on and off instead of only setting.
  function toggle(current, on, off) {
    return current === on ? off : on;
  }

  function bindToggle(button, key, on, off) {
    button.addEventListener("click", function (event) {
      event.preventDefault();
      var el = selectedElement();
      if (!el) return;
      var current = key === "font_weight" ? el.style.fontWeight : el.style.fontStyle;
      var next = toggle(current, on, off);
      button.setAttribute("aria-pressed", next === on ? "true" : "false");
      var style = {};
      style[key] = next;
      persist(style);
    });
  }

  function bindColor(id, styleKey) {
    var input = document.getElementById(id);
    if (!input) return;
    var trigger = document.querySelector('[data-gb-color-for="' + id + '"]');
    if (window.GBColorPicker) {
      window.GBColorPicker.enhance(input, { trigger: trigger });
    }
    input.addEventListener("gb-color-preview", function () {
      var style = {};
      style[styleKey] = input.value;
      applyLocalOnly(style);
    });
    input.addEventListener("gb-color-commit", function () {
      var style = {};
      style[styleKey] = input.value;
      persist(style);
    });
    // Fallback: if richer picker is unavailable, commit on native change only
    // (never on per-pixel input) to avoid flooding.
    input.addEventListener("change", function () {
      if (input._gbColorEnhanced) return;
      var style = {};
      style[styleKey] = input.value;
      persist(style);
    });
  }

  function initToolbar() {
    var bold = document.getElementById("slBoldBtn");
    if (bold) bindToggle(bold, "font_weight", "bold", "normal");
    var italic = document.getElementById("slItalicBtn");
    if (italic) bindToggle(italic, "font_style", "italic", "normal");

    Array.prototype.forEach.call(
      document.querySelectorAll("[data-slides-align]"),
      function (button) {
        button.addEventListener("click", function (event) {
          event.preventDefault();
          persist({ text_align: button.getAttribute("data-slides-align") || "left" });
        });
      },
    );

    Array.prototype.forEach.call(
      document.querySelectorAll("[data-slides-style]"),
      function (button) {
        button.addEventListener("click", function (event) {
          event.preventDefault();
          var spec = (button.getAttribute("data-slides-style") || "").split(":");
          if (spec.length !== 2) return;
          var style = {};
          style[spec[0]] = spec[1];
          persist(style);
        });
      },
    );

    var size = document.getElementById("slFontSize");
    if (size) {
      size.addEventListener("change", function () {
        var value = parseFloat(size.value);
        if (!value) return;
        persist({ font_size: value });
      });
    }

    bindColor("slTextColor", "color");
    bindColor("slFillColor", "fill");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initToolbar);
  } else {
    initToolbar();
  }
})();
