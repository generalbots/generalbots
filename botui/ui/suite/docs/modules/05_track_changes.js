"use strict";

/**
 * Module 05: Track Changes for Word Processor.
 * Provides: revision tracking, accept/reject, change history, version diff.
 */

function revisionId() {
  return "rev-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8);
}

function makeRevision(type, author, before, after, position) {
  return {
    id: revisionId(),
    type: type,
    author: author || "Unknown",
    timestamp: new Date().toISOString(),
    before: before != null ? String(before) : null,
    after: after != null ? String(after) : null,
    position: position || 0,
    accepted: null,
    rejected: null,
  };
}

function trackInsertion(state, position, text, author) {
  if (!state.revisions) state.revisions = [];
  if (!state.trackingEnabled) return null;
  const rev = makeRevision("insert", author, null, text, position);
  state.revisions.push(rev);
  return rev;
}

function trackDeletion(state, position, text, author) {
  if (!state.revisions) state.revisions = [];
  if (!state.trackingEnabled) return null;
  const rev = makeRevision("delete", author, text, null, position);
  state.revisions.push(rev);
  return rev;
}

function trackReplacement(state, position, before, after, author) {
  if (!state.revisions) state.revisions = [];
  if (!state.trackingEnabled) return null;
  const rev = makeRevision("replace", author, before, after, position);
  state.revisions.push(rev);
  return rev;
}

function trackFormatting(state, position, formatBefore, formatAfter, author) {
  if (!state.revisions) state.revisions = [];
  if (!state.trackingEnabled) return null;
  const rev = makeRevision("format", author, JSON.stringify(formatBefore), JSON.stringify(formatAfter), position);
  state.revisions.push(rev);
  return rev;
}

function acceptRevision(state, revisionId) {
  const rev = (state.revisions || []).find((r) => r.id === revisionId);
  if (!rev) return false;
  rev.accepted = new Date().toISOString();
  return true;
}

function rejectRevision(state, revisionId) {
  const rev = (state.revisions || []).find((r) => r.id === revisionId);
  if (!rev) return false;
  rev.rejected = new Date().toISOString();
  return true;
}

function acceptAll(state) {
  if (!state.revisions) return 0;
  let count = 0;
  for (const rev of state.revisions) {
    if (rev.accepted == null && rev.rejected == null) {
      rev.accepted = new Date().toISOString();
      count++;
    }
  }
  return count;
}

function rejectAll(state) {
  if (!state.revisions) return 0;
  let count = 0;
  for (const rev of state.revisions) {
    if (rev.accepted == null && rev.rejected == null) {
      rev.rejected = new Date().toISOString();
      count++;
    }
  }
  return count;
}

function pendingRevisions(state) {
  return (state.revisions || []).filter((r) => r.accepted == null && r.rejected == null);
}

function revisionsByAuthor(state) {
  const map = new Map();
  for (const rev of state.revisions || []) {
    if (!map.has(rev.author)) map.set(rev.author, []);
    map.get(rev.author).push(rev);
  }
  return map;
}

function revisionSummary(state) {
  const revs = state.revisions || [];
  return {
    total: revs.length,
    pending: revs.filter((r) => r.accepted == null && r.rejected == null).length,
    accepted: revs.filter((r) => r.accepted != null).length,
    rejected: revs.filter((r) => r.rejected != null).length,
    byType: revs.reduce((acc, r) => {
      acc[r.type] = (acc[r.type] || 0) + 1;
      return acc;
    }, {}),
    byAuthor: Array.from(revisionsByAuthor(state).entries()).map(([a, list]) => ({ author: a, count: list.length })),
  };
}

function diffText(before, after) {
  if (!before) return [{ type: "insert", value: after }];
  if (!after) return [{ type: "delete", value: before }];
  const a = String(before);
  const b = String(after);
  const al = a.split("");
  const bl = b.split("");
  const m = al.length;
  const n = bl.length;
  const lcs = [];
  for (let i = 0; i <= m; i++) {
    lcs.push(new Array(n + 1).fill(0));
  }
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (al[i - 1] === bl[j - 1]) {
        lcs[i][j] = lcs[i - 1][j - 1] + 1;
      } else {
        lcs[i][j] = Math.max(lcs[i - 1][j], lcs[i][j - 1]);
      }
    }
  }
  const ops = [];
  let i = m;
  let j = n;
  while (i > 0 && j > 0) {
    if (al[i - 1] === bl[j - 1]) {
      ops.push({ type: "equal", value: al[i - 1] });
      i--;
      j--;
    } else if (lcs[i - 1][j] >= lcs[i][j - 1]) {
      ops.push({ type: "delete", value: al[i - 1] });
      i--;
    } else {
      ops.push({ type: "insert", value: bl[j - 1] });
      j--;
    }
  }
  while (i > 0) {
    ops.push({ type: "delete", value: al[i - 1] });
    i--;
  }
  while (j > 0) {
    ops.push({ type: "insert", value: bl[j - 1] });
    j--;
  }
  return ops.reverse();
}

window.DocsTrackChanges = {
  trackInsertion,
  trackDeletion,
  trackReplacement,
  trackFormatting,
  acceptRevision,
  rejectRevision,
  acceptAll,
  rejectAll,
  pendingRevisions,
  revisionsByAuthor,
  revisionSummary,
  diffText,
};
