/**
 * Runtime tests for the SHIPPED docs model (modules 68..74).
 * Every assertion below calls a real exported helper on the real IIFEs.
 */
"use strict";

var h = require("./00_harness");
var loadModel = h.loadModel;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deepEqual = h.deepEqual;
var note = h.note;
var event = h.event;

function ids(list) { return list.map(function (item) { return item.id; }); }
function put(NS, type, payload, author) {
  var descriptor = { type: type, author: author || "You" };
  Object.keys(payload || {}).forEach(function (key) { descriptor[key] = payload[key]; });
  return NS.dispatch(descriptor);
}

// ── versioned schema + constructors ────────────────────────────────────

test("RUNTIME schema: versioned doc, typed blocks/marks, pure constructors", function () {
  var NS = loadModel().NS;
  var doc = NS.createDoc();
  equal(doc.schema_version, 1, "schema version");
  equal(NS.BLOCK_TYPES.length, 8, "block kind count");
  equal(NS.MARK_TYPES.length, 7, "inline mark count");
  assert(NS.validate(doc).ok, "fresh doc validates");
  equal(NS.createBlock("unknown", "x", "b-fallback").type, "paragraph", "unknown type falls back");

  var table = NS.createBlock("table", "", "b-table");
  deepEqual(table.attrs, { rows: 1, cols: 1, cells: {} }, "table attrs");
  var marks = [{ type: "bold" }];
  var span = NS.createSpan("text", marks, "s-fixed");
  marks.push({ type: "italic" });
  deepEqual(span.marks, [{ type: "bold" }], "marks cloned, not aliased");
  assert(NS.uid("b").indexOf("b-") === 0, "uid prefix");
  assert(NS.uid("b") !== NS.uid("b"), "uids are unique under stubbed crypto");
});

test("RUNTIME schema: validator rejects version, id, type, span, mark and anchor faults", function () {
  var NS = loadModel().NS;
  function check(mutate) {
    var doc = NS.createDoc();
    mutate(doc);
    return NS.validate(doc);
  }
  var version = check(function (d) { d.schema_version = 2; });
  assert(!version.ok && version.errors[0] === "schema:version:2", "version diagnostic");

  var blockId = check(function (d) { d.blocks.push(NS.createBlock("paragraph", "x", d.blocks[0].id)); });
  assert(!blockId.ok && blockId.errors.indexOf("schema:block-id:1") >= 0, "duplicate block id");

  var type = check(function (d) { d.blocks[0].type = "quote"; });
  assert(!type.ok && type.errors.indexOf("schema:block-type:0") >= 0, "unknown block type");

  var spanId = check(function (d) { d.blocks[0].spans.push(NS.createSpan("x", [], d.blocks[0].spans[0].id)); });
  assert(!spanId.ok && spanId.errors.indexOf("schema:span-id:0:1") >= 0, "duplicate span id");

  var text = check(function (d) { d.blocks[0].spans[0].text = 7; });
  assert(!text.ok && text.errors.indexOf("schema:span-text:0:0") >= 0, "non-string span text");

  var mark = check(function (d) { d.blocks[0].spans[0].marks = [{ type: "blink" }]; });
  assert(!mark.ok && mark.errors.indexOf("schema:mark-type:0:0:0") >= 0, "unknown inline mark");

  var anchor = check(function (d) { d.comments.push(NS.createComment("b-missing", "x", "Actor", "c-x")); });
  assert(!anchor.ok && anchor.errors.indexOf("schema:comment-anchor:0") >= 0, "dangling comment anchor");
});

test("RUNTIME state: document-scoped localStorage key and persisted state", function () {
  var hCase = loadModel();
  var NS = hCase.NS;
  var title = hCase.dom.makeElement("input");
  title.setAttribute("id", "docTitle");
  title.value = "doc-X";
  hCase.doc.body.appendChild(title);
  NS.loadState();
  NS.persist();
  assert(hCase.store["docs2423:doc-X:state"], "state persisted under doc-scoped key");
  assert(JSON.parse(hCase.store["docs2423:doc-X:state"]).ops instanceof Array, "stored ops array");
  deepEqual(NS.defaultState().undo, {}, "fresh undo stacks");
});

// ── operation log, inverses, undo/redo ─────────────────────────────────

test("RUNTIME ops: dispatch records author/timestamp/seq/inverse; history is a copy", function () {
  var hCase = loadModel();
  var NS = hCase.NS;
  hCase.clock.set(1234567890123);
  var block = NS.createBlock("paragraph", "body", "b-one");
  var op = put(NS, "insert", { afterId: null, block: block }, "Alpha");
  equal(op.seq, 1, "first seq");
  equal(op.author, "Alpha", "author recorded");
  equal(op.timestamp, 1234567890123, "clock stub used");
  equal(op.inverse.type, "delete", "inverse captured");
  equal(NS.state.ops.length, 1, "single log entry");
  equal(NS.undoDepth("Alpha"), 1, "author undo depth");
  var history = NS.history();
  history[0].seq = 999;
  equal(NS.state.ops[0].seq, 1, "history cloned");
  assert(NS.getDoc().blocks.some(function (b) { return b.id === "b-one"; }), "insert applied");
});

test("RUNTIME ops: delete/undo/redo is invertible through the reducer", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0].id;
  var block = NS.createBlock("paragraph", "gone", "b-del");
  put(NS, "insert", { afterId: root, block: block });
  var removed = put(NS, "delete", { ids: [block.id] });
  equal(removed.inverse.type, "restore-many", "delete inverse");
  equal(NS.getDoc().blocks.length, 1, "delete applied");
  NS.undo();
  equal(NS.getDoc().blocks.length, 2, "undo restores");
  assert(NS.validate(NS.getDoc()).ok, "restored document valid");
  NS.redo();
  equal(NS.getDoc().blocks.length, 1, "redo re-deletes");
});

test("[UAT:DOC-02] RUNTIME ops: format/unformat, move and table cells invert", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0];
  put(NS, "format", { spanIds: [root.spans[0].id], mark: { type: "bold" } });
  assert(NS.getDoc().blocks[0].spans[0].marks.some(function (m) { return m.type === "bold"; }), "format applied");
  NS.undo();
  equal(NS.getDoc().blocks[0].spans[0].marks.length, 0, "format undone");
  NS.redo();
  equal(NS.getDoc().blocks[0].spans[0].marks.length, 1, "format redone");

  var a = NS.createBlock("paragraph", "A", "b-a");
  var b = NS.createBlock("paragraph", "B", "b-b");
  var c = NS.createBlock("paragraph", "C", "b-c");
  put(NS, "insert", { afterId: root.id, block: a });
  put(NS, "insert", { afterId: a.id, block: b });
  put(NS, "insert", { afterId: b.id, block: c });
  put(NS, "move", { id: a.id, beforeId: c.id });
  deepEqual(ids(NS.getDoc().blocks.slice(1)), [b.id, a.id, c.id], "move applied");
  NS.undo();
  deepEqual(ids(NS.getDoc().blocks.slice(1)), [a.id, b.id, c.id], "move undone");

  var table = NS.createBlock("table", "", "b-table");
  put(NS, "insert", { afterId: root.id, block: table });
  put(NS, "table", { blockId: table.id, row: 0, col: 0, text: "cell" });
  equal(NS.getBlock(NS.getDoc(), table.id).attrs.cells["0:0"].text, "cell", "table cell written");
  NS.undo();
  equal(NS.getBlock(NS.getDoc(), table.id).attrs.cells["0:0"].text, "", "table cell inverted");
});

test("RUNTIME ops: comment/uncomment keeps an id-based anchor", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0];
  var comment = NS.createComment(root.id, "note", "Alpha", "c-one");
  equal(comment.blockId, root.id, "comment anchors to id, not HTML");
  put(NS, "comment", { comment: comment }, "Alpha");
  assert(NS.validate(NS.getDoc()).ok, "commented doc valid");
  put(NS, "uncomment", { id: comment.id }, "Alpha");
  equal(NS.getDoc().comments.length, 0, "uncomment applied");
  NS.undo("Alpha");
  deepEqual(NS.getDoc().comments, [comment], "comment restored identically");
});

test("RUNTIME undo: interleaved authors never revert a peer's operations", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0].id;
  var alpha1 = NS.createBlock("paragraph", "a1", "b-a1");
  var beta1 = NS.createBlock("paragraph", "b1", "b-b1");
  var alpha2 = NS.createBlock("paragraph", "a2", "b-a2");
  put(NS, "insert", { afterId: root, block: alpha1 }, "Alpha");
  put(NS, "insert", { afterId: alpha1.id, block: beta1 }, "Beta");
  put(NS, "insert", { afterId: beta1.id, block: alpha2 }, "Alpha");
  deepEqual(NS.history().map(function (op) { return op.author; }), ["Alpha", "Beta", "Alpha"], "audit authors");

  NS.undo("Alpha");
  assert(ids(NS.getDoc().blocks).indexOf(alpha2.id) < 0, "Alpha's last op undone");
  assert(ids(NS.getDoc().blocks).indexOf(beta1.id) >= 0, "peer op preserved");
  NS.undo("Beta");
  assert(ids(NS.getDoc().blocks).indexOf(beta1.id) < 0, "Beta's op undone");
  assert(ids(NS.getDoc().blocks).indexOf(alpha1.id) >= 0, "Alpha's earlier op preserved");
  NS.undo("Alpha");
  deepEqual(ids(NS.getDoc().blocks), [root], "only root remains");
  NS.redo("Alpha");
  assert(ids(NS.getDoc().blocks).indexOf(alpha1.id) >= 0, "Alpha redo independent");
  equal(NS.undoDepth("Beta"), 0, "Beta stack untouched by Alpha redo");
});

// ── stable anchors and ids ─────────────────────────────────────────────

test("RUNTIME ids: paste preserves block ids and undo/redo restores them", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0].id;
  var pasted = NS.normalizePaste("# Title\n- item\n\nplain line");
  equal(pasted.length, 3, "paste normalised to blocks");
  var before = ids(pasted);
  put(NS, "paste", { afterId: root, blocks: pasted }, "Alpha");
  deepEqual(ids(NS.getDoc().blocks.slice(1)), before, "paste keeps stable ids");
  assert(NS.validate(NS.getDoc()).ok, "pasted doc valid");
  NS.undo("Alpha");
  deepEqual(ids(NS.getDoc().blocks), [root], "paste inverted");
  NS.redo("Alpha");
  deepEqual(ids(NS.getDoc().blocks.slice(1)), before, "redo recreates identical ids");
});

test("RUNTIME anchors: replace re-anchors comments and the inverse flips them back", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0];
  put(NS, "comment", { comment: NS.createComment(root.id, "note", "Alpha", "c-anchor") }, "Alpha");
  var replacement = NS.createBlock("paragraph", "new", "b-new");
  put(NS, "replace", { id: root.id, block: replacement }, "Alpha");
  equal(NS.getDoc().comments[0].blockId, replacement.id, "forward re-anchor");
  assert(NS.validate(NS.getDoc()).ok, "replaced doc valid");
  NS.undo("Alpha");
  equal(NS.getDoc().comments[0].blockId, root.id, "inverse re-anchor");
  assert(NS.getBlock(NS.getDoc(), root.id) !== null, "original block restored");
  NS.redo("Alpha");
  equal(NS.getDoc().comments[0].blockId, replacement.id, "redo re-anchors again");
});

test("RUNTIME anchors: a comment survives 50 unrelated inserts above it", function () {
  var NS = loadModel().NS;
  var root = NS.getDoc().blocks[0].id;
  var target = NS.createBlock("paragraph", "clause 4.2", "b-target");
  put(NS, "insert", { afterId: root, block: target });
  put(NS, "comment", { comment: NS.createComment(target.id, "note", "Alpha", "c-target") }, "Alpha");
  for (var i = 0; i < 50; i += 1) {
    put(NS, "insert", { afterId: null, block: NS.createBlock("paragraph", "edit " + i, "b-edit-" + i) });
  }
  var doc = NS.getDoc();
  equal(NS.findBlockIndex(doc, target.id), 51, "target shifted by inserts");
  equal(doc.comments[0].blockId, target.id, "anchor unchanged");
  equal(NS.plainText(NS.getBlock(doc, target.id)), "clause 4.2", "target text unchanged");
  assert(NS.validate(doc).ok, "50-edit document valid");
});

test("RUNTIME KNOWN DEFECT: split undo corrupts ids (observed, not endorsed)", function () {
  var NS = loadModel().NS;
  var root = NS.state.doc.blocks[0];
  root.spans[0].text = "abcdef";
  var rightId = "b-right";
  put(NS, "split", { id: root.id, offset: 3, rightId: rightId, tailSpanId: "s-tail" });
  var split = NS.state.doc;
  deepEqual(split.blocks.map(function (b) { return NS.plainText(b); }), ["abc", "def"], "split text");
  equal(split.blocks[1].spans[0].id, "s-tail", "tail span id supplied by caller");
  equal(split.blocks[0].spans[0].id, root.spans[0].id, "head span id kept");
  NS.undo();
  var after = NS.state.doc;
  var repeats = after.blocks.filter(function (b) { return b.id === root.id; }).length;
  equal(repeats, 2, "observed: inverse duplicates the original block id");
  assert(!NS.validate(after).ok, "observed: validator rejects the corrupted document");
  note("CONFIRMED BUG #2423: split's inverse (70_model_ops.js previewInverse) replaces only the right block; the head survives, duplicating the source block id and text on undo.");
});

test("RUNTIME KNOWN DEFECT: merge undo duplicates the right block's spans", function () {
  var NS = loadModel().NS;
  var root = NS.state.doc.blocks[0];
  root.id = "b-left";
  root.spans[0].id = "s-left";
  root.spans[0].text = "alpha";
  var right = NS.createBlock("paragraph", "beta", "b-right");
  right.spans[0].id = "s-right";
  put(NS, "insert", { afterId: root.id, block: right });
  put(NS, "merge", { id: root.id, withId: right.id });
  equal(NS.plainText(NS.getDoc().blocks[0]), "alphabeta", "merge appends right spans");
  NS.undo();
  var left = NS.getBlock(NS.state.doc, root.id);
  var restoredRight = NS.getBlock(NS.state.doc, right.id);
  equal(NS.plainText(left), "alphabeta", "observed: left keeps appended spans");
  equal(NS.plainText(restoredRight), "beta", "observed: right restored with same span id");
  note("CONFIRMED BUG #2423: merge's inverse restores the right block but does not remove the right spans appended to the left block, duplicating text and span ids.");
});

test("RUNTIME KNOWN DEFECT: validator misses span-id collisions across blocks", function () {
  var NS = loadModel().NS;
  var doc = NS.createDoc();
  doc.blocks[0].id = "b-one";
  doc.blocks[0].spans[0].id = "s-shared";
  var other = NS.createBlock("paragraph", "other", "b-two");
  other.spans[0].id = "s-shared";
  doc.blocks.push(other);
  assert(NS.validate(doc).ok, "observed: duplicate span id in another block is accepted");
  note("CONFIRMED BUG #2423: 69_model_schema.js validate() scopes span-id uniqueness to each block, so the duplicate produced by merge undo is never reported.");
});

// ── one-way projections + paste/a11y helpers ───────────────────────────

test("RUNTIME projections: HTML/Markdown/JSON are read-only views of the schema", function () {
  var NS = loadModel().NS;
  var doc = NS.getDoc();
  doc.blocks = [
    NS.createBlock("heading", "Title", "b-h"),
    NS.createBlock("paragraph", "", "b-p"),
  ];
  doc.blocks[1].spans = [{ id: "s-bold", text: "a < b", marks: [{ type: "bold" }, { type: "link", value: "x" }] }];
  doc.blocks.push(NS.createBlock("page-break", "", "b-pb"));
  var snapshot = JSON.stringify(doc);
  var html = NS.toHtml(doc);
  var md = NS.toMarkdown(doc);
  assert(html.indexOf("<h2>Title</h2>") >= 0, "heading projection");
  assert(html.indexOf("&lt;") >= 0 && html.indexOf("a < b") < 0, "projection escapes text");
  assert(html.indexOf("<strong>") >= 0 && html.indexOf("<a href=\"x\">") >= 0, "inline marks projected");
  assert(html.indexOf("data-page-break") >= 0, "page break projected");
  assert(md.indexOf("## Title") >= 0 && md.indexOf("**a < b**") >= 0, "markdown projection");
  equal(NS.toJson(doc), JSON.stringify(doc, null, 2), "JSON projection");
  equal(JSON.stringify(doc), snapshot, "projections do not mutate the schema");
});

test("RUNTIME projections: paste normalisation and a11y/selection summary", function () {
  var hCase = loadModel();
  var NS = hCase.NS;
  var blocks = NS.normalizePaste("# Head\r\n* one\r\n\r\nplain text");
  deepEqual(blocks.map(function (b) { return b.type; }), ["heading", "list", "paragraph"], "line kinds");
  deepEqual(blocks.map(function (b) { return NS.plainText(b); }), ["Head", "one", "plain text"], "line text");
  deepEqual(NS.normalizePaste("   ").length, 1, "empty paste yields one block");

  var root = NS.getDoc().blocks[0];
  put(NS, "format", { spanIds: [root.spans[0].id], mark: { type: "italic" } });
  NS.setSelection(root.id, -4);
  equal(NS.state.selection.offset, 0, "negative offset clamped");
  var summary = NS.a11ySummary(NS.getDoc());
  equal(summary.blocks, 1, "summary blocks");
  equal(summary.spans, 1, "summary spans");
  equal(summary.marks, 1, "summary marks");
  equal(summary.comments, 0, "summary comments");
  equal(summary.selection.blockId, root.id, "selection block id");
  NS.localNote();
  assert(hCase.gbCalls.length > 0, "gbT stub consulted for localised strings");
});

// ── dialog/inspector/boot behaviour ────────────────────────────────────

test("RUNTIME UI: focusMove trap contract is pure and wraps correctly", function () {
  var NS = loadModel().NS;
  deepEqual(NS.focusMove(["a", "b", "c"], "a", { key: "Tab" }), { action: "focus", index: 1, key: "b" }, "Tab forward");
  deepEqual(NS.focusMove(["a", "b", "c"], "a", { key: "Tab", shiftKey: true }), { action: "focus", index: 2, key: "c" }, "Tab backward wraps");
  deepEqual(NS.focusMove(["a"], "a", { key: "Escape" }), { action: "close" }, "Escape closes");
  deepEqual(NS.focusMove([], null, { key: "Tab" }), { action: "none" }, "empty focus set");
});

test("RUNTIME UI: dialog shell exposes role, label, status region and focus trap", function () {
  var hCase = loadModel();
  var NS = hCase.NS;
  var shell = NS.dialog.shell();
  var dialog = shell.root;
  equal(dialog.getAttribute("role"), "dialog", "dialog role");
  equal(dialog.getAttribute("aria-modal"), "true", "modal flag");
  equal(dialog.getAttribute("aria-labelledby"), dialog.querySelector("h3").id, "label linkage");
  var status = dialog.querySelector(".gb-model-status");
  equal(status.getAttribute("role"), "status", "status role");
  equal(status.getAttribute("aria-live"), "polite", "polite live region");
  equal(hCase.doc.activeElement, dialog.querySelectorAll("button")[0], "first focusable focused");
  var before = hCase.doc.body.children.length;
  hCase.doc.dispatch("keydown", event({ key: "Escape" }));
  equal(hCase.doc.body.children.length, before - 1, "Escape removes backdrop");
});

test("RUNTIME UI: inspector renders real history and boot dispatches open", function () {
  var hCase = loadModel();
  var NS = hCase.NS;
  put(NS, "insert", { afterId: null, block: NS.createBlock("paragraph", "x", "b-ui") }, "Alpha");
  var host = hCase.dom.makeElement("div");
  NS.renderInspector(host);
  assert(host.querySelector(".gb-model-op").textContent.indexOf("insert") >= 0, "op log row rendered from history");
  equal(host.querySelectorAll(".gb-model-pre").length, 4, "JSON/HTML/Markdown/paste previews");

  var opened = 0;
  NS.dialog.open = function () { opened += 1; return function () {}; };
  var target = hCase.dom.makeElement("div");
  target.setAttribute("data-gb-model-open", "");
  hCase.doc.dispatch("click", event({ target: target }));
  equal(opened, 1, "delegated click opens inspector");
  var key = event({ key: "m", ctrlKey: true, altKey: true });
  hCase.doc.dispatch("keydown", key);
  equal(opened, 2, "Ctrl+Alt+M opens inspector");
  assert(key.prevented, "shortcut default prevented");
});
