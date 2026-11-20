/**
 * Slides review tools (#2433) — deck-wide find/replace and sections.
 * Every RUNTIME case drives the real shipped DOM handlers in the shared sandbox.
 */
"use strict";

var h = require("./00_harness");
var doc = h.document;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var O = h.O;

var NEEDLE = "needle";
var OLD = "OldCo";
var NEW = "NewCo";

function element(type, text, id) {
  var el = doc.createElement("div");
  el.className = "sl-element";
  el.setAttribute("data-type", type);
  el.setAttribute("data-id", id);
  var node = doc.createElement("p");
  node.textContent = text;
  el.appendChild(node);
  return el;
}

function deck(texts) {
  var app = h.fixture();
  var host = doc.createElement("div");
  host.id = "sidebar-thumbs";
  app.appendChild(host);
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  canvas.setAttribute("data-slide-id", "0");
  app.appendChild(canvas);
  var thumb = doc.createElement("div");
  thumb.className = "sl-thumb";
  thumb.setAttribute("data-index", "0");
  texts.forEach(function (spec, index) {
    if (spec.type === "table") {
      var table = doc.createElement("div");
      table.className = "sl-element";
      table.setAttribute("data-type", "table");
      table.setAttribute("data-id", spec.id);
      var cell = doc.createElement("td");
      cell.textContent = spec.text;
      table.appendChild(cell);
      thumb.appendChild(table);
    } else {
      thumb.appendChild(element(spec.type, spec.text, spec.id));
    }
  });
  host.appendChild(thumb);
  return { app: app, host: host, thumb: thumb, canvas: canvas };
}

function openFind(find, replace, cases, whole) {
  h.F.open();
  doc.getElementById("slides-fr-find").value = find;
  doc.getElementById("slides-fr-replace").value = replace || "";
  doc.getElementById("slides-fr-case").checked = !!cases;
  doc.getElementById("slides-fr-whole").checked = !!whole;
  h.byKey("slides-outline-fr-find-btn").click();
}

function matches() {
  return doc.querySelectorAll(".slides-fr-match");
}

function dialogInput() {
  return doc.querySelector(".slides-tools-dialog-body input[type=\"text\"]");
}

// ── find / replace ──────────────────────────────────────────────────────
test("[RUNTIME] find() is case-sensitive when asked and case-insensitive by default", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "title", text: NEEDLE + " " + NEEDLE + "Fish " + NEEDLE.toUpperCase(), id: "t0" }]);
  openFind(NEEDLE, "", true, false);
  equal(matches().length, 2, "case-sensitive count");
  openFind(NEEDLE, "", false, false);
  equal(matches().length, 3, "case-insensitive count");
});

test("[RUNTIME] whole-word option rejects substring matches", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "text", text: NEEDLE + " " + NEEDLE + "Fish " + NEEDLE.toUpperCase(), id: "x0" }]);
  openFind(NEEDLE, "", false, true);
  equal(matches().length, 2, "only standalone words");
  assert(matches()[0].querySelector(".slides-fr-context").textContent.length > 0, "snippet rendered");
});

test("[RUNTIME] find() treats regex metacharacters as literal text", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "text", text: "a.b axb", id: "x0" }]);
  openFind("a.b", "", false, false);
  equal(matches().length, 1, "dot is literal");
});

test("[RUNTIME] empty query and no-match states are reported without mutations", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "text", text: "a.b axb", id: "x0" }]);
  openFind("", "", false, false);
  equal(matches().length, 0, "empty query");
  assert(doc.querySelector("#slides-tools-status").textContent.length > 0, "status written");
  openFind("absent-token", "", false, false);
  equal(matches().length, 0, "no match");
  assert(doc.querySelector(".slides-fr-results .slides-tools-empty"), "empty state shown");
});

test("[RUNTIME] replace-all covers slides, tables, charts and notes in one undo step", function () {
  h.resetUndo();
  h.clearStorage();
  var d = deck([
    { type: "title", text: OLD + " deck", id: "t0" },
    { type: "text", text: OLD + " and " + OLD, id: "x0" },
    { type: "table", text: OLD, id: "tab0" },
    { type: "chart", text: OLD, id: "cht0" }
  ]);
  O.save("notes", { "0": [{ id: "n1", text: OLD + " note" }] });
  openFind(OLD, NEW, false, false);
  equal(matches().length, 6, "matches across all local scopes");
  h.byKey("slides-outline-fr-all").click();
  equal(d.thumb.querySelectorAll("p")[0].textContent, NEW + " deck", "title replaced");
  equal(d.thumb.querySelectorAll("p")[1].textContent, NEW + " and " + NEW, "text replaced");
  equal(O.load("notes", {})["0"][0].text, NEW + " note", "note replaced");
  equal(h.stackState().undo, false, "one undoable action recorded");
  O.undo();
  equal(O.load("notes", {})["0"][0].text, OLD + " note", "note restored");
  equal(d.thumb.querySelectorAll("p")[1].textContent, OLD + " and " + OLD, "text restored");
});

test("[RUNTIME] per-match Replace edits one match and is undoable", function () {
  h.resetUndo();
  h.clearStorage();
  var d = deck([{ type: "text", text: NEEDLE + " " + NEEDLE, id: "x0" }]);
  openFind(NEEDLE, NEW, false, false);
  equal(matches().length, 2, "two matches");
  matches()[0].querySelectorAll("button")[1].click();
  equal(d.thumb.querySelector("p").textContent, NEW + " " + NEEDLE, "one match replaced");
  O.undo();
  equal(d.thumb.querySelector("p").textContent, NEEDLE + " " + NEEDLE, "restored");
});

test("[STATIC] find/replace reads only local slide and note refs", function () {
  var src = h.readFile("slides/modules/22_find_replace.js");
  assert(src.indexOf("O.elementRefs()") >= 0 && src.indexOf("O.noteRefs()") >= 0,
    "local scopes missing");
  ["fetch(", "XMLHttpRequest", "/api/", "sendBeacon"].forEach(function (needle) {
    assert(src.indexOf(needle) < 0, "server call leaked: " + needle);
  });
});

// ── sections ────────────────────────────────────────────────────────────
test("[RUNTIME] sections default to one local section and reappear on reopen", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "title", text: "a", id: "t0" }]);
  h.S.open();
  equal(doc.querySelectorAll(".slides-section-row").length, 1, "default row");
  // Shipped behaviour: 23_sections.js:11-17 derives the default section on
  // read only; it is persisted on the first mutating action.
  equal(O.load("sections", []).length, 0, "default is derived, not eagerly saved");
  O.closePanels();
  h.S.open();
  equal(doc.querySelectorAll(".slides-section-row").length, 1, "reopen stable");
});

test("[RUNTIME] annotate() partitions thumbnails by section start", function () {
  h.resetUndo();
  h.clearStorage();
  var d = deck([{ type: "title", text: "a", id: "t0" }]);
  for (var i = 1; i < 3; i++) {
    var thumb = doc.createElement("div");
    thumb.className = "sl-thumb";
    thumb.setAttribute("data-index", String(i));
    d.host.appendChild(thumb);
  }
  O.save("sections", [
    { id: "default", name: "First", start: 0 },
    { id: "sec-x", name: "Second", start: 2 }
  ]);
  h.S.annotate();
  var thumbs = d.host.querySelectorAll(".sl-thumb");
  equal(thumbs[0].getAttribute("data-section-id"), "default", "first section");
  equal(thumbs[1].getAttribute("data-section-id"), "default", "first section spans");
  equal(thumbs[2].getAttribute("data-section-id"), "sec-x", "second section");
  equal(thumbs[2].getAttribute("data-section-name"), "Second", "section name annotation");
  assert(thumbs[2].getAttribute("aria-label").indexOf("Second") >= 0, "localized chip label");
});

test("[RUNTIME] collapse state persists in local storage and reopens collapsed", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "title", text: "a", id: "t0" }]);
  h.S.open();
  doc.querySelectorAll(".slides-section-toggle")[0].click();
  equal(O.load("sections-collapsed", {})["default"], true, "collapse persisted");
  equal(doc.querySelectorAll(".slides-section-toggle")[0].getAttribute("aria-expanded"),
    "false", "aria-expanded updated");
  O.closePanels();
  h.S.open();
  equal(doc.querySelectorAll(".slides-section-toggle")[0].getAttribute("aria-expanded"),
    "false", "reopened collapsed");
});

test("[RUNTIME] the default section cannot be deleted", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "title", text: "a", id: "t0" }]);
  h.S.open();
  doc.querySelector(".slides-section-delete").click();
  equal(doc.querySelectorAll(".slides-section-row").length, 1, "default row retained");
  equal(O.load("sections", []).length, 0, "no section removal performed");
  assert(doc.querySelector("#slides-tools-status").textContent.length > 0, "status reported");
});

test("[RUNTIME] create, rename and reorder sections through the real dialogs", function () {
  h.resetUndo();
  h.clearStorage();
  deck([{ type: "title", text: "a", id: "t0" },
    { type: "text", text: "b", id: "x0" },
    { type: "text", text: "c", id: "x1" }]);
  h.S.open();
  h.byKey("slides-outline-section-create").click();
  var input = dialogInput();
  assert(input, "name dialog opened");
  input.value = "Second section";
  h.byKey("slides-outline-save").click();
  var sections = O.load("sections", []);
  equal(sections.length, 2, "section created");
  equal(sections[1].name, "Second section", "created name");
  doc.querySelectorAll(".slides-section-rename")[1].click();
  dialogInput().value = "Renamed section";
  h.byKey("slides-outline-save").click();
  equal(O.load("sections", [])[1].name, "Renamed section", "renamed");
  var beforeOrder = O.load("sections", []).map(function (section) { return section.id; });
  doc.querySelectorAll(".slides-section-up")[1].click();
  var afterOrder = O.load("sections", []).map(function (section) { return section.id; });
  equal(afterOrder[0], beforeOrder[1], "section moved up");
  O.undo();
  equal(O.load("sections", []).map(function (section) { return section.id; }).join("|"),
    beforeOrder.join("|"), "undo restored section order");
});

test("[STATIC] section PPTX round-trip is explicitly deferred to the backend", function () {
  var src = h.readFile("slides/modules/23_sections.js");
  assert(src.indexOf("server compile gate") >= 0, "deferred backend note missing");
  assert(src.indexOf("localStorage") >= 0, "local metadata store missing");
  ["fetch(", "XMLHttpRequest", "/api/"].forEach(function (needle) {
    assert(src.indexOf(needle) < 0, "server write leaked: " + needle);
  });
});
