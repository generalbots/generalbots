"use strict";
/* #2225 — gb-doc-delta for notes.
   The board keeps its notes in localStorage (`gb-sticky-notes`), so streamed
   ops must go through window.GBNotes: building detached cards here would
   disappear on the next render and never reach the persisted board. */
(function () {
  var keyByOpId = {};

  function textOf(op) {
    if (typeof op.text === "string") return op.text;
    if (typeof op.body === "string") return op.body;
    if (typeof op.title === "string") return op.title;
    return "";
  }

  function resolveKey(opId) {
    if (Object.prototype.hasOwnProperty.call(keyByOpId, opId)) return keyByOpId[opId];
    // Ops may address a note by its position or timestamp instead of the id
    // this renderer handed out on add_note.
    var direct = Number(opId);
    return isNaN(direct) ? null : direct;
  }

  function apply(payload) {
    var notes = window.GBNotes;
    if (!notes) return;
    (payload.ops || []).forEach(function (op) {
      if (!op) return;
      if (op.op === "add_note") {
        var key = notes.add(textOf(op));
        if (op.id) keyByOpId[op.id] = key;
        return;
      }
      if (op.op === "patch_note") {
        var patchKey = resolveKey(op.id);
        if (patchKey !== null) notes.patch(patchKey, textOf(op));
        return;
      }
      if (op.op === "delete_note") {
        var deleteKey = resolveKey(op.id);
        if (deleteKey !== null) {
          notes.remove(deleteKey);
          if (op.id) delete keyByOpId[op.id];
        }
        return;
      }
      if (op.op === "clear") {
        // A new stream starts a new id space; user notes are never wiped.
        keyByOpId = {};
      }
    });
  }

  function register() {
    if (window.GBDocDelta) window.GBDocDelta.register("notes", apply);
  }

  if (window.GBDocDelta) register();
  else document.addEventListener("DOMContentLoaded", register, { once: true });
})();
