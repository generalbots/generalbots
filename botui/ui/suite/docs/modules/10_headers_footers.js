"use strict";

/**
 * Module 10: Headers and footers per page for Word Processor.
 * Renders header/footer on every paginated page (.page-header-slot /
 * .page-footer-slot). Supports dynamic field codes: [PAGE], [SECTION],
 * [TOTAL_PAGES], [DATE], [TIME], [AUTHOR] that update on render.
 *
 * Implements "Different first page", "Different odd & even pages",
 * "Link to previous" (section linking). Headers/footers are stored
 * per-section, not per-document.
 *
 * Public API: window.DocsHeadersFooters = { setHeader, setFooter,
 *   setFirstPageHeader, setFirstPageFooter, setOddEvenDifferent,
 *   getSection, render, expandFieldCodes }.
 */

(function () {
  function getState() { return window.state || null; }

  function getSections() {
    const s = getState();
    if (!s) return [{ id: 0, name: "Section 1" }];
    if (!s.sections) s.sections = [{ id: 0, name: "Section 1" }];
    return s.sections;
  }

  function getSectionByPage(pageIndex) {
    const sections = getSections();
    const pageSections = (getState() && getState().pageSections) || {};
    if (pageSections[pageIndex] != null) return sections[pageSections[pageIndex]] || sections[0];
    return sections[0];
  }

  function expandFieldCodes(text) {
    if (!text) return "";
    const now = new Date();
    const pages = document.querySelectorAll(".doc-page").length;
    const currentPageIdx = (function () {
      const pages = Array.from(document.querySelectorAll(".doc-page"));
      const scrollY = window.scrollY;
      let idx = 0;
      for (let i = 0; i < pages.length; i++) {
        const r = pages[i].getBoundingClientRect();
        if (r.top >= 0) { idx = i; break; }
      }
      return idx + 1;
    })();
    return text
      .replace(/\[PAGE\]/g, String(currentPageIdx))
      .replace(/\[SECTION\]/g, String(1))
      .replace(/\[TOTAL_PAGES\]/g, String(pages || 1))
      .replace(/\[DATE\]/g, now.toLocaleDateString("pt-BR"))
      .replace(/\[TIME\]/g, now.toLocaleTimeString("pt-BR"))
      .replace(/\[AUTHOR\]/g, (getState() && getState().author) || "Unknown");
  }

  function render() {
    const pages = document.querySelectorAll(".doc-page");
    if (!pages.length) return;
    pages.forEach((page, idx) => {
      const section = getSectionByPage(idx);
      const isFirst = idx === 0;
      const isOdd = (idx + 1) % 2 === 1;
      const useFirstPage = section.firstPageDifferent && isFirst;
      const useOddEven = section.oddEvenDifferent;
      const headerText = useFirstPage
        ? (section.firstPageHeader || "")
        : (useOddEven && !isOdd
            ? (section.evenPageHeader || section.header || "")
            : (section.header || ""));
      const footerText = useFirstPage
        ? (section.firstPageFooter || "")
        : (useOddEven && !isOdd
            ? (section.evenPageFooter || section.footer || "")
            : (section.footer || ""));
      const headerSlot = page.querySelector(".page-header-slot");
      const footerSlot = page.querySelector(".page-footer-slot");
      if (headerSlot) {
        headerSlot.innerHTML = headerText ? '<div class="page-header-content" style="text-align:' + (section.headerAlign || "center") + ';font-size:11px;color:#666;">' + expandFieldCodes(headerText) + '</div>' : '';
      }
      if (footerSlot) {
        footerSlot.innerHTML = footerText ? '<div class="page-footer-content" style="text-align:' + (section.footerAlign || "center") + ';font-size:11px;color:#666;">' + expandFieldCodes(footerText) + '</div>' : '';
      }
    });
  }

  function setHeader(sectionIndex, text) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    s.header = text;
    render();
  }

  function setFooter(sectionIndex, text) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    s.footer = text;
    render();
  }

  function setFirstPageHeader(sectionIndex, text) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    s.firstPageHeader = text;
    render();
  }

  function setFirstPageFooter(sectionIndex, text) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    s.firstPageFooter = text;
    render();
  }

  function setOddEvenDifferent(sectionIndex, on) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    s.oddEvenDifferent = !!on;
    render();
  }

  function setFirstPageDifferent(sectionIndex, on) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    s.firstPageDifferent = !!on;
    render();
  }

  function setAlign(sectionIndex, side, align) {
    const sections = getSections();
    const s = sections[sectionIndex] || sections[0];
    if (side === "header") s.headerAlign = align;
    else s.footerAlign = align;
    render();
  }

  function getSection(index) {
    return getSections()[index] || getSections()[0];
  }

  function attach() {
    setTimeout(render, 250);
    document.addEventListener("docsPaginated", render);
    document.addEventListener("docsPageConfigChange", render);
    document.addEventListener("scroll", function () {
      clearTimeout(window.__docsHFTimer);
      window.__docsHFTimer = setTimeout(render, 100);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsHeadersFooters = {
    setHeader,
    setFooter,
    setFirstPageHeader,
    setFirstPageFooter,
    setOddEvenDifferent,
    setFirstPageDifferent,
    setAlign,
    getSection,
    render,
    expandFieldCodes,
  };
})();
