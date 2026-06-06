// botui/ui/suite/docs/modules/21_page_breaks_toc.js
// Page breaks and Table of Contents (TOC) for the Word Processor.
//
// Features:
//   1. Page breaks: insert a page break before/after a paragraph
//      or at the cursor. Page breaks render as visible markers in
//      the editor and as actual page breaks in PDF/DOCX export.
//   2. TOC: scan the document for headings (H1-H6), build a nested
//      TOC tree, and inject at cursor. Click-to-jump.
//
// API:
//   window.DocsPageBreaks.insertAtCursor()
//   window.DocsPageBreaks.insertBefore(el)
//   window.DocsPageBreaks.insertAfter(el)
//   window.DocsPageBreaks.remove(el)
//   window.DocsPageBreaks.getAll() -> [Element, ...]
//   window.DocsPageBreaks.renderMarker(breakEl)
//
//   window.DocsTOC.generate() -> HTML string of TOC
//   window.DocsTOC.insertAtCursor()
//   window.DocsTOC.update()
//   window.DocsTOC.bindClicks(tocContainer)
//
// Both modules operate on the contenteditable editor (#editorContent
// or whatever the docs module exposes).
"use strict";

(function () {
  const PAGE_BREAK_CLASS = "docs-page-break";
  const PAGE_BREAK_ATTR = "data-page-break";
  const TOC_ID = "docs-toc";
  const HEADING_TAGS = ["H1", "H2", "H3", "H4", "H5", "H6"];

  function getEditor() {
    return (
      document.querySelector("#editorContent") ||
      document.querySelector("[contenteditable]") ||
      document.querySelector(".editor-page")
    );
  }

  function makeBreakElement(position) {
    const el = document.createElement("div");
    el.className = PAGE_BREAK_CLASS;
    el.setAttribute(PAGE_BREAK_ATTR, "1");
    el.setAttribute("contenteditable", "false");
    el.setAttribute("data-position", position || "before");
    el.style.pageBreakBefore = "always";
    el.style.breakBefore = "page";
    el.innerHTML =
      '<div class="docs-page-break-line" title="Page break"></div>' +
      '<span class="docs-page-break-label">— Page Break —</span>';
    return el;
  }

  function insertAtCursor() {
    const editor = getEditor();
    if (!editor) return false;
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) {
      // Append at end
      const br = makeBreakElement("after");
      editor.appendChild(br);
      return br;
    }
    const range = sel.getRangeAt(0);
    if (!editor.contains(range.commonAncestorContainer)) {
      const br = makeBreakElement("after");
      editor.appendChild(br);
      return br;
    }
    const br = makeBreakElement("after");
    range.insertNode(br);
    // Move cursor after the break
    range.setStartAfter(br);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
    return br;
  }

  function insertBefore(target) {
    if (!target) return null;
    const br = makeBreakElement("before");
    target.parentNode.insertBefore(br, target);
    return br;
  }

  function insertAfter(target) {
    if (!target) return null;
    const br = makeBreakElement("after");
    if (target.nextSibling) {
      target.parentNode.insertBefore(br, target.nextSibling);
    } else {
      target.parentNode.appendChild(br);
    }
    return br;
  }

  function remove(breakEl) {
    if (!breakEl || !breakEl.parentNode) return false;
    breakEl.parentNode.removeChild(breakEl);
    return true;
  }

  function getAll() {
    const editor = getEditor();
    if (!editor) return [];
    return Array.prototype.slice.call(
      editor.querySelectorAll("[" + PAGE_BREAK_ATTR + "]")
    );
  }

  function renderMarker(breakEl) {
    if (!breakEl) return;
    breakEl.innerHTML =
      '<div class="docs-page-break-line"></div>' +
      '<span class="docs-page-break-label">— Page Break —</span>';
  }

  // -------- TOC --------

  function findHeadings() {
    const editor = getEditor();
    if (!editor) return [];
    return Array.prototype.slice.call(editor.querySelectorAll(HEADING_TAGS.join(",")));
  }

  function assignIds() {
    const headings = findHeadings();
    for (let i = 0; i < headings.length; i++) {
      const h = headings[i];
      if (!h.id) {
        h.id = "docs-heading-" + Date.now() + "-" + i;
      }
    }
    return headings;
  }

  function generate() {
    const headings = assignIds();
    if (headings.length === 0) {
      return '<p class="docs-toc-empty">No headings found. Use H1-H6 to create a table of contents.</p>';
    }
    let html = '<nav class="docs-toc" id="' + TOC_ID + '"><ol>';
    let openLevel = 0;
    for (let i = 0; i < headings.length; i++) {
      const h = headings[i];
      const level = parseInt(h.tagName.charAt(1), 10);
      const text = h.textContent || "(untitled)";
      if (level > openLevel) {
        for (let j = openLevel; j < level; j++) html += "<ol>";
        openLevel = level;
      } else if (level < openLevel) {
        for (let j = level; j < openLevel; j++) html += "</ol></li>";
        openLevel = level;
      } else if (i > 0) {
        html += "</li>";
      }
      html += '<li class="docs-toc-item docs-toc-h' + level + '">' +
        '<a href="#' + h.id + '" data-toc-target="' + h.id + '">' +
        escapeHtml(text) + '</a>';
    }
    for (let j = 0; j < openLevel; j++) html += "</li></ol>";
    html += "</nav>";
    return html;
  }

  function insertAtCursor() {
    const editor = getEditor();
    if (!editor) return false;
    const html = generate();
    const container = document.createElement("div");
    container.className = "docs-toc-wrapper";
    container.innerHTML = html;
    editor.appendChild(container);
    bindClicks(container);
    return container;
  }

  function update() {
    const existing = document.getElementById(TOC_ID);
    if (existing) {
      const parent = existing.closest(".docs-toc-wrapper") || existing.parentNode;
      if (parent) {
        parent.innerHTML = generate();
        bindClicks(parent);
      }
    }
  }

  function bindClicks(container) {
    if (!container) return;
    const links = container.querySelectorAll("[data-toc-target]");
    for (let i = 0; i < links.length; i++) {
      links[i].addEventListener("click", function (e) {
        e.preventDefault();
        const id = links[i].getAttribute("data-toc-target");
        const target = document.getElementById(id);
        if (target) {
          target.scrollIntoView({ behavior: "smooth", block: "start" });
          target.focus({ preventScroll: true });
        }
      });
    }
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  window.DocsPageBreaks = {
    insertAtCursor: insertAtCursor,
    insertBefore: insertBefore,
    insertAfter: insertAfter,
    remove: remove,
    getAll: getAll,
    renderMarker: renderMarker,
  };

  window.DocsTOC = {
    generate: generate,
    insertAtCursor: insertAtCursor,
    update: update,
    bindClicks: bindClicks,
  };
})();
