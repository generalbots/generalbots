"use strict";
/* Docs enterprise review workflow (#2425), pure review model.
 *
 * All logic here is DOM-free and operates on DocsReview.state so it can be
 * proven offline. Accepting/rejecting one change never touches a neighbour:
 * each action records a single-item (or explicitly keyed bulk) undo snapshot
 * and only rewrites the addressed ids. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;

  function state() { return NS.state; }
  function persist() { return NS.persist(); }

  /* ---- undo snapshots --------------------------------------------------- */
  function snapshot(entry) {
    var stack = state().undoStack;
    stack.push(entry);
    if (stack.length > 20) stack.shift();
    persist();
  }

  function undo() {
    var stack = state().undoStack;
    if (!stack.length) return false;
    var entry = stack.pop();
    if (entry.kind === "item") {
      var list = entry.collection === "redlines" ? state().redlines : state().changes;
      var item = list.filter(function (x) { return x.id === entry.id; })[0];
      if (item) item.status = entry.prevStatus;
    } else if (entry.kind === "thread") {
      var thread = state().threads.filter(function (x) { return x.id === entry.id; })[0];
      if (thread) thread.status = entry.prevStatus;
    } else if (entry.kind === "bulk") {
      var target = entry.collection === "redlines" ? state().redlines : state().changes;
      target.forEach(function (x) {
        if (Object.prototype.hasOwnProperty.call(entry.prevStatuses, x.id)) {
          x.status = entry.prevStatuses[x.id];
        }
      });
    } else if (entry.kind === "signoff") {
      state().signOff = entry.prevStatus;
    }
    persist();
    return true;
  }
  NS.undo = undo;

  /* ---- in-document changes --------------------------------------------- */
  function addChange(fields) {
    var input = fields || {};
    var change = {
      id: input.id || NS.uid("chg"),
      author: input.author || NS.me(),
      time: input.time || new Date().toISOString(),
      type: ["insert", "delete", "format"].indexOf(input.type) >= 0 ? input.type : "insert",
      before: input.before == null ? "" : String(input.before),
      after: input.after == null ? "" : String(input.after),
      anchorId: input.anchorId || "",
      offset: typeof input.offset === "number" ? input.offset : 0,
      status: "open",
      suggestOnly: input.suggestOnly === true
    };
    state().changes.push(change);
    persist();
    return change;
  }
  NS.addChange = addChange;

  function getChange(id) {
    return state().changes.filter(function (c) { return c.id === id; })[0] || null;
  }
  NS.getChange = getChange;

  function decideChange(id, decision) {
    var change = getChange(id);
    if (!change || change.status !== "open") return false;
    snapshot({ kind: "item", collection: "changes", id: id, prevStatus: change.status });
    change.status = decision === "reject" ? "rejected" : "accepted";
    persist();
    return true;
  }
  NS.acceptChange = function (id) { return decideChange(id, "accept"); };
  NS.rejectChange = function (id) { return decideChange(id, "reject"); };

  function decideByAuthor(author, decision) {
    var prev = {};
    var count = 0;
    state().changes.forEach(function (change) {
      if (change.status === "open" && change.author === author) {
        prev[change.id] = change.status;
        change.status = decision === "reject" ? "rejected" : "accepted";
        count += 1;
      }
    });
    if (!count) return 0;
    snapshot({ kind: "bulk", collection: "changes", prevStatuses: prev });
    persist();
    return count;
  }
  NS.acceptByAuthor = function (author) { return decideByAuthor(author, "accept"); };
  NS.rejectByAuthor = function (author) { return decideByAuthor(author, "reject"); };

  function authors() {
    var seen = {};
    state().changes.forEach(function (c) { seen[c.author] = true; });
    return Object.keys(seen).sort();
  }
  NS.changeAuthors = authors;

  /* ---- stable anchors --------------------------------------------------- */
  /* A comment stores a stable anchorId (a DOM node id) plus an offset inside
   * that node. Edits applied to a DIFFERENT anchor cannot detach it; edits in
   * the same anchor before the offset shift the offset by their delta. */
  function adjustAnchor(anchor, edits) {
    var result = { anchorId: anchor.anchorId, offset: anchor.offset };
    (edits || []).forEach(function (edit) {
      if (edit.anchorId !== anchor.anchorId) return;
      if (typeof edit.at === "number" && edit.at <= result.offset) {
        result.offset = Math.max(0, result.offset + (edit.delta || 0));
      }
    });
    return result;
  }
  NS.adjustAnchor = adjustAnchor;

  /* ---- threaded comments ------------------------------------------------ */
  function mentions(text) {
    var found = String(text == null ? "" : text).match(/@[\w.-]+/g) || [];
    var seen = {};
    return found.map(function (token) { return token.slice(1); }).filter(function (name) {
      if (seen[name]) return false;
      seen[name] = true;
      return true;
    });
  }
  NS.mentions = mentions;

  function addThread(fields) {
    var input = fields || {};
    var text = input.text || "";
    var thread = {
      id: input.id || NS.uid("thr"),
      anchorId: input.anchorId || "",
      offset: typeof input.offset === "number" ? input.offset : 0,
      quote: input.quote || "",
      firstText: input.firstText == null ? text : String(input.firstText),
      author: input.author || NS.me(),
      time: input.time || new Date().toISOString(),
      status: "open",
      mentions: mentions(text),
      replies: []
    };
    state().threads.push(thread);
    persist();
    return thread;
  }
  NS.addThread = addThread;

  function getThread(id) {
    return state().threads.filter(function (t) { return t.id === id; })[0] || null;
  }
  NS.getThread = getThread;

  function reply(threadId, fields) {
    var thread = getThread(threadId);
    if (!thread) return null;
    var input = fields || {};
    var text = input.text || "";
    var entry = {
      id: NS.uid("rep"),
      author: input.author || NS.me(),
      time: input.time || new Date().toISOString(),
      text: text,
      mentions: mentions(text)
    };
    thread.replies.push(entry);
    persist();
    return entry;
  }
  NS.reply = reply;

  function decideThread(id, status) {
    var thread = getThread(id);
    if (!thread || thread.status === status) return false;
    snapshot({ kind: "thread", id: id, prevStatus: thread.status });
    thread.status = status;
    persist();
    return true;
  }
  NS.resolveThread = function (id) { return decideThread(id, "resolved"); };
  NS.reopenThread = function (id) { return decideThread(id, "open"); };

  function filterThreads(filter) {
    var me = NS.me();
    return state().threads.filter(function (t) {
      if (filter === "mine") {
        return t.author === me || t.replies.some(function (r) { return r.author === me; });
      }
      if (filter === "mentions") {
        return t.mentions.indexOf(me) >= 0 ||
          t.replies.some(function (r) { return r.mentions.indexOf(me) >= 0; });
      }
      return t.status === "open";
    });
  }
  NS.filterThreads = filterThreads;

  /* ---- compare / redline ------------------------------------------------ */
  function toLines(text) {
    return String(text == null ? "" : text).split(/\r?\n/);
  }

  /* LCS diff. Bounded: very large inputs collapse to one delete + one insert
   * hunk so a pasted book cannot freeze the tab. */
  function computeRedline(left, right, options) {
    var a = toLines(left);
    var b = toLines(right);
    var ops = [];
    var opts = options || {};
    var author = opts.author || NS.me();
    if (a.length * b.length > 160000) {
      ops.push({ id: NS.uid("hunk"), type: "delete", text: a.join("\n"), status: "open", author: author });
      ops.push({ id: NS.uid("hunk"), type: "insert", text: b.join("\n"), status: "open", author: author });
      state().redlines = ops;
      persist();
      return ops;
    }
    var table = [];
    var i;
    var j;
    for (i = 0; i <= a.length; i += 1) {
      table[i] = new Array(b.length + 1).fill(0);
    }
    for (i = a.length - 1; i >= 0; i -= 1) {
      for (j = b.length - 1; j >= 0; j -= 1) {
        table[i][j] = a[i] === b[j]
          ? table[i + 1][j + 1] + 1
          : Math.max(table[i + 1][j], table[i][j + 1]);
      }
    }
    i = 0; j = 0;
    function push(type, text) {
      var last = ops[ops.length - 1];
      if (last && last.type === type && !last.id) { last.text += "\n" + text; return; }
      ops.push({ type: type, text: text });
    }
    while (i < a.length && j < b.length) {
      if (a[i] === b[j]) { push("equal", a[i]); i += 1; j += 1; }
      else if (table[i + 1][j] >= table[i][j + 1]) { push("delete", a[i]); i += 1; }
      else { push("insert", b[j]); j += 1; }
    }
    while (i < a.length) { push("delete", a[i]); i += 1; }
    while (j < b.length) { push("insert", b[j]); j += 1; }
    ops.forEach(function (op) {
      if (op.type !== "equal") {
        op.id = NS.uid("hunk");
        op.status = "open";
        op.author = author;
      }
    });
    state().redlines = ops;
    persist();
    return ops;
  }
  NS.computeRedline = computeRedline;

  function hunks() {
    return state().redlines.filter(function (op) { return !!op.id; });
  }
  NS.hunks = hunks;

  function decideHunk(id, decision) {
    var hunk = hunks().filter(function (h) { return h.id === id; })[0];
    if (!hunk || hunk.status !== "open") return false;
    snapshot({ kind: "item", collection: "redlines", id: id, prevStatus: hunk.status });
    hunk.status = decision === "reject" ? "rejected" : "accepted";
    persist();
    return true;
  }
  NS.acceptHunk = function (id) { return decideHunk(id, "accept"); };
  NS.rejectHunk = function (id) { return decideHunk(id, "reject"); };

  /* Merge preview: equal text is kept; a delete is dropped only when accepted;
   * an insert is added only when accepted. Rejected hunks keep the original. */
  function mergeResult() {
    var out = [];
    state().redlines.forEach(function (op) {
      if (op.type === "equal") { out.push(op.text); return; }
      if (op.type === "delete") {
        if (op.status !== "accepted") out.push(op.text);
        return;
      }
      if (op.status === "accepted") out.push(op.text);
    });
    return out.join("\n");
  }
  NS.mergeResult = mergeResult;

  /* ---- dashboard + sign-off -------------------------------------------- */
  function computeDashboard() {
    var byAuthor = {};
    var totalOpen = 0;
    state().changes.forEach(function (c) {
      var row = byAuthor[c.author] || (byAuthor[c.author] = {
        open: 0, accepted: 0, rejected: 0, total: 0
      });
      row.total += 1;
      if (c.status === "open") { row.open += 1; totalOpen += 1; }
      else if (c.status === "accepted") row.accepted += 1;
      else row.rejected += 1;
    });
    var unresolved = state().threads.filter(function (t) { return t.status === "open"; }).length;
    var minutes = Math.floor((Date.now() - state().reviewStartedAt) / 60000);
    return {
      byAuthor: byAuthor,
      totalOpen: totalOpen,
      unresolvedThreads: unresolved,
      timeInReviewMinutes: Math.max(0, minutes),
      signOff: state().signOff
    };
  }
  NS.computeDashboard = computeDashboard;

  function setSignOff(status) {
    if (["pending", "approved", "changes"].indexOf(status) < 0) return false;
    snapshot({ kind: "signoff", prevStatus: state().signOff });
    state().signOff = status;
    persist();
    return true;
  }
  NS.setSignOff = setSignOff;
})(window);
