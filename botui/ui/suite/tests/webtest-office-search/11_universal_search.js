/**
 * RUNTIME + KNOWN BUG tests executing the REAL shipped js/gb-search.js.
 * The search bar is the browser face of GET /api/ui/search (unified_search.rs).
 */
"use strict";

var h = require("./00_harness");
var fx = require("./02_sandbox");
var test = h.test;
var knownBug = h.knownBug;
var assert = h.assert;
var equal = h.assertEqual;

function clickableItems(panel) {
  return panel.children.filter(function (node) { return typeof node.onclick === "function"; });
}

test("#2505: shipped universal search bar mounts unmodified", function () {
  var f = fx.create();
  assert(f.win.GBUniversalSearch, "window.GBUniversalSearch missing after module load");
  assert(typeof f.win.GBUniversalSearch.mount === "function", "mount missing");
  assert(typeof f.win.GBUniversalSearch.close === "function", "close missing");
  f.mountSearch();
  assert(f.searchInput.listeners.input.length === 1, "input listener not installed");
  assert(f.searchInput.listeners.keydown.length === 1, "keydown listener not installed");
});

test("#2505: queries shorter than the minimum do not call the search API", function () {
  var f = fx.create();
  f.mountSearch();
  f.searchQuery("a");
  equal(f.fetchCalls.length, 0, "one-character query must not fetch");
  equal(f.timers.length, 0, "one-character query must not schedule work");
  f.searchQuery("");
  equal(f.fetchCalls.length, 0, "empty query must not fetch");
});

test("#2505: debounced search requests the unified endpoint with a bearer token", function () {
  var f = fx.create();
  f.mountSearch();
  f.searchQuery("termination notice");
  equal(f.timers.length, 1, "one debounce timer expected");
  equal(f.timerDelays[0], 300, "shipped debounce delay");
  f.flushTimers();
  equal(f.fetchCalls.length, 1, "one search fetch expected");
  equal(f.fetchCalls[0].url, "/api/ui/search?q=termination%20notice", "search URL");
  equal(f.fetchCalls[0].options.headers.Authorization, "Bearer fixed-token", "auth header");
});

test("#2505: results are grouped by app with shipped labels and rendered", function () {
  var f = fx.create();
  f.mountSearch();
  f.searchQuery("termination");
  f.flushTimers();
  var panel = f.searchPanel();
  assert(panel, "search panel missing");
  equal(panel.style.display, "block", "panel must be visible");
  assert(panel.innerHTML.indexOf("People") >= 0, "People app label missing");
  assert(panel.innerHTML.indexOf("Drive") >= 0, "Drive app label missing");
  assert(panel.innerHTML.indexOf("Ada Lovelace") >= 0, "result title missing");
  equal(clickableItems(panel).length, 2, "one clickable result per returned row");
});

test("#2505: person results deep-link straight to the record id", function () {
  var f = fx.create();
  f.mountSearch();
  f.searchQuery("ada");
  f.flushTimers();
  clickableItems(f.searchPanel())[0].onclick();
  equal(f.deepLinks.length, 1, "person deep link count");
  equal(f.deepLinks[0].api, "WindowManager", "person deep link API");
  equal(f.deepLinks[0].app, "people", "person deep link app");
  equal(f.deepLinks[0].params.person_id, "p-1", "record id must survive the deep link");
  equal(f.deepLinks[0].options.ownerId, "search", "deep link owner");
});

test("#2505: non-person results fall back to the UI orchestrator", function () {
  var f = fx.create();
  var focused = [];
  f.win.GBUiOrchestrator = { focusEntity: function (row) { focused.push(row); } };
  f.mountSearch();
  f.searchQuery("variance");
  f.flushTimers();
  clickableItems(f.searchPanel())[1].onclick();
  equal(focused.length, 1, "orchestrator must focus one entity");
  equal(focused[0].type, "file", "focused entity type");
  equal(f.deepLinks.length, 0, "non-person results must not deep-link");
});

test("#2505: Escape closes the rendered search panel", function () {
  var f = fx.create();
  f.mountSearch();
  f.searchQuery("termination");
  f.flushTimers();
  equal(f.searchPanel().style.display, "block", "panel open before Escape");
  f.searchInput.dispatchEvent(f.event({ type: "keydown", key: "Escape" }));
  equal(f.searchPanel().style.display, "none", "Escape must close the panel");
});

test("#2505: a 401 response degrades to the sign-in hint without crashing", function () {
  var f = fx.create({ searchStatus: 401 });
  f.mountSearch();
  f.searchQuery("termination");
  f.flushTimers();
  var panel = f.searchPanel();
  assert(panel, "panel must still exist on 401");
  equal(panel.style.display, "block", "401 hint must be visible");
  assert(panel.innerHTML.indexOf("Sign in to search") >= 0, "sign-in hint missing");
});

test("#2505: the empty-result panel escapes the typed query", function () {
  var marker = String.fromCharCode(60) + "img src=x" + String.fromCharCode(62);
  var f = fx.create({ searchResults: [] });
  f.mountSearch();
  f.searchQuery(marker);
  f.flushTimers();
  var html = f.searchPanel().innerHTML;
  assert(html.indexOf("&lt;img") >= 0, "query must be escaped");
  assert(html.indexOf(marker) < 0, "raw markup must not reach the panel");
});

knownBug("#2505 KNOWN BUG: clearing a query below the minimum resurrects a stale search", function () {
  /* Source facts in shipped gb-search.js: onInput() only calls clearTimeout(timer)
     in the q.length >= MIN_QUERY_LEN branch (line ~192). The short-query branch
     (line ~189) closes the panel but leaves the pending debounce alive. doSearch()
     then reopens the panel with the OLD query while the input shows the new one. */
  var f = fx.create();
  f.mountSearch();
  f.searchQuery("termination");
  equal(f.timers.length, 1, "first query schedules a debounce");
  f.searchQuery("t");
  equal(f.timers.length, 1, "OBSERVED: short query did NOT clear the pending timer");
  f.flushTimers();
  equal(f.fetchCalls.length, 1, "OBSERVED: stale query still reached the API");
  equal(f.fetchCalls[0].url, "/api/ui/search?q=termination", "OBSERVED: old query fetched");
  equal(f.searchInput.value, "t", "input shows the shortened query");
  equal(f.searchPanel().style.display, "block", "OBSERVED: stale results reopened the panel");
  h.note("KNOWN BUG (reported, not patched): botui/ui/suite/js/gb-search.js:187-195 — onInput() closes the panel when the query has fewer than 2 characters but never clearTimeout(timer). A pending debounce from a longer query still fires doSearch(oldQuery), which repaints the panel while the input shows the shortened text. Move clearTimeout(timer) above the length guard in production.");
});
