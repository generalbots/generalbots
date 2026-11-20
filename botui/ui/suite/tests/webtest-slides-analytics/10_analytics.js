/**
 * Slides presentation analytics — pure-helper invariant tests (#2489).
 * Exercises only the DOM-free helpers of the four shipped analytics modules;
 * no production logic is modified, duplicated or reimplemented here.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var A = h.loadAnalytics();

function event(type, viewerId, ts, slide, duration) {
  return {
    type: type,
    viewerId: viewerId,
    ts: ts,
    slide: slide || 0,
    duration: duration || 0
  };
}

// Fixed synthetic stream: two distinct viewers, three opens (one revisit),
// per-slide enter/dwell/leave, and no dependence on wall-clock randomness.
function fixedStream() {
  var base = A.BASE_TS;
  return [
    event("open", "viewer-a", base),
    event("enter", "viewer-a", base + 100, 0),
    event("dwell", "viewer-a", base + 1100, 0, 1000),
    event("leave", "viewer-a", base + 1100, 0),
    event("open", "viewer-b", base + 200),
    event("enter", "viewer-b", base + 300, 1),
    event("dwell", "viewer-b", base + 1300, 1, 1000),
    event("open", "viewer-a", base + 2000)
  ];
}

function seeded() {
  return A.seed(A.fresh(), 20);
}

// ── API surface and load-time purity ──────────────────────────────────
test("[STATIC] analytics layers 66-69 are DOM-free and window-guarded", function () {
  h.ANALYTICS_MODULES.forEach(function (rel) {
    var src = h.readFile(rel);
    assert(src.indexOf('typeof window !== "undefined"') >= 0,
      rel + " lacks the window/globalThis guard");
    assert(src.indexOf("document.") < 0, rel + " references document at source level");
  });
});

test("all four shipped namespaces load into one real A object", function () {
  ["fresh", "load", "save", "compute", "csv", "prune", "seed", "qa", "polls", "tr",
    "num", "pseudonym", "bucketDevice", "bucketGeo"].forEach(function (key) {
    assert(A[key] !== undefined, "A." + key + " missing after vm load");
  });
  assert(typeof A.compute === "function" && typeof A.seed === "function", "compute/seed not functions");
  assert(typeof A.qa.upvote === "function" && typeof A.polls.results === "function",
    "qa/polls helpers not functions");
});

// ── fresh() shape ─────────────────────────────────────────────────────
test("fresh() has a stable, empty, deck-scoped shape", function () {
  var state = A.fresh();
  assert(state.version === 1, "unexpected state version");
  assert(state.slideCount === A.SLIDE_COUNT, "slideCount mismatch");
  assert(state.options.anonymise === true && state.options.retentionDays === 90,
    "option defaults changed");
  ["viewers", "events", "questions", "polls"].forEach(function (key) {
    assert(Array.isArray(state[key]) && state[key].length === 0, key + " not empty");
  });
  assert(state.follow.presenterSlide === 0 && Array.isArray(state.follow.devices),
    "follow shape changed");
  assert(state.console.startedAt === A.BASE_TS, "console anchor changed");
  assert(JSON.stringify(A.fresh()) === JSON.stringify(state), "fresh() is not deterministic");
});

test("fresh() carries no personal fields", function () {
  var keys = Object.keys(A.fresh()).join(",");
  ["email", "name", "viewerId"].forEach(function (forbidden) {
    assert(keys.indexOf(forbidden) < 0, forbidden + " leaked into fresh state");
  });
});

// ── base helpers ──────────────────────────────────────────────────────
test("num() coerces valid numbers and falls back for malformed ones", function () {
  assert(A.num("12", 0) === 12, "numeric string not coerced");
  assert(A.num("x", 7) === 7, "non-numeric string not defaulted");
  assert(A.num(NaN, 7) === 7, "NaN not defaulted");
  assert(A.num(Infinity, 7) === 7, "Infinity not defaulted");
});

test("pseudonym() is deterministic and opaque", function () {
  var first = A.pseudonym("viewer-1");
  assert(first === A.pseudonym("viewer-1"), "pseudonym not reproducible");
  assert(first !== A.pseudonym("viewer-2"), "pseudonym collided for distinct ids");
  assert(first.indexOf("anon-") === 0, "pseudonym prefix missing");
});

test("device and geo buckets map fixed raw values deterministically", function () {
  assert(A.bucketDevice("iPhone 15") === "mobile", "iPhone not mobile");
  assert(A.bucketDevice("iPad Air") === "tablet", "iPad not tablet");
  assert(A.bucketDevice("MacBook Pro") === "desktop", "MacBook not desktop");
  assert(A.bucketGeo("São Paulo, Brazil") === "South America", "Brazil bucket wrong");
  assert(A.bucketGeo("Berlin, Germany") === "Europe", "Berlin bucket wrong");
  assert(A.bucketGeo("unknown-place") === "Africa", "fallback bucket wrong");
});

// ── view-count aggregation ────────────────────────────────────────────
test("view counts are the number of open events", function () {
  var state = A.fresh();
  state.events = fixedStream();
  assert(A.compute(state).views === 3, "expected three opens in fixed stream");
});

test("view counts sum deterministically across independent event sets", function () {
  var left = A.fresh();
  var right = A.fresh();
  var both = A.fresh();
  left.events = [event("open", "v1", 1), event("open", "v2", 2)];
  right.events = [event("open", "v3", 3), event("open", "v4", 4), event("open", "v5", 5)];
  both.events = left.events.concat(right.events);
  assert(A.compute(left).views === 2, "left view count wrong");
  assert(A.compute(right).views === 3, "right view count wrong");
  assert(A.compute(both).views === 5, "combined view count is not the sum");
});

test("unique viewers dedupe repeated opens when no viewer rows are present", function () {
  var state = A.fresh();
  state.events = fixedStream();
  var report = A.compute(state);
  assert(report.views === 3, "opens miscounted");
  assert(report.uniqueViewers === 2, "revisits not deduped");
  assert(report.views > report.uniqueViewers, "dedupe had no effect");
});

test("seeded fixture yields the documented 20 viewers and 23 opens", function () {
  var report = A.compute(seeded());
  assert(report.views === 23, "expected 20 opens plus three revisits");
  assert(report.uniqueViewers === 20, "twenty seeded viewers expected");
});

test("per-slide engagement sums deterministically over the seeded stream", function () {
  var report = A.compute(seeded());
  var entered = report.perSlide.reduce(function (sum, row) { return sum + row.entered; }, 0);
  assert(report.perSlide.length === A.SLIDE_COUNT, "perSlide length mismatch");
  assert(entered === 134, "entered total drifted from the fixed fixture: " + entered);
  assert(report.dropOffSlide === 3 && report.dropOffCount === 5,
    "drop-off slide/count drifted from the fixed fixture");
});

test("completion rate and privacy buckets are stable for the seeded fixture", function () {
  var report = A.compute(seeded());
  assert(report.completedViewers === 12, "completed viewer count changed");
  assert(report.completionRate === 60, "completion rate changed: " + report.completionRate);
  assert(report.deviceBuckets.mobile === 11 && report.deviceBuckets.tablet === 3 &&
    report.deviceBuckets.desktop === 6, "device buckets changed");
  assert(report.geoBuckets["North America"] === 7 && report.geoBuckets.Europe === 6,
    "geo buckets changed");
  var geoTotal = Object.keys(report.geoBuckets).reduce(function (sum, key) {
    return sum + report.geoBuckets[key];
  }, 0);
  assert(geoTotal === 20, "geo buckets do not sum to the viewer count");
});

test("first and last view labels are derived from the stream, not the clock", function () {
  var state = A.fresh();
  state.events = fixedStream();
  var report = A.compute(state);
  assert(report.firstView.length > 0 && report.lastView.length > 0, "view labels empty");
  assert(report.firstView === A.dateLabel(A.BASE_TS), "first view label wrong");
  assert(report.lastView === A.dateLabel(A.BASE_TS + 2000), "last view label wrong");
  assert(report.firstView !== report.lastView, "last view did not advance");
});

// ── percentages and zero denominators ────────────────────────────────
test("empty state produces finite zero percentages, never NaN", function () {
  var report = A.compute(A.fresh());
  assert(report.views === 0 && report.uniqueViewers === 0, "empty counts not zero");
  assert(report.completionRate === 0 && isFinite(report.completionRate),
    "completion rate is NaN or non-zero");
  assert(report.dropOffCount === 0 && report.firstView === "" && report.lastView === "",
    "empty report has phantom values");
  report.perSlide.forEach(function (row) {
    assert(row.avgDwellMs === 0 && isFinite(row.avgDwellMs), "avgDwellMs not a finite zero");
  });
});

test("dwell averages are rounded and finite for a fixed stream", function () {
  var state = A.fresh();
  state.events = fixedStream();
  var report = A.compute(state);
  assert(report.perSlide[0].dwellMs === 1000 && report.perSlide[0].avgDwellMs === 1000,
    "slide 0 dwell not aggregated");
  assert(report.perSlide[1].entered === 1 && report.perSlide[1].exits === 0,
    "slide 1 counters wrong");
});

// ── reproducibility and exports ──────────────────────────────────────
test("a fixed synthetic event stream produces a byte-identical report", function () {
  var state = A.fresh();
  state.events = fixedStream();
  var first = JSON.stringify(A.compute(state));
  var second = JSON.stringify(A.compute(state));
  assert(first === second, "report changed between identical runs");
  assert(JSON.stringify(A.compute(seeded())) === JSON.stringify(A.compute(seeded())),
    "seeded report is not reproducible");
});

test("summary CSV has one header and one row per slide", function () {
  var csv = A.csv.summary(A.compute(seeded()));
  var lines = csv.trim().split("\n");
  assert(lines.length === A.SLIDE_COUNT + 1, "unexpected summary CSV row count");
  // csv.summary() quotes every cell via its cell() helper, so the header is
  // `"slide","entered","dwell_ms","avg_dwell_ms","exits"` — assert content,
  // not an unquoted prefix.
  assert(lines[0].indexOf("slide") >= 0 && lines[0].indexOf("entered") >= 0 &&
    lines[0].indexOf("dwell_ms") >= 0 && lines[0].indexOf("avg_dwell_ms") >= 0 &&
    lines[0].indexOf("exits") >= 0, "summary header missing");
});

test("anonymised viewer CSV omits raw contacts and uses pseudonyms", function () {
  var state = seeded();
  var csv = A.csv.viewers(state, true);
  assert(csv.indexOf("@example.test") < 0, "raw contact leaked into anonymised CSV");
  assert(csv.indexOf("anon-") >= 0, "pseudonyms missing from anonymised CSV");
  var raw = A.csv.viewers(state, false);
  assert(raw.indexOf("@example.test") >= 0, "raw export lost contact column");
});

test("prune() removes only events older than the retention cutoff", function () {
  var state = A.fresh();
  var day = 86400000;
  state.options.retentionDays = 1;
  state.events = [event("open", "v1", 1000), event("open", "v2", 1000 + day + 1)];
  var removed = A.prune(state, 1000 + day + 100);
  assert(removed === 1, "expected one expired event");
  assert(state.events.length === 1 && state.events[0].ts === 1000 + day + 1,
    "wrong event survived pruning");
});

// ── polls ─────────────────────────────────────────────────────────────
function choicePoll() {
  var state = A.fresh();
  var poll = A.polls.create(state, {
    question: "poll-question",
    options: ["opt-a", "opt-b", "opt-c"],
    createdAt: 1000
  });
  return { state: state, poll: poll };
}

test("poll create validates question and option count; rating has five options", function () {
  var state = A.fresh();
  assert(A.polls.create(state, { question: "", options: ["a", "b"] }) === null,
    "empty question accepted");
  assert(A.polls.create(state, { question: "q", options: ["a"] }) === null,
    "single option accepted");
  var rating = A.polls.create(state, { question: "q", type: "rating" });
  assert(rating.options.length === 5, "rating options are not 1..5");
  assert(rating.status === "draft", "new poll not a draft");
});

test("poll launch and close transitions are one-way and guarded", function () {
  var context = choicePoll();
  var id = context.poll.id;
  assert(A.polls.close(context.state, id, 2000) === false, "draft poll closed");
  assert(A.polls.launch(context.state, id, 5000) === true, "draft poll not launched");
  assert(A.polls.launch(context.state, id, 6000) === false, "live poll relaunched");
  assert(A.polls.close(context.state, id, 7000) === true, "live poll not closed");
  assert(context.poll.status === "closed" && context.poll.closedAt === 7000,
    "close state not persisted");
  assert(A.polls.byStatus(context.state, "closed").length === 1, "byStatus failed");
});

test("poll response is accepted only while live with a known option", function () {
  var context = choicePoll();
  var id = context.poll.id;
  assert(A.polls.respond(context.state, id, { value: "opt-a" }) === false,
    "draft poll accepted a response");
  A.polls.launch(context.state, id, 5000);
  assert(A.polls.respond(context.state, id, { value: "bogus", viewerId: "viewer-x" }) === false,
    "unknown option accepted");
  assert(A.polls.respond(context.state, id, { value: "opt-a", viewerId: "viewer-1", ts: 5500 }),
    "valid response rejected");
});

test("poll tally counts each option and ignores malformed ballots", function () {
  var context = choicePoll();
  A.polls.launch(context.state, context.poll.id, 5000);
  [["viewer-1", "opt-a", 5500], ["viewer-2", "opt-a", 5600],
    ["viewer-3", "opt-b", 5700]].forEach(function (row) {
    A.polls.respond(context.state, context.poll.id,
      { viewerId: row[0], value: row[1], ts: row[2] });
  });
  context.poll.responses.push({ id: "bad-1", viewerId: "viewer-x", value: "bogus", ts: 5800 });
  context.poll.responses.push({ id: "bad-2", viewerId: "viewer-y", value: undefined, ts: 5900 });
  var result = A.polls.results(context.state, context.poll.id);
  var counts = result.options.map(function (row) { return row.count; });
  assert(counts.join(",") === "2,1,0", "option tally wrong: " + counts.join(","));
  result.options.forEach(function (row) {
    assert(isFinite(row.percent), "percent is not finite for " + row.label);
  });
  assert(result.options[0].percent === 40 && result.options[1].percent === 20,
    "percentages drifted for the fixed ballot set");
});

test("[OBSERVED] poll total counts every stored ballot, including malformed ones", function () {
  var context = choicePoll();
  A.polls.launch(context.state, context.poll.id, 5000);
  A.polls.respond(context.state, context.poll.id, { value: "opt-a", ts: 5500 });
  context.poll.responses.push({ id: "bad-1", viewerId: "viewer-x", value: "bogus", ts: 5800 });
  var result = A.polls.results(context.state, context.poll.id);
  assert(result.total === 2 && result.options[0].percent === 50,
    "observed total/denominator changed — re-check the reported gap");
});

test("poll result reports finite zeros before any response", function () {
  var context = choicePoll();
  var result = A.polls.results(context.state, context.poll.id);
  assert(result.total === 0 && result.latencyMs === 0 && result.withinBudget === false,
    "empty poll result wrong");
  result.options.forEach(function (row) {
    assert(row.percent === 0 && isFinite(row.percent), "empty poll percent not a finite zero");
  });
});

test("poll response latency budget is computed from the first response", function () {
  var context = choicePoll();
  A.polls.launch(context.state, context.poll.id, 5000);
  A.polls.respond(context.state, context.poll.id, { value: "opt-a", ts: 5500 });
  var within = A.polls.results(context.state, context.poll.id);
  assert(within.latencyMs === 500 && within.withinBudget === true, "latency budget not met");
  var late = A.fresh();
  var slow = A.polls.create(late, { question: "q", options: ["a", "b"] });
  A.polls.launch(late, slow.id, 5000);
  A.polls.respond(late, slow.id, { value: "a", ts: 8000 });
  var over = A.polls.results(late, slow.id);
  assert(over.latencyMs === 3000 && over.withinBudget === false, "over-budget not flagged");
});

test("poll CSV exports are anonymised by default and stable", function () {
  var context = choicePoll();
  A.polls.launch(context.state, context.poll.id, 5000);
  A.polls.respond(context.state, context.poll.id,
    { viewerId: "viewer-1", value: "opt-a", ts: 5500 });
  var csv = A.polls.resultsCsv(context.state, context.poll.id);
  assert(csv.trim().split("\n").length === 4, "results CSV row count wrong");
  var responses = A.polls.responsesCsv(context.state, context.poll.id, true);
  assert(responses.indexOf("viewer-1") < 0 && responses.indexOf("anon-") >= 0,
    "responses CSV not anonymised");
  assert(responses === A.polls.responsesCsv(context.state, context.poll.id, true),
    "responses CSV not reproducible");
});

// ── Q&A ───────────────────────────────────────────────────────────────
function qaState() {
  var state = A.fresh();
  var first = A.qa.submit(state, {
    text: "first-question", viewerId: "viewer-1", name: "name-1", email: "mail-1", ts: 100
  });
  var second = A.qa.submit(state, { text: "second-question", policy: "anonymous", ts: 200 });
  return { state: state, first: first, second: second };
}

test("Q&A submit validates text and records submission policy", function () {
  var context = qaState();
  assert(A.qa.submit(context.state, { text: "  " }) === null, "blank question accepted");
  assert(context.first.anonymous === false && context.first.submittedBy.viewerId === "viewer-1",
    "attributable policy lost");
  assert(context.second.anonymous === true && context.second.submittedBy === null,
    "anonymous policy lost");
  assert(context.first.votes === 0 && Array.isArray(context.first.voters), "vote fields wrong");
});

test("Q&A attribution is opaque for anonymous submitters", function () {
  var context = qaState();
  assert(A.qa.attribution(context.first) === "name-1", "attributed name not used");
  var anonymousLabel = A.qa.attribution(context.second);
  assert(typeof anonymousLabel === "string" && anonymousLabel.length > 0,
    "anonymous fallback label missing");
  assert(anonymousLabel.indexOf("viewer-") < 0, "anonymous attribution leaked an id");
});

test("Q&A upvotes are deduped per viewer and deterministic", function () {
  var context = qaState();
  assert(A.qa.upvote(context.state, context.first.id, "viewer-9") === true, "first upvote lost");
  assert(A.qa.upvote(context.state, context.first.id, "viewer-9") === false,
    "duplicate upvote counted twice");
  assert(context.first.votes === 1 && context.first.voters.length === 1, "vote count drifted");
  assert(A.qa.upvote(context.state, context.second.id, "viewer-9") === true,
    "second question upvote lost");
  assert(/^anon-[0-9a-z]+$/.test(context.first.voters[0]), "voter id is not a pseudonym");
});

test("shipped Q&A ordering helpers preserve deterministic insertion order", function () {
  var context = qaState();
  var ids = function (rows) { return rows.map(function (row) { return row.id; }).join(","); };
  assert(ids(A.qa.queue(context.state)) === context.first.id + "," + context.second.id,
    "queue order drifted");
  assert(ids(A.qa.archive(context.state)) === "", "archive not empty for new questions");
  assert(A.qa.queue(context.state)[0].ts === 100 && A.qa.queue(context.state)[1].ts === 200,
    "timestamps not carried in order");
  assert(JSON.stringify(A.qa.queue(context.state)) === JSON.stringify(A.qa.queue(context.state)),
    "queue is not reproducible");
});

test("Q&A moderation moves questions through queue, approved and archive", function () {
  var context = qaState();
  assert(A.qa.moderate(context.state, context.second.id, "reject") === true, "reject failed");
  assert(A.qa.moderate(context.state, context.first.id, "approve") === true, "approve failed");
  assert(A.qa.moderate(context.state, context.first.id, "unknown") === false,
    "unknown decision accepted");
  assert(A.qa.count(context.state) === 1, "rejected question still counted");
  assert(A.qa.queue(context.state).length === 0 && A.qa.approved(context.state).length === 1,
    "moderation buckets wrong");
  assert(A.qa.upvote(context.state, context.second.id, "viewer-3") === false,
    "rejected question accepted an upvote");
  assert(A.qa.answer(context.state, context.second.id, "answer") === false,
    "rejected question was answered");
  assert(A.qa.answer(context.state, context.first.id, "answer") === true, "answer failed");
  assert(A.qa.archive(context.state).length === 1, "answered archive missing the item");
  assert(A.qa.get(context.state, "missing-id") === null, "unknown question id resolved");
});

test("[STATIC][GAP] no Q&A votes/timestamp ranking helper ships", function () {
  var src = h.readFile("slides/modules/68_analytics_qa.js");
  assert(src.indexOf(".sort(") < 0,
    "sort helper now present — replace this static gap check with runtime ranking tests");
  assert(src.indexOf("QA.queue") >= 0 && src.indexOf("QA.filter") >= 0,
    "shipped ordering helpers changed — re-read the module before reporting this gap");
});
