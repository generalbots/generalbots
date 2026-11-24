/**
 * Slides delivery mode — pure-helper invariant tests (#2432).
 * Runtime: the eight real shipped delivery modules in a VM sandbox. DOM-only
 * render helpers and unimplemented capabilities are covered by explicitly
 * labelled STATIC source checks at the bottom (never executed as runtime).
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var g = h.loadModules();
var D = g.D;
var state = g.state;

function lastKey() {
  return state.statuses.length ? state.statuses[state.statuses.length - 1].key : null;
}
function activeShow(order, current) {
  g.reset();
  state.current = current || 0;
  var show = D.customShows.create("Demo", order);
  D.customShows.start(show.id);
  return show;
}
function sectionDeck() {
  g.reset();
  state.slideCount = 10;
  D.sections.create("A", 0);
  D.sections.create("C", 7);
  D.sections.create("B", 3);
  return D.sections.ordered();
}

// ── API surface ───────────────────────────────────────────────────────
test("all eight shipped modules bind one shared delivery namespace", function () {
  ["tr", "escapeHtml", "formatTime", "blank", "load", "save", "patch",
    "customShows", "sections", "captions", "rehearsal", "handouts", "recording"
  ].forEach(function (name) {
    assert(D[name], "GBSlidesDelivery." + name + " missing");
  });
  assert(D.console && typeof D.console.elapsed === "function", "D.console missing");
});

test("pure helper entry points are functions", function () {
  ["parseOrder", "create", "find", "list", "remove", "start", "advance",
    "finish", "nextIndex", "prevIndex", "isActive"].forEach(function (name) {
    assert(typeof D.customShows[name] === "function", "customShows." + name + " missing");
  });
  ["ordered", "owner", "bounds", "neighbours", "create", "remove",
    "jumpNext", "jumpPrev"].forEach(function (name) {
    assert(typeof D.sections[name] === "function", "sections." + name + " missing");
  });
  ["state", "style", "isOn", "toggle", "setStyle"].forEach(function (name) {
    assert(typeof D.captions[name] === "function", "captions." + name + " missing");
  });
  ["targets", "actual", "setTarget", "elapsed", "lap", "rows", "apply"].forEach(function (name) {
    assert(typeof D.rehearsal[name] === "function", "rehearsal." + name + " missing");
  });
  ["perPage", "setPerPage", "pageCount"].forEach(function (name) {
    assert(typeof D.handouts[name] === "function", "handouts." + name + " missing");
  });
  ["state", "isActive", "elapsed", "start", "stop", "resume"].forEach(function (name) {
    assert(typeof D.recording[name] === "function", "recording." + name + " missing");
  });
});

// ── Delivery state helpers ────────────────────────────────────────────
test("blank() has the documented empty shape and deck storage key is stable", function () {
  g.reset();
  var fresh = D.blank();
  assert(fresh.shows.length === 0 && fresh.sections.length === 0, "blank collections");
  assert(fresh.captions.on === false && fresh.captions.style.size === 20, "blank captions");
  assert(fresh.rehearsal.appliedAt === 0 && fresh.handouts.perPage === 4, "blank defaults");
  assert(fresh.recording.active === false && fresh.recording.duration === 0, "blank recording");
  assert(D.storeKey() === "slides-delivery:demo-deck", "unexpected deck storage key");
});

test("escapeHtml escapes all five HTML-sensitive characters", function () {
  assert(D.escapeHtml("<a href='x'>&\"") === "&lt;a href=&#39;x&#39;&gt;&amp;&quot;",
    "escapeHtml output mismatch");
  assert(D.escapeHtml(null) === "" && D.escapeHtml(7) === "7", "escapeHtml coercion");
});

test("formatTime clamps negative and invalid input and formats minutes", function () {
  assert(D.formatTime(0) === "0:00", "zero time");
  assert(D.formatTime(59999) === "0:59", "sub-minute time");
  assert(D.formatTime(60000) === "1:00", "minute rollover");
  assert(D.formatTime(3661000) === "61:01", "large duration");
  assert(D.formatTime(-5) === "0:00" && D.formatTime("bad") === "0:00", "clamped time");
});

test("load() round-trips a saved document through the storage stub", function () {
  g.reset();
  var data = D.blank();
  data.shows.push({ id: "s", name: "n", order: [2, 0] });
  data.captions.on = true;
  assert(D.save(data) === true, "save failed");
  assert(JSON.parse(g.storage.raw()).shows[0].order[1] === 0, "persisted JSON mismatch");
  var loaded = D.load();
  assert(loaded.shows.length === 1 && loaded.shows[0].order[0] === 2, "shows not reloaded");
  assert(loaded.captions.on === true, "caption flag not reloaded");
});

test("load() falls back on corrupt JSON and merges partial persisted fields", function () {
  g.reset();
  g.storage.setRaw("{not json");
  var fresh = D.load();
  assert(fresh.shows.length === 0 && fresh.captions.on === false, "corrupt JSON fallback");
  g.storage.setRaw(JSON.stringify({
    captions: { on: true, style: { color: "#abc", size: 999 } },
    rehearsal: { targets: { 2: 30 }, actual: { 2: 31 }, appliedAt: 77 },
    handouts: { perPage: 6 },
    recording: { active: false, startedAt: 0, duration: 1234 }
  }));
  var merged = D.load();
  assert(merged.captions.on === true && merged.captions.style.color === "#abc", "caption merge");
  assert(merged.rehearsal.targets[2] === 30 && merged.rehearsal.appliedAt === 77, "rehearsal merge");
  assert(merged.handouts.perPage === 6 && merged.recording.duration === 1234, "handout/recording merge");
  assert(merged.shows.length === 0, "absent arrays stay empty");
});

// ── Navigation bounds ─────────────────────────────────────────────────
test("[UAT:SLIDE-09] console next/previous indices clamp at both deck bounds", function () {
  g.reset();
  state.slideCount = 5;
  state.current = 0;
  assert(D.console.previousIndex() === 0 && D.console.nextIndex() === 1, "lower bound");
  state.current = 2;
  assert(D.console.previousIndex() === 1 && D.console.nextIndex() === 3, "middle slide");
  state.current = 4;
  assert(D.console.previousIndex() === 3 && D.console.nextIndex() === 4, "upper bound");
});

test("[UAT:SLIDE-09] console timer accumulates only running time across pause and reset", function () {
  g.reset();
  assert(D.console.elapsed() === 0, "initial elapsed");
  g.setClock(1000);
  D.console.start();
  g.setClock(6000);
  assert(D.console.elapsed() === 5000, "running elapsed");
  D.console.pause();
  g.setClock(9000);
  assert(D.console.elapsed() === 5000, "paused elapsed");
  D.console.reset();
  assert(D.console.elapsed() === 0, "reset elapsed");
  D.console.start();
  g.setClock(11000);
  assert(D.console.elapsed() === 2000, "restarted elapsed");
  D.console.pause();
});

// ── Custom shows ──────────────────────────────────────────────────────
test("parseOrder preserves an explicit slide subset order", function () {
  g.reset();
  var result = D.customShows.parseOrder("3, 1; 5", 6);
  assert(result.invalid === false, "valid order rejected");
  assert(result.order.join(",") === "2,0,4", "order not preserved");
  assert(D.customShows.parseOrder("2\n4", 6).order.join(",") === "1,3", "newline separator");
});

test("parseOrder rejects out-of-range, duplicate and non-numeric entries", function () {
  g.reset();
  ["7", "2, 2", "abc", "", "0"].forEach(function (text) {
    var result = D.customShows.parseOrder(text, 6);
    assert(result.invalid === true && result.order.length === 0, "accepted " + text);
  });
});

test("create/find/remove keep an ordered subset with a defensive copy", function () {
  g.reset();
  var input = [2, 0, 4];
  var show = D.customShows.create("Demo", input);
  input.push(9);
  assert(show.order.length === 3 && show.order[2] === 4, "input array was aliased");
  assert(D.customShows.list().length === 1, "show not stored");
  assert(D.customShows.find(show.id).id === show.id, "find failed");
  D.customShows.remove(show.id);
  assert(D.customShows.list().length === 0 && D.customShows.find(show.id) === null, "remove failed");
});

test("start() plays the subset first slide and records the return slide", function () {
  var show = activeShow([2, 0, 4], 1);
  assert(D.customShows.isActive() === true, "show not active");
  assert(state.jumps[0] === 2 && state.current === 2, "first show slide not played");
  assert(D.customShows.active().returnTo === 1, "return slide not recorded");
  assert(D.customShows.active().order.join(",") === show.order.join(","), "active order mismatch");
  assert(state.renderCalls > 0, "console was not asked to re-render");
  assert(lastKey() === "slides-delivery-shows-started", "start status missing");
});

test("next/prev indices walk the show order then return to the deck", function () {
  activeShow([2, 0, 4], 1);
  assert(D.customShows.nextIndex(2) === 0, "second show slide");
  assert(D.customShows.nextIndex(0) === 4, "third show slide");
  assert(D.customShows.nextIndex(4) === 1, "returnTo after last show slide");
  assert(D.customShows.prevIndex(2) === 2, "previous at first show slide clamps");
  assert(D.customShows.prevIndex(0) === 2, "previous from second show slide");
  assert(D.customShows.prevIndex(4) === 0, "previous from last show slide");
});

test("advance() on the last show slide finishes and returns to the original slide", function () {
  activeShow([2, 0, 4], 1);
  D.customShows.advance();
  assert(state.current === 0 && D.customShows.isActive() === true, "first advance");
  D.customShows.advance();
  assert(state.current === 4 && D.customShows.isActive() === true, "second advance");
  D.customShows.advance();
  assert(D.customShows.isActive() === false, "show did not finish");
  assert(state.current === 1 && state.jumps[state.jumps.length - 1] === 1, "deck not restored");
  assert(lastKey() === "slides-delivery-shows-finished", "finish status missing");
});

// ── Sections ──────────────────────────────────────────────────────────
test("ordered() sorts sections deterministically by start", function () {
  var all = sectionDeck();
  assert(all.map(function (s) { return s.start; }).join(",") === "0,3,7", "start order");
  assert(D.sections.ordered().map(function (s) { return s.id; }).join(",") ===
    all.map(function (s) { return s.id; }).join(","), "ordering is not deterministic");
});

test("owner() maps every slide to exactly one section", function () {
  var all = sectionDeck();
  assert(D.sections.owner(0, all).start === 0 && D.sections.owner(2, all).start === 0, "first section");
  assert(D.sections.owner(3, all).start === 3 && D.sections.owner(6, all).start === 3, "second section");
  assert(D.sections.owner(7, all).start === 7 && D.sections.owner(9, all).start === 7, "last section");
});

test("bounds() partitions the deck with no gaps or overlaps", function () {
  var all = sectionDeck();
  var ranges = all.map(function (section) { return D.sections.bounds(section, all); });
  assert(ranges[0].from === 0 && ranges[0].to === 2, "first range");
  assert(ranges[1].from === 3 && ranges[1].to === 6, "second range");
  assert(ranges[2].from === 7 && ranges[2].to === 9, "last range extends to deck end");
  ranges.forEach(function (range, index) {
    if (index > 0) assert(range.from === ranges[index - 1].to + 1, "partition gap/overlap");
  });
});

test("section neighbours drive navigation and the default section is protected", function () {
  sectionDeck();
  state.current = 4;
  var neighbours = D.sections.neighbours(4);
  assert(neighbours.previous.start === 0 && neighbours.next.start === 7, "neighbours");
  assert(D.sections.jumpNext() === true && state.current === 7, "jumpNext");
  state.current = 4;
  assert(D.sections.jumpPrev() === true && state.current === 0, "jumpPrev");
  state.current = 0;
  assert(D.sections.jumpPrev() === false, "jumpPrev at first section");
  state.current = 9;
  assert(D.sections.jumpNext() === false, "jumpNext at last section");
  assert(D.sections.remove("default") === false, "default section was removable");
});

// ── Captions ──────────────────────────────────────────────────────────
test("caption style sanitizes colours and clamps size to the supported range", function () {
  g.reset();
  var style = D.captions.style();
  assert(style.color === "#f8fafc" && style.background === "#111827" && style.size === 20, "defaults");
  D.patch(function (data) {
    data.captions.style = { color: "javascript:alert(1)", background: "red", size: 999 };
  });
  style = D.captions.style();
  assert(style.color === "#f8fafc" && style.background === "#111827" && style.size === 48, "sanitized");
  D.patch(function (data) { data.captions.style = { color: "#abc", size: 3 }; });
  style = D.captions.style();
  assert(style.color === "#abc" && style.size === 10, "lower clamp");
});

test("toggle() switches captions on/off and reports each state", function () {
  g.reset();
  assert(D.captions.isOn() === false, "initial caption state");
  assert(D.captions.toggle() === true && D.captions.isOn() === true, "toggle on");
  assert(lastKey() === "slides-delivery-captions-on", "on status missing");
  assert(D.captions.toggle() === false && D.captions.isOn() === false, "toggle off");
  assert(lastKey() === "slides-delivery-captions-off", "off status missing");
  assert(D.load().captions.on === false, "toggle not persisted");
});

// ── Rehearsal ─────────────────────────────────────────────────────────
test("setTarget() stores floored non-negative seconds per slide", function () {
  g.reset();
  assert(D.rehearsal.setTarget(2, "45.9") === 45, "fractional target");
  assert(D.rehearsal.setTarget(0, -3) === 0, "negative target");
  assert(D.rehearsal.targets()[2] === 45 && D.rehearsal.targets()[0] === 0, "targets not stored");
  assert(D.rehearsal.targets()[1] === undefined, "unset slide gained a target");
});

test("lap() records per-slide durations and cumulative totals", function () {
  g.reset();
  state.slideCount = 10;
  // The shipped lap() only records when the previous lap timestamp is > 0,
  // so arm the stopwatch at a realistic positive clock before advancing it.
  g.setClock(1000);
  D.rehearsal.reset();
  g.advanceClock(7000);
  state.current = 1;
  D.rehearsal.lap();
  g.advanceClock(3000);
  state.current = 2;
  D.rehearsal.lap();
  var actual = D.rehearsal.actual();
  assert(actual["0"] === 7 && actual["1"] === 3, "per-slide durations");
  assert(actual["2"] === undefined, "current slide recorded too early");
  var total = 0;
  Object.keys(actual).forEach(function (key) { total += actual[key]; });
  assert(total === 10, "total duration mismatch");
});

test("rows() computes per-slide delta and over/under/missing status", function () {
  g.reset();
  D.rehearsal.setTarget(0, 5);
  D.rehearsal.setTarget(1, 10);
  D.patch(function (data) { data.rehearsal.actual = { 0: 7, 1: 8, 2: 3 }; });
  var rows = D.rehearsal.rows();
  assert(rows.length === 3, "expected three summary rows");
  assert(rows[0].index === 0 && rows[0].delta === 2 && rows[0].status === "over", "over row");
  assert(rows[1].index === 1 && rows[1].delta === -2 && rows[1].status === "under", "under row");
  assert(rows[2].index === 2 && rows[2].target === null && rows[2].actual === 3 &&
    rows[2].status === "missing", "missing-target row");
});

test("apply() records a local appliedAt timestamp and reports status", function () {
  g.reset();
  g.setClock(12345);
  assert(D.rehearsal.apply() === true, "apply failed");
  assert(D.load().rehearsal.appliedAt === 12345, "appliedAt not persisted");
  assert(lastKey() === "slides-delivery-rehearsal-applied", "apply status missing");
});

// ── Handouts ──────────────────────────────────────────────────────────
test("[UAT:SLIDE-06] pageCount() computes pages for 1/2/4/6-up layouts", function () {
  assert(D.handouts.ALLOWED.join(",") === "1,2,4,6", "allowed layout set");
  assert(D.handouts.pageCount(10, 1) === 10, "1-up");
  assert(D.handouts.pageCount(10, 2) === 5, "2-up");
  assert(D.handouts.pageCount(10, 4) === 3, "4-up");
  assert(D.handouts.pageCount(10, 6) === 2, "6-up");
  assert(D.handouts.pageCount(6, 6) === 1 && D.handouts.pageCount(7, 6) === 2, "exact fill");
  assert(D.handouts.pageCount(0, 4) === 0 && D.handouts.pageCount("bad", 4) === 0, "empty deck");
});

test("setPerPage() persists allowed values and rejects unsupported ones", function () {
  g.reset();
  assert(D.handouts.perPage() === 4, "default layout");
  assert(D.handouts.setPerPage(6) === 6, "6-up not accepted");
  assert(D.handouts.perPage() === 6 && D.load().handouts.perPage === 6, "layout not persisted");
  assert(D.handouts.setPerPage(3) === 6 && D.handouts.perPage() === 6, "unsupported layout accepted");
  assert(D.handouts.setPerPage("4") === 4, "numeric string not accepted");
  D.patch(function (data) { data.handouts.perPage = 3; });
  assert(D.handouts.perPage() === 4, "corrupt stored layout not fallback");
});

// ── Recording ─────────────────────────────────────────────────────────
test("start/stop toggle recording state and account for duration", function () {
  g.reset();
  g.setClock(1000);
  assert(D.recording.isActive() === false, "initial recording state");
  assert(D.recording.start() === true && D.recording.isActive() === true, "start");
  assert(D.recording.start() === false, "double start accepted");
  g.setClock(11000);
  assert(D.recording.elapsed() === 10000, "running duration");
  assert(D.recording.stop() === true && D.recording.isActive() === false, "stop");
  assert(D.load().recording.duration === 10000, "duration not persisted");
  assert(D.recording.elapsed() === 10000, "stopped duration");
  g.setClock(20000);
  assert(D.recording.elapsed() === 10000, "duration kept growing after stop");
  assert(D.recording.stop() === false, "double stop accepted");
});

test("resume() restores an active recording from persisted state", function () {
  g.reset();
  D.patch(function (data) {
    data.recording.active = true;
    data.recording.startedAt = 1000;
    data.recording.duration = 2500;
  });
  g.setClock(4000);
  assert(D.recording.isActive() === true, "persisted active flag ignored");
  assert(D.recording.resume() === true, "resume failed");
  assert(D.recording.elapsed() === 3000, "resumed duration");
  assert(D.load().recording.duration === 2500, "resume overwrote stored duration");
  D.recording.stop();
});

// ── STATIC source assertions (DOM-only render paths / absent features) ─
test("STATIC: jump() clamps the target before navigating", function () {
  var src = g.source("slides/modules/51_delivery_state.js");
  assert(/D\.jump = function/.test(src), "jump definition missing");
  assert(/Math\.max\(0,\s*Math\.min\(count - 1,/.test(src), "bounds clamp missing in jump()");
});

test("STATIC: captions render one sample cue; no timed cue lookup ships", function () {
  var src = g.source("slides/modules/55_delivery_captions.js");
  assert(/stm-transcript-cue sd-caption-cue/.test(src), "sample cue markup missing");
  assert(/"slides-delivery-captions-sample"/.test(src), "sample cue label missing");
  assert(typeof D.captions.cueAt === "undefined", "unexpected runtime cue lookup");
  assert(!/parseVtt|cueAt|activeCue/.test(src), "timed cue engine unexpectedly in delivery module");
});

test("STATIC: handout render lays out page*size+cell and marks empty slots", function () {
  var src = g.source("slides/modules/57_delivery_handouts.js");
  assert(/Math\.ceil\(count \/ size\)/.test(src), "page count formula missing");
  assert(/page \* size \+ cell/.test(src), "slide-to-cell mapping missing");
  assert(/cell < size/.test(src) && /sd-handout-cell-empty/.test(src), "empty-slot handling missing");
});
