"use strict";

/**
 * Module 18: Selection-based inline formatting for Docs (P0 critical).
 * Replaces the deprecated `document.execCommand()` for bold, italic,
 * underline, strike, subscript, superscript, font, size, color, and
 * background. Operates on the current Selection's Range by walking
 * text nodes, splitting them at the range boundaries, and applying
 * styles via wrapper <span> elements (or by toggling attributes).
 *
 * Public API: window.DocsSelection = {
 *   exec, getRange, isCollapsed, getActiveFormats, queryCommandState
 * }.
 */

(function () {
  function getEditor() { return document.querySelector(".doc-editor, .docs-editor, [contenteditable='true']"); }

  function getRange() {
    const sel = window.getSelection ? window.getSelection() : null;
    if (!sel || sel.rangeCount === 0) return null;
    return sel.getRangeAt(0);
  }

  function isCollapsed() {
    const sel = window.getSelection ? window.getSelection() : null;
    return !sel || sel.isCollapsed;
  }

  function ancestorNode(node, predicate) {
    while (node && node !== document.body) {
      if (predicate(node)) return node;
      node = node.parentNode;
    }
    return null;
  }

  function splitTextAt(textNode, offset) {
    if (offset <= 0 || offset >= textNode.length) return textNode;
    const rest = textNode.splitText(offset);
    return rest.previousSibling;
  }

  function rangeToTextNodes(range) {
    const out = [];
    if (!range) return out;
    const start = range.startContainer;
    const end = range.endContainer;
    if (start === end && start.nodeType === 3) {
      out.push({ node: start, start: range.startOffset, end: range.endOffset });
      return out;
    }
    const walker = document.createTreeWalker(
      range.commonAncestorContainer,
      NodeFilter.SHOW_TEXT,
      { acceptNode: function (n) { return range.intersectsNode(n) ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT; } }
    );
    let n = walker.nextNode();
    let started = false, ended = false;
    while (n) {
      if (!started && n === start) { started = true; }
      if (started) {
        const startOff = (n === start) ? range.startOffset : 0;
        const endOff = (n === end) ? range.endOffset : n.length;
        out.push({ node: n, start: startOff, end: endOff });
      }
      if (n === end) { ended = true; break; }
      n = walker.nextNode();
    }
    if (!started && !ended && range.commonAncestorContainer.nodeType === 3) {
      out.push({ node: range.commonAncestorContainer, start: range.startOffset, end: range.endOffset });
    }
    return out;
  }

  function wrapRangeInSpan(range, attrs) {
    const pieces = rangeToTextNodes(range);
    for (const p of pieces) {
      if (p.start === p.end) continue;
      const before = p.start > 0 ? splitTextAt(p.node, p.start) : p.node;
      const after = p.end < before.length ? before.splitText(p.end - p.start) : null;
      const target = after ? before : (p.start > 0 ? before : p.node);
      if (target.parentNode && target.parentNode.classList && target.parentNode.classList.contains("doc-format")) {
        for (const k in attrs) target.parentNode.setAttribute(k, attrs[k]);
        continue;
      }
      const span = document.createElement("span");
      for (const k in attrs) span.setAttribute(k, attrs[k]);
      span.className = "doc-format";
      target.parentNode.insertBefore(span, target);
      span.appendChild(target);
    }
  }

  function unwrapRange(selector) {
    const sel = window.getSelection ? window.getSelection() : null;
    if (!sel) return;
    const range = sel.getRangeAt(0);
    const elements = new Set();
    const walker = document.createTreeWalker(range.commonAncestorContainer, NodeFilter.SHOW_ELEMENT, null);
    let n = walker.nextNode();
    while (n) {
      if (n.matches && n.matches(selector)) elements.add(n);
      n = walker.nextNode();
    }
    for (const el of elements) {
      if (!range.intersectsNode(el)) continue;
      const parent = el.parentNode;
      if (!parent) continue;
      while (el.firstChild) parent.insertBefore(el.firstChild, el);
      parent.removeChild(el);
      parent.normalize();
    }
  }

  function isInside(selector) {
    const sel = window.getSelection ? window.getSelection() : null;
    if (!sel || sel.rangeCount === 0) return false;
    const range = sel.getRangeAt(0);
    return ancestorNode(range.startContainer, function (n) { return n.nodeType === 1 && n.matches && n.matches(selector); }) !== null;
  }

  function collapseRange() {
    const sel = window.getSelection ? window.getSelection() : null;
    if (!sel || sel.rangeCount === 0) return;
    const range = sel.getRangeAt(0);
    if (!range.collapsed) range.collapse(true);
  }

  const FORMATS = {
    bold: { tag: "b", attr: null },
    italic: { tag: "i", attr: null },
    underline: { tag: "u", attr: null },
    strike: { tag: "s", attr: null },
    subscript: { tag: "sub", attr: null },
    superscript: { tag: "sup", attr: null },
  };

  function exec(command, value) {
    const editor = getEditor();
    if (!editor) return false;
    if (command === "bold" || command === "italic" || command === "underline" || command === "strike" || command === "subscript" || command === "superscript") {
      const cfg = FORMATS[command];
      const sel = isInside(cfg.tag);
      if (isCollapsed()) {
        collapseRange();
        return true;
      }
      const range = getRange();
      if (!range) return false;
      if (sel) unwrapRange(cfg.tag);
      else wrapRangeInSpan(range, {});
      return true;
    }
    if (command === "formatBlock") {
      const tag = (value || "P").toUpperCase();
      const sel = window.getSelection ? window.getSelection() : null;
      if (!sel) return false;
      const range = sel.getRangeAt(0);
      let block = ancestorNode(range.startContainer, function (n) { return /^(P|H1|H2|H3|H4|H5|H6|BLOCKQUOTE|PRE)$/i.test(n.nodeName); });
      if (!block) {
        const p = document.createElement(tag);
        range.surroundContents(p);
      } else {
        const newBlock = document.createElement(tag);
        while (block.firstChild) newBlock.appendChild(block.firstChild);
        block.parentNode.replaceChild(newBlock, block);
      }
      return true;
    }
    if (command === "fontSize") {
      const size = String(value);
      if (isCollapsed()) return true;
      const range = getRange();
      if (!range) return false;
      if (isInside("[data-size='" + size + "']")) {
        unwrapRange("[data-size]");
        return true;
      }
      unwrapRange("[data-size]");
      wrapRangeInSpan(range, { "data-size": size, style: "font-size:" + size + "px;" });
      return true;
    }
    if (command === "fontName") {
      const family = String(value);
      if (isCollapsed()) return true;
      const range = getRange();
      if (!range) return false;
      if (isInside("[data-font='" + family + "']")) unwrapRange("[data-font]");
      else { unwrapRange("[data-font]"); wrapRangeInSpan(range, { "data-font": family, style: "font-family:'" + family + "';" }); }
      return true;
    }
    if (command === "foreColor") {
      const color = String(value);
      if (isCollapsed()) return true;
      const range = getRange();
      if (!range) return false;
      unwrapRange("[data-color]");
      wrapRangeInSpan(range, { "data-color": color, style: "color:" + color + ";" });
      return true;
    }
    if (command === "hiliteColor" || command === "backColor") {
      const color = String(value);
      if (isCollapsed()) return true;
      const range = getRange();
      if (!range) return false;
      unwrapRange("[data-bg]");
      wrapRangeInSpan(range, { "data-bg": color, style: "background-color:" + color + ";" });
      return true;
    }
    if (command === "createLink") {
      const href = String(value);
      if (!href) return false;
      const range = getRange();
      if (!range) return false;
      if (range.collapsed) {
        const a = document.createElement("a");
        a.href = href;
        a.textContent = href;
        a.rel = "noopener noreferrer";
        range.insertNode(a);
      } else {
        const a = document.createElement("a");
        a.href = href;
        a.rel = "noopener noreferrer";
        try { range.surroundContents(a); }
        catch (_e) {
          a.appendChild(range.extractContents());
          range.insertNode(a);
        }
      }
      return true;
    }
    if (command === "unlink") {
      unwrapRange("a[href]");
      return true;
    }
    if (command === "removeFormat") {
      unwrapRange("b, i, u, s, sub, sup, [data-size], [data-font], [data-color], [data-bg], a[href]");
      return true;
    }
    if (command === "insertHTML") {
      const range = getRange();
      if (!range) return false;
      range.deleteContents();
      const safe = sanitizeHtmlFragment(String(value));
      const tmp = document.createElement("div");
      tmp.appendChild(safe);
      const frag = document.createDocumentFragment();
      while (tmp.firstChild) frag.appendChild(tmp.firstChild);
      range.insertNode(frag);
      return true;
    }
    if (command === "insertText") {
      const range = getRange();
      if (!range) return false;
      range.deleteContents();
      const t = document.createTextNode(String(value));
      range.insertNode(t);
      range.setStartAfter(t);
      range.collapse(true);
      const sel = window.getSelection ? window.getSelection() : null;
      if (sel) { sel.removeAllRanges(); sel.addRange(range); }
      return true;
    }
    if (command === "justifyLeft" || command === "justifyCenter" || command === "justifyRight" || command === "justifyFull") {
      const align = { justifyLeft: "left", justifyCenter: "center", justifyRight: "right", justifyFull: "justify" }[command];
      const sel = window.getSelection ? window.getSelection() : null;
      if (!sel) return false;
      const range = sel.getRangeAt(0);
      let block = ancestorNode(range.startContainer, function (n) { return /^(P|H1|H2|H3|H4|H5|H6|DIV|LI|BLOCKQUOTE)$/i.test(n.nodeName); });
      if (!block) block = editor;
      block.style.textAlign = align;
      block.setAttribute("data-align", align);
      return true;
    }
    if (command === "indent") {
      const range = getRange();
      if (!range) return false;
      const block = ancestorNode(range.startContainer, function (n) { return /^(P|H1|H2|H3|H4|H5|H6|DIV|LI)$/i.test(n.nodeName); });
      if (!block) return false;
      const cur = parseInt(block.getAttribute("data-indent") || "0", 10);
      block.setAttribute("data-indent", String(cur + 1));
      block.style.paddingLeft = ((cur + 1) * 32) + "px";
      return true;
    }
    if (command === "outdent") {
      const range = getRange();
      if (!range) return false;
      const block = ancestorNode(range.startContainer, function (n) { return /^(P|H1|H2|H3|H4|H5|H6|DIV|LI)$/i.test(n.nodeName); });
      if (!block) return false;
      const cur = Math.max(0, parseInt(block.getAttribute("data-indent") || "0", 10) - 1);
      block.setAttribute("data-indent", String(cur));
      block.style.paddingLeft = (cur * 32) + "px";
      return true;
    }
    return false;
  }

  function queryCommandState(command) {
    const cfg = FORMATS[command];
    if (cfg) return isInside(cfg.tag);
    if (command === "fontSize") {
      const el = document.querySelector("[data-size]");
      return el && el.getAttribute("data-size");
    }
    return false;
  }

  function getActiveFormats() {
    const out = {};
    for (const cmd in FORMATS) out[cmd] = queryCommandState(cmd);
    return out;
  }

  function attach() {
    document.addEventListener("keydown", function (e) {
      if (!(e.ctrlKey || e.metaKey)) return;
      const k = e.key.toLowerCase();
      if (k === "b") { e.preventDefault(); exec("bold"); }
      else if (k === "i") { e.preventDefault(); exec("italic"); }
      else if (k === "u") { e.preventDefault(); exec("underline"); }
      else if (k === "k") { e.preventDefault(); const url = window.prompt("URL do link:"); if (url) exec("createLink", url); }
    });
    const tb = document.querySelectorAll("[data-format-cmd]");
    tb.forEach(function (b) {
      b.addEventListener("mousedown", function (e) { e.preventDefault(); });
      b.addEventListener("click", function (e) {
        e.preventDefault();
        const cmd = b.getAttribute("data-format-cmd");
        const val = b.getAttribute("data-format-value") || null;
        exec(cmd, val);
      });
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  function sanitizeHtmlFragment(html) {
    const tmp = document.createElement("div");
    tmp.innerHTML = html;
    const blocked = new Set(["SCRIPT", "STYLE", "IFRAME", "OBJECT", "EMBED", "LINK", "META", "BASE", "FORM", "INPUT", "TEXTAREA", "SELECT", "BUTTON"]);
    const walk = (node) => {
      const children = Array.from(node.childNodes);
      for (const child of children) {
        if (child.nodeType === 1) {
          if (blocked.has(child.tagName)) {
            child.remove();
            continue;
          }
          for (const attr of Array.from(child.attributes)) {
            const n = attr.name.toLowerCase();
            if (n.startsWith("on") || n === "srcdoc" || n === "xlink:href" || n === "formaction") {
              child.removeAttribute(attr.name);
              continue;
            }
            if ((n === "href" || n === "src") && /^\s*javascript:/i.test(attr.value)) {
              child.removeAttribute(attr.name);
            }
          }
          if (child.tagName === "A" && child.getAttribute("href") && !/^(https?:|mailto:|tel:|#|\/)/i.test(child.getAttribute("href"))) {
            child.removeAttribute("href");
          }
          walk(child);
        } else if (child.nodeType === 8) {
          child.remove();
        }
      }
    };
    walk(tmp);
    return tmp;
  }

  window.DocsSelection = { exec, getRange, isCollapsed, getActiveFormats, queryCommandState, wrapRangeInSpan, unwrapRange, sanitizeHtmlFragment };
})();
