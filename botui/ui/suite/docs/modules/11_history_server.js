"use strict";
/* Server-side undo/redo fallback (#1138).
 * The native editing stack (browser) is session-local; this module talks to
 * POST /api/docs/history so undo/redo survive reloads. Keyboard shortcuts
 * hook the editor and fall back to document.execCommand when the server
 * reports nothing to walk (204).
 */
(function () {
  var busy = false;

  function docId() {
    var view = document.getElementById("doc-view");
    return view ? view.getAttribute("data-id") : null;
  }

  function applyRestored(doc) {
    var article = document.querySelector("article[contenteditable]");
    if (!article || !doc || typeof doc.content !== "string") return false;
    article.innerHTML = doc.content;
    article.dispatchEvent(new Event("input", { bubbles: true }));
    return true;
  }

  function walk(action) {
    var id = docId();
    if (!id || busy) return;
    busy = true;
    fetch("/api/docs/history", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify({ id: id, action: action }),
    })
      .then(function (r) {
        if (r.status === 204) return null;
        if (!r.ok) throw new Error("history HTTP " + r.status);
        return r.json();
      })
      .then(function (doc) {
        if (doc && applyRestored(doc)) return;
        // Server has nothing: fall back to the native editor stack.
        document.execCommand(action === "undo" ? "undo" : "redo");
      })
      .catch(function () {})
      .finally(function () { busy = false; });
  }

  function bind() {
    var article = document.querySelector("article[contenteditable]");
    if (!article || article.dataset.gbHistBound === "1") return;
    article.dataset.gbHistBound = "1";
    article.addEventListener("keydown", function (e) {
      var mod = e.ctrlKey || e.metaKey;
      if (!mod || e.key.toLowerCase() !== "z") return;
      e.preventDefault();
      walk(e.shiftKey ? "redo" : "undo");
    });
  }

  var mo = new MutationObserver(bind);
  mo.observe(document.body, { childList: true, subtree: true });
  window.GBDocsServerHistory = { undo: function(){walk("undo");}, redo: function(){walk("redo");} };
})();
