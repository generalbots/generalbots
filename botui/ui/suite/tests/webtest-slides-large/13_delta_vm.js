/**
 * #2529 real shipped JS executed in a vm sandbox -- [BASELINE-OBSERVE].
 *
 * 01_state.js and 13_delta.js are loaded UNMODIFIED into a deterministic
 * context. These tests prove local behaviour of the shipped incremental-save
 * module: op dispatch, request shape, index parsing and the failure toast.
 * They do NOT claim any server, network or runtime-performance result.
 */
"use strict";

var h = require("./00_harness");
var sb = require("./02_sandbox");

var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;

test("[BASELINE-OBSERVE] #2529 vm: 13_delta.js registers one slides op handler with no fallback listener", function () {
  var fx = sb.fixture();
  equal(fx.calls.registered.length, 1, "one GBDocDelta registration");
  equal(fx.calls.registered[0].kind, "slides", "registered under the slides kind");
  equal(fx.calls.listeners.length, 0, "standalone document fallback skipped when GBDocDelta exists");
  equal(typeof fx.calls.registered[0].fn, "function", "handler is callable");
  assert(fx.win.SlidesEditor && typeof fx.win.SlidesEditor.addSlide === "function", "SlidesEditor.addSlide bound");
  equal(fx.win.SlidesEditor.addSlide, fx.win.addSlide, "legacy window.addSlide alias is the same function");
});

test("[BASELINE-OBSERVE] #2529 vm: addSlide without a position posts a default layout and no position", function () {
  var fx = sb.fixture();
  fx.win.SlidesEditor.addSlide({ layout: "two_column" }, null);
  equal(fx.calls.fetch.length, 1, "one POST issued");
  equal(fx.calls.fetch[0].url, "/api/slides/slide/add", "coarse slide-add route used");
  equal(fx.calls.fetch[0].opts.method, "POST", "POST method");
  var body = sb.bodyAt(fx, 0);
  equal(body.presentation_id, "deck-fixture", "deck id resolved from window helper");
  equal(body.layout, "two_column", "layout carried from the op");
  equal(Object.prototype.hasOwnProperty.call(body, "position"), false, "no position key when absent");
  equal(fx.calls.refresh.length, 1, "canvas refreshed once after success");
  equal(fx.calls.refresh[0], undefined, "refresh accepts the append sentinel");
});

test("[BASELINE-OBSERVE] #2529 vm: addSlide parses numeric and after positions but drops invalid ones", function () {
  var fx = sb.fixture();
  fx.win.SlidesEditor.addSlide({ position: "3", layout: "x" }, { file: "deck-op" });
  fx.win.SlidesEditor.addSlide({ after: "5" }, null);
  fx.win.SlidesEditor.addSlide({ position: "not-a-number" }, null);
  equal(fx.calls.fetch.length, 3, "three POSTs issued");
  equal(sb.bodyAt(fx, 0).position, 3, "position parsed with parseInt");
  equal(sb.bodyAt(fx, 0).presentation_id, "deck-op", "payload.file overrides window deck id");
  equal(sb.bodyAt(fx, 1).position, 5, "after is accepted as an insert index");
  equal(Object.prototype.hasOwnProperty.call(sb.bodyAt(fx, 2), "position"), false, "NaN position omitted");
  equal(fx.calls.refresh[0], 3, "refresh receives the parsed position");
});

test("[BASELINE-OBSERVE] #2529 vm: fill/finalize ops dispatch to rename, slide-add and toast", function () {
  var fx = sb.fixture();
  var apply = fx.calls.registered[0].fn;
  apply({ ops: [
    { op: "fill", field: "title", value: "New Title" },
    { op: "fill", field: "body", value: "Only body" },
    { op: "finalize" }
  ] });
  equal(fx.calls.fetch.length, 2, "rename plus one slide-add issued");
  equal(fx.calls.fetch[0].url, "/api/slides/rename", "title fill posts rename route");
  equal(sb.bodyAt(fx, 0).title, "New Title", "title value sent verbatim");
  equal(fx.calls.fetch[1].url, "/api/slides/slide/add", "body fill posts slide-add route");
  equal(fx.calls.toast.length, 1, "finalize shows one toast");
  equal(fx.calls.toast[0].kind, "success", "finalize toast is a success");
});

test("[BASELINE-OBSERVE] #2529 vm: body fill creates one add per paragraph but drops the text", function () {
  var fx = sb.fixture();
  var apply = fx.calls.registered[0].fn;
  apply({ ops: [{ op: "fill", field: "body", value: "First\n\nSecond" }] });
  equal(fx.calls.fetch.length, 2, "two slides created from two paragraphs");
  equal(sb.bodyAt(fx, 0).layout, "title_content", "generated slides use title_content layout");
  equal(Object.prototype.hasOwnProperty.call(sb.bodyAt(fx, 0), "title"), false,
    "OBSERVED: computed title is not included in the POST body");
  equal(Object.prototype.hasOwnProperty.call(sb.bodyAt(fx, 0), "body"), false,
    "OBSERVED: computed paragraph text is not included in the POST body");
  h.note("FINDING (reported, not patched): 13_delta.js fillSlides() computes title/body for each generated slide but addSlide() only sends presentation_id/layout/position, so generated slide text never reaches the server.");
});

test("[BASELINE-OBSERVE] #2529 vm: an HTTP failure surfaces an error toast and skips refresh", function () {
  var fx = sb.fixture({
    response: { ok: false, status: 413, json: function () { return sb.thenable({}); } }
  });
  fx.win.SlidesEditor.addSlide({}, null);
  equal(fx.calls.refresh.length, 0, "no refresh on failure");
  equal(fx.calls.toast.length, 1, "one error toast");
  assert(fx.calls.toast[0].message.indexOf("HTTP 413") >= 0, "status surfaced in the toast");
  equal(fx.calls.toast[0].kind, "error", "toast kind is error");
});

test("[BASELINE-OBSERVE] #2529 vm: 01_state.js exposes fixed slide geometry and eight handles", function () {
  var fx = sb.fixture();
  equal(fx.ctx.SLIDE_W, 960, "slide width constant");
  equal(fx.ctx.SLIDE_H, 540, "slide height constant");
  equal(fx.ctx.SCALE_MIN, 0.25, "minimum zoom scale");
  equal(fx.ctx.SCALE_MAX, 2.0, "maximum zoom scale");
  equal(fx.ctx.HANDLE_POSITIONS.length, 8, "eight selection handles");
  equal(fx.ctx.HANDLE_POSITIONS[0].pos, "nw", "first handle is north-west");
});
