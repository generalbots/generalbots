"use strict";
/* GB Notepad (#1154): multi-file plain-text scratchpad, localStorage-backed. */
window.GBNotepad = window.GBNotepad || {};
(function (app) {
  var IDX = "gb-notepad-index";
  function loadIndex() {
    try { return gbStoreGet(IDX, {}); } catch (e) { return {}; }
  }
  function saveIndex(i) {
    try { localStorage.setItem(IDX, JSON.stringify(i)); return true; }
    catch (e) { console.error("Notepad save failed:", e); return false; }
  }

  app.init = function () {
    var root = document.getElementById("gb-notepad-root") ||
      (document.currentScript ? document.currentScript.closest(".gb-notepad") : null);
    if (!root || root.dataset.npInit === "1") return;
    root.dataset.npInit = "1";

    var sel = root.querySelector("#gb-notepad-files");
    var ta = root.querySelector("#gb-notepad-text");
    var status = root.querySelector("#gb-notepad-status");

    function names() { return Object.keys(loadIndex()).sort(); }
    function current() { return sel.value; }
    function renderList(keep) {
      sel.innerHTML = "";
      names().forEach(function (n) {
        var o = document.createElement("option");
        o.value = n; o.textContent = n;
        sel.appendChild(o);
      });
      if (keep && names().indexOf(keep) !== -1) sel.value = keep;
      // A gb-doc-delta may have filled the editor before this boot pass ran;
      // seeding the stored note here would silently discard it.
      if (ta.dataset.gbDelta !== "1") {
        ta.value = loadIndex()[sel.value] || "";
      } else {
        // Delta content is visible but was never written to the note store.
        persist();
      }
    }
    function t(key, fallback) {
      return window.gbT ? window.gbT(key, fallback) : fallback;
    }
    function mark(dirty) {
      status.textContent = dirty
        ? t("notepad-editing", "Editing…")
        : t("canvas-saved", "Saved");
    }

    function persist() {
      var i = loadIndex();
      if (!current()) return;
      i[current()] = ta.value;
      var ok = saveIndex(i);
      if (!ok) {
        clearTimeout(app._t);
        status.textContent = t("notepad-save-failed", "Not saved — storage full");
        return;
      }
      mark(true);
      clearTimeout(app._t);
      app._t = setTimeout(function () { mark(false); }, 500);
    }

    ta.addEventListener("input", persist);
    ta.addEventListener("change", persist);
    sel.addEventListener("change", function () { ta.value = loadIndex()[current()] || ""; });
    root.querySelector("#gb-notepad-new").addEventListener("click", function () {
      var name = prompt(t("notepad-new-name", "Note name:"), "note-" + new Date().toISOString().slice(0, 10));
      if (!name) return;
      var i = loadIndex();
      if (i[name] === undefined) { i[name] = ""; saveIndex(i); }
      renderList(name); mark(false);
    });
    root.querySelector("#gb-notepad-del").addEventListener("click", function () {
      if (!current()) return;
      if (!confirm(t("notepad-delete-confirm", "Delete this note?") + " " + current())) return;
      var i = loadIndex(); delete i[current()]; saveIndex(i);
      var next = names()[0] || "";
      renderList(next);
      try { localStorage.setItem("gb-notepad-last", next); } catch (e) { console.error("Notepad save failed:", e); }
    });

    if (!names().length) {
      var i = loadIndex(); i["welcome"] = t("notepad-welcome", "Welcome to Notepad.\nEverything is stored locally per user."); saveIndex(i);
    }
    // #2394: a deleted note must not stay the remembered selection.
    var last = localStorage.getItem("gb-notepad-last");
    if (last && names().indexOf(last) === -1) {
      last = names()[0];
      try { localStorage.setItem("gb-notepad-last", last); } catch (e) { console.error("Notepad save failed:", e); }
    }
    renderList(last || names()[0]);
    sel.addEventListener("change", function () { try { localStorage.setItem("gb-notepad-last", sel.value); } catch (e) {} });
  };
})(window.GBNotepad);

(function () {
  function boot() { window.GBNotepad.init(); }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot, { once: true });
  else boot();
})();
