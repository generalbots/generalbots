/**
 * Slides review tools (#2433) — notes workspace, accessibility panel, deck
 * properties and the Review-menu bootstrap. RUNTIME cases drive the real
 * shipped DOM handlers; STATIC cases only confirm shipped source hooks.
 */
"use strict";

var h = require("./00_harness");
var doc = h.document;
var win = h.window;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var O = h.O;

function addButton(row, key) {
  var buttons = row.querySelectorAll("button");
  for (var i = 0; i < buttons.length; i++) {
    if (buttons[i].getAttribute("data-i18n") === key) return buttons[i];
  }
  return null;
}

function canvasDeck(slideId) {
  var app = h.fixture();
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  canvas.setAttribute("data-slide-id", String(slideId == null ? 0 : slideId));
  app.appendChild(canvas);
  return { app: app, canvas: canvas };
}

function noteRows() {
  return doc.querySelectorAll(".slides-notes-row");
}

// ── notes workspace ─────────────────────────────────────────────────────
test("[RUNTIME] notes workspace renders an empty state and a local badge", function () {
  h.resetUndo();
  h.clearStorage();
  canvasDeck(0);
  var panel = h.N.open();
  assert(panel.querySelector(".slides-notes-list"), "notes list missing");
  assert(panel.querySelector("input[type=\"search\"]"), "search input missing");
  assert(h.byKey("slides-outline-notes-add"), "add button missing");
  assert(panel.querySelector(".slides-tools-local-badge"), "local badge missing");
  assert(doc.querySelector(".slides-notes-list .slides-tools-empty"), "empty state missing");
});

test("[UAT:SLIDE-04][RUNTIME] add note stores it under the current slide index and is undoable", function () {
  h.resetUndo();
  h.clearStorage();
  canvasDeck(3);
  h.N.open();
  h.byKey("slides-outline-notes-add").click();
  var area = doc.querySelector(".slides-tools-dialog-body textarea");
  assert(area, "note dialog textarea missing");
  area.value = " <note-text> ";
  h.byKey("slides-outline-save").click();
  var map = O.load("notes", {});
  equal(map["3"].length, 1, "note stored for slide 3");
  equal(map["3"][0].text, "<note-text>", "text trimmed");
  assert(/^n[0-9a-z]+$/.test(map["3"][0].id), "deterministic note id");
  equal(noteRows().length, 1, "row rendered");
  equal(noteRows()[0].getAttribute("data-slide"), "3", "row slide association");
  equal(noteRows()[0].getAttribute("data-note"), map["3"][0].id, "row note id");
  O.undo();
  equal(O.load("notes", {})["3"].length, 0, "undo removed note");
  O.redo();
  equal(O.load("notes", {})["3"].length, 1, "redo restored note");
});

test("[RUNTIME] notes search filters by text and numeric slide order is stable", function () {
  h.resetUndo();
  h.clearStorage();
  canvasDeck(0);
  O.save("notes", {
    "10": [{ id: "n10", text: "<alpha>" }],
    "2": [{ id: "n2", text: "<beta>" }]
  });
  h.N.open();
  equal(noteRows().length, 2, "both notes rendered");
  equal(noteRows()[0].getAttribute("data-slide"), "2", "numeric order: 2 before 10");
  var search = doc.querySelector("input[type=\"search\"]");
  search.value = "alpha";
  search.dispatchEvent({ type: "input", bubbles: true });
  equal(noteRows().length, 1, "filtered to one note");
  equal(noteRows()[0].getAttribute("data-note"), "n10", "correct note matched");
});

test("[RUNTIME] note jump, edit and delete drive the shared undo stack", function () {
  h.resetUndo();
  h.clearStorage();
  canvasDeck(0);
  O.save("notes", { "0": [{ id: "n1", text: "<original>" }] });
  h.N.open();
  noteRows()[0].querySelector(".slides-notes-jump").click();
  equal(win.__jumps[0], 0, "jump to slide 0");
  addButton(noteRows()[0], "slides-outline-edit").click();
  doc.querySelector(".slides-tools-dialog-body textarea").value = "<edited>";
  h.byKey("slides-outline-save").click();
  equal(O.load("notes", {})["0"][0].text, "<edited>", "note edited");
  O.undo();
  equal(O.load("notes", {})["0"][0].text, "<original>", "edit undone");
  addButton(noteRows()[0], "slides-outline-delete").click();
  equal(O.load("notes", {})["0"].length, 0, "note deleted");
  O.undo();
  equal(O.load("notes", {})["0"][0].text, "<original>", "delete undone");
});

test("[RUNTIME] rich-note save copies the modal text into the local workspace", function () {
  h.resetUndo();
  h.clearStorage();
  var d = canvasDeck(1);
  var notesBtn = doc.createElement("button");
  notesBtn.id = "notesBtn";
  d.app.appendChild(notesBtn);
  var modal = doc.createElement("div");
  modal.id = "slides-modal-container";
  d.app.appendChild(modal);
  var area = doc.createElement("textarea");
  area.setAttribute("name", "notes");
  area.value = "<rich-text>";
  modal.appendChild(area);
  O.save("notes", { "1": [{ id: "nseed", text: "<seed>" }] });
  h.N.open();
  addButton(noteRows()[0], "slides-outline-notes-rich-edit").click();
  assert(JSON.parse(notesBtn.getAttribute("hx-vals")).slide_index === "1", "hx-vals slide index");
  modal.dispatchEvent({ type: "htmx:afterRequest", bubbles: true, detail: { successful: true } });
  var stored = O.load("notes", {})["1"] || [];
  equal(stored.length, 2, "rich note appended to the slide list");
  assert(/^rich-/.test(stored[1].id), "rich note id prefix");
  equal(stored[1].text, "<rich-text>", "rich note text");
  equal(stored[1].rich, true, "rich marker");
});

// ── accessibility panel ─────────────────────────────────────────────────
function a11yDeck() {
  var app = h.fixture();
  var host = doc.createElement("div");
  host.id = "sidebar-thumbs";
  app.appendChild(host);
  var canvas = doc.createElement("div");
  canvas.id = "slide-canvas";
  canvas.setAttribute("data-slide-id", "0");
  canvas.setAttribute("data-bg", "#ffffff");
  app.appendChild(canvas);
  var thumb = doc.createElement("div");
  thumb.className = "sl-thumb";
  thumb.setAttribute("data-index", "0");
  host.appendChild(thumb);
  function block(type, id) {
    var el = doc.createElement("div");
    el.className = "sl-element";
    el.setAttribute("data-type", type);
    el.setAttribute("data-id", id);
    thumb.appendChild(el);
    return el;
  }
  var title = block("title", "t0");
  title.style.color = "#ffffff";
  var para = doc.createElement("p");
  para.textContent = "<title-sample>";
  title.appendChild(para);
  var image = block("image", "img0");
  var img = doc.createElement("img");
  image.appendChild(img);
  var chart = block("chart", "cht0");
  var table = block("table", "tab0");
  var media = doc.createElement("img");
  media.className = "sl-media";
  thumb.appendChild(media);
  return { app: app, thumb: thumb, img: img, image: image, chart: chart,
    table: table, media: media };
}

function listRows() {
  return doc.querySelectorAll(".slides-a11y-list .slides-a11y-row");
}

test("[RUNTIME] accessibility panel collects image, chart, table and media gaps", function () {
  h.resetUndo();
  h.clearStorage();
  a11yDeck();
  var panel = h.A.open();
  equal(listRows().length, 4, "four missing text alternatives");
  assert(panel.querySelector(".slides-tools-local-badge"), "local badge present");
  assert(h.byKey("slides-outline-a11y-summary"), "summary uses a catalog key");
  assert(doc.querySelector("#slides-tools-status").textContent.length > 0, "status written");
});

test("[RUNTIME] inline alt text saves locally, hides the row and is undoable", function () {
  h.resetUndo();
  h.clearStorage();
  var d = a11yDeck();
  h.A.open();
  var row = listRows()[0];
  var inline = row.querySelector(".slides-a11y-inline");
  assert(inline, "inline alt field missing");
  inline.value = "<alt-text>";
  addButton(row, "slides-outline-save").click();
  equal(listRows().length, 3, "row removed after alt saved");
  equal(d.img.getAttribute("alt"), "<alt-text>", "DOM alt applied");
  assert(Object.keys(O.load("a11y-alt", {})).length === 1, "alt stored locally");
  O.undo();
  equal(listRows().length, 4, "undo restored the row");
  equal(d.img.getAttribute("alt"), "", "undo reset DOM alt to the empty string");
});

test("[RUNTIME] mark decorative sets aria-hidden and is undoable", function () {
  h.resetUndo();
  h.clearStorage();
  var d = a11yDeck();
  h.A.open();
  var chartRow = listRows()[1];
  addButton(chartRow, "slides-outline-a11y-decorate").click();
  equal(listRows().length, 3, "decorated row hidden");
  equal(d.chart.getAttribute("aria-hidden"), "true", "aria-hidden applied");
  equal(d.chart.getAttribute("role"), "presentation", "presentation role applied");
  assert(Object.keys(O.load("a11y-decorative", {})).length === 1, "flag stored locally");
  O.undo();
  equal(listRows().length, 4, "undo restored the row");
  equal(d.chart.hasAttribute("aria-hidden"), false, "undo removed aria-hidden");
});

test("[RUNTIME] low-contrast text raises a warning; clear text does not", function () {
  h.resetUndo();
  h.clearStorage();
  a11yDeck();
  var panel = h.A.open();
  var warnings = panel.querySelectorAll(".slides-a11y-warnings .slides-a11y-row");
  equal(warnings.length, 1, "white-on-white warning");
  assert(h.byKey("slides-outline-a11y-ratio"), "ratio uses a catalog key");
  var d = a11yDeck();
  panel = h.A.open();
  warnings = panel.querySelectorAll(".slides-a11y-warnings .slides-a11y-row");
  equal(warnings.length, 1, "warning recomputed on reopen");
  d.app.querySelectorAll(".sl-element")[0].style.color = "#000000";
  panel = h.A.open();
  warnings = panel.querySelectorAll(".slides-a11y-warnings .slides-a11y-row");
  equal(warnings.length, 0, "black-on-white clears the warning");
  assert(panel.querySelector(".slides-a11y-warnings .slides-tools-empty"), "clear state rendered");
});

test("[RUNTIME] collect() only sees app slide elements, not a raw injected img", function () {
  h.resetUndo();
  h.clearStorage();
  var d = canvasDeck(0);
  var stray = doc.createElement("img");
  d.canvas.appendChild(stray);
  var panel = h.A.open();
  equal(listRows().length, 0, "raw img outside the slide model is not collected");
  assert(panel.querySelector(".slides-a11y-warnings .slides-tools-empty"), "no contrast warnings");
  // Reported as a real potential bug: 26_accessibility.js:16-60 calls
  // O.slides(), which derives decks from .sl-thumb/.sl-element. An image added
  // straight to the DOM (e.g. by a third-party editor) is invisible to the
  // audit. No production change is made here.
});

test("[STATIC] contrast helpers and collect are shipped in the accessibility module", function () {
  var src = h.readFile("slides/modules/26_accessibility.js");
  ["function collect(", "function parseColor(", "function luminance(",
    "function contrast(", "function contrastWarnings(", "function refKey(",
    "data-bg", "sl-media", "4.5"].forEach(function (needle) {
    assert(src.indexOf(needle) >= 0, "shipped helper missing: " + needle);
  });
});

// ── deck properties ─────────────────────────────────────────────────────
test("[RUNTIME] properties dialog exposes six labelled local-only fields", function () {
  h.resetUndo();
  h.clearStorage();
  h.fixture();
  var dialog = h.P.open();
  equal(doc.querySelectorAll(".slides-props-row").length, 6, "six fields");
  assert(doc.querySelector(".slides-tools-local-badge"), "local badge present");
  var inputs = dialog.body.querySelectorAll("input, textarea");
  equal(inputs.length, 6, "six controls");
  assert(inputs[0].hasAttribute("data-i18n-aria-label"), "aria-label key present");
  assert(inputs[0].getAttribute("aria-label").length > 0, "aria-label resolved");
});

test("[RUNTIME] properties default the title from presTitle and save/undo locally", function () {
  h.resetUndo();
  h.clearStorage();
  var app = h.fixture();
  var presTitle = doc.createElement("input");
  presTitle.id = "presTitle";
  presTitle.value = "<deck-title>";
  app.appendChild(presTitle);
  var dialog = h.P.open();
  var inputs = dialog.body.querySelectorAll("input, textarea");
  equal(inputs[0].value, "<deck-title>", "title seeded from host");
  inputs[1].value = "<author>";
  h.byKey("slides-outline-save").click();
  var props = O.load("props", {});
  equal(props.title, "<deck-title>", "title saved");
  equal(props.author, "<author>", "author saved");
  equal(Object.keys(props).length, 6, "all six fields saved");
  O.undo();
  equal(O.load("props", {}).title, "<deck-title>", "undo restored the pre-save props");
});

test("[RUNTIME] properties cancel leaves the stored values untouched", function () {
  h.resetUndo();
  h.clearStorage();
  h.fixture();
  O.save("props", { title: "<kept>" });
  var dialog = h.P.open();
  dialog.body.querySelectorAll("input")[0].value = "<discarded>";
  h.byKey("slides-outline-cancel").click();
  equal(O.load("props", {}).title, "<kept>", "cancel did not mutate store");
});

// ── review bootstrap ────────────────────────────────────────────────────
function toolbarFixture() {
  var app = h.fixture();
  var main = doc.createElement("div");
  main.className = "slides-toolbar-row-main";
  app.appendChild(main);
  var right = doc.createElement("div");
  right.className = "toolbar-right";
  main.appendChild(right);
  return { app: app, right: right };
}

function buildMenu() {
  var f = toolbarFixture();
  doc.dispatchEvent({ type: "DOMContentLoaded", bubbles: false });
  return f;
}

test("[RUNTIME] Review menu is built once on DOMContentLoaded with seven items", function () {
  h.resetUndo();
  h.clearStorage();
  buildMenu();
  var menu = doc.getElementById("slides-review-menu");
  assert(menu, "review menu missing");
  equal(menu.getAttribute("role"), "menu", "menu role");
  equal(menu.querySelectorAll(".slides-review-item").length, 7, "seven menu items");
  var trigger = doc.getElementById("slides-review-btn");
  equal(trigger.getAttribute("aria-haspopup"), "true", "has popup");
  equal(trigger.getAttribute("aria-expanded"), "false", "starts collapsed");
  assert(menu.hasAttribute("hidden"), "menu hidden initially");
  doc.dispatchEvent({ type: "DOMContentLoaded", bubbles: false });
  equal(doc.querySelectorAll("#slides-review-menu").length, 1, "build is idempotent");
});

test("[RUNTIME] Review menu opens, focuses first item and closes on Escape", function () {
  h.resetUndo();
  h.clearStorage();
  buildMenu();
  var trigger = doc.getElementById("slides-review-btn");
  var menu = doc.getElementById("slides-review-menu");
  trigger.click();
  equal(menu.hasAttribute("hidden"), false, "menu opened");
  equal(trigger.getAttribute("aria-expanded"), "true", "expanded state");
  equal(doc.activeElement, menu.querySelector("button"), "first item focused");
  menu.dispatchEvent({ type: "keydown", key: "ArrowDown", bubbles: false });
  equal(doc.activeElement, menu.querySelectorAll("button")[1], "ArrowDown moved focus");
  menu.dispatchEvent({ type: "keydown", key: "ArrowUp", bubbles: false });
  equal(doc.activeElement, menu.querySelectorAll("button")[0], "ArrowUp moved focus");
  menu.dispatchEvent({ type: "keydown", key: "Escape", bubbles: false });
  equal(menu.hasAttribute("hidden"), true, "Escape closed menu");
  equal(doc.activeElement, trigger, "focus returned to trigger");
});

test("[RUNTIME] menu items dispatch the matching shipped review action", function () {
  h.resetUndo();
  h.clearStorage();
  buildMenu();
  var menu = doc.getElementById("slides-review-menu");
  menu.querySelectorAll("button")[4].click();
  assert(doc.querySelector(".slides-notes-panel"), "notes item opened notes");
  buildMenu();
  doc.getElementById("slides-review-menu").querySelectorAll("button")[6].click();
  assert(doc.querySelector(".slides-tools-dialog"), "properties item opened dialog");
  assert(doc.getElementById("slides-review-menu").hasAttribute("hidden"), "menu closed after action");
});

test("[RUNTIME] htmx:afterSwap restores annotations without the optional bulk module", function () {
  h.resetUndo();
  h.clearStorage();
  buildMenu();
  doc.dispatchEvent({ type: "htmx:afterSwap", bubbles: false });
  h.flushTimers();
  equal(doc.getElementById("slides-review-menu").hasAttribute("hidden"), true,
    "restore did not reopen the menu");
  ["outline", "find", "sections", "notes", "accessibility", "properties", "bulk"].forEach(function (name) {
    equal(typeof h.R[name], "function", "GBSlidesReview." + name + " exported");
  });
});

test("[STATIC] review bootstrap defers bulk to the optional shipped module", function () {
  var src = h.readFile("slides/modules/28_outline_init.js");
  assert(src.indexOf("window.GBSlidesBulk.open()") >= 0, "bulk entry point present");
  assert(src.indexOf("if (window.GBSlidesBulk)") >= 0, "bulk restore is guarded");
  assert(src.indexOf("DOMContentLoaded") >= 0 && src.indexOf("htmx:afterSwap") >= 0,
    "bootstrap listeners missing");
});
