/**
 * Executes the shipped Docs large-file preview modules verbatim in a stubbed
 * Node vm. These are RUNTIME PASS results for logic that IS present. They do
 * not measure time/RSS/bytes and do not claim a real DOCX import — the fixture
 * only accounts size by block count, exactly as the shipped source states.
 */
"use strict";

var H = require("./00_harness");

var MODULES = [
  "botui/ui/suite/docs/modules/84_large_state.js",
  "botui/ui/suite/docs/modules/85_large_fixture.js",
  "botui/ui/suite/docs/modules/86_large_window.js",
  "botui/ui/suite/docs/modules/87_large_save.js",
  "botui/ui/suite/docs/modules/88_large_degradation.js"
];

function makeSandbox() {
  var win = {};
  win.localStorage = {
    getItem: function () { return null; },
    setItem: function () {}
  };
  win.gbT = function (key, fallback) { return fallback; };
  var doc = {
    createElement: function () {
      return {
        setAttribute: function () {}, appendChild: function () {},
        textContent: "", className: "", id: "", style: {}
      };
    },
    getElementById: function () { return null; },
    querySelector: function () { return null; }
  };
  win.document = doc;
  return { window: win, document: doc, localStorage: win.localStorage, console: console };
}

var ctx = H.loadModules(MODULES, makeSandbox());
var NS = ctx.window.DocsLarge;

H.runtime("DocsLarge namespace loaded from shipped modules", function () {
  H.assert(!!NS && !!NS.Window && !!NS.Fixture && !!NS.Save && !!NS.Degradation,
    "all large namespaces must be registered");
  H.equal(NS.CEILING_BYTES, 2 * 1024 * 1024, "documented local ceiling");
});

H.runtime("visibleRange() bounds mounted blocks independent of document size", function () {
  var total = NS.Fixture.blockCount(20);
  var small = NS.Window.visibleRange(0, 280, total, 28, 4);
  var deep = NS.Window.visibleRange(28 * 20000, 280, total, 28, 4);
  var bound = Math.ceil(280 / 28) + 8;
  H.assert(small.count > 0 && small.count <= bound, "small range bounded by viewport+overscan");
  H.assert(deep.count > 0 && deep.count <= bound, "deep-scroll range stays bounded");
  H.assert(deep.end <= total, "range never exceeds document");
  H.assert(total > 10000, "20 MB fixture accounts size by block count");
});

H.runtime("mountPlan() mounts only the visible window, container not editable", function () {
  var range = NS.Window.visibleRange(560, 280, NS.Fixture.blockCount(20), 28, 4);
  var plan = NS.Window.mountPlan(20, range);
  H.equal(plan.containerEditable, false, "container must not be contenteditable");
  H.equal(plan.blocks.length, range.count, "mounted blocks equal the visible range only");
  H.assert(plan.blocks.length < NS.Fixture.blockCount(20), "mounted set is a strict subset");
  H.assert(plan.blocks.every(function (b) { return b.editable === true; }), "mounted blocks editable");
});

H.runtime("blocksInRange() allocates a window, not the whole document", function () {
  var all = NS.Fixture.blocksInRange(20, 0, NS.Fixture.blockCount(20));
  var windowSlice = NS.Fixture.blocksInRange(20, 100, 118);
  H.equal(windowSlice.length, 18, "explicit range yields only 18 blocks");
  H.assert(windowSlice.length < all.length, "window is smaller than the full simulated document");
});

H.runtime("save plan: stale ETag -> 409, over-ceiling full -> 413, incremental ok", function () {
  function state(overrides) {
    var base = NS.defaultState();
    Object.keys(overrides || {}).forEach(function (k) { base[k] = overrides[k]; });
    return base;
  }
  var stale = NS.Save.plan(state({ etag: "v2", etagSent: "v1", fullBytes: 10 }));
  H.equal(stale.status, 409, "stale save rejected");
  var huge = NS.Save.plan(state({ fullBytes: NS.CEILING_BYTES + 1 }));
  H.equal(huge.status, 413, "full body over local ceiling rejected");
  var inc = NS.Save.plan(state({ queue: [{ blockId: "blk-1", bytes: 100 }], fullBytes: 99 * 1024 * 1024 }));
  H.equal(inc.kind, "incremental", "queued change uses incremental plan");
  H.equal(inc.ok, true, "small incremental plan accepted");
});

H.runtime("degradation gate opens large content read-only", function () {
  H.equal(NS.Degradation.evaluate(20, 10).readOnly, true, "20 MB over 10 MB threshold is read-only");
  H.equal(NS.Degradation.evaluate(5, 10).readOnly, false, "5 MB within threshold is editable");
});
