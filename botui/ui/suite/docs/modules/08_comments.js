"use strict";
/* Docs advanced module: 08_comments — range-anchored threaded comments.
 *
 * Reaches sheet parity: selecting text shows a floating "💬 Comment" chip that
 * opens a comment anchored to the selected character range, and the toolbar
 * Comments button aggregates every range comment into the document-level panel
 * via include_children.
 *
 * Range comments are addressed as resourceType "docs:range" with resourceId
 * "{docId}:{start}-{end}" (global character offsets), so the shared
 * /api/collab/* backend groups them under the document (resourceType "docs").
 */
(function () {
  var chip = null;
  var pending = null; // { start, end } captured when the chip was shown
  var debounceTimer = null;

  function article() {
    return document.querySelector("article[contenteditable]");
  }

  function docId() {
    var a = article();
    if (a && a.dataset && a.dataset.docId) return a.dataset.docId;
    var t = document.getElementById("docTitle");
    if (t && t.value) return t.value;
    return "current";
  }

  function hideChip() {
    if (chip) { chip.remove(); chip = null; }
    pending = null;
  }

  function showChip(rect, start, end) {
    hideChip();
    pending = { start: start, end: end };
    chip = document.createElement("button");
    chip.type = "button";
    chip.textContent = "\uD83D\uDCAC Comment";
    chip.className = "docs-comment-chip";
    chip.style.cssText =
      "position:fixed;z-index:70;background:#3b82f6;color:#fff;border:none;border-radius:999px;" +
      "padding:6px 12px;font-size:12px;font-weight:600;cursor:pointer;box-shadow:0 4px 12px rgba(0,0,0,.35);";
    chip.style.left = rect.left + "px";
    chip.style.top = Math.max(8, rect.bottom + 6) + "px";
    chip.addEventListener("mousedown", function (e) { e.preventDefault(); });
    chip.addEventListener("click", function () {
      var sel = pending || { start: 0, end: 0 };
      hideChip();
      openRangeComments(sel.start, sel.end);
    });
    document.body.appendChild(chip);
  }

  function openRangeComments(start, end) {
    if (!window.GBCollabComments) return;
    var s = Math.min(start, end);
    var e = Math.max(start, end);
    window.GBCollabComments.open({
      resourceType: "docs:range",
      resourceId: docId() + ":" + s + "-" + e,
      title: "Comment on selection (" + s + "\u2013" + e + ")",
    });
  }

  function refresh() {
    var a = article();
    if (!a) { hideChip(); return; }
    var sel = window.getSelection();
    if (!sel || sel.rangeCount === 0 || sel.isCollapsed) { hideChip(); return; }
    var range = sel.getRangeAt(0);
    if (!a.contains(range.commonAncestorContainer)) { hideChip(); return; }
    var offs = getSelectionCharacterOffsets(a);
    if (offs.start === offs.end) { hideChip(); return; }
    var rects = range.getClientRects();
    if (!rects || !rects.length) { hideChip(); return; }
    showChip(rects[rects.length - 1], offs.start, offs.end);
  }

  function schedule() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(refresh, 120);
  }

  document.addEventListener("selectionchange", schedule);
  document.addEventListener("scroll", function () { if (chip) refresh(); }, true);
})();
