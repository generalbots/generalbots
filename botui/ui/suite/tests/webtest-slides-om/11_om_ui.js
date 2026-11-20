/* Slides object model — media ops, canvas/dialog boot wiring and source gates.
 * RUNTIME checks execute shipped bodies; STATIC checks are labelled and only
 * inspect shipped source for DOM-only/backend-deferred behaviour. */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var e = h.env();
var OM = e.OM;
var Ops = e.Ops;
var gb = e.gbCalls;

function shape(id, x, y, w, hh, extra) {
  return Object.assign({
    id: id, type: "shape", x: x, y: y, w: w, h: hh, z: 1,
    groupId: null, locked: false, hidden: false
  }, extra || {});
}
function seed(objects, groups, guides) {
  var s = OM.slide(0);
  s.size = { id: "16:9", w: 960, h: 540 };
  s.objects = objects || [];
  s.groups = groups || [];
  s.guides = guides || [];
  return s;
}
function buttonFor(key) {
  return e.document.body.querySelector("[data-i18n='" + key + "']");
}

// ── Media operations (pure, shared checkpoint) ───────────────────────
test("RUNTIME: smartGuides snaps to nearby edges and honours its threshold", function () {
  h.reset();
  seed([shape("fixed", 100, 100, 100, 100), shape("moving", 201, 190, 60, 60)]);
  var snapped = Ops.smartGuides(["moving"], -3, 12, 6);
  equal(snapped.dx, -1, "x snap adjustment wrong");
  equal(snapped.dy, 10, "y snap adjustment wrong");
  assert(snapped.guides.length === 2 && snapped.guides[0].at === 200 && snapped.guides[1].at === 200,
    "guide positions wrong: " + JSON.stringify(snapped.guides));
  var loose = Ops.smartGuides(["moving"], -3, 12, 1);
  equal(loose.dx, -3, "threshold should reject a two-pixel snap");
  equal(loose.guides.length, 0, "rejected snap should emit no guides");
  Ops.setting("smartGuides", false);
  var off = Ops.smartGuides(["moving"], -3, 12, 6);
  assert(off.dx === -3 && off.dy === 12 && off.guides.length === 0, "disabled smart guides still snapped");
});

test("RUNTIME: connectors follow anchors when an endpoint moves", function () {
  h.reset();
  var s = seed([shape("a", 0, 0, 100, 50), shape("b", 200, 0, 100, 50)]);
  var id = Ops.addConnector("a", "b", "right", "left", { stroke: "<ink>" });
  var con = s.objects[2];
  equal(con.id, id, "connector id mismatch");
  assert(con.from.anchor === "right" && con.to.anchor === "left", "anchors not stored");
  assert(con.x === 100 && con.y === 25 && con.w === 100 && con.h === 0, "connector geometry wrong");
  equal(con.stroke, "<ink>", "style override lost");
  OM.move(s, ["a"], 10, 0);
  equal(con.x, 110, "connector did not follow its endpoint");
  equal(con.w, 90, "connector width not recomputed");
});

test("RUNTIME: addFreeform bounds its points and clones the point array", function () {
  h.reset();
  var points = [{ x: 50, y: 60 }, { x: 150, y: 120 }, { x: 100, y: 30 }];
  var id = Ops.addFreeform(points, { stroke: "<line>" });
  var node = OM.slide(0).objects[0];
  assert(node.type === "freeform" && node.id === id, "freeform not created");
  assert(node.x === 50 && node.y === 30 && node.w === 100 && node.h === 90, "freeform bounds wrong");
  points[0].x = 999;
  equal(node.points[0].x, 50, "points aliased into the caller array");
  equal(node.stroke, "<line>", "freeform style lost");
  assert(Ops.addFreeform([{ x: 0, y: 0 }]) === null, "one-point freeform should be rejected");
});

test("RUNTIME: crop creates normalized crop state, mask/replace/reset round-trip", function () {
  h.reset();
  seed([shape("img", 10, 20, 100, 80, { type: "image", src: "<src>" })]);
  var id = "img";
  assert(Ops.crop(id, { w: 0.5, h: 0.25, shape: "circle" }), "crop refused an image");
  var crop = OM.slide(0).objects[0].crop;
  assert(crop.x === 0 && crop.y === 0 && crop.w === 0.5 && crop.h === 0.25, "crop rect wrong");
  assert(JSON.stringify(crop.original) === JSON.stringify({ x: 10, y: 20, w: 100, h: 80, src: "<src>" }),
    "crop original wrong");
  Ops.mask(id, "rounded");
  equal(crop.shape, "rounded", "mask did not delegate to crop");
  Ops.replaceImage(id, "<next>");
  equal(OM.slide(0).objects[0].src, "<next>", "image not replaced");
  equal(crop.original.src, "<next>", "crop original source not updated");
  assert(Ops.resetCrop(id) && OM.slide(0).objects[0].crop === null, "crop not cleared");
  assert(OM.slide(0).objects[0].w === 100 && OM.slide(0).objects[0].src === "<next>", "original geometry not restored");
  assert(!Ops.crop("missing", { w: 1 }) && !Ops.resetCrop(id), "invalid crop targets should be refused");
});

test("KNOWN-BUG (runtime): image crop patch is not clamped to 0..1", function () {
  h.reset();
  seed([shape("img", 0, 0, 100, 80, { type: "image" })]);
  Ops.crop("img", { x: -2, w: 5 });
  var crop = OM.slide(0).objects[0].crop;
  equal(crop.x, -2, "shipped crop clamped x (revisit bug)");
  equal(crop.w, 5, "shipped crop clamped w (revisit bug)");
  /* The dialog clamps width/height to <=1 but never x/y or the lower bound;
   * Ops.crop itself accepts any patch value. */
});

test("RUNTIME: hyperlink, alt text and line style are single mutations", function () {
  h.reset();
  seed([shape("a", 0, 0, 100, 50), shape("b", 200, 0, 100, 50)]);
  assert(Ops.hyperlink("a", "url", "<target>") && Ops.alt("a", "<desc>", true), "link/alt refused");
  assert(Ops.setLineStyle({ lineStyle: "dashed", arrowEnd: true }, ["a", "b"]), "line style refused");
  var a = OM.slide(0).objects[0];
  assert(a.hyperlink.kind === "url" && a.hyperlink.target === "<target>", "hyperlink not stored");
  assert(a.alt === "<desc>" && a.decorative === true, "alt text not stored");
  assert(a.lineStyle === "dashed" && a.arrowEnd && OM.slide(0).objects[1].lineStyle === "dashed",
    "line style did not reach all targets");
  equal(OM.undoStack.length, 3, "each media mutation should add one checkpoint");
});

test("RUNTIME: setSlideSize rescales geometry and rejects invalid sizes", function () {
  h.reset();
  seed([shape("a", 100, 50, 200, 100)], [], [{ axis: "x", at: 480 }]);
  var result = Ops.setSlideSize("4:3", 0);
  assert(result.scale.sx === 0.75 && result.scale.sy === 1, "scale report wrong");
  assert(OM.slide(0).objects[0].x === 75 && OM.slide(0).objects[0].w === 150, "shape not rescaled");
  equal(OM.slide(0).guides[0].at, 360, "guide not rescaled");
  assert(Ops.setSlideSize({ w: "<bad>", h: 1 }, 0) === null, "invalid size should be refused");
  assert(Ops.setSlideSize({ id: "custom", w: 800, h: 600 }, 0).to.w === 800, "custom size refused");
});

test("KNOWN-BUG (runtime): setSlideSize rescales normalized crop rectangles", function () {
  h.reset();
  seed([shape("img", 0, 0, 100, 80, { type: "image", src: "<src>" })]);
  Ops.crop("img", { w: 1, h: 1 });
  Ops.setSlideSize("4:3", 0);
  equal(OM.slide(0).objects[0].crop.w, 0.75, "shipped crop stayed normalized (revisit bug)");
  /* Crop w/h are fractions of the source image; multiplying them by the
   * slide-size ratio changes the crop instead of keeping it at 100%. */
});

// ── Canvas render + boot wiring ──────────────────────────────────────
test("RUNTIME: canvas renders local-only layers, selection, connectors and guides", function () {
  h.reset();
  var s = seed([shape("a", 10, 20, 100, 50, { z: 2 }), shape("child", 300, 20, 50, 50, { groupId: "g" })],
    [{ id: "g", parentId: null, children: ["child"], z: 1, locked: false, hidden: false }],
    [{ axis: "x", at: 120 }]);
  Ops.addConnector("a", "child", "right", "left");
  OM.setSelection(["a"]);
  e.Canvas.render();
  var layer = e.canvas.querySelector(".sl-om-layer");
  assert(layer && layer.getAttribute("role") === "group", "layer role missing");
  equal(layer.getAttribute("aria-label"), "Object layer (local only)", "layer label not resolved through gbT");
  assert(layer.classList.contains("show-rulers"), "rulers setting not applied");
  var nodes = layer.querySelector(".sl-om-objects");
  equal(nodes.children.length, 3, "expected two top-level nodes plus a connector");
  var a = nodes.querySelector('[data-om-id="a"]');
  assert(a && a.style.left === "10px" && a.style.zIndex === "2", "node geometry/style wrong");
  assert(a.classList.contains("is-selected"), "selection class missing");
  assert(nodes.querySelector('[data-om-id="g"]'), "group node missing");
  var lines = layer.querySelector(".sl-om-connectors");
  var line = lines.querySelectorAll("line")[0];
  assert(line && line.getAttribute("x1") === "110" && line.getAttribute("x2") === "300", "connector line wrong");
  assert(lines.querySelector("defs"), "arrow marker definition missing");
  equal(layer.querySelector(".sl-om-guides").children.length, 1, "guide line missing");
  assert(gb.some(function (call) { return call.key === "slides-om-layer-aria"; }), "layer label not requested via gbT");
  assert(s.guides.length === 1, "render mutated guide state");
});

test("RUNTIME: grid setting toggles local layer classes only", function () {
  h.reset();
  seed([]);
  Ops.setting("grid", true);
  e.Canvas.render();
  var layer = e.canvas.querySelector(".sl-om-layer");
  assert(layer.classList.contains("show-grid") && layer.classList.contains("snap-grid"), "grid/snap classes missing");
  Ops.setting("grid", false);
  e.Canvas.render();
  assert(!layer.classList.contains("show-grid"), "grid class not removed");
  e.Canvas.clearSmartGuides();
  e.Canvas.render();
  assert(layer, "clearSmartGuides broke rendering");
});

test("RUNTIME: canvas keyboard nudges and deletes only the selection", function () {
  h.reset();
  seed([shape("a", 100, 100, 100, 50), shape("b", 500, 100, 100, 50)]);
  OM.setSelection(["a"]);
  h.dispatch(e.document, "keydown", { key: "ArrowRight" });
  equal(OM.slide(0).objects[0].x, 110, "ten-pixel nudge failed");
  h.dispatch(e.document, "keydown", { key: "ArrowDown", shiftKey: true });
  equal(OM.slide(0).objects[0].y, 101, "one-pixel nudge failed");
  h.dispatch(e.document, "keydown", { key: "Delete" });
  equal(OM.slide(0).objects.length, 1, "delete did not remove only the selection");
  equal(OM.slide(0).objects[0].id, "b", "wrong object deleted");
});

test("RUNTIME: rubber-band selection traverses top-level objects by rect overlap", function () {
  h.reset();
  seed([shape("a", 10, 10, 50, 50), shape("b", 100, 100, 50, 50), shape("c", 300, 300, 50, 50)]);
  e.Canvas.render();
  var layer = e.canvas.querySelector(".sl-om-layer");
  h.dispatch(e.canvas, "pointerdown", { button: 0, target: layer, clientX: 0, clientY: 0 });
  h.dispatch(e.window, "pointermove", { clientX: 200, clientY: 200 });
  h.dispatch(e.window, "pointerup", { clientX: 200, clientY: 200 });
  equal(OM.selection.join(","), "a,b", "rubber band selected the wrong nodes");
});

// ── Dialog + init wiring ─────────────────────────────────────────────
test("RUNTIME: dialog opens with a11y roles, polite status and no native prompts", function () {
  h.reset();
  e.Dialog.open();
  assert(e.Dialog.isOpen(), "dialog did not open");
  var probe = e.Dialog.probe();
  equal(probe.role, "dialog", "dialog role wrong");
  equal(probe.ariaModal, "true", "aria-modal missing");
  equal(probe.statusRole, "status", "status role missing");
  equal(probe.statusLive, "polite", "status is not polite");
  assert(probe.focusTrap && probe.restoresFocus && probe.localBadge, "focus/badge probe failed");
  assert(gb.some(function (call) { return call.key === "slides-om-title"; }), "title not resolved via gbT");
  equal(e.window.__prompts, 0, "native prompt used by the dialog");
  e.Dialog.close();
  assert(!e.Dialog.isOpen(), "dialog did not close");
});

test("RUNTIME: media section buttons call shipped ops and clamp crop input", function () {
  h.reset();
  seed([shape("img", 0, 0, 100, 80, { type: "image" })]);
  OM.setSelection(["img"]);
  var groupCalls = [];
  var cropCalls = [];
  var originalGroup = Ops.group;
  var originalCrop = Ops.crop;
  Ops.group = function (ids) { groupCalls.push(ids.slice()); return 1; };
  Ops.crop = function (id, patch) { cropCalls.push([id, patch]); return true; };
  try {
    e.Dialog.open();
    var groupButton = buttonFor("slides-om-group");
    assert(groupButton, "group control missing from the dialog");
    h.dispatch(groupButton, "click", {});
    assert(groupCalls.length === 1 && groupCalls[0].join(",") === "img", "group button did not call Ops.group");
    var width = e.document.body.querySelector('[data-i18n-aria-label="slides-om-crop-width"]');
    assert(width, "crop width input missing");
    width.value = "5";
    h.dispatch(buttonFor("slides-om-crop"), "click", {});
    assert(cropCalls.length === 1 && cropCalls[0][0] === "img" && cropCalls[0][1].w === 1,
      "crop input not clamped by dialog: " + JSON.stringify(cropCalls));
    assert(e.document.body.querySelector(".sl-om-status").getAttribute("aria-live") === "polite",
      "live status region missing");
  } finally {
    Ops.group = originalGroup;
    Ops.crop = originalCrop;
    e.Dialog.close();
  }
});

test("RUNTIME: toolbar click and dialog-scoped undo/redo/select-all shortcuts", function () {
  h.reset();
  h.dispatch(e.toolbar, "click", {});
  assert(e.Dialog.isOpen(), "toolbar did not open the editor");
  Ops.addShape("shape", 10, 10);
  h.dispatch(e.document, "keydown", { key: "z", ctrlKey: true });
  equal(OM.slide(0).objects.length, 0, "Ctrl+Z did not undo while the dialog was open");
  h.dispatch(e.document, "keydown", { key: "y", ctrlKey: true });
  equal(OM.slide(0).objects.length, 1, "Ctrl+Y did not redo");
  h.dispatch(e.document, "keydown", { key: "a", ctrlKey: true });
  equal(OM.selection.length, 1, "Ctrl+A did not select all");
  e.Dialog.close();
  h.dispatch(e.document, "keydown", { key: "z", ctrlKey: true });
  equal(OM.slide(0).objects.length, 1, "shortcut leaked outside the open dialog");
});

// ── STATIC source assertions (DOM-only / backend-deferred behaviour) ─
test("STATIC: pure OM modules are DOM-free and mutate only through checkpoints", function () {
  ["slides/modules/46_om_state.js", "slides/modules/47_om_ops.js",
    "slides/modules/47_om_ops_media.js"].forEach(function (rel) {
    var src = h.source(rel);
    assert(src.indexOf("fetch(") < 0 && src.indexOf("XMLHttpRequest") < 0, rel + " performs network I/O");
    assert(src.indexOf("innerHTML") < 0 && src.indexOf("eval(") < 0, rel + " writes markup or evaluates code");
    assert(src.indexOf("window.alert") < 0, rel + " uses a native prompt");
    if (rel.indexOf("46_") >= 0) return;
    assert(src.indexOf("document") < 0 && src.indexOf("localStorage") < 0, rel + " is not DOM-free");
    var mutations = (src.match(/OM\.mutate\(/g) || []).length;
    assert(mutations >= 8, rel + " has only " + mutations + " checkpointed mutations");
  });
  assert(h.source("slides/modules/46_om_state.js").indexOf('"slides:object-model:v1:"') >= 0,
    "deck-local store prefix missing");
});

test("STATIC: canvas keeps objects in local-only .sl-om classes", function () {
  var src = h.source("slides/modules/48_om_canvas.js");
  ["sl-om-layer", "sl-om-node", "sl-om-connectors", "sl-om-guides", "sl-om-rubber",
    "data-om-id", "aria-hidden"].forEach(function (needle) {
    assert(src.indexOf('"' + needle) >= 0 || src.indexOf('"' + needle + '"') >= 0,
      "canvas marker missing: " + needle);
  });
  assert(src.indexOf('"sl-element"') < 0, "canvas writes server-persisted element classes");
  assert(src.indexOf("fetch(") < 0 && src.indexOf("persistElement") < 0, "canvas writes to the server");
  assert(src.indexOf('"slides-om-local-only"') >= 0, "local-only badge key missing");
});

test("STATIC: dialog shell declares a11y roles, data-i18n and no native prompts", function () {
  var shell = h.source("slides/modules/49_om_dialog.js");
  ['"role", "dialog"', '"aria-modal"', '"role", "status"', '"aria-live", "polite"',
    '"Escape"', '"Tab"', "previous.focus", '"data-i18n"'].forEach(function (needle) {
    assert(shell.indexOf(needle) >= 0, "dialog a11y marker missing: " + needle);
  });
  assert(!new RegExp("\\b(alert|confirm|prompt)\\s*\\(").test(shell), "dialog uses a native prompt");
  var media = h.source("slides/modules/49_om_dialog_media.js");
  assert(media.indexOf('"data-i18n-aria-label"') >= 0 && media.indexOf('"data-i18n-placeholder"') >= 0,
    "media dialog does not tag translated aria/placeholder slots");
  assert(media.indexOf("UI.sections.objects") >= 0 && media.indexOf("UI.sections.size") >= 0,
    "media sections are not registered with the shell");
});

test("STATIC: init shortcuts are scoped to the open dialog", function () {
  var src = h.source("slides/modules/50_om_init.js");
  assert(src.indexOf("Dialog.isOpen()") >= 0, "shortcuts are not guarded by dialog state");
  assert(src.indexOf("stopImmediatePropagation") >= 0 && src.indexOf("__omInitKeyBound") >= 0,
    "shortcut capture or one-time binding missing");
  assert(src.indexOf('"slObjectModelBtn"') >= 0, "toolbar binding missing");
  assert(src.indexOf("OM.init()") >= 0, "state is not initialised on boot");
});
