"use strict";

/**
 * Module 24: Slide numbering for Slides.
 * Inserts auto-updating slide numbers as a footer element on all slides
 * (with custom format like 1/N, 01/N, "A-1"). Suppressed on the title
 * slide by default. Updates on navigation.
 *
 * Public API: window.SlidesNumbering = { enable, disable, setFormat,
 *   setSuppression, refresh, renderOn }.
 */

(function () {
  function getState() { return window.state || null; }
  let _enabled = false;
  let _format = "n-of-N";
  let _suppressTitle = true;

  function formatNumber(n, total, fmt) {
    switch (fmt) {
      case "n-only": return String(n);
      case "n-of-N": return n + "/" + total;
      case "zero-padded": return String(n).padStart(2, "0") + "/" + String(total).padStart(2, "0");
      case "letter-n": return String.fromCharCode(64 + n) + "-" + n;
      case "alpha-only": {
        let s = "";
        let v = n;
        while (v > 0) { const r = (v - 1) % 26; s = String.fromCharCode(65 + r) + s; v = Math.floor((v - 1) / 26); }
        return s;
      }
      case "Page-n": return "Page " + n;
      default: return n + "/" + total;
    }
  }

  function setFormat(fmt) { _format = fmt || "n-of-N"; refresh(); }
  function setSuppression(sup) { _suppressTitle = !!sup; refresh(); }
  function enable() { _enabled = true; refresh(); }
  function disable() {
    _enabled = false;
    document.querySelectorAll(".slide-number-display").forEach((n) => n.remove());
  }

  function refresh() {
    const s = getState();
    if (!s) return;
    const slides = s.slides || [];
    for (let i = 0; i < slides.length; i++) {
      if (_suppressTitle && slides[i].layout === "title") {
        const c = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
        if (c) c.querySelectorAll(".slide-number-display").forEach((n) => n.remove());
        continue;
      }
      renderOn(i);
    }
  }

  function renderOn(slideIndex) {
    if (!_enabled) return;
    const s = getState();
    if (!s) return;
    const slide = (s.slides || [])[slideIndex];
    if (!slide) return;
    const total = (s.slides || []).length;
    const txt = formatNumber((slideIndex || 0) + 1, total, _format);
    if (!slide._numberEl) {
      slide._numberEl = document.createElement("div");
      slide._numberEl.className = "slide-number-display";
      slide._numberEl.style.cssText = "position:absolute;right:8px;bottom:8px;font-size:11px;color:#5f6368;pointer-events:auto;z-index:5;background:rgba(255,255,255,0.7);padding:2px 6px;border-radius:3px;";
    }
    slide._numberEl.textContent = txt;
    if (s.currentSlide === slideIndex) {
      const c = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
      if (c && !c.contains(slide._numberEl)) c.appendChild(slide._numberEl);
    }
  }

  function attach() {
    const prev = document.querySelector("[data-action='prev-slide'], .prev-slide");
    const next = document.querySelector("[data-action='next-slide'], .next-slide");
    function reRender() {
      const s = getState();
      if (!s) return;
      const c = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
      if (c) c.querySelectorAll(".slide-number-display").forEach((n) => n.remove());
      const idx = s.currentSlide || 0;
      renderOn(idx);
    }
    if (prev) prev.addEventListener("click", reRender);
    if (next) next.addEventListener("click", reRender);
    const toggle = document.querySelector("[data-toggle='slide-numbering']");
    if (toggle) toggle.addEventListener("change", function (e) { e.target.checked ? enable() : disable(); });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesNumbering = { enable, disable, setFormat, setSuppression, refresh, renderOn, formatNumber };
})();
