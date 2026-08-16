"use strict";
/* docs state — constants, helpers, delta edit functions */

var SIDEBAR_TAB_KEY = "docs_sidebar_tab";
var SAVE_DEBOUNCE_MS = 1500;
var TITLE_BG = "#0f172a";
var TITLE_COLOR = "#f8fafc";
var TITLE_BORDER = "#334155";
var SAVE_OK_COLOR = "#94a3b8";
var SAVE_ERR_COLOR = "#f87171";

function getCaretCharacterOffsetWithin(element) {
  var caretOffset = 0;
  var doc = element.ownerDocument || element.document;
  var win = doc.defaultView || doc.parentWindow;
  if (win && win.getSelection) {
    var sel = win.getSelection();
    if (sel.rangeCount > 0) {
      var range = sel.getRangeAt(0);
      var preCaretRange = range.cloneRange();
      preCaretRange.selectNodeContents(element);
      preCaretRange.setEnd(range.startContainer, range.startOffset);
      caretOffset = preCaretRange.toString().length;
    }
  }
  return caretOffset;
}

function getPlainTextLength(root) {
  return (root.textContent || "").length;
}

// Global [start,end) character offsets of the current selection within element
// (mirrors getCaretCharacterOffsetWithin, which returns only the collapsed
// start). Used to broadcast selection ranges to remote collaborators.
function getSelectionCharacterOffsets(element) {
  var doc = element.ownerDocument || document;
  var win = doc.defaultView || doc.parentWindow;
  var result = { start: 0, end: 0 };
  if (win && win.getSelection) {
    var sel = win.getSelection();
    if (sel.rangeCount > 0) {
      var range = sel.getRangeAt(0);
      var pre = range.cloneRange();
      pre.selectNodeContents(element);
      pre.setEnd(range.startContainer, range.startOffset);
      result.start = pre.toString().length;
      pre.setEnd(range.endContainer, range.endOffset);
      result.end = pre.toString().length;
    }
  }
  return result;
}

function applyDeltaEdit(article, position, content, removeLength) {
  var doc = article.ownerDocument || article.document;
  if (!doc) return;
  var remove = Math.max(0, removeLength || 0);
  var insert = content == null ? "" : String(content);
  var walker = doc.createTreeWalker(article, NodeFilter.SHOW_TEXT, null);
  var range = doc.createRange();
  var remaining = position;
  var startNode = null;
  var startOffset = 0;
  while (walker.nextNode()) {
    var node = walker.currentNode;
    var len = node.textContent.length;
    if (remaining <= len) {
      startNode = node;
      startOffset = remaining;
      break;
    }
    remaining -= len;
  }
  if (!startNode) {
    startNode = article;
    startOffset = article.childNodes.length;
  }
  range.setStart(startNode, startOffset);
  var endNode = startNode;
  var endOffset = startOffset;
  var toRemove = remove;
  if (toRemove > 0) {
    var walker2 = doc.createTreeWalker(article, NodeFilter.SHOW_TEXT, null);
    var collected = 0;
    var lastNode = null;
    var lastOffset = 0;
    while (walker2.nextNode()) {
      var node2 = walker2.currentNode;
      var len2 = node2.textContent.length;
      if (collected + len2 > position) {
        lastNode = node2;
        lastOffset = position - collected;
        collected = position + (len2 - lastOffset);
        break;
      }
      collected += len2;
      lastNode = node2;
      lastOffset = len2;
    }
    var remainingRemove = toRemove;
    var cursor = lastNode;
    var cursorOffset = lastOffset || 0;
    while (remainingRemove > 0 && cursor) {
      var len3 = (cursor.textContent || "").length;
      var take = Math.min(len3 - cursorOffset, remainingRemove);
      if (take < len3 - cursorOffset) {
        cursor = cursor.splitText(cursorOffset + take);
        cursorOffset = 0;
      } else {
        var next = walker2.nextNode();
        cursor.deleteData(cursorOffset, len3 - cursorOffset);
        cursor = next;
        cursorOffset = 0;
      }
      remainingRemove -= take;
    }
    endNode = cursor || lastNode;
    endOffset = cursorOffset;
  }
  range.setEnd(endNode, endOffset);
  var frag = doc.createDocumentFragment();
  if (insert.length > 0) {
    frag.appendChild(doc.createTextNode(insert));
  }
  range.deleteContents();
  if (insert.length > 0) range.insertNode(frag);
}

function setCaretPosition(element, offset) {
  var doc = element.ownerDocument || element.document;
  var win = doc.defaultView || doc.parentWindow;
  if (win && win.getSelection) {
    var sel = win.getSelection();
    var range = doc.createRange();
    var currentOffset = 0;
    var nodeToSelect = null;
    var offsetWithinNode = 0;
    function traverse(node) {
      if (nodeToSelect) return;
      if (node.nodeType === 3) {
        var length = node.textContent.length;
        if (currentOffset + length >= offset) {
          nodeToSelect = node;
          offsetWithinNode = offset - currentOffset;
        } else {
          currentOffset += length;
        }
      } else {
        for (var i = 0; i < node.childNodes.length; i++) {
          traverse(node.childNodes[i]);
        }
      }
    }
    traverse(element);
    if (nodeToSelect) {
      range.setStart(nodeToSelect, offsetWithinNode);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    }
  }
}

function $(s, r) { return (r || document).querySelector(s); }
function $$(s, r) { return Array.from((r || document).querySelectorAll(s)); }
