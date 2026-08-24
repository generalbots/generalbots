"use strict";
/* GB Notepad (#1154): multi-file plain-text scratchpad, localStorage-backed. */
window.GBNotepad = window.GBNotepad || {};
(function (app) {
  var IDX = "gb-notepad-index";
  function loadIndex() {
    try { return JSON.parse(localStorage.getItem(IDX) || "{}"); } catch (e) { return {}; }
  }
  function saveIndex(i) { try { localStorage.setItem(IDX, JSON.stringify(i)); } catch (e) {} }

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
      ta.value = loadIndex()[sel.value] || "";
    }
    function mark(dirty) { status.textContent = dirty ? "Editing…" : "Saved"; }

    ta.addEventListener("input", function () {
      var i = loadIndex();
      if (!current()) return;
      i[current()] = ta.value;
      saveIndex(i);
      mark(true);
      clearTimeout(app._t);
      app._t = setTimeout(function () { mark(false); }, 500);
    });
    sel.addEventListener("change", function () { ta.value = loadIndex()[current()] || ""; });
    root.querySelector("#gb-notepad-new").addEventListener("click", function () {
      var name = prompt("Note name:", "note-" + new Date().toISOString().slice(0, 10));
      if (!name) return;
      var i = loadIndex();
      if (i[name] === undefined) { i[name] = ""; saveIndex(i); }
      renderList(name); mark(false);
    });
    root.querySelector("#gb-notepad-del").addEventListener("click", function () {
      if (!current()) return;
      if (!confirm("Delete note '" + current() + "'?")) return;
      var i = loadIndex(); delete i[current()]; saveIndex(i); renderList(names()[0] || "");
    });

    if (!names().length) {
      var i = loadIndex(); i["welcome"] = "Welcome to Notepad.\nEverything is stored locally per user."; saveIndex(i);
    }
    renderList(localStorage.getItem("gb-notepad-last") || names()[0]);
    sel.addEventListener("change", function () { try { localStorage.setItem("gb-notepad-last", sel.value); } catch (e) {} });
  };
})(window.GBNotepad);

(function () {
  function boot() { window.GBNotepad.init(); }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot, { once: true });
  else boot();
})();
