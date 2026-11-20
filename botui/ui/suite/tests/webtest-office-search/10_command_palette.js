/**
 * RUNTIME tests executing the REAL shipped js/command-palette.js.
 * Every assertion below observes production code running in the vm sandbox.
 */
"use strict";

var h = require("./00_harness");
var fx = require("./02_sandbox");
var test = h.test;
var knownBug = h.knownBug;
var assert = h.assert;
var equal = h.assertEqual;

test("#2505: shipped command palette loads unmodified and exposes its shell API", function () {
  var f = fx.create();
  assert(f.win.GBCommandPalette, "window.GBCommandPalette missing after module load");
  assert(typeof f.win.GBCommandPalette.toggle === "function", "toggle missing");
  assert(typeof f.win.GBCommandPalette.open === "function", "open missing");
  assert(typeof f.win.GBCommandPalette.close === "function", "close missing");
  assert(f.ctx.window === f.win, "vm window must be the shipped window object");
});

test("#2505: Ctrl+K opens the palette from the shipped catalog and translates via gbT", function () {
  var f = fx.create();
  var shortcut = f.ctrlK();
  equal(f.fetchCalls.length, 1, "catalog fetch count");
  equal(f.fetchCalls[0].url, "/api/apps/catalog", "catalog URL");
  assert(f.overlay(), "no overlay appended");
  assert(f.items().length > 0, "no rendered palette items");
  ["palette-placeholder", "palette-open-app", "palette-kind-app", "palette-hint"].forEach(function (key) {
    assert(f.gbTCalls.some(function (call) { return call.key === key; }), "gbT key not used: " + key);
  });
  equal(shortcut.stopImmediatePropagationCalls, 1, "Ctrl+K must stop immediate propagation");
  equal(shortcut.preventDefaultCalls, 1, "Ctrl+K must prevent the browser default");
  f.ctrlK();
  assert(!f.overlay(), "second Ctrl+K must close the palette");
});

test("#2505: palette query matches across apps and their commands (cross-app surface)", function () {
  var f = fx.create();
  f.ctrlK();
  f.typeQuery("slide");
  var html = f.overlay().innerHTML;
  assert(html.indexOf("Slides") >= 0, "Slides app missing");
  assert(html.indexOf("New deck") >= 0, "Slides command missing");
  assert(f.items().length === 2, "expected exactly app + command, got " + f.items().length);
  f.typeQuery("zzz-no-match-token");
  assert(f.items().length === 0, "no-match query must yield no items");
  assert(f.overlay().innerHTML.indexOf("No matching apps or commands") >= 0, "empty state missing");
});

test("#2505: toolwindow apps are excluded when the registry has normal apps", function () {
  var f = fx.create();
  f.ctrlK();
  assert(f.overlay().innerHTML.indexOf("Vibe tool") < 0, "toolwindow app leaked into palette");
});

knownBug("#2505 KNOWN BUG: catalog fallback re-admits vibe- apps when no normal app remains", function () {
  /* appsForQuery() filters toolwindow and vibe- entries from APPS_REGISTRY, but
     its `if (!apps.length)` fallback re-reads state.catalog without that filter.
     With a registry containing only Vibe apps the excluded entries come back. */
  var vibed = fx.create({ catalog: [{ id: "vibe-project", title: "Vibe Project", commands: [] }] });
  vibed.ctrlK();
  equal(vibed.items().length, 1, "OBSERVED: excluded app reappeared through the fallback");
  assert(vibed.overlay().innerHTML.indexOf("Vibe Project") >= 0,
    "OBSERVED: vibe- prefix leaked into the palette");
  h.note("KNOWN BUG (reported, not patched): js/command-palette.js:58-61 — the empty-registry fallback filters only enabled/compiled, silently undoing the #1312 toolwindow and vibe- exclusions. Apply the same predicate to both lists in production.");
});

test("#2505: keyboard navigation moves the active item and Enter runs it", function () {
  var f = fx.create();
  f.ctrlK();
  var first = f.items()[0];
  assert(first.classList.contains("gb-palette-item-active"), "first item must start active");
  var input = f.overlay().querySelector("#gb-palette-input");
  input.dispatchEvent(f.event({ type: "keydown", key: "ArrowDown", target: input }));
  assert(f.items()[1].classList.contains("gb-palette-item-active"), "ArrowDown must move selection");
  input.dispatchEvent(f.event({ type: "keydown", key: "Enter", target: input }));
  equal(f.launches.length, 1, "Enter must launch one app");
  assert(f.launches[0].appId.length > 0, "launch missing app id");
});

test("#2505: a command deep link without placeholders opens the exact app", function () {
  var f = fx.create();
  f.ctrlK();
  f.typeQuery("deck");
  var items = f.items();
  equal(items.length, 2, "app + command expected");
  var input = f.overlay().querySelector("#gb-palette-input");
  input.dispatchEvent(f.event({ type: "keydown", key: "ArrowDown", target: input }));
  input.dispatchEvent(f.event({ type: "keydown", key: "Enter", target: input }));
  equal(f.deepLinks.length, 1, "deep link count");
  equal(f.deepLinks[0].api, "window", "no-placeholder path must use window.openDeepLink");
  equal(f.deepLinks[0].app, "slides", "deep link app");
  equal(JSON.stringify(f.deepLinks[0].params), "{}", "deep link params");
});

test("#2505: deep-link placeholders are prompted in sequence and collected", function () {
  var f = fx.create();
  f.ctrlK();
  f.typeQuery("Open document");
  equal(f.items().length, 1, "one command expected");
  var input = f.overlay().querySelector("#gb-palette-input");
  input.dispatchEvent(f.event({ type: "keydown", key: "Enter", target: input }));
  equal(f.prompts.length, 1, "one placeholder prompt expected");
  assert(f.prompts[0].indexOf("doc_id") >= 0, "prompt must name the placeholder");
  assert(f.gbTCalls.some(function (call) { return call.key === "palette-value-prompt"; }),
    "prompt translation key used");
  equal(f.deepLinks.length, 1, "deep link count");
  equal(f.deepLinks[0].app, "docs", "deep link app");
  equal(f.deepLinks[0].params.doc_id, "doc-42", "collected value must reach the deep link");
});

test("#2505: optional {key?} placeholders are excluded from prompting", function () {
  var f = fx.create();
  f.ctrlK();
  f.typeQuery("Scoped document");
  equal(f.items().length, 1, "one command expected");
  var input = f.overlay().querySelector("#gb-palette-input");
  input.dispatchEvent(f.event({ type: "keydown", key: "Enter", target: input }));
  equal(f.prompts.length, 1, "only the required placeholder is asked");
  assert(f.prompts[0].indexOf("record?") < 0, "optional placeholder must not be prompted");
  equal(f.deepLinks[0].params.doc_id, "doc-42", "required value collected");
  assert(!Object.prototype.hasOwnProperty.call(f.deepLinks[0].params, "record"),
    "optional value must stay absent");
});

test("#2505: derived endpoint commands are executed through the chat api surface", function () {
  var f = fx.create();
  var chatEvents = 0;
  f.chatInput.addEventListener("input", function () { chatEvents++; });
  f.ctrlK();
  f.typeQuery("Recalculate");
  var input = f.overlay().querySelector("#gb-palette-input");
  input.dispatchEvent(f.event({ type: "keydown", key: "Enter", target: input }));
  assert(f.chatInput.value.indexOf("/api/sheet/recalculate") >= 0, "api path missing in chat text");
  equal(chatEvents, 1, "chat input event must be dispatched");
});

test("#2505: palette HTML-escapes titles from the app catalog", function () {
  var marker = String.fromCharCode(60) + "img src=x" + String.fromCharCode(62);
  var f = fx.create({ catalog: [{ id: "evil", title: marker, commands: [] }] });
  f.ctrlK();
  var html = f.overlay().innerHTML;
  assert(html.indexOf("&lt;img") >= 0, "title must be HTML-escaped");
  assert(html.indexOf(marker) < 0, "raw markup must not reach the palette");
});

test("#2505: guided tours are replayable from the palette with force", function () {
  var f = fx.create({ tourIds: ["drive"] });
  f.ctrlK();
  assert(f.overlay().innerHTML.indexOf("Show tour: Drive") >= 0, "tour result missing");
  assert(f.gbTCalls.some(function (call) { return call.key === "palette-show-tour"; }),
    "tour translation key used");
  var match = f.overlay().innerHTML.match(
    /data-index="(\d+)"[^>]*><span class="gb-palette-label"><b>Show tour:/);
  assert(match, "tour item not found");
  var index = parseInt(match[1], 10);
  var tourItem = f.items().filter(function (item) {
    return item.getAttribute("data-index") === String(index);
  })[0];
  assert(tourItem, "tour item node missing");
  tourItem.dispatchEvent(f.event({ type: "click", target: tourItem }));
  equal(f.tourStarts.length, 1, "tour must start once");
  equal(f.tourStarts[0].appId, "drive", "tour app");
  equal(f.tourStarts[0].options.force, true, "tour must replay with force");
});
