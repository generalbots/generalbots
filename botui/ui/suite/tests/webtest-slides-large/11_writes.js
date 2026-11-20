/**
 * #2529 per-edit write granularity -- [BASELINE-OBSERVE].
 *
 * The wire payload is element-scoped, but the server read-modify-writes the
 * entire deck from Drive. Both halves are measured here so the distinction is
 * explicit and future fixes cannot silently regress either side.
 */
"use strict";

var h = require("./00_harness");
var s = require("./01_sources");

var test = h.test;
var assert = h.assert;

test("[BASELINE-OBSERVE] #2529: server element ops read and write the entire deck", function () {
  var ops = s.rust("elementOps");
  var handlers = s.rust("handlers");
  assert(s.count(ops, "load_pres!(") >= 3, "element handlers load the whole presentation");
  assert(s.count(ops, "save_pres!(") >= 5, "element handlers save the whole presentation");
  assert(s.has(handlers, "load_presentation_by_id(drive, $user_id, &requested_id)"),
    "load_pres! reads one whole deck object from Drive");
  assert(s.has(handlers, "save_presentation_to_drive(drive, $user_id, $presentation)"),
    "save_pres! writes one whole deck object back to Drive");
});

test("[BASELINE-OBSERVE] #2529: the element-update request payload is element-scoped", function () {
  var src = s.readModule("03_canvas.js");
  assert(s.has(src, "SlideCanvas.persistElement = function"), "persistElement is the edit path");
  assert(s.has(src, "\"/api/slides/element/update\""), "edits POST to the element route");
  assert(s.has(src, "element_id: el.dataset.id"), "payload names a single element");
  assert(s.has(src, "x: parseInt(el.style.left, 10)"), "payload carries geometry only");
});

test("[BASELINE-OBSERVE] #2529: collaboration sends element-scoped content, not the full deck", function () {
  var src = s.readModule("03_canvas.js");
  assert(s.count(src, "GBCollab.send(\"slide_update\"") >= 2, "drag and text edits publish slide_update");
  assert(s.has(src, "content: JSON.stringify(payload)"), "broadcast content is the element payload");
  assert(s.has(src, "element_id: el.dataset.id"), "broadcast identifies the one edited element");
});

test("[BASELINE-OBSERVE] #2529: no manifest or per-element persistence module is shipped", function () {
  var js = s.allJs();
  var ops = s.rust("elementOps");
  ["element-manifest", "deck-manifest", "patch_element", "persist_element_delta"].forEach(function (marker) {
    assert(!s.has(js, marker), "no shipped JS marker " + marker);
    assert(!s.has(ops, marker), "no shipped Rust marker " + marker);
  });
});
