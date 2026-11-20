/**
 * Suite UI a11y static audit — scanner (#2331).
 * Run with: node botui/ui/suite/tests/webtest-ui-aria.js
 *
 * The patterns below are deliberately written from character classes and
 * token boundaries: this file is itself scanned by the i18n gate, so it never
 * embeds a literal attribute assignment with a key inside it.
 *
 * Every number is an OBSERVED static token count. A declaration may live in a
 * comment or a template string; a write may be announced by a helper in a
 * different file. These are proxies, not runtime/AX-tree proof.
 */
"use strict";

var h = require("./00_harness");

var RX = {
  liveToken: /\baria-live\b/g,
  liveDecl: /\baria-live\s*=\s*(["'])(polite|assertive|off)\1|["']aria-live["']\s*:\s*["'](polite|assertive|off)["']|setAttribute\(\s*["']aria-live["']\s*,\s*["'](polite|assertive|off)["']/gi,
  roleDecl: /\brole\s*=\s*(["'])(status|alert|log)\1|["']role["']\s*:\s*["'](status|alert|log)["']|setAttribute\(\s*["']role["']\s*,\s*["'](status|alert|log)["']/gi,
  i18nAttr: /\bdata-i18n(?:-[a-z-]+)?\s*=\s*(["'])[^"']+\1/gi,
  gbtCall: /\bgbT\s*\(\s*(["'])[a-z0-9][a-z0-9._-]*\1/g,
  write: /\.(?:textContent|innerText|innerHTML)\s*\+?=/g,
  writeBracket: /\[\s*["'](?:textContent|innerText|innerHTML)["']\s*\]\s*\+?=/g,
};

/*
 * Measurement semantics (what each OBSERVED number actually means):
 * - ariaLiveDeclarations counts literal polite/assertive/off values only;
 *   computed values (for example a ternary) are not resolvable statically.
 * - dataI18nAttrs counts quoted attribute values with a simple key token;
 *   concatenated keys are intentionally not inferred.
 * - gbTCalls counts calls whose first argument is a literal key token;
 *   variable or template keys are not inferred.
 * - writesOutsideRegionFiles is a file-level upper bound: a write can still be
 *   announced by a helper loaded in another file (GBToasts, collab-a11y).
 */

function count(text, rx) {
  rx.lastIndex = 0;
  var found = text.match(rx);
  return found ? found.length : 0;
}

function values(text, rx) {
  rx.lastIndex = 0;
  var out = {};
  var match;
  while ((match = rx.exec(text)) !== null) {
    var value = (match[2] || match[3] || match[4] || "other").toLowerCase();
    out[value] = (out[value] || 0) + 1;
    if (match[0].length === 0) rx.lastIndex++;
  }
  return out;
}

function scan() {
  var files = h.readTree();
  var m = {
    filesScanned: files.length,
    htmlFiles: 0,
    jsFiles: 0,
    vendorFiles: 0,
    topSegments: 0,
    ariaLiveMentions: 0,
    ariaLiveFiles: 0,
    ariaLiveDeclarations: 0,
    livePolite: 0,
    liveAssertive: 0,
    liveOff: 0,
    roleStatus: 0,
    roleAlert: 0,
    roleLog: 0,
    roleStatusAlertFiles: 0,
    dataI18nAttrs: 0,
    htmlWithI18n: 0,
    gbTCalls: 0,
    jsWithGbt: 0,
    dynamicWrites: 0,
    writesInRegionFiles: 0,
    writesOutsideRegionFiles: 0,
    filesWithWritesNoRegion: 0,
    bySegment: {},
  };
  var segments = {};

  files.forEach(function (file) {
    var text = file.text;
    var liveVals = values(text, RX.liveDecl);
    var roleVals = values(text, RX.roleDecl);
    var liveCount = (liveVals.polite || 0) + (liveVals.assertive || 0) +
      (liveVals.off || 0) + (liveVals.other || 0);
    var statusCount = roleVals.status || 0;
    var alertCount = roleVals.alert || 0;
    var hasRegion = liveCount > 0 || statusCount > 0 || alertCount > 0;
    var writes = count(text, RX.write) + count(text, RX.writeBracket);
    var i18n = count(text, RX.i18nAttr);
    var gbt = count(text, RX.gbtCall);
    var segment = file.rel.split("/")[0];

    segments[segment] = (segments[segment] || 0) + 1;
    m.bySegment[segment] = (m.bySegment[segment] || 0) + writes;
    if (file.ext === ".html") m.htmlFiles++;
    if (file.ext === ".js") m.jsFiles++;
    if (file.vendor) m.vendorFiles++;

    var mentions = count(text, RX.liveToken);
    m.ariaLiveMentions += mentions;
    if (mentions > 0) m.ariaLiveFiles++;
    m.ariaLiveDeclarations += liveCount;
    m.livePolite += liveVals.polite || 0;
    m.liveAssertive += liveVals.assertive || 0;
    m.liveOff += liveVals.off || 0;
    m.roleStatus += statusCount;
    m.roleAlert += alertCount;
    m.roleLog += roleVals.log || 0;
    if (statusCount + alertCount > 0) m.roleStatusAlertFiles++;
    m.dataI18nAttrs += i18n;
    if (file.ext === ".html" && i18n > 0) m.htmlWithI18n++;
    m.gbTCalls += gbt;
    if (file.ext === ".js" && gbt > 0) m.jsWithGbt++;
    m.dynamicWrites += writes;
    if (hasRegion) m.writesInRegionFiles += writes;
    else {
      m.writesOutsideRegionFiles += writes;
      if (writes > 0) m.filesWithWritesNoRegion++;
    }
  });

  m.topSegments = Object.keys(segments).length;
  h.setMetrics(m);
  return m;
}

var metrics = scan();

h.test("[OBSERVE] scanner found both HTML and JS sources", function () {
  h.assert(metrics.htmlFiles > 0 && metrics.jsFiles > 0, "empty or single-kind source tree");
});

h.test("[OBSERVE] file-kind totals reconcile", function () {
  h.assert(metrics.htmlFiles + metrics.jsFiles === metrics.filesScanned, "file count mismatch");
});

h.test("[OBSERVE] live-region value partition reconciles", function () {
  var part = metrics.livePolite + metrics.liveAssertive + metrics.liveOff;
  h.assert(part === metrics.ariaLiveDeclarations, "live value partition mismatch");
});

h.test("[OBSERVE] dynamic-write partition reconciles", function () {
  var part = metrics.writesInRegionFiles + metrics.writesOutsideRegionFiles;
  h.assert(part === metrics.dynamicWrites, "write partition mismatch");
});

h.test("[OBSERVE] at least one shipped source declares an announced region", function () {
  h.assert(metrics.ariaLiveDeclarations + metrics.roleStatus + metrics.roleAlert > 0,
    "no announced region found anywhere in the shipped suite");
});

module.exports = { metrics: metrics, RX: RX };
