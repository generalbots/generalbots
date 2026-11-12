"use strict";
/* docs editor — attach handlers, page count, article access */

function getActiveArticle() {
  return document.querySelector("article[contenteditable]");
}

/* #2184 — Live generation draws the document through UI hints: the plan opens
   Docs with no file and then fills one semantic target, the document body:
     {"op":"fill","field":"document body","value":"# Title\n\nparagraph"}
   The value is markdown-lite (headings, bullets, numbered items, blank-line
   paragraphs, **bold**), rendered here into blocks so a plan produces a real
   document instead of one wall of text. */
function docsEscapeHtml(s) {
  return String(s == null ? "" : s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function docsRenderInline(text) {
  var out = docsEscapeHtml(text);
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/(^|[^*])\*([^*]+)\*/g, "$1<em>$2</em>");
  out = out.replace(/`([^`]+)`/g, "<code>$1</code>");
  return out;
}

function docsRenderBlocks(text) {
  var html = [];
  var listTag = "";
  function closeList() {
    if (listTag) {
      html.push("</" + listTag + ">");
      listTag = "";
    }
  }
  String(text == null ? "" : text)
    .split(/\r?\n/)
    .forEach(function (line) {
      var t = line.replace(/\s+$/, "");
      var heading = /^(#{1,6})\s+(.*)$/.exec(t);
      if (heading) {
        closeList();
        var level = heading[1].length;
        html.push("<h" + level + ">" + docsRenderInline(heading[2]) + "</h" + level + ">");
        return;
      }
      var bullet = /^\s*[-*]\s+(.*)$/.exec(t);
      if (bullet) {
        if (listTag !== "ul") { closeList(); listTag = "ul"; html.push("<ul>"); }
        html.push("<li>" + docsRenderInline(bullet[1]) + "</li>");
        return;
      }
      var numbered = /^\s*\d+[.)]\s+(.*)$/.exec(t);
      if (numbered) {
        if (listTag !== "ol") { closeList(); listTag = "ol"; html.push("<ol>"); }
        html.push("<li>" + docsRenderInline(numbered[1]) + "</li>");
        return;
      }
      if (!t.trim()) {
        closeList();
        return;
      }
      closeList();
      html.push("<p>" + docsRenderInline(t) + "</p>");
    });
  closeList();
  return html.join("");
}

/* Creates the editable body of a blank Docs window and gives it the semantic
   identity the UI-hint executor looks up (`data-gb-field="body"`), plus a
   structured `__gbFill` hook so a fill step renders blocks instead of
   replacing the text with one blob. Idempotent: safe on every window open. */
function ensureDocArticle() {
  var host = document.getElementById("docs-content");
  if (!host) return null;
  var article = host.querySelector("article[contenteditable]");
  if (!article) {
    article = document.createElement("article");
    article.className = "docs-doc-view";
    article.id = "doc-content";
    article.setAttribute("contenteditable", "true");
    host.appendChild(article);
    if (typeof attachEditorHandlers === "function") attachEditorHandlers(host);
  }
  article.setAttribute("data-gb-field", "body");
  if (!article.getAttribute("aria-label")) article.setAttribute("aria-label", "document body");
  if (!article.__gbFill) {
    article.__gbFill = function (value) {
      article.innerHTML = docsRenderBlocks(value);
      // A drafted document carries its own title: promote the leading heading
      // instead of leaving the window on "Untitled Document".
      var titleEl = document.getElementById("docTitle");
      var heading = /^\s*#\s+(.+)$/m.exec(String(value || ""));
      if (titleEl && heading && /^untitled/i.test(titleEl.value || "")) {
        titleEl.value = heading[1].trim();
      }
      article.dispatchEvent(new Event("input", { bubbles: true }));
      if (typeof docsAfterContentLoad === "function") docsAfterContentLoad(article);
      if (typeof updatePageCount === "function") updatePageCount();
    };
  }
  return article;
}

window.GBDocsEnsureArticle = ensureDocArticle;
window.GBDocsRenderBlocks = docsRenderBlocks;

function updatePageCount() {
  var el = document.getElementById("pageCount");
  if (!el) return;
  var article = getActiveArticle();
  if (!article) {
    el.textContent = docsT("docs-page-count", "1 page", { count: 1 });
    return;
  }
  var h = article.scrollHeight;
  var pageH = parseInt(article.dataset.pageH, 10) || 1123;
  var count = Math.max(1, Math.ceil(h / pageH));
  el.textContent = docsT(
    "docs-page-count",
    count + " page" + (count !== 1 ? "s" : ""),
    { count: count },
  );
}

function attachEditorHandlers(host) {
  if (!host) return;
  var article = host.querySelector("article[contenteditable]");
  if (!article) return;
  var lastTextLength = getPlainTextLength(article);
  var pendingEdit = null;
  article.addEventListener("beforeinput", function (e) {
    pendingEdit = { inputType: e.inputType, data: e.data, targetRanges: e.getTargetRanges ? e.getTargetRanges() : [] };
  });
  article.addEventListener("input", function (e) {
    scheduleSave(article.dataset.docId, article.innerHTML);
    updatePageCount();
    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      var sel = window.getSelection();
      var pos = sel && sel.rangeCount ? getCaretCharacterOffsetWithin(article) : 0;
      var newLength = getPlainTextLength(article);
      var ed = pendingEdit || {};
      var content = null;
      var removeLen = 0;
      if (ed.inputType === "insertText" && ed.data) {
        content = ed.data;
      } else if (ed.inputType === "insertFromPaste" || ed.inputType === "insertFromDrop") {
        content = (article.textContent || "").substring(pos, pos + (newLength - lastTextLength));
      } else if (ed.inputType === "deleteContentBackward") {
        removeLen = lastTextLength - newLength;
      } else if (ed.inputType === "deleteContentForward") {
        removeLen = lastTextLength - newLength;
      } else if (ed.inputType && ed.inputType.indexOf("delete") >= 0) {
        removeLen = lastTextLength - newLength;
      } else if (newLength !== lastTextLength) {
        content = (article.textContent || "").substring(pos, pos + Math.max(0, newLength - lastTextLength));
        removeLen = Math.max(0, lastTextLength - newLength);
      }
      if (content !== null || removeLen > 0) {
        window.GBCollab.debouncedTypingStart(pos);
        window.GBCollab.sendEdit({ position: pos, content: content, length: content ? content.length : 0, removeLength: removeLen });
      }
    }
    lastTextLength = getPlainTextLength(article);
    pendingEdit = null;
  });
  article.addEventListener("blur", function () {
    if (saveTimer) {
      clearTimeout(saveTimer);
      scheduleSave(article.dataset.docId, article.innerHTML);
    }
    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      window.GBCollab.sendTypingStop();
    }
  });
  article.addEventListener("keyup", function () {
    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      var offs = getSelectionCharacterOffsets(article);
      if (offs.start !== offs.end) window.GBCollab.sendSelection(offs.start, offs.end);
    }
  });
  article.addEventListener("click", function () {
    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      window.GBCollab.sendCursor(getCaretCharacterOffsetWithin(article));
    }
  });
  updatePageCount();
}
