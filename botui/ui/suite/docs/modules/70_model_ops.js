"use strict";
/* Docs structured document model (#2423), operation log + inverse engine.
 *
 * dispatch() is the ONLY write path: it captures an inverse from the live
 * document, applies the op through the single reducer apply(), appends an
 * audited entry (author/timestamp/seq/inverse) and records it on the author's
 * undo stack. Undo/redo apply inverses through the SAME reducer, never by
 * restoring a document snapshot, so one author's undo cannot revert a peer's
 * interleaved operations. */
(function (window) {
  var NS = window.DocsModel;
  if (!NS) return;

  function doc() { return NS.ensureDoc(); }
  function nextSeq() {
    var ops = NS.state.ops;
    return ops.length ? ops[ops.length - 1].seq + 1 : 1;
  }
  function eachSpan(root, cb) {
    (root.blocks || []).forEach(function (block) {
      (block.spans || []).forEach(function (span) { cb(block, span); });
    });
  }
  function reanchor(root, list) {
    (list || []).forEach(function (rule) {
      (root.comments || []).forEach(function (comment) {
        if (comment.id === rule.commentId) comment.blockId = rule.to;
      });
    });
  }
  function anchorsFor(root, from, to) {
    return (root.comments || []).filter(function (comment) { return comment.blockId === from; })
      .map(function (comment) { return { commentId: comment.id, from: from, to: to }; });
  }
  function flip(list) {
    return (list || []).map(function (rule) {
      return { commentId: rule.commentId, from: rule.to, to: rule.from };
    });
  }
  function clone(value) { return NS.clone(value); }

  /* ---- the single reducer ------------------------------------------------ */
  function apply(op) {
    var root = doc();
    var p = op.payload || {};
    var index, block;
    switch (op.type) {
      case "insert":
        index = p.afterId == null ? 0 : NS.findBlockIndex(root, p.afterId) + 1;
        root.blocks.splice(index, 0, clone(p.block));
        break;
      case "paste":
        index = p.afterId == null ? 0 : NS.findBlockIndex(root, p.afterId) + 1;
        (p.blocks || []).forEach(function (item, i) { root.blocks.splice(index + i, 0, clone(item)); });
        break;
      case "delete": {
        var ids = p.ids || [];
        root.blocks = root.blocks.filter(function (item) { return ids.indexOf(item.id) < 0; });
        break;
      }
      case "restore-many":
        index = p.afterId == null ? 0 : NS.findBlockIndex(root, p.afterId) + 1;
        (p.blocks || []).forEach(function (item, i) { root.blocks.splice(index + i, 0, clone(item)); });
        reanchor(root, p.reanchors);
        break;
      case "replace":
        index = NS.findBlockIndex(root, p.id);
        if (index >= 0) root.blocks.splice(index, 1, clone(p.block));
        reanchor(root, p.reanchors);
        break;
      case "format":
      case "unformat":
        eachSpan(root, function (owner, span) {
          if ((p.spanIds || []).indexOf(span.id) < 0) return;
          var exists = span.marks.some(function (mark) {
            return mark.type === p.mark.type && mark.value === p.mark.value;
          });
          if (op.type === "format" && !exists) span.marks.push(clone(p.mark));
          if (op.type === "unformat" && exists) {
            span.marks = span.marks.filter(function (mark) {
              return !(mark.type === p.mark.type && mark.value === p.mark.value);
            });
          }
          void owner;
        });
        break;
      case "move":
        index = NS.findBlockIndex(root, p.id);
        if (index < 0) break;
        block = root.blocks.splice(index, 1)[0];
        var target = p.beforeId == null ? root.blocks.length : NS.findBlockIndex(root, p.beforeId);
        if (target < 0) target = root.blocks.length;
        root.blocks.splice(target, 0, block);
        break;
      case "table":
        block = NS.getBlock(root, p.blockId);
        if (block && block.type === "table") {
          if (!block.attrs.cells) block.attrs.cells = {};
          block.attrs.cells[p.row + ":" + p.col] = { text: String(p.text == null ? "" : p.text) };
        }
        break;
      case "comment":
        if (!root.comments) root.comments = [];
        root.comments.push(clone(p.comment));
        break;
      case "uncomment":
        root.comments = (root.comments || []).filter(function (item) { return item.id !== p.id; });
        break;
      case "split":
        applySplit(root, p);
        break;
      case "merge":
        applyMerge(root, p);
        break;
      default:
        break;
    }
    return root;
  }
  NS.applyOpInternal = apply;

  function applySplit(root, p) {
    var index = NS.findBlockIndex(root, p.id);
    if (index < 0) return;
    var block = root.blocks[index];
    var remaining = Math.max(0, Math.min(Number(p.offset) || 0, NS.plainText(block).length));
    var head = [], tail = [];
    block.spans.forEach(function (span) {
      if (remaining >= span.text.length) { head.push(span); remaining -= span.text.length; return; }
      if (remaining > 0) {
        head.push({ id: span.id, text: span.text.slice(0, remaining), marks: clone(span.marks) });
        tail.push({ id: p.tailSpanId || NS.uid("s"), text: span.text.slice(remaining), marks: clone(span.marks) });
        remaining = 0;
        return;
      }
      tail.push(span);
    });
    var right = { id: p.rightId || NS.uid("b"), type: block.type, spans: tail.length ? tail : [NS.createSpan("")], attrs: clone(block.attrs) };
    block.spans = head.length ? head : [NS.createSpan("")];
    root.blocks.splice(index + 1, 0, right);
  }

  function applyMerge(root, p) {
    var indexA = NS.findBlockIndex(root, p.id);
    var indexB = NS.findBlockIndex(root, p.withId);
    if (indexA < 0 || indexB < 0 || indexB <= indexA) return;
    var left = root.blocks[indexA], right = root.blocks[indexB];
    left.spans = left.spans.concat(clone(right.spans));
    root.blocks.splice(indexB, 1);
    reanchor(root, anchorsFor(root, p.withId, p.id));
  }

  /* ---- inverse capture (before the forward mutation) -------------------- */
  function previewInverse(root, descriptor) {
    var d = descriptor;
    var block, comment, moves;
    switch (d.type) {
      case "insert":
      case "paste": {
        var ids = d.type === "insert" ? [d.block.id] : (d.blocks || []).map(function (b) { return b.id; });
        return { type: "delete", payload: { ids: ids } };
      }
      case "delete": {
        var ids = d.ids || [];
        var first = NS.findBlockIndex(root, ids[0]);
        var before = first > 0 ? root.blocks[first - 1].id : null;
        var removed = root.blocks.filter(function (b) { return ids.indexOf(b.id) >= 0; });
        return { type: "restore-many", payload: { afterId: before, blocks: clone(removed) } };
      }
      case "replace":
        moves = anchorsFor(root, d.id, d.block.id);
        return { type: "replace", payload: { id: d.block.id, block: clone(NS.getBlock(root, d.id)), reanchors: flip(moves) } };
      case "format":
        return { type: "unformat", payload: { spanIds: d.spanIds, mark: clone(d.mark) } };
      case "unformat":
        return { type: "format", payload: { spanIds: d.spanIds, mark: clone(d.mark) } };
      case "move": {
        var at = NS.findBlockIndex(root, d.id);
        var next = at >= 0 && at + 1 < root.blocks.length ? root.blocks[at + 1].id : null;
        return { type: "move", payload: { id: d.id, beforeId: next } };
      }
      case "table":
        block = NS.getBlock(root, d.blockId);
        var cell = block && block.attrs.cells ? block.attrs.cells[d.row + ":" + d.col] : null;
        return { type: "table", payload: { blockId: d.blockId, row: d.row, col: d.col, text: cell ? cell.text : "" } };
      case "comment":
        return { type: "uncomment", payload: { id: d.comment.id } };
      case "uncomment":
        comment = (root.comments || []).filter(function (c) { return c.id === d.id; })[0];
        return { type: "comment", payload: { comment: clone(comment) } };
      case "split":
        return { type: "replace", payload: { id: d.rightId, block: clone(NS.getBlock(root, d.id)), reanchors: [] } };
      case "merge":
        block = clone(NS.getBlock(root, d.withId));
        moves = anchorsFor(root, d.withId, d.id);
        return { type: "restore-many", payload: { afterId: d.id, blocks: [block], reanchors: flip(moves) } };
      default:
        return { type: "replace", payload: { id: d.id, block: clone(NS.getBlock(root, d.id)), reanchors: [] } };
    }
  }
  NS.previewInverse = previewInverse;

  /* ---- public write path ------------------------------------------------ */
  function dispatch(descriptor) {
    var root = doc();
    var type = descriptor.type;
    var payload = clone(descriptor);
    delete payload.type;
    /* Replacements and merges move comment anchors on the forward path too;
     * capture them here so the reducer is the only place that rewrites them. */
    if (type === "replace") payload.reanchors = anchorsFor(root, descriptor.id, descriptor.block.id);
    if (type === "merge") payload.reanchors = anchorsFor(root, descriptor.withId, descriptor.id);
    var inverse = previewInverse(root, descriptor);
    var author = NS.rememberAuthor(descriptor.author || NS.me());
    var op = { seq: nextSeq(), author: author, timestamp: Date.now(), type: type, payload: payload, inverse: inverse };
    NS.applyOpInternal(op);
    NS.state.ops.push(op);
    if (!NS.state.undo[author]) NS.state.undo[author] = [];
    if (!NS.state.redo[author]) NS.state.redo[author] = [];
    NS.state.undo[author].push(op.seq);
    NS.state.redo[author] = [];
    NS.persist();
    return op;
  }
  NS.dispatch = dispatch;

  function findOp(seq) {
    return (NS.state.ops || []).filter(function (op) { return op.seq === seq; })[0] || null;
  }

  function undo(author) {
    author = NS.rememberAuthor(author || NS.me());
    var stack = NS.state.undo[author] || [];
    if (!stack.length) return null;
    var seq = stack.pop();
    var op = findOp(seq);
    if (!op) return null;
    NS.applyOpInternal(op.inverse);
    if (!NS.state.redo[author]) NS.state.redo[author] = [];
    NS.state.redo[author].push(seq);
    NS.persist();
    return op;
  }
  NS.undo = undo;

  function redo(author) {
    author = NS.rememberAuthor(author || NS.me());
    var stack = NS.state.redo[author] || [];
    if (!stack.length) return null;
    var seq = stack.pop();
    var op = findOp(seq);
    if (!op) return null;
    NS.applyOpInternal({ type: op.type, payload: op.payload });
    if (!NS.state.undo[author]) NS.state.undo[author] = [];
    NS.state.undo[author].push(seq);
    NS.persist();
    return op;
  }
  NS.redo = redo;

  NS.history = function () { return clone(NS.state.ops || []); };
  NS.undoDepth = function (author) { return (NS.state.undo[author] || []).length; };
  NS.redoDepth = function (author) { return (NS.state.redo[author] || []).length; };
})(typeof window !== "undefined" ? window : globalThis);
