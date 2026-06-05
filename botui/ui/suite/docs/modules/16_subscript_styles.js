"use strict";

/**
 * Module 16: Subscript, superscript, multi-level lists, multi-column,
 * and styles panel for Word Processor. Provides a self-contained
 * "Advanced formatting" toolbar group with all the inline-format
 * primitives that were missing in the original toolbar.
 *
 *   - applySubscript() / applySuperscript(): wraps the current
 *     selection in <sub>/<sup>.
 *   - increaseListLevel() / decreaseListLevel(): Tab/Shift+Tab in
 *     lists to nest/unnest. Stores list level and numbering type on
 *     the <ol>/<ul> element.
 *   - setColumnCount(n): CSS column-count on a section.
 *   - applyStyle(name): applies a paragraph style (Heading 1-6,
 *     Title, Subtitle, Quote, Body Text) by adjusting font size,
 *     weight, color, and line-height.
 *
 * Public API: window.DocsAdvancedFormat = { applySubscript,
 *   applySuperscript, increaseListLevel, decreaseListLevel,
 *   setColumnCount, applyStyle, listAvailableStyles }.
 */

(function () {
  const STYLES = {
    Title: { fontSize: 28, fontWeight: "bold", color: "#1a1a1a" },
    Subtitle: { fontSize: 18, fontWeight: "normal", color: "#555" },
    "Heading 1": { fontSize: 24, fontWeight: "bold", color: "#1a1a1a" },
    "Heading 2": { fontSize: 20, fontWeight: "bold", color: "#1a1a1a" },
    "Heading 3": { fontSize: 16, fontWeight: "bold", color: "#1a1a1a" },
    "Heading 4": { fontSize: 14, fontWeight: "bold", color: "#1a1a1a" },
    "Heading 5": { fontSize: 13, fontWeight: "bold", color: "#1a1a1a" },
    "Heading 6": { fontSize: 12, fontWeight: "bold", color: "#555" },
    Quote: { fontSize: 14, fontStyle: "italic", color: "#444", borderLeft: "4px solid #888", paddingLeft: "12px" },
    "Body Text": { fontSize: 14, fontWeight: "normal", color: "#1a1a1a" },
  };

  function getCurrentBlock() {
    const sel = window.getSelection();
    if (!sel || !sel.anchorNode) return null;
    let n = sel.anchorNode;
    while (n && n !== document.body) {
      if (n.nodeType === 1 && /^(P|DIV|LI|H[1-6]|BLOCKQUOTE)$/.test(n.tagName)) return n;
      n = n.parentNode;
    }
    return null;
  }

  function wrapSelection(tagName) {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return;
    const range = sel.getRangeAt(0);
    const wrapper = document.createElement(tagName);
    try {
      wrapper.appendChild(range.extractContents());
      range.insertNode(wrapper);
    } catch (_e) { /* silent */ }
  }

  function applySubscript() { wrapSelection("sub"); }
  function applySuperscript() { wrapSelection("sup"); }

  function applyStyle(name) {
    const style = STYLES[name];
    if (!style) return false;
    const block = getCurrentBlock();
    if (!block) return false;
    Object.assign(block.style, style);
    if (name === "Quote" && !block.querySelector(":scope > blockquote")) {
      block.style.borderLeft = "4px solid #888";
      block.style.paddingLeft = "12px";
    }
    if (name.startsWith("Heading")) {
      const level = parseInt(name.split(" ")[1]);
      const newTag = "H" + level;
      if (block.tagName !== newTag) {
        const replacement = document.createElement(newTag);
        while (block.firstChild) replacement.appendChild(block.firstChild);
        Object.assign(replacement.style, style);
        block.parentNode.replaceChild(replacement, block);
      }
    }
    return true;
  }

  function listAvailableStyles() { return Object.keys(STYLES); }

  function ensureListItem() {
    const sel = window.getSelection();
    if (!sel || !sel.anchorNode) return null;
    let n = sel.anchorNode;
    while (n && n !== document.body) {
      if (n.nodeType === 1 && n.tagName === "LI") return n;
      n = n.parentNode;
    }
    return null;
  }

  function findParentList(item) {
    let n = item;
    while (n && n !== document.body) {
      if (n.nodeType === 1 && /^(UL|OL)$/.test(n.tagName)) return n;
      n = n.parentNode;
    }
    return null;
  }

  function indentLevel(item, delta) {
    if (!item) return false;
    const list = findParentList(item);
    if (!list) return false;
    const currentLevel = parseInt(item.dataset.level || "0");
    const newLevel = Math.max(0, currentLevel + delta);
    if (newLevel === currentLevel) return false;
    if (delta > 0) {
      const prevLi = item.previousElementSibling;
      if (prevLi) {
        let subList = prevLi.querySelector(":scope > ul, :scope > ol");
        if (!subList) {
          subList = document.createElement(list.tagName);
          subList.dataset.level = newLevel;
          prevLi.appendChild(subList);
        }
        subList.appendChild(item);
        item.dataset.level = newLevel;
      }
    } else {
      const parentList = list.parentNode.closest("ul, ol, li");
      if (parentList && parentList.tagName === "LI") {
        const grand = parentList.parentNode;
        if (grand) {
          grand.insertBefore(item, parentList.nextSibling);
          item.dataset.level = Math.max(0, (parseInt(item.dataset.level) || 0) - 1);
        }
      }
    }
    return true;
  }

  function increaseListLevel() { return indentLevel(ensureListItem(), 1); }
  function decreaseListLevel() { return indentLevel(ensureListItem(), -1); }

  function setColumnCount(n) {
    const block = getCurrentBlock();
    if (!block) return false;
    block.style.columnCount = String(n);
    block.style.columnGap = "24px";
    if (n > 1) block.style.columnRule = "1px solid #ddd";
    return true;
  }

  function setLineSpacing(value) {
    const block = getCurrentBlock();
    if (!block) return false;
    block.style.lineHeight = String(value);
    return true;
  }

  function setParagraphSpacing(before, after) {
    const block = getCurrentBlock();
    if (!block) return false;
    if (before != null) block.style.marginTop = before + "px";
    if (after != null) block.style.marginBottom = after + "px";
    return true;
  }

  function attach() {
    document.addEventListener("keydown", function (e) {
      if (!e.target) return;
      const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
      if (!editor || !editor.contains(e.target)) return;
      if (e.key === "Tab") {
        const li = ensureListItem();
        if (li) {
          e.preventDefault();
          if (e.shiftKey) decreaseListLevel();
          else increaseListLevel();
        }
      }
      if ((e.ctrlKey || e.metaKey) && e.shiftKey) {
        if (e.key.toLowerCase() === "b") { e.preventDefault(); applySubscript(); }
        if (e.key.toLowerCase() === "p") { e.preventDefault(); applySuperscript(); }
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsAdvancedFormat = {
    applySubscript, applySuperscript,
    increaseListLevel, decreaseListLevel,
    setColumnCount, setLineSpacing, setParagraphSpacing,
    applyStyle, listAvailableStyles,
  };
})();
