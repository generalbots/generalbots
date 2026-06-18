"use strict";
/* docs editor — attach handlers, page count, article access */

function getActiveArticle() {
  return document.querySelector("article[contenteditable]");
}

function updatePageCount() {
  var el = document.getElementById("pageCount");
  if (!el) return;
  var article = getActiveArticle();
  if (!article) { el.textContent = "1 page"; return; }
  var h = article.scrollHeight;
  var pageH = 1056;
  var count = Math.max(1, Math.ceil(h / pageH));
  el.textContent = count + " page" + (count !== 1 ? "s" : "");
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
      var sel = window.getSelection();
      if (sel && sel.rangeCount) {
        var range = sel.getRangeAt(0);
        var start = range.startOffset;
        var end = range.endOffset;
        if (start !== end) window.GBCollab.sendSelection(start, end);
      }
    }
  });
  article.addEventListener("click", function () {
    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      var sel = window.getSelection();
      if (sel && sel.rangeCount) {
        window.GBCollab.sendCursor(sel.getRangeAt(0).startOffset);
      }
    }
  });
  updatePageCount();
}
