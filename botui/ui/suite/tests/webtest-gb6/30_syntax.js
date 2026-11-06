/**
 * webtest_gb6 — syntax validation of shipped feature scripts
 * Split from webtest_gb6.js for #1731; keeps the original test order.
 */
"use strict";

const {
  test,
  readFile,
  allJsFiles,
} = require("./00_harness");

// ═══════════════════════════════════════════════════════════════════
// JS SYNTAX VALIDATION
// ═══════════════════════════════════════════════════════════════════

allJsFiles.forEach(function (f) {
  test("SYNTAX: " + f, function () {
    var src = readFile(f);
    try { new Function(src); } catch (e) {
      throw new Error("Syntax error: " + e.message);
    }
  });
});
