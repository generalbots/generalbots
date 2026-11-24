/**
 * [UAT:NOTE-02] "Add a sticky note: call the accountant on Friday"
 * Steps exercised: (1) the note is created with the exact phrase text;
 * (2) it is still visible after closing and reopening the app.
 * Uses the real notes/notes.js add/render/patch code against a localStorage stub.
 */
"use strict";

var env = require("./00_env");
var h = env.h;

h.test("[UAT:NOTE-02] add a note and see it again after reopening", function () {
  var store = env.makeStore();
  var doc = h.makeDocument({
    readyState: "complete",
    ids: ["gb-notes", "notesAdd", "notesCount", "notesBoard"],
  });
  var sandbox = env.baseSandbox(doc, store);
  env.loadAll(sandbox, ["notes/notes.js"]);

  var app = sandbox.GBNotes;
  h.assert(app && typeof app.add === "function", "GBNotes.add not exported");

  // Step 1 — the phrase becomes one note with exactly that text.
  var key = app.add("call the accountant on Friday");
  var saved = JSON.parse(store.data["gb-sticky-notes"]);
  h.assert(saved.length === 1, "expected exactly one note, got " + saved.length);
  h.assert(
    saved[0].text === "call the accountant on Friday",
    "note text mismatch: " + saved[0].text,
  );
  h.assert(
    doc.getElementById("notesCount").textContent === "1 note",
    "counter did not update after add",
  );

  // Step 2 — a fresh app instance (close + reopen) restores it from storage.
  var reopened = env.makeStore(JSON.parse(JSON.stringify(store.data)));
  var doc2 = h.makeDocument({
    readyState: "complete",
    ids: ["gb-notes", "notesAdd", "notesCount", "notesBoard"],
  });
  var sandbox2 = env.baseSandbox(doc2, reopened);
  env.loadAll(sandbox2, ["notes/notes.js"]);

  var board = doc2.getElementById("notesBoard");
  h.assert(
    board.innerHTML.indexOf("call the accountant on Friday") !== -1,
    "note not rendered after reopen: " + board.innerHTML,
  );
  h.assert(
    doc2.getElementById("notesCount").textContent === "1 note",
    "counter not restored after reopen",
  );
  h.assert(
    sandbox2.GBNotes.patch(String(key), "call the accountant on Saturday") === true,
    "reopened note cannot be addressed by its key",
  );
});
