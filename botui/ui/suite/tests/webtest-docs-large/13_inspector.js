/**
 * RUNTIME #2533: pure focus/doc-id helpers, document-scoped persistence,
 * dialog shell, inspector render and delegated boot wiring (modules 84/90/91).
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var dom = require("./01_dom");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var deep = h.deepEqual;
var fixture = f.fixture;
var byClass = dom.byClass;

function openPanel(fx) {
  f.article(fx, "doc-A");
  var shell = fx.NS.dialog.shell();
  var refs = fx.NS.dialog.render(shell);
  fx.NS.dialog.refresh();
  return { shell: shell, refs: refs };
}

test("RUNTIME #2533: focusMove is a pure Escape/Tab trap contract", function () {
  var NS = fixture().NS;
  deep(NS.focusMove(["a", "b", "c"], "b", { key: "Escape" }), { action: "close" }, "Escape closes");
  deep(NS.focusMove(["a", "b", "c"], "b", { key: "Tab" }),
    { action: "focus", index: 2, key: "c" }, "Tab moves forward");
  deep(NS.focusMove(["a", "b", "c"], "b", { key: "Tab", shiftKey: true }),
    { action: "focus", index: 0, key: "a" }, "Shift+Tab moves backward");
  deep(NS.focusMove(["a", "b", "c"], "c", { key: "Tab" }),
    { action: "focus", index: 0, key: "a" }, "Tab wraps at the end");
  deep(NS.focusMove([], null, { key: "Tab" }), { action: "none" }, "empty key list is inert");
  deep(NS.focusMove(["a"], "z", { key: "Tab", shiftKey: true }),
    { action: "focus", index: 0, key: "a" }, "unknown current focuses the last key");
});

test("RUNTIME #2533: document id, scoped storage key and local-only notes", function () {
  var fx = fixture();
  f.article(fx, "doc-A");
  equal(fx.NS.docId(), "doc-A", "article dataset wins");
  equal(fx.NS.storeKey(), "docs-large:doc-A", "storage key is document-scoped");
  var titled = fixture();
  f.titleInput(titled, "doc-title-value");
  equal(titled.NS.docId(), "doc-title-value", "title input is the fallback resolver");
  equal(fixture().NS.docId(), "current", "default id when no article is present");
  var status = f.registerNode(fx, "docs-large-status", fx.doc.createElement("div"));
  fx.NS.status("<status>");
  equal(status.textContent, "<status>", "status writes to the polite region");
  equal(fx.NS.localNote(), "Local only — preview, not the server load path", "local-only note");
  assert(fx.NS.deferredNote().indexOf("Deferred:") === 0, "deferred work is named, not shipped");
});

test("RUNTIME #2533: stored state reloads and is sanitised under its own key", function () {
  var fx = fixture();
  f.article(fx, "doc-A");
  var NS = fx.NS;
  var state = NS.defaultState();
  state.sizeMb = 10;
  state.queue = [{ blockId: "blk-0", bytes: 68 }];
  NS.state = state;
  equal(NS.persist(), true, "persist writes to localStorage");
  assert(fx.store["docs-large:doc-A"], "document-scoped key written");
  var reloaded = NS.loadState();
  equal(reloaded.sizeMb, 10, "valid size survives a reload");
  equal(reloaded.queue.length, 1, "valid queue survives a reload");

  fx.store["docs-large:doc-A"] = JSON.stringify({
    version: 9, sizeMb: 99, mode: "weird", etag: "", etagSent: "",
    queue: "x", editingEnabled: true, fullBytes: -5,
  });
  var clean = NS.loadState();
  equal(clean.version, 1, "version reset to the shipped constant");
  equal(clean.sizeMb, 5, "unknown size falls back to the first declared size");
  equal(clean.mode, "automatic", "unknown mode falls back to automatic");
  equal(clean.etag, "v1", "empty ETag falls back to v1");
  equal(clean.etagSent, "v1", "empty sent ETag falls back to v1");
  deep(clean.queue, [], "non-array queue discarded");
  equal(clean.editingEnabled, true, "editing flag preserved");
  equal(clean.fullBytes, 0, "negative byte count clamped");

  fx.store["docs-large:doc-B"] = JSON.stringify({ sizeMb: 20 });
  fx.doc.querySelectorAll("article")[0].dataset.docId = "doc-B";
  equal(NS.loadState().sizeMb, 20, "a different document reads a different key");
});

test("RUNTIME #2533: dialog shell is modal, polite and focus-trapped", function () {
  var fx = fixture();
  var prior = fx.doc.createElement("button");
  fx.doc.body.appendChild(prior);
  prior.focus();
  var shell = fx.NS.dialog.shell();
  var closeButton = shell.root.querySelectorAll("button")[0];
  equal(shell.root.getAttribute("role"), "dialog", "dialog role");
  equal(shell.root.getAttribute("aria-modal"), "true", "modal dialog");
  var labelledBy = shell.root.getAttribute("aria-labelledby");
  assert(labelledBy && labelledBy.indexOf("gb-large-title-") === 0, "labelled by a generated title id");
  equal(shell.root.querySelectorAll("h3")[0].id, labelledBy, "title id matches aria-labelledby");
  equal(f.i18nKey(shell.root.querySelectorAll("h3")[0]), "docs-large-title", "title uses a catalog key");
  equal(shell.status.id, "docs-large-status", "status region is addressable");
  equal(shell.status.getAttribute("role"), "status", "status region role");
  equal(shell.status.getAttribute("aria-live"), "polite", "status region is polite");
  assert(fx.doc.body.querySelector(".gb-large-backdrop"), "backdrop appended to host");
  equal(fx.doc.activeElement, closeButton, "first focusable receives focus");
  fx.doc.dispatch("keydown", { key: "Tab", preventDefault: function () {} });
  equal(fx.doc.activeElement, closeButton, "single focusable stays trapped");
  fx.doc.dispatch("keydown", { key: "Escape", preventDefault: function () {} });
  equal(fx.doc.body.querySelectorAll(".gb-large-backdrop").length, 0, "Escape removes the dialog");
  equal(fx.doc.activeElement, prior, "focus restored to the trigger");
});

test("RUNTIME #2533: inspector renders scoped controls, status and baseline", function () {
  var panel = openPanel(fixture());
  var refs = panel.refs;
  var options = refs.controls.size.querySelectorAll("option");
  equal(options.length, 3, "three declared fixture sizes");
  deep(options.map(f.i18nKey), ["docs-large-size-5", "docs-large-size-10", "docs-large-size-20"],
    "options use catalog keys");
  equal(f.i18nKey(refs.controls.load), "docs-large-load", "load button catalog key");
  equal(f.i18nKey(refs.controls.mode), "docs-large-mode-auto", "mode button reflects automatic state");
  equal(f.i18nKey(refs.controls.incremental), "docs-large-save-incremental", "incremental key");
  equal(f.i18nKey(refs.controls.full), "docs-large-save-full", "full key");
  equal(f.i18nKey(refs.deg.enable), "docs-large-enable-anyway", "override key");
  equal(refs.controls.failure.getAttribute("role"), "status", "failure line is a status region");
  equal(refs.controls.failure.getAttribute("aria-live"), "polite", "failure line is polite");
  var baseline = byClass(panel.shell.body, "gb-large-baseline")[0];
  assert(baseline && baseline.querySelector("table"), "baseline table embedded in dialog");
  equal(refs.win.host.children.length, 14, "default fixture mounts a bounded window");
  equal(refs.win.mounted.textContent, "14 blocks", "mounted count shown");
  equal(refs.insp.range.textContent, "Mounted range: 0–14 of 5120", "inspector shows the mounted range");
  equal(refs.insp.depth.textContent, "0", "inspector shows queue depth");
  equal(refs.controls.bytes.textContent, "Incremental bytes: 0", "wire-byte estimate shown");
});

test("RUNTIME #2533: load, mode, ETag and save buttons drive the shipped state", function () {
  var fx = fixture();
  var panel = openPanel(fx);
  var refs = panel.refs;
  refs.controls.size.value = "20";
  refs.controls.load.dispatch("click", {});
  equal(fx.NS.state.sizeMb, 20, "load selects the 20 MB fixture");
  equal(fx.NS.state.editingEnabled, false, "large fixture starts read-only");
  equal(refs.insp.depth.textContent, "0", "loading resets the queue display");
  assert(refs.win.host.children.length <= 14, "large fixture still mounts a bounded window");
  equal(refs.win.host.children[0].getAttribute("contenteditable"), "false", "large fixture is read-only");
  assert(byClass(panel.shell.body, "gb-large-banner-warn").length === 1, "degraded banner shown");
  assert(refs.deg.enable.style.display === "", "override action is visible");

  refs.controls.mode.dispatch("click", {});
  equal(f.i18nKey(refs.controls.mode), "docs-large-mode-manual", "mode toggle updates the key");

  refs.controls.etag.value = "v9";
  refs.controls.etag.dispatch("change", {});
  refs.controls.incremental.dispatch("click", {});
  assert(refs.controls.failure.textContent.indexOf("Stale save rejected") === 0,
    "stale save surfaces a human message");
  equal(refs.insp.depth.textContent, "1", "rejected save keeps the queue");
  equal(fx.NS.state.etag, "v1", "rejected save does not advance the ETag");

  refs.controls.etag.value = "v1";
  refs.controls.etag.dispatch("change", {});
  refs.controls.incremental.dispatch("click", {});
  equal(refs.controls.failure.textContent, "Saved 68 bytes", "successful incremental message");
  equal(refs.insp.depth.textContent, "0", "successful incremental drains the queue");
  equal(fx.NS.state.etag, "v2", "successful save advances the ETag");
});

test("RUNTIME #2533: full-body save over the ceiling reports 413, not a silent failure", function () {
  var panel = openPanel(fixture());
  var refs = panel.refs;
  refs.controls.size.value = "20";
  refs.controls.load.dispatch("click", {});
  refs.controls.full.dispatch("click", {});
  equal(refs.controls.failure.textContent,
    "Document too large for full save — use incremental / split", "actionable 413 message");
  assert(refs.controls.failure._classes.indexOf("gb-large-failure-error") >= 0, "error class applied");
  equal(refs.insp.depth.textContent, "0", "failed full save leaves the queue intact");
});

test("RUNTIME #2533: override and scroll input re-render the visible window", function () {
  var panel = openPanel(fixture());
  var refs = panel.refs;
  refs.controls.size.value = "20";
  refs.controls.load.dispatch("click", {});
  refs.deg.enable.dispatch("click", {});
  equal(refs.win.host.children[0].getAttribute("contenteditable"), "true", "override enables editing");
  equal(refs.deg.enable.style.display, "none", "override hides itself once used");
  refs.win.scroll.value = "100";
  refs.win.scroll.dispatch("input", {});
  equal(refs.win.mounted.textContent, "10 blocks", "scrolling to the tail mounts fewer rows");
  equal(refs.win.host.children[0].getAttribute("data-block-id"), "blk-020470",
    "tail window uses stable index-derived ids");
  equal(refs.insp.range.textContent, "Mounted range: 20470–20480 of 20480", "inspector tracks the scroll");
});

test("RUNTIME #2533: module 91 delegates open clicks and block-input enqueue", function () {
  var openFx = fixture();
  f.boot(openFx);
  var trigger = openFx.doc.createElement("button");
  trigger.setAttribute("data-gb-large-open", "panel");
  openFx.doc.body.appendChild(trigger);
  var prevented = 0;
  openFx.doc.dispatch("click", {
    target: trigger, preventDefault: function () { prevented += 1; }, stopPropagation: function () {},
  });
  assert(openFx.doc.body.querySelector(".gb-large-backdrop"), "delegated click opens the panel");
  equal(prevented, 1, "default browser action prevented");

  var editFx = fixture();
  f.boot(editFx);
  var block = editFx.doc.createElement("div");
  block.setAttribute("class", "gb-large-block");
  block.setAttribute("data-block-id", "blk-000007");
  block.textContent = "<edited>";
  editFx.doc.body.appendChild(block);
  editFx.doc.dispatch("input", { target: block });
  equal(editFx.NS.state.queue.length, 1, "block input enqueues one op");
  equal(editFx.NS.state.queue[0].blockId, "blk-000007", "op carries the stable block id");
  equal(editFx.NS.state.queue[0].bytes, 48 + 8, "op bytes follow the shipped overhead formula");
  assert(editFx.store["docs-large:current"], "edit is persisted under the document-scoped key");
  var plain = editFx.doc.createElement("div");
  editFx.doc.body.appendChild(plain);
  editFx.doc.dispatch("input", { target: plain });
  equal(editFx.NS.state.queue.length, 1, "non-block input is ignored");
});
