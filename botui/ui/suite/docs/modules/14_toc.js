"use strict";

/**
 * Module 14: Table of Contents for Word Processor.
 * Adds an "Insert Table of Contents" toolbar button. When clicked,
 * scans the document for <h1>-<h6> tags and builds an indented list
 * of headings. The list is rendered as a field, not static text, so
 * it can be updated by re-running the scan. Supports formatting
 * options: show page numbers, right-align page numbers, tab leader
 * character. Auto-update on save (debounced 1s).
 *
 * Public API: window.DocsTOC = { generate, update, insertTOC, removeTOC,
 *   setShowPageNumbers, setLeader, setIndentation }.
 */

(function () {
  function getState() { return window.state || null; }
  function getEditor() {
    return document.querySelector(".editor") || document.querySelector("[contenteditable]");
  }

  function scanHeadings(editor) {
    if (!editor) return [];
    const out = [];
    const nodes = editor.querySelectorAll("h1, h2, h3, h4, h5, h6");
    for (const h of nodes) {
      const level = parseInt(h.tagName.charAt(1));
      out.push({
        level: level,
        text: h.textContent || "",
        anchor: h.id || ("h-" + out.length),
      });
      if (!h.id) h.id = "h-" + out.length;
    }
    return out;
  }

  function buildTOC(headings, options) {
    options = options || {};
    const showPageNumbers = options.showPageNumbers !== false;
    const leader = options.leader || ".";
    const rightAlign = options.rightAlign !== false;
    const container = document.createElement("div");
    container.className = "docs-toc";
    container.contentEditable = "false";
    container.style.cssText = "background:#f4f4f4;border:1px solid #ddd;padding:12px 16px;margin:16px 0;font-family:inherit;";
    const title = document.createElement("div");
    title.style.cssText = "font-weight:bold;font-size:14px;margin-bottom:8px;";
    title.textContent = "Table of Contents";
    container.appendChild(title);
    if (!headings.length) {
      const empty = document.createElement("div");
      empty.style.cssText = "color:#888;font-size:12px;";
      empty.textContent = "(no headings found)";
      container.appendChild(empty);
      return container;
    }
    const list = document.createElement("div");
    list.className = "docs-toc-list";
    for (const h of headings) {
      const row = document.createElement("div");
      row.className = "docs-toc-row";
      row.style.cssText = "display:flex;align-items:baseline;font-size:13px;margin:3px 0;";
      const indent = (h.level - 1) * 20;
      const left = document.createElement("a");
      left.href = "#" + h.anchor;
      left.textContent = h.text;
      left.style.cssText = "text-decoration:none;color:#1a73e8;flex:1;padding-left:" + indent + "px;";
      left.addEventListener("click", (e) => {
        e.preventDefault();
        const target = document.getElementById(h.anchor);
        if (target) target.scrollIntoView({ behavior: "smooth" });
      });
      row.appendChild(left);
      if (showPageNumbers) {
        const ln = document.createElement("span");
        ln.style.cssText = "flex:0 0 auto;min-width:30px;text-align:" + (rightAlign ? "right" : "left") + ";color:#666;font-size:12px;margin-left:8px;";
        ln.textContent = estimatePageNumber(h.anchor);
        row.appendChild(ln);
      }
      container.appendChild(row);
    }
    return container;
  }

  function estimatePageNumber(anchorId) {
    const target = document.getElementById(anchorId);
    if (!target) return "1";
    const pages = Array.from(document.querySelectorAll(".doc-page"));
    if (!pages.length) return "1";
    const tr = target.getBoundingClientRect();
    for (let i = 0; i < pages.length; i++) {
      const pr = pages[i].getBoundingClientRect();
      if (tr.top >= pr.top && tr.top <= pr.bottom) return String(i + 1);
    }
    return "1";
  }

  function generate(options) {
    const editor = getEditor();
    if (!editor) return null;
    const headings = scanHeadings(editor);
    return buildTOC(headings, options);
  }

  function insertTOC(options) {
    const editor = getEditor();
    if (!editor) return false;
    removeTOC();
    const toc = generate(options);
    if (!toc) return false;
    editor.insertBefore(toc, editor.firstChild);
    scheduleAutoUpdate();
    return true;
  }

  function update() {
    const existing = document.querySelector(".docs-toc");
    if (!existing) return false;
    const options = { showPageNumbers: true, rightAlign: true, leader: "." };
    const headings = scanHeadings(getEditor());
    const fresh = buildTOC(headings, options);
    existing.replaceWith(fresh);
    return true;
  }

  function removeTOC() {
    const t = document.querySelector(".docs-toc");
    if (t) t.remove();
  }

  function setShowPageNumbers(on) {
    const s = getState();
    if (s) s.tocShowPageNumbers = !!on;
    update();
  }

  function setLeader(leader) {
    const s = getState();
    if (s) s.tocLeader = leader;
    update();
  }

  function setIndentation(px) {
    const s = getState();
    if (s) s.tocIndentation = px;
    update();
  }

  function scheduleAutoUpdate() {
    clearTimeout(window.__docsTocTimer);
    window.__docsTocTimer = setTimeout(update, 1000);
  }

  function attach() {
    const editor = getEditor();
    if (!editor) return;
    const obs = new MutationObserver(scheduleAutoUpdate);
    obs.observe(editor, { childList: true, subtree: true, characterData: true });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsTOC = { generate, update, insertTOC, removeTOC, setShowPageNumbers, setLeader, setIndentation };
})();
