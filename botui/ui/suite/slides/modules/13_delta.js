"use strict";
/* #2172 — apply gb-doc-delta/v1 ops to the live deck.
 *
 * UAT SLIDE-02: `add_slide` was handed to `window.SlidesEditor.addSlide` /
 * `window.addSlide`, and neither function existed anywhere in the app, so a
 * generated insert reached the deck store but never the deck on screen —
 * "inserting a new slide does nothing". The op now performs the same append
 * the toolbar dialog performs (POST /api/slides/slide/add) and re-renders the
 * canvas and the thumbnail rail from the stored deck, so a reload keeps it.
 */
(function () {
  function deckId(payload) {
    return (payload && payload.file) || (window.getSlidesPresentationId && window.getSlidesPresentationId()) || window.__SLIDES_PRESENTATION_ID || "";
  }

  // `position` is the 0-based insert index the op log carries; `after` is the
  // 1-based "insert after slide N" the chat phrase uses (same number, since
  // inserting after slide N is index N). Neither means "append".
  function insertIndex(op) {
    var raw = op ? (op.position !== undefined ? op.position : op.after) : undefined;
    if (raw === undefined || raw === null || raw === "") return null;
    var parsed = parseInt(raw, 10);
    return isNaN(parsed) ? null : parsed;
  }

  function addSlide(op, payload) {
    var position = insertIndex(op);
    var body = {
      presentation_id: deckId(payload),
      layout: (op && op.layout) || "two_column"
    };
    if (position !== null) body.position = position;
    return fetch("/api/slides/slide/add", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function () {
        if (window.refreshSlideView) {
          window.refreshSlideView(position === null ? undefined : position);
        }
      })
      .catch(function (error) {
        if (window.GBToasts && window.GBToasts.show) {
          window.GBToasts.show("Slides", "Insert failed: " + error.message, "error");
        }
      });
  }

  window.SlidesEditor = window.SlidesEditor || {};
  window.SlidesEditor.addSlide = addSlide;
  window.addSlide = addSlide;

(function () {
  function registerWhenReady(attempt) {
    if (window.GBDocDelta && typeof window.GBDocDelta.register === "function") {
      window.GBDocDelta.register("slides", apply);
      return;
    }
    if ((attempt || 0) >= 20) return;
    setTimeout(function () { registerWhenReady((attempt || 0) + 1); }, 250);
  }

  function fillTitle(value) {
    var input = document.getElementById("deck-title") ||
      document.querySelector("#presentation-title, .presentation-title, [data-presentation-title]");
    if (input) {
      if ("value" in input) input.value = String(value || "");
      else input.textContent = String(value || "");
    }
    var id = (window.getSlidesPresentationId && window.getSlidesPresentationId()) || window.__SLIDES_PRESENTATION_ID || "";
    fetch("/api/slides/rename", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ presentation_id: id, title: String(value || "") })
    }).catch(function (err) {
      console.error("[gb-slides] Rename failed:", err);
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show("Slides", "Could not rename presentation", "error");
      }
    });
  }

  function fillSlides(value) {
    var text = String(value || "");
    // Treat a newline-separated string or an array as slide bodies.
    var slides = Array.isArray(value) ? value : text.split(/\n{2,}/).filter(function (part) { return part.trim(); });
    if (!slides.length) slides = [text];
    slides.forEach(function (body) {
      addSlide({ layout: "title_content", title: String(body).split("\n")[0].slice(0, 80), body: body }, null);
    });
  }

  function apply(payload) {
    var ops = payload.ops || [];
    ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "add_slide") {
        addSlide(op, payload);
      } else if (op.op === "finalize" && window.GBToasts) {
        window.GBToasts.show("Slides", "Generation complete", "success");
      } else if (op.op === "fill") {
        var field = String(op.field || "").toLowerCase();
        if (field.indexOf("title") >= 0) {
          fillTitle(op.value);
        } else if (field.indexOf("body") >= 0 || field.indexOf("slide") >= 0) {
          fillSlides(op.value);
        }
      }
    });
  }

  registerWhenReady(0);
  // Standalone fallback only: register() already routes batches to apply().
  if (!window.GBDocDelta) {
    document.addEventListener("gb-doc-delta", function (e) {
      apply((e && e.detail) || {});
    });
  }
})();
})();
