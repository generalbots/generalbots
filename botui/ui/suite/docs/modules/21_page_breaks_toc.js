// botui/ui/suite/docs/modules/21_page_breaks_toc.js
// Page breaks and Table of Contents (TOC) — refactored to delegate
// page-break persistence to botserver via window.DocsAPI.formatCells.
// TOC generation now uses /api/docs/toc/generate for the authoritative
// outline; client-side generation is kept as a fallback when the
// server is unreachable.
//
// API:
//   window.DocsPageBreaks.insertAtCursor()       -> Promise<{ok,...}>
//   window.DocsPageBreaks.insertBefore(el)       -> Promise
//   window.DocsPageBreaks.insertAfter(el)        -> Promise
//   window.DocsPageBreaks.remove(el)             -> Promise
//   window.DocsPageBreaks.getAll()               -> Promise<[el,...]>
//
//   window.DocsTOC.generate()                    -> Promise<string>
//   window.DocsTOC.insertAtCursor()              -> Promise<el>
//   window.DocsTOC.update()                      -> Promise<void>
//   window.DocsTOC.bindClicks(container)         -> void
"use strict";

(function () {
  const PAGE_BREAK_CLASS = "docs-page-break";
  const PAGE_BREAK_ATTR = "data-page-break";
  const TOC_ID = "docs-toc";
  const HEADING_TAGS = ["H1", "H2", "H3", "H4", "H5", "H6"];

  function getAPI() {
    return window.DocsAPI || null;
  }

  function getDocId() {
    const el = document.getElementById("docName");
    return (el && el.value) ? el.value : "default";
  }

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
    if (!editor) return Promise.resolve(null);
    const br = makeBreakElement("after");
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0) {
      const range = sel.getRangeAt(0);
      if (editor.contains(range.commonAncestorContainer)) {
        range.insertNode(br);
        range.setStartAfter(br);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
      } else {
        editor.appendChild(br);
      }
    } else {
      editor.appendChild(br);
    }
    notifyChange();
    return Promise.resolve(br);
  }

  function insertBefore(target) {
    if (!target) return Promise.resolve(null);
    const br = makeBreakElement("before");
    target.parentNode.insertBefore(br, target);
    notifyChange();
    return Promise.resolve(br);
  }

  function insertAfter(target) {
    if (!target) return Promise.resolve(null);
    const br = makeBreakElement("after");
    if (target.nextSibling) {
      target.parentNode.insertBefore(br, target.nextSibling);
    } else {
      target.parentNode.appendChild(br);
    }
    notifyChange();
    return Promise.resolve(br);
  }

  function remove(breakEl) {
    if (!breakEl || !breakEl.parentNode) return Promise.resolve(false);
    breakEl.parentNode.removeChild(breakEl);
    notifyChange();
    return Promise.resolve(true);
  }

  function notifyChange() {
    const editor = getEditor();
    if (!editor) return;
    const ev = new CustomEvent("docs-structure-changed", {
      bubbles: true,
      cancelable: true,
      detail: { type: "page-break" },
    });
    editor.dispatchEvent(ev);
    const API = getAPI();
    if (!API) return;
    const html = editor.innerHTML;
    API.autosave(getDocId(), { html: html, change_type: "page-break" }).catch(function () {
      // autosave failed; editor's own save will catch up
    });
  }

  function getAll() {
    const editor = getEditor();
    if (!editor) return Promise.resolve([]);
    return Promise.resolve(Array.prototype.slice.call(editor.querySelectorAll("[" + PAGE_BREAK_ATTR + "]")));
  }

  function findHeadings() {
    const editor = getEditor();
    if (!editor) return [];
    return Array.prototype.slice.call(editor.querySelectorAll(HEADING_TAGS.join(",")));
  }

  function assignIds(headings) {
    for (let i = 0; i < headings.length; i++) {
      const h = headings[i];
      if (!h.id) h.id = "docs-heading-" + Date.now() + "-" + i;
    }
    return headings;
  }

  function buildHtml(headings) {
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

  function generate() {
    const API = getAPI();
    if (API) {
      return API.generateToc(getDocId()).then(function (r) {
        if (r.ok && r.data && r.data.toc_html) return r.data.toc_html;
        return buildHtml(assignIds(findHeadings()));
      });
    }
    return Promise.resolve(buildHtml(assignIds(findHeadings())));
  }

  function insertAtCursorToc() {
    const editor = getEditor();
    if (!editor) return Promise.resolve(null);
    return generate().then(function (html) {
      const container = document.createElement("div");
      container.className = "docs-toc-wrapper";
      container.innerHTML = html;
      editor.appendChild(container);
      bindClicks(container);
      return container;
    });
  }

  function update() {
    const existing = document.getElementById(TOC_ID);
    if (!existing) return Promise.resolve();
    const parent = existing.closest(".docs-toc-wrapper") || existing.parentNode;
    if (!parent) return Promise.resolve();
    return generate().then(function (html) {
      parent.innerHTML = html;
      bindClicks(parent);
      const API = getAPI();
      if (API) API.updateToc(getDocId());
    });
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
  };

  window.DocsTOC = {
    generate: generate,
    insertAtCursor: insertAtCursorToc,
    update: update,
    bindClicks: bindClicks,
  };
})();
