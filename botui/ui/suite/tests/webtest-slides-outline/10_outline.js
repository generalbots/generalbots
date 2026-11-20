/**
 * Slides review tools (#2433) — shared foundation + outline view.
 * All RUNTIME cases drive the real exported helpers / real DOM handlers.
 */
"use strict";

var h = require("./00_harness");
var doc = h.document;
var win = h.window;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var O = h.O;

function textElement(text, id, type) {
  var el = doc.createElement("div");
  el.className = "sl-element";
  el.setAttribute("data-type", type || "text");
  if (id) el.setAttribute("data-id", id);
  var node = doc.createElement("p");
  node.textContent = text;
  el.appendChild(node);
  return el;
}

function deck(count) {
  var app = h.fixture();
  var host = doc.createElement("div");
  host.id = "sidebar-thumbs";
  app.appendChild(host);
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  canvas.setAttribute("data-slide-id", "0");
  canvas.setAttribute("data-bg", "#ffffff");
  app.appendChild(canvas);
  for (var i = 0; i < count; i++) {
    var thumb = doc.createElement("div");
    thumb.className = "sl-thumb";
    thumb.setAttribute("data-index", String(i));
    thumb.appendChild(textElement("<title-" + i + ">", "t" + i, "title"));
    thumb.appendChild(textElement("<bullet-" + i + ">", "b" + i, "text"));
    host.appendChild(thumb);
  }
  return { app: app, host: host, canvas: canvas, thumbs: host.children };
}

function rows() {
  return doc.querySelectorAll(".slides-outline-row");
}

// ── shared namespace ───────────────────────────────────────────────────
test("[RUNTIME] all eight shipped IIFEs expose their namespaces in one sandbox", function () {
  ["V", "F", "S", "N", "A", "P", "R"].forEach(function (name) {
    assert(h[name] && typeof h[name] === "object", name + " namespace missing");
  });
  assert(Object.keys(O).length >= 25, "shared outline API is incomplete");
  ["load", "save", "escape", "slides", "textFields", "elementRefs", "noteRefs",
    "writeText", "pushUndo", "undo", "redo", "dialog", "openPanel"].forEach(function (key) {
    equal(typeof O[key], "function", "O." + key + " is not a function");
  });
});

test("[RUNTIME] escape() encodes every HTML metacharacter and nullish input", function () {
  equal(O.escape("<a href=\"x\">&'"), "&lt;a href=&quot;x&quot;&gt;&amp;&#39;", "escape output");
  equal(O.escape(null), "", "null escape");
  equal(O.escape(undefined), "", "undefined escape");
});

test("[RUNTIME] deckId() prefers the provider, then the global, then a default", function () {
  h.fixture();
  equal(O.deckId(), "deck-A", "provider id");
  var provider = win.getSlidesPresentationId;
  delete win.getSlidesPresentationId;
  win.__SLIDES_PRESENTATION_ID = "deck-B";
  equal(O.deckId(), "deck-B", "global id fallback");
  delete win.__SLIDES_PRESENTATION_ID;
  equal(O.deckId(), "current", "default id");
  win.getSlidesPresentationId = provider;
});

test("[RUNTIME] load/save round-trip JSON under a deck-scoped key and fail safe", function () {
  h.clearStorage();
  equal(O.save("notes", { "1": [{ id: "n1", text: "<note>" }] }), true, "save result");
  assert(h.storageKeys().indexOf("slides-outline:notes:deck-A") >= 0, "deck-scoped key missing");
  equal(O.load("notes", {})["1"][0].text, "<note>", "roundtrip value");
  win.localStorage.setItem("slides-outline:corrupt:deck-A", "{broken");
  equal(O.load("corrupt", "<fallback>"), "<fallback>", "corrupt JSON fallback");
});

test("[RUNTIME] textFields() maps table cells, chart titles and text blocks", function () {
  h.fixture();
  var table = doc.createElement("div");
  table.setAttribute("data-type", "table");
  var cell = doc.createElement("td");
  cell.textContent = "<cell>";
  table.appendChild(cell);
  var chart = doc.createElement("div");
  chart.setAttribute("data-type", "chart");
  var chartTitle = doc.createElement("div");
  chartTitle.setAttribute("style", "font-weight:600");
  chartTitle.textContent = "<chart-title>";
  chart.appendChild(chartTitle);
  equal(O.textFields(table)[0].field, "cell:0", "table cell field");
  equal(O.textFields(chart)[0].field, "chart-title", "chart title field");
  equal(O.textFields(textElement("<body>", "x", "text"))[0].field, "text", "text field");
  equal(O.textFields(doc.createElement("div")).length, 0, "unknown type has no field");
});

test("[RUNTIME] slides() derives indices and mirrors the current canvas", function () {
  var d = deck(2);
  d.thumbs[1].setAttribute("data-index", "7");
  d.canvas.setAttribute("data-slide-id", "7");
  d.canvas.appendChild(textElement("<canvas-text>", "c1", "text"));
  var slides = O.slides();
  equal(slides.length, 2, "slide count");
  equal(slides[0].index, 0, "first index");
  equal(slides[1].index, 7, "data-index wins over position");
  equal(slides[1].canvasElements.length, 1, "canvas mirrored into current slide");
  equal(slides[0].canvasElements.length, 0, "non-current slide has no canvas copy");
  d.app.removeChild(d.host);
  var fallback = O.slides();
  equal(fallback.length, 1, "canvas-only fallback");
  equal(fallback[0].elements.length, 1, "canvas-only elements");
});

test("[RUNTIME] elementRefs() and noteRefs() flatten all local text refs", function () {
  h.clearStorage();
  deck(2);
  var refs = O.elementRefs();
  equal(refs.length, 4, "two text fields per slide");
  assert(refs.every(function (ref) { return ref.kind === "element"; }), "element kind");
  equal(refs[0].field, "text", "field name");
  O.save("notes", { "3": [{ id: "n7", text: "<note-text>" }] });
  var notes = O.noteRefs();
  equal(notes.length, 1, "note count");
  equal(notes[0].slideIndex, 3, "note slide index");
  equal(notes[0].noteId, "n7", "note id");
});

test("[RUNTIME] writeText() edits a note by id and a DOM field by before-snapshot", function () {
  h.clearStorage();
  var d = deck(1);
  O.save("notes", { "0": [{ id: "n1", text: "<old-note>" }] });
  O.writeText({ kind: "note", slideIndex: 0, noteId: "n1" }, "<new-note>");
  equal(O.load("notes", {})["0"][0].text, "<new-note>", "note rewritten");
  var ref = O.elementRefs()[0];
  ref.before = ref.text;
  O.writeText(ref, "<new-text>");
  equal(d.thumbs[0].querySelector("p").textContent, "<new-text>", "DOM text rewritten");
  ref.before = "<stale>";
  O.writeText(ref, "<never>");
  equal(d.thumbs[0].querySelector("p").textContent, "<new-text>", "stale snapshot ignored");
});

test("[RUNTIME] status() creates one polite live region and keeps the last message", function () {
  h.fixture();
  O.status("<status-one>");
  O.status("<status-two>");
  var regions = doc.querySelectorAll("#slides-tools-status");
  equal(regions.length, 1, "single status region");
  equal(regions[0].getAttribute("role"), "status", "status role");
  equal(regions[0].getAttribute("aria-live"), "polite", "polite live region");
  equal(regions[0].textContent, "<status-two>", "last message");
});

test("[RUNTIME] pushUndo/undo/redo are LIFO and a new action clears redo", function () {
  h.fixture();
  h.resetUndo();
  var counter = { n: 0 };
  var action = { label: "<action>", undo: function () { counter.n--; }, redo: function () { counter.n++; } };
  counter.n = 1;
  O.pushUndo(action);
  O.undo();
  equal(counter.n, 0, "undo ran");
  equal(h.stackState().redo, false, "redo available");
  O.redo();
  equal(counter.n, 1, "redo ran");
  O.undo();
  O.pushUndo(action);
  equal(h.stackState().redo, true, "redo cleared by new action");
  h.resetUndo();
});

test("[RUNTIME] undo history is capped at one hundred actions", function () {
  h.fixture();
  h.resetUndo();
  for (var i = 0; i < 105; i++) {
    O.pushUndo({ label: "<action>", undo: function () {}, redo: function () {} });
  }
  var drained = 0;
  while (!h.stackState().undo && drained < 200) { O.undo(); drained++; }
  equal(drained, 100, "capped undo depth");
});

test("[RUNTIME] jump() refreshes the host view and reports a 1-based slide", function () {
  h.fixture();
  O.jump(4);
  equal(win.__jumps.length, 1, "refresh called once");
  equal(win.__jumps[0], 4, "refresh index");
  equal(doc.querySelector("#slides-tools-status").textContent, "Jumped to slide 5", "status");
});

test("[RUNTIME] openPanel() and closePanels() manage a single panel host", function () {
  h.fixture();
  O.openPanel("<first>", function () { return doc.createElement("section"); });
  O.openPanel("<second>", function () { return doc.createElement("section"); });
  equal(doc.querySelectorAll("#slides-tools-panel-host").length, 1, "one host");
  equal(doc.querySelector("#slides-tools-panel-host").children[0].id, "<second>", "replaced panel");
  O.closePanels();
  equal(doc.querySelector("#slides-tools-panel-host").children.length, 0, "panels closed");
});

test("[RUNTIME] dialog() is modal, labelled, Escape-dismissable and restores focus", function () {
  var app = h.fixture();
  var trigger = doc.createElement("button");
  app.appendChild(trigger);
  trigger.focus();
  var dialog = O.dialog({ titleKey: "slides-outline-title" });
  equal(dialog.box.getAttribute("role"), "dialog", "dialog role");
  equal(dialog.box.getAttribute("aria-modal"), "true", "modal");
  equal(dialog.box.getAttribute("aria-labelledby"), dialog.box.querySelector("h2").id, "labelled");
  doc.dispatchEvent({ type: "keydown", key: "Escape", bubbles: false });
  equal(doc.querySelector(".slides-tools-overlay"), null, "overlay removed");
  equal(doc.activeElement, trigger, "focus restored");
});

test("[RUNTIME] confirm() cancel path dismisses the real dialog", function () {
  h.fixture();
  O.confirm({ titleKey: "slides-outline-title", messageKey: "slides-outline-section-required" });
  var overlay = doc.querySelector(".slides-tools-overlay");
  assert(overlay, "confirm dialog not created");
  overlay.querySelectorAll(".slides-tools-btn")[0].click();
  equal(doc.querySelector(".slides-tools-overlay"), null, "cancel dismissed");
});

test("[STATIC] shared foundation writes locally and never calls a server endpoint", function () {
  var src = h.readFile("slides/modules/20_outline_shared.js");
  assert(src.indexOf("window.localStorage") >= 0, "local store missing");
  ["fetch(", "XMLHttpRequest", "/api/", "sendBeacon"].forEach(function (needle) {
    assert(src.indexOf(needle) < 0, "server write leaked: " + needle);
  });
});

// ── outline view ────────────────────────────────────────────────────────
test("[RUNTIME] outline view derives a title/bullet tree from the rendered deck", function () {
  h.resetUndo();
  h.clearStorage();
  deck(2);
  h.V.open();
  var list = doc.querySelector(".slides-outline-list");
  assert(list, "outline list missing");
  equal(list.getAttribute("role"), "tree", "tree role");
  var all = rows();
  equal(all.length, 4, "two titles + two bullets");
  equal(all[0].getAttribute("aria-level"), "1", "slide level");
  equal(all[1].getAttribute("aria-level"), "2", "bullet level");
  equal(all[0].getAttribute("data-id"), "s0", "slide row id");
  equal(all[1].getAttribute("data-id"), "b0-0", "bullet row id");
  equal(all[1].getAttribute("data-slide"), "0", "slide association");
  assert(all[1].style["--so-indent"] === "18px", "bullet indent");
});

test("[RUNTIME] Word paste maps tab depth and bullet markers into outline levels", function () {
  h.resetUndo();
  h.clearStorage();
  deck(1);
  h.V.open();
  var paste = doc.querySelector(".slides-outline-paste");
  var payload = ["<word-a>", "\t- <word-b>", "\t\t1. <word-c>", "", "* <word-d>"].join("\n");
  paste.dispatchEvent({
    type: "paste", bubbles: true,
    clipboardData: { getData: function () { return payload; } }
  });
  var all = rows();
  equal(all.length, 6, "four bullets added");
  equal(all[2].getAttribute("aria-level"), "2", "unstyled line level");
  equal(all[3].getAttribute("aria-level"), "3", "one-tab level");
  equal(all[4].getAttribute("aria-level"), "4", "two-tab level");
  equal(all[5].getAttribute("aria-level"), "2", "asterisk marker stripped");
  equal(O.load("outline", {}).rows.length, 6, "pasted rows persisted");
});

test("[RUNTIME] promote/demote clamp at levels 0..5 and persist", function () {
  h.resetUndo();
  h.clearStorage();
  deck(1);
  h.V.open();
  rows()[1].querySelector(".slides-outline-demote").click();
  equal(O.load("outline", {}).rows[1].level, 2, "demoted");
  equal(rows()[1].getAttribute("aria-level"), "3", "aria level after demote");
  rows()[1].querySelector(".slides-outline-promote").click();
  equal(O.load("outline", {}).rows[1].level, 1, "promoted back");
  rows()[0].querySelector(".slides-outline-promote").click();
  equal(O.load("outline", {}).rows[0].level, 0, "slide clamps at zero");
  rows()[1].querySelector(".slides-outline-demote").click();
  rows()[1].querySelector(".slides-outline-demote").click();
  rows()[1].querySelector(".slides-outline-demote").click();
  rows()[1].querySelector(".slides-outline-demote").click();
  rows()[1].querySelector(".slides-outline-demote").click();
  equal(O.load("outline", {}).rows[1].level, 5, "bullet clamps at five");
});

test("[RUNTIME] move up/down reorders rows (icon path ships no undo entry)", function () {
  h.resetUndo();
  h.clearStorage();
  deck(2);
  h.V.open();
  var before = rows().map(function (row) { return row.getAttribute("data-id"); });
  rows()[0].querySelector(".slides-outline-down").click();
  var moved = rows().map(function (row) { return row.getAttribute("data-id"); });
  equal(moved[0], before[1], "row moved down");
  equal(moved[1], before[0], "neighbour shifted up");
  // Shipped behaviour: 21_outline_view.js:159-164 never calls O.pushUndo for
  // the icon/keyboard path (drag/drop at :209 does). Reported as a real bug;
  // this assertion records the observable state, it does not excuse it.
  equal(h.stackState().undo, true, "no undo entry pushed by icon reorder");
  O.undo();
  equal(rows().map(function (row) { return row.getAttribute("data-id"); }).join("|"),
    moved.join("|"), "order unchanged by empty undo stack");
});

test("[RUNTIME] drag and drop reorders rows as one undoable action", function () {
  h.resetUndo();
  h.clearStorage();
  deck(2);
  h.V.open();
  var before = rows().map(function (row) { return row.getAttribute("data-id"); });
  var transfer = { effectAllowed: "", dropEffect: "", setData: function (type, value) { this.data = value; },
    getData: function () { return this.data; } };
  rows()[0].dispatchEvent({ type: "dragstart", bubbles: true, dataTransfer: transfer });
  rows()[2].dispatchEvent({ type: "drop", bubbles: true, dataTransfer: transfer });
  var moved = rows().map(function (row) { return row.getAttribute("data-id"); });
  equal(moved[0], before[1], "dragged row moved");
  equal(moved[2], before[0], "target position taken");
  O.undo();
  equal(rows().map(function (row) { return row.getAttribute("data-id"); }).join("|"),
    before.join("|"), "undo restored drag order");
});

test("[RUNTIME] rebuild from slides discards local-only outline rows", function () {
  h.resetUndo();
  h.clearStorage();
  deck(2);
  h.V.open();
  doc.querySelector(".slides-outline-paste").dispatchEvent({
    type: "paste", bubbles: true,
    clipboardData: { getData: function () { return "<local-only>"; } }
  });
  equal(rows().length, 5, "pasted row present");
  h.byKey("slides-outline-rebuild").click();
  equal(rows().length, 4, "rebuilt from shipped slide model");
  assert(O.load("outline", {}).rows.every(function (row) {
    return row.text.indexOf("local-only") < 0;
  }), "local row removed from store");
});

test("[RUNTIME] global Ctrl+Z/Ctrl+Y route through the shared undo stack", function () {
  h.fixture();
  h.resetUndo();
  var counter = { n: 0 };
  O.pushUndo({ label: "<action>", undo: function () { counter.n--; }, redo: function () { counter.n++; } });
  counter.n = 1;
  var host = doc.createElement("div");
  host.className = "slides-tools-panel-host";
  h.window.GBSlidesOutline.scope().appendChild(host);
  doc.dispatchEvent({ type: "keydown", key: "z", ctrlKey: true, target: host, bubbles: false });
  equal(counter.n, 0, "Ctrl+Z undone");
  doc.dispatchEvent({ type: "keydown", key: "y", ctrlKey: true, target: host, bubbles: false });
  equal(counter.n, 1, "Ctrl+Y redone");
});

test("[STATIC] outline view ships keyboard, drag and paste accessibility hooks", function () {
  var src = h.readFile("slides/modules/21_outline_view.js");
  ["role\", \"treeitem", "aria-level", "aria-grabbed", "dragstart", "altKey",
    "clipboardData", "slides-outline-paste-bullets"].forEach(function (needle) {
    assert(src.indexOf(needle) >= 0, "shipped hook missing: " + needle);
  });
});
