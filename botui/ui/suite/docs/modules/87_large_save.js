"use strict";
/* Docs large .docx load path (#2533), incremental save queue (simulation).
 *
 * Changed-block ops are queued (last write wins per block) and sent instead of
 * the full body. The full body is the first-save fallback. Stale ETag/If-Match
 * and the 2 MB ceiling produce clear, stable statuses; a failed save NEVER
 * clears the queue (no silent data loss). All numbers are byte estimates from
 * the local fixture, not measured network traffic. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;
  var S = (NS.Save = NS.Save || {});
  S.OP_OVERHEAD = 48;

  function opBytes(textLen) { return S.OP_OVERHEAD + Math.max(0, Math.floor(Number(textLen) || 0)); }
  S.opBytes = opBytes;

  function enqueue(state, blockId, textLen) {
    var op = { blockId: String(blockId || ""), bytes: opBytes(textLen) };
    var queue = Array.isArray(state.queue) ? state.queue.slice() : [];
    var found = -1;
    for (var i = 0; i < queue.length; i += 1) {
      if (queue[i].blockId === op.blockId) { found = i; break; }
    }
    if (found >= 0) queue[found] = op;
    else queue.push(op);
    state.queue = queue;
    return op;
  }
  S.enqueue = enqueue;

  function incrementalBytes(state) {
    return (state.queue || []).reduce(function (sum, op) { return sum + (Number(op.bytes) || 0); }, 0);
  }
  S.incrementalBytes = incrementalBytes;

  function setMode(state, mode) { state.mode = mode === "manual" ? "manual" : "automatic"; return state.mode; }
  S.setMode = setMode;
  function toggleMode(state) { return setMode(state, state.mode === "manual" ? "automatic" : "manual"); }
  S.toggleMode = toggleMode;

  /* Build the request plan without mutating state. */
  function plan(state) {
    var queued = Array.isArray(state.queue) ? state.queue.length : 0;
    var kind = queued > 0 ? "incremental" : "full";
    var bytes = kind === "incremental" ? incrementalBytes(state) : Math.max(0, Number(state.fullBytes) || 0);
    var stale = String(state.etagSent) !== String(state.etag);
    var overCeiling = bytes > NS.CEILING_BYTES;
    var result = {
      kind: kind,
      bytes: bytes,
      queued: queued,
      stale: stale,
      overCeiling: overCeiling,
      ok: true,
      status: 200,
      messageKey: "docs-large-save-ok",
      fallback: "Saved { $n } bytes"
    };
    if (stale) {
      result.ok = false;
      result.status = 409;
      result.messageKey = "docs-large-save-stale";
      result.fallback = "Stale save rejected — ETag mismatch, no data lost";
    } else if (overCeiling && kind === "full") {
      result.ok = false;
      result.status = 413;
      result.messageKey = "docs-large-save-413";
      result.fallback = "Document too large for full save — use incremental / split";
    } else if (overCeiling && kind === "incremental") {
      result.ok = false;
      result.status = 413;
      result.messageKey = "docs-large-save-inc-413";
      result.fallback = "Change batch too large — split it; nothing lost";
    }
    return result;
  }
  S.plan = plan;

  /* Commit only a successful plan. Failures leave the queue intact. */
  function commit(state, result) {
    if (!result || !result.ok) {
      return { ok: false, lostData: false, queueDepth: (state.queue || []).length, etag: state.etag };
    }
    if (result.kind === "incremental") state.queue = [];
    state.etag = NS.bumpEtag(state.etag);
    state.etagSent = state.etag;
    return { ok: true, lostData: false, queueDepth: (state.queue || []).length, etag: state.etag };
  }
  S.commit = commit;

  S.message = function (result) { return NS.tr(result.messageKey, result.fallback, { n: result.bytes }); };
})(typeof window !== "undefined" ? window : globalThis);
