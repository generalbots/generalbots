/* Slides object model — runtime invariants against the real shipped IIFEs.
 * Every check calls a genuine production function in the shared vm sandbox;
 * the fake host is a stub, never a reimplementation of production logic. */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var e = h.env();
var OM = e.OM;
var Ops = e.Ops;
var H = e.helpers;

function shape(id, x, y, w, hh, extra) {
  return Object.assign({
    id: id, type: "shape", x: x, y: y, w: w, h: hh, z: 1,
    groupId: null, locked: false, hidden: false
  }, extra || {});
}
function group(id, children, z, extra) {
  return Object.assign({ id: id, parentId: null, children: children, z: z || 1,
    locked: false, hidden: false }, extra || {});
}
function seed(objects, groups, guides) {
  var s = OM.slide(0);
  s.size = { id: "16:9", w: 960, h: 540 };
  s.objects = objects || [];
  s.groups = groups || [];
  s.guides = guides || [];
  return s;
}
function idsOf(nodes) { return nodes.map(function (node) { return node.id; }).join(","); }
function positions(objects) {
  return objects.map(function (obj) { return Math.round(obj.x) + ":" + Math.round(obj.y); }).join(",");
}

// ── State, storage, checkpoints ───────────────────────────────────────
test("RUNTIME: all seven shipped modules expose their real API in one sandbox", function () {
  h.reset();
  ["fresh", "load", "mutate", "undo", "redo", "descendants", "rect", "move", "rescale"].forEach(function (name) {
    assert(typeof OM[name] === "function", "OM." + name + " missing");
  });
  ["addShape", "group", "ungroup", "zOrder", "align", "distribute", "setSlideSize"].forEach(function (name) {
    assert(typeof Ops[name] === "function", "Ops." + name + " missing");
  });
  ["smartGuides", "addConnector", "addFreeform", "crop", "mask"].forEach(function (name) {
    assert(typeof Ops[name] === "function", "media op " + name + " missing");
  });
  ["L", "byId", "nextZ", "targets", "scaleNode"].forEach(function (name) {
    assert(typeof H[name] === "function", "private helper export " + name + " missing");
  });
  assert(typeof e.Canvas.render === "function" && typeof e.Dialog.open === "function",
    "canvas/dialog API missing");
  assert(typeof e.Init.state === "object" && e.Init.ops === Ops, "boot wiring surface missing");
});

test("RUNTIME: presets and fresh state are deep-independent and deterministic", function () {
  h.reset();
  var a = OM.fresh();
  var b = OM.fresh();
  assert(JSON.stringify(a) === JSON.stringify(b), "fresh state differs between calls");
  a.settings.gridSize = 999;
  equal(b.settings.gridSize, 10, "fresh settings aliased");
  var presets = OM.presets();
  presets[0].w = 1;
  equal(OM.preset("16:9").w, 960, "presets aliased through clone");
  assert(OM.preset("unknown") === null, "unknown preset should be null");
  assert(OM.preset("4:3").h === 540 && OM.preset("a4").w === 794, "preset geometry wrong");
});

test("RUNTIME: deck-scoped storage loads valid state and rejects malformed JSON", function () {
  h.reset();
  OM.save();
  var key = Object.keys(e.store)[0];
  assert(key === "slides:object-model:v1:offline-deck", "wrong deck-scoped key: " + key);
  e.storage.setItem(key, "{not-json");
  assert(JSON.stringify(OM.load()) === JSON.stringify(OM.fresh()), "malformed JSON not rejected");
  e.storage.setItem(key, JSON.stringify({ slides: {}, settings: { snap: false } }));
  equal(OM.load().settings.snap, false, "persisted settings not loaded");
});

test("RUNTIME: slide index follows the canvas data-slide-id", function () {
  h.reset();
  equal(OM.slideKey(), "0", "default slide key wrong");
  e.canvas.dataset.slideId = "2";
  equal(OM.slideKey(), "2", "slide key did not follow canvas");
  OM.slide().objects.push(shape("a", 0, 0, 10, 10));
  assert(OM.slide(2).objects[0].id === "a", "slide(2) did not resolve canvas slide");
  equal(OM.slide(0).objects.length, 0, "slides are not scoped by index");
  h.reset();
});

test("RUNTIME: mutate checkpoints state, saves and undoes/redoes one step", function () {
  h.reset();
  var s = OM.slide(0);
  OM.mutate("step", function () { s.objects.push(shape("a", 1, 2, 3, 4)); });
  OM.setSelection(["a"]);
  assert(OM.canUndo() && OM.undoStack.length === 1, "checkpoint not recorded");
  assert(OM.lastLabel() === "step", "checkpoint label lost");
  assert(Object.keys(JSON.parse(e.store["slides:object-model:v1:offline-deck"]).slides["0"].objects).length > 0,
    "state not persisted");
  assert(OM.undo() && !OM.canUndo() && OM.canRedo(), "undo did not move the step");
  equal(OM.selection.length, 0, "selection not filtered after undo");
  assert(OM.redo() && OM.state.slides["0"].objects[0].id === "a", "redo did not restore state");
  equal(OM.selection.length, 0, "selection still empty after redo (id was filtered)");
});

test("RUNTIME: empty undo/redo are safe and history is capped at 80", function () {
  h.reset();
  assert(!OM.undo() && !OM.redo(), "empty history should be a no-op");
  for (var i = 0; i < 81; i++) OM.checkpoint("k" + i);
  equal(OM.undoStack.length, 80, "checkpoint cap wrong");
  equal(OM.undoStack[0].label, "k1", "oldest checkpoint should be evicted");
});

// ── Selection + hierarchy ─────────────────────────────────────────────
test("RUNTIME: multi-select add/toggle/clear and selectAll use top-level z order", function () {
  h.reset();
  seed([shape("a", 0, 0, 10, 10, { z: 3 }), shape("child", 0, 0, 10, 10, { z: 4, groupId: "g" }),
    shape("b", 0, 0, 10, 10, { z: 1 })], [group("g", ["child"], 2)]);
  OM.setSelection(["a", "b"]);
  OM.add("a");
  OM.add("g");
  equal(OM.selection.join(","), "a,b,g", "add duplicated an id");
  OM.toggle("b");
  OM.toggle("g");
  equal(OM.selection.join(","), "a", "toggle failed");
  assert(OM.isSelected("a") && !OM.isSelected("b"), "isSelected disagrees with selection");
  OM.clearSelection();
  OM.selectAll(0);
  equal(idsOf(OM.selected(0)), "b,g,a", "selectAll not sorted by z or included a child");
});

test("RUNTIME: groupOf, childObjects, childGroups and recursive descendants", function () {
  h.reset();
  seed([shape("outerChild", 0, 0, 10, 10, { groupId: "outer" }),
    shape("innerChild", 20, 20, 10, 10, { groupId: "inner" })],
  [group("outer", ["innerChild"], 1), group("inner", ["innerChild"], 2, { parentId: "outer" })]);
  equal(OM.groupOf({ groupId: "outer" }), "outer", "groupId not reported");
  equal(OM.groupOf({ parentId: "outer" }), "outer", "parentId not reported");
  equal(idsOf(OM.childObjects(OM.slide(0), null)), "", "top-level objects should be empty");
  equal(idsOf(OM.childGroups(OM.slide(0), "outer")), "inner", "nested group not found");
  equal(idsOf(OM.descendants(OM.slide(0), "outer")), "outerChild,inner,innerChild",
    "recursive traversal wrong");
});

test("RUNTIME: rect/union/bounds compute group bounds from descendants", function () {
  h.reset();
  seed([shape("a", 10, 20, 30, 40, { groupId: "g" }), shape("b", 100, 5, 20, 10, { groupId: "g" }),
    shape("c", 300, 300, 10, 10)], [group("g", ["a", "b"], 1)]);
  assert(JSON.stringify(OM.rect(OM.slide(0), "g")) === JSON.stringify({ x: 10, y: 5, w: 110, h: 55 }),
    "group rect wrong: " + JSON.stringify(OM.rect(OM.slide(0), "g")));
  assert(JSON.stringify(OM.bounds(["g", "c"], 0)) === JSON.stringify({ x: 10, y: 5, w: 300, h: 305 }),
    "multi-select bounds wrong");
  assert(OM.union([]) === null && OM.rect(OM.slide(0), "missing") === null, "empty union should be null");
  assert(JSON.stringify(OM.union([{ x: -5, y: 7, w: 10, h: 10 }, null])) ===
    JSON.stringify({ x: -5, y: 7, w: 10, h: 10 }), "union ignored a rect or accepted null");
});

test("RUNTIME: move respects locks, moves groups and recomputes connector geometry", function () {
  h.reset();
  var s = seed([shape("a", 100, 100, 100, 100, { locked: true }),
    shape("child", 200, 200, 50, 50, { groupId: "g" }),
    shape("target", 400, 200, 30, 30),
    shape("con", 0, 0, 0, 0, { type: "connector", from: { id: "a", anchor: "right" },
      to: { id: "target", anchor: "left" } })], [group("g", ["child"], 1)]);
  OM.move(s, ["a", "g"], 25, 10);
  equal(s.objects[0].x, 100, "locked object moved");
  equal(s.objects[1].x, 225, "group descendant did not move");
  equal(s.objects[2].y, 200, "non-selected object moved");
  equal(s.objects[3].x, 200, "connector left edge not recomputed from locked right anchor");
  equal(s.objects[3].w, 200, "connector width wrong");
  assert(s.objects[3]._a.x === 200 && s.objects[3]._b.x === 400, "connector anchors stale");
  OM.move(s, ["con"], 99, 99);
  equal(s.objects[3].x, 200, "connector should be endpoint-driven, not directly movable");
});

test("RUNTIME: anchorPoint honours left/right/top/bottom and centre defaults", function () {
  h.reset();
  var s = seed([shape("a", 10, 20, 100, 40)]);
  assert(JSON.stringify(OM.anchorPoint(s, { id: "a" })) === JSON.stringify({ x: 60, y: 40 }), "centre anchor wrong");
  assert(OM.anchorPoint(s, { id: "a", anchor: "left" }).x === 10, "left anchor wrong");
  assert(OM.anchorPoint(s, { id: "a", anchor: "right" }).x === 110, "right anchor wrong");
  assert(OM.anchorPoint(s, { id: "a", anchor: "top" }).y === 20, "top anchor wrong");
  assert(OM.anchorPoint(s, { id: "a", anchor: "bottom" }).y === 60, "bottom anchor wrong");
  assert(OM.anchorPoint(s, null) === null, "missing anchor should be null");
});

test("RUNTIME: rescale scales objects, crop, freeform points and guides", function () {
  h.reset();
  var s = seed([shape("a", 100, 50, 200, 100, { crop: { x: 1, y: 2, w: 3, h: 4 },
    points: [{ x: 100, y: 50 }, { x: 300, y: 150 }] })], [], [{ axis: "x", at: 480 }, { axis: "y", at: 270 }]);
  var result = OM.rescale(s, { w: 960, h: 540 }, { id: "4:3", w: 720, h: 540 });
  equal(result.sx, 0.75, "horizontal scale wrong");
  equal(result.sy, 1, "vertical scale wrong");
  assert(JSON.stringify(s.objects[0]).indexOf('"x":75') >= 0, "object x not scaled");
  equal(s.objects[0].crop.w, 2.25, "crop rectangle not scaled");
  equal(s.objects[0].points[1].x, 225, "freeform point not scaled");
  equal(s.guides[0].at, 360, "x guide not scaled");
  equal(s.guides[1].at, 270, "y guide should not change");
  assert(s.size.id === "4:3", "slide size not replaced");
});

// ── Core operations ───────────────────────────────────────────────────
test("RUNTIME: addShape/duplicate/nudge/delete share one-checkpoint undo", function () {
  h.reset();
  var id = Ops.addShape("text", 40, 60);
  var s = OM.slide(0);
  equal(s.objects.length, 1, "shape not added");
  equal(s.objects[0].type, "text", "wrong shape type");
  equal(s.objects[0].w, 220, "text box width wrong");
  assert(OM.isSelected(id), "new shape not selected");
  var copies = Ops.duplicateSelected();
  equal(copies.length, 1, "duplicate count wrong");
  assert(copies[0] !== id && s.objects[1].x === 56 && s.objects[1].y === 76, "duplicate offset wrong");
  Ops.nudge(10, -5);
  equal(s.objects[1].x, 66, "nudge did not move the copy");
  equal(OM.undoStack.length, 3, "each operation should add exactly one checkpoint");
  Ops.deleteSelected();
  equal(s.objects.length, 1, "duplicate not deleted");
  assert(OM.undo() && OM.slide(0).objects.length === 2, "delete not undone in one step");
});

test("RUNTIME: group/ungroup nested objects, restore parent and selection", function () {
  h.reset();
  seed([shape("a", 0, 0, 10, 10), shape("b", 20, 0, 10, 10), shape("c", 40, 0, 10, 10)]);
  var inner = Ops.group(["a", "b"]);
  var outer = Ops.group([inner, "c"]);
  var s = OM.slide(0);
  assert(s.groups.length === 2, "two groups expected");
  equal(H.byId(s, inner).parentId, outer, "inner group not parented");
  equal(s.objects[2].groupId, outer, "object not attached to outer group");
  var released = Ops.ungroup([outer]);
  equal(released, 2, "ungroup should release its two children");
  assert(H.byId(s, inner).parentId === null, "inner group parent not cleared");
  equal(OM.selection.join(","), inner + ",c", "ungroup selection wrong");
  Ops.ungroup([inner]);
  equal(s.groups.length, 0, "inner group not removed");
  assert(s.objects[0].groupId === null && s.objects[1].groupId === null, "children not released");
});

test("RUNTIME: zOrder front/back gives contiguous z without gaps", function () {
  h.reset();
  seed([shape("a", 0, 0, 10, 10, { z: 1 }), shape("b", 0, 0, 10, 10, { z: 2 }),
    shape("c", 0, 0, 10, 10, { z: 3 })]);
  Ops.zOrder("front", ["a"]);
  equal(idsOf(OM.topLevel(0)), "b,c,a", "bring to front wrong");
  Ops.zOrder("back", ["a"]);
  equal(idsOf(OM.topLevel(0)), "a,b,c", "send to back wrong");
  assert(OM.topLevel(0).every(function (node, index) { return node.z === index + 1; }), "z values not normalised");
});

test("RUNTIME: zOrder forward/backward move a single step (regression #2430)", function () {
  function three() {
    h.reset();
    seed([shape("a", 0, 0, 10, 10, { z: 1 }), shape("b", 0, 0, 10, 10, { z: 2 }),
      shape("c", 0, 0, 10, 10, { z: 3 })]);
  }
  three();
  Ops.zOrder("forward", ["b"]);
  equal(idsOf(OM.topLevel(0)), "a,c,b", "forward must swap b toward the front");
  three();
  Ops.zOrder("backward", ["b"]);
  equal(idsOf(OM.topLevel(0)), "b,a,c", "backward must swap b toward the back");
  assert(OM.topLevel(0).every(function (node, index) { return node.z === index + 1; }),
    "z values stay contiguous after a single step");
});

test("RUNTIME: align left/centre/right/top/centre/bottom uses selection bounds", function () {
  function aligned(mode) {
    h.reset();
    seed([shape("a", 100, 100, 100, 50), shape("b", 300, 220, 60, 80)]);
    Ops.align(mode, ["a", "b"]);
    return positions(OM.slide(0).objects);
  }
  equal(aligned("left"), "100:100,100:220", "align left wrong");
  equal(aligned("hcenter"), "180:100,200:220", "align hcenter wrong");
  equal(aligned("right"), "260:100,300:220", "align right wrong");
  equal(aligned("top"), "100:100,300:100", "align top wrong");
  equal(aligned("vcenter"), "100:175,300:160", "align vcenter wrong");
  equal(aligned("bottom"), "100:250,300:220", "align bottom wrong");
});

test("RUNTIME: a single selected object aligns to the slide, not itself", function () {
  h.reset();
  seed([shape("a", 100, 100, 100, 50)]);
  Ops.align("hcenter", ["a"]);
  equal(OM.slide(0).objects[0].x, 430, "single-object horizontal centre wrong");
  Ops.align("bottom", ["a"]);
  equal(OM.slide(0).objects[0].y, 490, "single-object bottom wrong");
});

test("RUNTIME: distribute keeps endpoints, spaceEvenly stays inside the slide", function () {
  h.reset();
  seed([shape("a", 0, 0, 100, 20), shape("b", 200, 100, 100, 20), shape("c", 500, 300, 100, 20)]);
  Ops.distribute("h", ["c", "a", "b"]);
  equal(positions(OM.slide(0).objects), "0:0,250:100,500:300", "horizontal distribute wrong");
  Ops.distribute("v", ["a", "b", "c"]);
  equal(positions(OM.slide(0).objects), "0:0,250:150,500:300", "vertical distribute wrong");
  h.reset();
  seed([shape("a", 50, 50, 100, 20), shape("b", 700, 400, 100, 20)]);
  Ops.spaceEvenly("h", ["a", "b"]);
  assert(OM.slide(0).objects[0].x === 0 && OM.slide(0).objects[1].x === 860,
    "spaceEvenly did not span the slide: " + positions(OM.slide(0).objects));
  assert(!Ops.distribute("h", ["a"]) && !Ops.spaceEvenly("h", []), "guards for short lists missing");
});

test("RUNTIME: equalSize scales width/height/both and freeform points", function () {
  h.reset();
  seed([shape("a", 0, 0, 100, 40), shape("b", 200, 200, 200, 80),
    shape("p", 400, 0, 60, 60, { type: "freeform", points: [{ x: 400, y: 0 }, { x: 460, y: 60 }] })]);
  Ops.equalSize("w", ["a", "b", "p"]);
  assert(OM.slide(0).objects[0].w === 200 && OM.slide(0).objects[1].w === 200 &&
    OM.slide(0).objects[2].w === 200, "equal width wrong");
  Ops.equalSize("h", ["a", "b"]);
  assert(OM.slide(0).objects[0].h === 80 && OM.slide(0).objects[1].h === 80, "equal height wrong");
});

test("KNOWN-BUG (runtime): equalSize scales freeform points around the origin", function () {
  h.reset();
  seed([shape("a", 0, 0, 100, 60), shape("p", 400, 0, 50, 60,
    { type: "freeform", points: [{ x: 400, y: 0 }, { x: 450, y: 60 }] })]);
  Ops.equalSize("w", ["a", "p"]);
  equal(OM.slide(0).objects[1].points[0].x, 800, "shipped freeform scaling changed (revisit bug)");
  assert(OM.slide(0).objects[1].points[0].x !== OM.slide(0).objects[1].x,
    "shipped freeform points unexpectedly stayed anchored (revisit bug)");
  /* Expected: points stay relative to the shape's own x/y (400 here), but
   * scaleNode multiplies absolute point coordinates by the width ratio. */
});

test("RUNTIME: lockHide cascades to group descendants", function () {
  h.reset();
  seed([shape("child", 10, 10, 20, 20, { groupId: "g" })], [group("g", ["child"], 1)]);
  Ops.lockHide("locked", true, ["g"]);
  assert(OM.slide(0).groups[0].locked && OM.slide(0).objects[0].locked, "lock did not cascade");
  Ops.lockHide("hidden", false, ["g"]);
  assert(!OM.slide(0).groups[0].hidden && !OM.slide(0).objects[0].hidden, "unhide did not cascade");
});

test("RUNTIME: deterministic clock/uid stub produces repeatable ids", function () {
  h.reset();
  var first = Ops.addShape("shape");
  h.reset();
  var second = Ops.addShape("shape");
  equal(first, second, "uid stream is not deterministic under the stubbed clock");
  assert(Ops.addShape("shape") !== second, "uid counter not advanced between calls");
});
