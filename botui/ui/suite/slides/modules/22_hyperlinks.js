"use strict";

/**
 * Module 22: Hyperlinks for Slides.
 * Toolbar button to insert/edit/remove hyperlinks on text elements
 * and shapes. Supports external URL, internal slide anchor (slide://N),
 * mailto, and download. Rendered as <a> inside the text element with
 * tooltip preview on hover.
 *
 * Public API: window.SlidesHyperlink = { insert, edit, remove, render, getHrefs }.
 */

(function () {
  function getState() { return window.state || null; }
  function getSlide() {
    const s = getState();
    return s ? (s.slides || [])[s.currentSlide || 0] : null;
  }
  function getActiveElement() {
    return document.querySelector(".slide-element.editing") || document.querySelector(".slide-element.selected");
  }
  function getSelectedText() {
    const sel = window.getSelection ? window.getSelection() : null;
    return sel ? sel.toString() : "";
  }

  function buildLinkData(href, text) {
    let protocol = "external";
    let target = href;
    if (href.startsWith("mailto:")) protocol = "mailto";
    else if (href.startsWith("slide://")) {
      protocol = "anchor";
      target = href.replace("slide://", "");
    } else if (href.startsWith("http://") || href.startsWith("https://")) protocol = "external";
    else protocol = "internal";
    return { protocol, target, text: text || href, rel: "noopener noreferrer" };
  }

  function insert(href, text) {
    if (!href) return false;
    const link = buildLinkData(href, text || getSelectedText());
    const slide = getSlide();
    if (!slide) return false;
    if (!slide.hyperlinks) slide.hyperlinks = [];
    slide.hyperlinks.push({
      id: "h-" + Date.now(),
      elementId: link.elementId || null,
      ...link,
    });
    const el = getActiveElement();
    if (el) {
      if (!link.elementId) link.elementId = el.dataset.elementId || el.id;
      const sel = window.getSelection();
      if (sel && sel.rangeCount > 0 && !sel.isCollapsed) {
        const range = sel.getRangeAt(0);
        const a = document.createElement("a");
        a.href = href;
        a.textContent = link.text;
        a.className = "slide-link";
        a.style.cssText = "color:#1a73e8;text-decoration:underline;";
        a.addEventListener("click", function (e) { e.preventDefault(); navigate(link); });
        try { range.surroundContents(a); }
        catch (_e) { a.textContent = link.text; el.appendChild(a); }
      } else {
        const a = document.createElement("a");
        a.href = href;
        a.textContent = link.text;
        a.className = "slide-link";
        a.style.cssText = "color:#1a73e8;text-decoration:underline;";
        a.addEventListener("click", function (e) { e.preventDefault(); navigate(link); });
        el.appendChild(a);
      }
    }
    return link;
  }

  function edit(existing) {
    if (!existing) return false;
    const newHref = window.prompt("Editar link (use slide://N para âncora interna, mailto:email, ou URL):", existing.target);
    if (!newHref) return false;
    existing.target = newHref;
    existing.protocol = newHref.startsWith("mailto:") ? "mailto"
      : newHref.startsWith("slide://") ? "anchor"
      : (newHref.startsWith("http://") || newHref.startsWith("https://")) ? "external"
      : "internal";
    const a = document.querySelector('a.slide-link[href="' + (existing.target || "") + '"]');
    if (a) a.href = newHref;
    return existing;
  }

  function remove(elementId) {
    const slide = getSlide();
    if (!slide || !slide.hyperlinks) return false;
    const before = slide.hyperlinks.length;
    slide.hyperlinks = slide.hyperlinks.filter((h) => h.elementId !== elementId);
    document.querySelectorAll('a.slide-link').forEach((a) => {
      const parent = a.closest(".slide-element");
      if (parent && parent.dataset.elementId === elementId) {
        const text = document.createTextNode(a.textContent);
        a.replaceWith(text);
      }
    });
    return slide.hyperlinks.length < before;
  }

  function navigate(link) {
    if (!link) return;
    if (link.protocol === "anchor") {
      const slideNum = parseInt(link.target, 10);
      const s = getState();
      if (s && slideNum >= 0 && slideNum < (s.slides || []).length) {
        s.currentSlide = slideNum;
        if (typeof window.SlidesNavigate === "object" && window.SlidesNavigate.goTo) {
          window.SlidesNavigate.goTo(slideNum);
        }
      }
    } else if (link.protocol === "mailto") {
      window.location.href = link.target;
    } else {
      window.open(link.target, "_blank", "noopener,noreferrer");
    }
  }

  function render() {
    const slide = getSlide();
    if (!slide || !slide.hyperlinks) return;
    for (const link of slide.hyperlinks) {
      const el = document.querySelector('[data-element-id="' + link.elementId + '"]');
      if (!el) continue;
      if (el.querySelector('a.slide-link[href="' + link.target + '"]')) continue;
      const a = document.createElement("a");
      a.href = link.target;
      a.textContent = link.text;
      a.title = link.target;
      a.className = "slide-link";
      a.style.cssText = "color:#1a73e8;text-decoration:underline;";
      a.addEventListener("click", function (e) { e.preventDefault(); navigate(link); });
      el.appendChild(a);
    }
  }

  function getHrefs() {
    const slide = getSlide();
    return slide && slide.hyperlinks ? slide.hyperlinks : [];
  }

  function prompt() {
    const href = window.prompt("URL do link (slide://N, mailto:email, ou https://...):", "https://");
    if (!href) return null;
    const text = window.prompt("Texto visível (vazio = URL):", "");
    return insert(href, text);
  }

  function attach() {
    const btn = document.querySelector("[data-toolbar-action='hyperlink']");
    if (btn) btn.addEventListener("click", function (e) { e.preventDefault(); prompt(); });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesHyperlink = { insert, edit, remove, render, getHrefs, navigate, prompt };
})();
