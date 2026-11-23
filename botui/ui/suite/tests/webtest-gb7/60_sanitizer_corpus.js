/**
 * #2743 — mXSS corpus and sanitizer provenance checks.
 *
 * The payload corpus lives in ../sanitizer-mxss-corpus.json and is executed
 * against a real DOM by agent1 during browser UAT. This offline module proves
 * the corpus is shipped, the sanitizer is versioned with a matching SHA-256,
 * and the hardenings the corpus depends on are present in the source.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const crypto = require("crypto");
const { test, assert, readFile, SUITE_DIR } = require("./00_harness");

const SANITIZER = "chat/chat-sanitize.js";
const MANIFEST = "chat/chat-sanitize.manifest.json";
const CORPUS = "tests/sanitizer-mxss-corpus.json";

test("ISSUE #2743: sanitizer is versioned and its checksum is recorded", function () {
  const source = readFile(SANITIZER);
  const manifest = JSON.parse(readFile(MANIFEST));
  assert(manifest.version && manifest.sha256, "manifest missing version/checksum");
  assert(source.indexOf("GB_SANITIZE_VERSION") !== -1, "sanitizer has no version constant");
  const actual = crypto.createHash("sha256").update(source).digest("hex");
  assert(actual === manifest.sha256, "sanitizer SHA-256 drifted from manifest");
  assert(source.indexOf("CDN") === -1, "sanitizer must not reference a CDN");
});

test("ISSUE #2743: sanitizer scrubs the final serialization (mXSS)", function () {
  const source = readFile(SANITIZER);
  ["foreignObject", "svg", "math", "annotation-xml", "template", "noscript"].forEach(function (tag) {
    assert(source.indexOf('"' + tag + '"') !== -1, tag + " not in DROP list");
  });
  assert(source.indexOf("parseAndScrub(once)") !== -1, "final serialization not re-scrubbed");
  assert(source.indexOf('name.indexOf("xmlns")') !== -1, "namespace attributes not stripped");
  assert(source.indexOf("@import") !== -1, "CSS url()/@import not stripped");
});

test("ISSUE #2743: mXSS bypass corpus is shipped and complete", function () {
  const full = path.join(SUITE_DIR, CORPUS);
  assert(fs.existsSync(full), "corpus file missing: " + CORPUS);
  const corpus = JSON.parse(fs.readFileSync(full, "utf8"));
  assert(corpus.cases.length >= 10, "corpus too small");
  corpus.cases.forEach(function (item) {
    assert(item.id && item.payload && Array.isArray(item.expect_absent),
      "incomplete corpus case: " + JSON.stringify(item));
    assert(item.expect_absent.length > 0, "no inertness assertion: " + item.id);
  });
  const ids = corpus.cases.map(function (item) { return item.id; });
  assert(new Set(ids).size === ids.length, "duplicate corpus IDs");
});

test("ISSUE #2707: multichat has a fail-closed textContent path", function () {
  const source = readFile("chat/modules/multichat/02_dom.js");
  assert(source.indexOf("inner.textContent = displayContent") !== -1,
    "multichat can still fall back to raw innerHTML");
  assert(source.indexOf("typeof window.GBSanitize.html !== \"function\"") !== -1,
    "multichat does not check sanitizer availability");
});
