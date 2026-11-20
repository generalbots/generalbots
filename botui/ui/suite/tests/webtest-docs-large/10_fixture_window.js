/**
 * RUNTIME #2533: fixture generation + windowed reads/mount.
 * Every assertion executes the shipped modules 84..86; no logic is copied.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var deep = h.deepEqual;
var fixture = f.fixture;

test("RUNTIME #2533: fixture sizes map to exact block counts and nominal bytes", function () {
  var F = fixture().NS.Fixture;
  equal(F.bytesForMb(5), 5 * 1024 * 1024, "5 MB bytes");
  equal(F.bytesForMb(10), 10 * 1024 * 1024, "10 MB bytes");
  equal(F.bytesForMb(20), 20 * 1024 * 1024, "20 MB bytes");
  equal(F.blockCount(5), 5120, "5 MB block count");
  equal(F.blockCount(10), 10240, "10 MB block count");
  equal(F.blockCount(20), 20480, "20 MB block count");
  deep(F.manifest(10), {
    sizeMb: 10, blockCount: 10240, nominalBytes: 10485760, nominalBlockBytes: 1024,
  }, "manifest descriptor is tiny and derived from size");
});

test("RUNTIME #2533: stable block ids and deterministic short block text", function () {
  var F = fixture().NS.Fixture;
  equal(F.blockId(0), "blk-000000", "first id padded");
  equal(F.blockId(20478), "blk-020478", "large index padded to six digits");
  equal(F.blockId(-3), "blk-000000", "negative index clamped");
  equal(F.blockText(0), "Paragraph 1 — report", "first block text");
  equal(F.blockText(13), "Paragraph 14 — draft", "last word of the cycle");
  equal(F.blockText(14), "Paragraph 15 — report", "word cycle wraps");
  assert(F.blockText(20479).length < 40, "no huge string is materialised per block");
});

test("RUNTIME #2533: only the explicit block range is materialised", function () {
  var F = fixture().NS.Fixture;
  var first = F.blocksInRange(5, 0, 3);
  equal(first.length, 3, "three blocks requested");
  equal(first[0].id, "blk-000000", "first requested id");
  equal(first[2].index, 2, "last requested index");
  equal(F.blocksInRange(5, -5, 3).length, 3, "negative start clamped");
  equal(F.blocksInRange(5, 10, 99999).length, 5110, "end clamped to block count");
  var tail = F.blocksInRange(20, 20478, 20480);
  equal(tail.length, 2, "tail range returns two blocks");
  equal(tail[1].id, "blk-020479", "last id of a 20 MB document");
});

test("RUNTIME #2533: visible window stays bounded regardless of document size", function () {
  var W = fixture().NS.Window;
  var small = W.visibleRange(0, 168, 5120, 28, 4);
  var large = W.visibleRange(0, 168, 20480, 28, 4);
  equal(small.count, 14, "5 MB viewport mounts 14 rows (6 + 2*4 overscan)");
  equal(large.count, 14, "20 MB viewport mounts the same 14 rows");
  assert(large.count <= 14 && small.count <= 14, "mounted count is size-independent");
  var middle = W.visibleRange(2800, 168, 20480, 28, 4);
  equal(middle.start, 96, "scroll start derives from scrollTop / row height");
  equal(middle.end, 110, "scroll end is start plus bounded span");
  equal(middle.count, 14, "scroll window remains bounded");
  deep(W.visibleRange(0, 168, 0, 28, 4), { start: 0, end: 0, count: 0 }, "empty document");
  deep(W.visibleRange(-100, 0, 100, 28, 4), { start: 0, end: 9, count: 9 },
    "negative scroll and zero viewport clamp to one row plus overscan");
});

test("RUNTIME #2533: mount plan keeps ids stable and the container non-editable", function () {
  var NS = fixture().NS;
  var range = NS.Window.visibleRange(2800, 168, 20480, 28, 4);
  var plan = NS.Window.mountPlan(20, range);
  equal(plan.containerEditable, false, "container itself is never contenteditable");
  equal(plan.total, 20480, "plan reports the full document size");
  equal(plan.blocks.length, 14, "only the visible window is in the plan");
  equal(plan.blocks[0].id, "blk-000096", "first mounted id is index-derived");
  equal(plan.blocks[13].id, "blk-000109", "last mounted id is index-derived");
  assert(plan.blocks.every(function (block) { return block.editable === true; }),
    "every mounted block is individually editable");
  equal(NS.Window.label(plan, NS.tr), "Mounted range: 96–110 of 20480",
    "label substitutes shipped params through the shipped tr helper");
});

test("RUNTIME #2533: mount writes blocks only, with scoped contenteditable", function () {
  var fx = fixture();
  var NS = fx.NS;
  var plan = NS.Window.mountPlan(20, NS.Window.visibleRange(2800, 168, 20480, 28, 4));
  var host = fx.doc.createElement("div");
  NS.Window.mount(host, plan, true);
  equal(host.children.length, 14, "one DOM node per mounted block");
  equal(host.getAttribute("contenteditable"), null, "host container has no contenteditable");
  equal(host.children[0].getAttribute("data-block-id"), "blk-000096", "block id written as attribute");
  equal(host.children[0].getAttribute("contenteditable"), "true", "editable blocks opt in");
  assert(host.children[0].textContent.indexOf("Paragraph 97 —") === 0, "block text is rendered");
  NS.Window.mount(host, plan, false);
  equal(host.children.length, 14, "re-mount replaces rather than appends");
  equal(host.children[0].getAttribute("contenteditable"), "false", "read-only mount disables blocks");
});
