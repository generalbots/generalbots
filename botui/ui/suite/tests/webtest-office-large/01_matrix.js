/**
 * Data-driven matrix definition for the #2534 readiness audit.
 * Every (app x capability) pair must be registered by a probe module; the
 * report cross-checks that the matrix is complete.
 */
"use strict";

var CAPABILITIES = [
  { id: "windowed_reads", label: "Windowed / paged reads" },
  { id: "progress_cancel", label: "Progress + cancel" },
  { id: "virtualized_render", label: "Virtualized rendering" },
  { id: "memory_caps", label: "Memory caps / byte budgets" },
  { id: "documented_oom", label: "Documented OOM path" }
];

var APPS = [
  { id: "Docs", largeModules: "botui/ui/suite/docs/modules" },
  { id: "Sheets", largeModules: "botui/ui/suite/sheet" },
  { id: "Slides", largeModules: "botui/ui/suite/slides/modules" }
];

/** Absence patterns used by GAP probes; presence patterns by PASS probes. */
var PATTERNS = {
  windowedRead: "windowed|/load\\?|range=|start_row|blocksInRange|visibleRange",
  progressCancel: "AbortController|progress-?bar|aria-valuenow|cancelImport|abortImport|abort-controller",
  virtualization: "virtual-grid|visibleRowRange|renderWindow|mountPlan|OVERSCAN",
  serverWindow: "/load\\?range|loadRange|start_row=|end_row=|&range=|&offset=",
  memoryBudget: "peak[_-]?rss|admission|cgroup|byte[_-]?budget|MAX_REQUEST_BYTES|Insufficient Storage",
  oomPath: "\\bOOM\\b|out of memory|507|Insufficient Storage|MemoryError"
};

module.exports = {
  CAPABILITIES: CAPABILITIES,
  APPS: APPS,
  PATTERNS: PATTERNS,

  /** Scan every shipped JS file of an app for `pattern`; return match count. */
  scanAppJs: function (H, appId, pattern) {
    var root = null;
    APPS.some(function (app) {
      if (app.id === appId) { root = app.largeModules; return true; }
      return false;
    });
    if (!root) return 0;
    var count = 0;
    var files = H.fs.readdirSync(H.path.join(H.REPO_DIR, root), { withFileTypes: true });
    files.forEach(function (entry) {
      if (!entry.isFile() || entry.name.slice(-3) !== ".js") return;
      count += H.countLines(root + "/" + entry.name, pattern);
    });
    return count;
  }
};
