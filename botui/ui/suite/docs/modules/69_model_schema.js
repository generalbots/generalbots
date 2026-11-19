"use strict";
/* Docs structured document model (#2423), versioned schema + validator.
 *
 * The document is a JSON projection: blocks carry stable ids, inline spans
 * carry stable ids and typed marks, and comments anchor to a block id (never to
 * an HTML string). Every constructor here is pure so the validator and the
 * stable-id contract can be proven in an offline DOM-stub harness. */
(function (window) {
  var NS = window.DocsModel;
  if (!NS) return;

  var VERSION = NS.SCHEMA_VERSION;
  NS.BLOCK_TYPES = ["paragraph", "heading", "list", "table", "image", "footnote-reference", "page-break", "section-break"];
  NS.MARK_TYPES = ["bold", "italic", "underline", "strike", "code", "link", "field"];
  NS.OP_TYPES = ["insert", "delete", "replace", "format", "unformat", "move", "table", "comment", "uncomment", "split", "merge", "restore", "reanchor"];

  function clone(value) { return JSON.parse(JSON.stringify(value)); }
  NS.clone = clone;

  function createSpan(text, marks, id) {
    return { id: id || NS.uid("s"), text: String(text == null ? "" : text), marks: Array.isArray(marks) ? clone(marks) : [] };
  }
  NS.createSpan = createSpan;

  function createBlock(type, text, id) {
    var kind = NS.BLOCK_TYPES.indexOf(type) >= 0 ? type : "paragraph";
    var block = { id: id || NS.uid("b"), type: kind, spans: [createSpan(text || "")], attrs: {} };
    if (kind === "table") block.attrs = { rows: 1, cols: 1, cells: {} };
    return block;
  }
  NS.createBlock = createBlock;

  function createComment(blockId, text, author, id) {
    return { id: id || NS.uid("c"), blockId: blockId, text: String(text || ""), author: author || NS.ME, createdAt: Date.now() };
  }
  NS.createComment = createComment;

  function createDoc() {
    return { schema_version: VERSION, blocks: [createBlock("paragraph", "")], comments: [] };
  }
  NS.createDoc = createDoc;

  NS.ensureDoc = function () {
    if (!NS.state.doc || !NS.validate(NS.state.doc).ok) NS.state.doc = createDoc();
    return NS.state.doc;
  };

  /* ---- readers (never expose the live document for mutation) ------------- */
  NS.getDoc = function () { return clone(NS.ensureDoc()); };
  NS.findBlockIndex = function (doc, id) {
    var blocks = doc && doc.blocks ? doc.blocks : [];
    for (var i = 0; i < blocks.length; i += 1) if (blocks[i].id === id) return i;
    return -1;
  };
  NS.getBlock = function (doc, id) {
    var index = NS.findBlockIndex(doc, id);
    return index < 0 ? null : doc.blocks[index];
  };
  NS.plainText = function (block) {
    if (!block || !Array.isArray(block.spans)) return "";
    return block.spans.map(function (span) { return span.text; }).join("");
  };

  /* ---- selection model: block id + inline offset ------------------------- */
  NS.selectionAnchor = function (blockId, offset) {
    return { blockId: blockId || null, offset: Math.max(0, Number(offset) || 0) };
  };
  NS.setSelection = function (blockId, offset) {
    NS.state.selection = NS.selectionAnchor(blockId, offset);
    return NS.state.selection;
  };

  /* ---- schema validation ------------------------------------------------- */
  function validate(doc) {
    var errors = [];
    if (!doc || typeof doc !== "object" || Array.isArray(doc)) {
      return { ok: false, errors: ["schema:document-not-object"] };
    }
    if (doc.schema_version !== VERSION) errors.push("schema:version:" + doc.schema_version);
    if (!Array.isArray(doc.blocks)) return { ok: false, errors: errors.concat(["schema:blocks-not-array"]) };
    var blockIds = {};
    doc.blocks.forEach(function (block, index) {
      if (!block || typeof block !== "object") { errors.push("schema:block-object:" + index); return; }
      if (typeof block.id !== "string" || !block.id || blockIds[block.id]) errors.push("schema:block-id:" + index);
      else blockIds[block.id] = true;
      if (NS.BLOCK_TYPES.indexOf(block.type) < 0) errors.push("schema:block-type:" + index);
      if (!Array.isArray(block.spans)) { errors.push("schema:spans:" + index); return; }
      var spanIds = {};
      block.spans.forEach(function (span, sIndex) {
        if (!span || typeof span !== "object") { errors.push("schema:span-object:" + index + ":" + sIndex); return; }
        if (typeof span.id !== "string" || !span.id || spanIds[span.id]) errors.push("schema:span-id:" + index + ":" + sIndex);
        else spanIds[span.id] = true;
        if (typeof span.text !== "string") errors.push("schema:span-text:" + index + ":" + sIndex);
        if (!Array.isArray(span.marks)) { errors.push("schema:marks:" + index + ":" + sIndex); return; }
        span.marks.forEach(function (mark, mIndex) {
          if (!mark || NS.MARK_TYPES.indexOf(mark.type) < 0) errors.push("schema:mark-type:" + index + ":" + sIndex + ":" + mIndex);
        });
      });
    });
    if (!Array.isArray(doc.comments)) errors.push("schema:comments-not-array");
    else doc.comments.forEach(function (comment, index) {
      if (!comment || typeof comment.blockId !== "string" || !blockIds[comment.blockId]) {
        errors.push("schema:comment-anchor:" + index);
      }
    });
    return { ok: errors.length === 0, errors: errors };
  }
  NS.validate = validate;
})(typeof window !== "undefined" ? window : globalThis);
