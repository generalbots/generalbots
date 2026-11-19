"use strict";
/* Docs large .docx load path (#2533), slow-device degradation.
 *
 * Above a content-size threshold the local preview opens read-only with a
 * clear banner and an explicit "enable editing anyway" action; below it,
 * editing is enabled. This is local UI gating only — no real device profiling. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;
  var D = (NS.Degradation = NS.Degradation || {});

  function evaluate(sizeMb, thresholdMb) {
    var threshold = Number(thresholdMb) || NS.DEFAULT_THRESHOLD_MB;
    var size = Number(sizeMb) || 0;
    var readOnly = size > threshold;
    return {
      sizeMb: size,
      thresholdMb: threshold,
      readOnly: readOnly,
      bannerKey: readOnly ? "docs-large-degraded-banner" : "docs-large-edit-ready",
      bannerFallback: readOnly
        ? "Editing will be slow — opened in read-only preview"
        : "Editing enabled — document within threshold",
      actionKey: readOnly ? "docs-large-enable-anyway" : null,
      actionFallback: "Enable editing anyway"
    };
  }
  D.evaluate = evaluate;

  function start(sizeMb, thresholdMb, state) {
    var verdict = evaluate(sizeMb, thresholdMb);
    state.editingEnabled = !verdict.readOnly;
    return verdict;
  }
  D.start = start;

  function enableAnyway(state) { state.editingEnabled = true; return state.editingEnabled; }
  D.enableAnyway = enableAnyway;

  function editable(verdict, state) { return !verdict.readOnly || !!state.editingEnabled; }
  D.editable = editable;
})(typeof window !== "undefined" ? window : globalThis);
