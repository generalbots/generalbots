/**
 * RUNTIME #2533: incremental save queue, ETag/If-Match, 2 MB ceiling.
 * All state transitions execute shipped module 87; no logic is copied.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var fixture = f.fixture;

test("RUNTIME #2533: op bytes and last-write-wins queue semantics", function () {
  var S = fixture().NS.Save;
  equal(S.opBytes(0), 48, "overhead only for empty text");
  equal(S.opBytes(100), 148, "text length added to overhead");
  equal(S.opBytes(-5), 48, "negative length clamped");
  equal(S.opBytes("abc"), 48, "non-numeric length normalises to overhead only");
  var state = { queue: [] };
  S.enqueue(state, "blk-a", 10);
  var replaced = S.enqueue(state, "blk-a", 20);
  equal(replaced.bytes, 68, "second write to the same block wins");
  equal(state.queue.length, 1, "no duplicate block entries");
  S.enqueue(state, "blk-b", 5);
  equal(state.queue.length, 2, "a different block appends");
  equal(S.incrementalBytes(state), 121, "incremental bytes sum the queued ops");
});

test("RUNTIME #2533: save mode toggles between automatic and manual", function () {
  var S = fixture().NS.Save;
  var state = { mode: "automatic" };
  equal(S.setMode(state, "manual"), "manual", "setMode accepts manual");
  equal(S.toggleMode(state), "automatic", "toggle returns to automatic");
  equal(S.setMode(state, "bogus"), "automatic", "unknown mode normalises to automatic");
});

test("RUNTIME #2533: full save is the first-save fallback and is well under the ceiling", function () {
  var NS = fixture().NS;
  var state = NS.defaultState();
  state.fullBytes = 1000;
  var result = NS.Save.plan(state);
  equal(result.kind, "full", "empty queue means a full save");
  equal(result.bytes, 1000, "full body bytes reported");
  equal(result.stale, false, "matching ETag is not stale");
  equal(result.overCeiling, false, "small body inside the ceiling");
  equal(result.ok, true, "full save accepted");
  equal(result.status, 200, "200 status");
  equal(result.messageKey, "docs-large-save-ok", "stable success key");
  equal(NS.Save.message(result), "Saved 1000 bytes", "message substitutes the byte count");
});

test("RUNTIME #2533: stale ETag is rejected with a stable 409 and no data loss", function () {
  var NS = fixture().NS;
  var state = NS.defaultState();
  state.fullBytes = 1000;
  NS.Save.enqueue(state, "blk-x", 10);
  state.etagSent = "v9";
  var result = NS.Save.plan(state);
  equal(result.ok, false, "stale save rejected");
  equal(result.status, 409, "409 conflict status");
  equal(result.messageKey, "docs-large-save-stale", "stable stale key");
  var outcome = NS.Save.commit(state, result);
  equal(outcome.ok, false, "commit refuses a stale plan");
  equal(outcome.lostData, false, "no silent data loss claimed");
  equal(outcome.queueDepth, 1, "queue is preserved after rejection");
  equal(outcome.etag, "v1", "server ETag unchanged");
});

test("RUNTIME #2533: 2 MB ceiling yields 413 for full and incremental bodies", function () {
  var NS = fixture().NS;
  var full = NS.defaultState();
  full.fullBytes = NS.CEILING_BYTES + 1;
  var fullResult = NS.Save.plan(full);
  equal(fullResult.status, 413, "oversize full body gets 413");
  equal(fullResult.messageKey, "docs-large-save-413", "actionable full-body key");

  var incremental = NS.defaultState();
  NS.Save.enqueue(incremental, "blk-big", 2 * 1024 * 1024);
  var incResult = NS.Save.plan(incremental);
  equal(incResult.kind, "incremental", "queued changes take the incremental path");
  equal(incResult.status, 413, "oversize change batch gets 413");
  equal(incResult.messageKey, "docs-large-save-inc-413", "actionable split key");
  var outcome = NS.Save.commit(incremental, incResult);
  equal(outcome.queueDepth, 1, "rejected incremental keeps its queue");
});

test("RUNTIME #2533: a small incremental save succeeds and clears only its own queue", function () {
  var NS = fixture().NS;
  var state = NS.defaultState();
  state.fullBytes = 20 * 1024 * 1024;
  NS.Save.enqueue(state, "blk-0", 20);
  NS.Save.enqueue(state, "blk-1", 20);
  var result = NS.Save.plan(state);
  equal(result.kind, "incremental", "incremental path chosen despite a huge full body");
  equal(result.bytes, 136, "two 68-byte ops sent, not the full body");
  assert(result.bytes < state.fullBytes, "incremental bytes are far below full body");
  equal(result.ok, true, "incremental save accepted over the ceiling");
  var outcome = NS.Save.commit(state, result);
  equal(outcome.ok, true, "commit succeeds");
  equal(outcome.queueDepth, 0, "successful incremental drains the queue");
  equal(outcome.etag, "v2", "ETag bumped on success");
  equal(state.etagSent, "v2", "sent ETag follows the committed ETag");
  equal(NS.Save.plan(state).stale, false, "next plan is no longer stale");
});

test("RUNTIME #2533: ETag bump handles simple versions and malformed values", function () {
  var NS = fixture().NS;
  equal(NS.bumpEtag("v1"), "v2", "increments a simple version");
  equal(NS.bumpEtag("v9"), "v10", "multi-digit version");
  equal(NS.bumpEtag(""), "v2", "empty ETag defaults to v2");
  equal(NS.bumpEtag("abc"), "v2", "malformed ETag defaults to v2");
});
