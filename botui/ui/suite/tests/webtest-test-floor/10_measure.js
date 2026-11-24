/**
 * [BASELINE-OBSERVE] characterizations for #2544.
 *
 * These assert the numbers the shipped tree ACTUALLY has today so any change
 * to the test surface is surfaced for review. They are observations, not a
 * ratchet: no baseline file is read or written. When production legitimately
 * adds a test or a crate, these exact values must be updated in the same PR.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var all = h.all;

var ZERO = [
  "botchannels-core", "botcoresession", "bothandoff",
  "bothr", "botitsm", "botmultimodal", "botnvidia", "botpublic",
];
var UNEXEMPTED = [];

test("[BASELINE-OBSERVE] 119 crates scanned across botserver/crates/*, botserver and bottest", function () {
  var m = all();
  equal(m.crates.length, 119, "crate count");
  assert(m.crates.indexOf("botserver") >= 0 && m.crates.indexOf("bottest") >= 0,
    "root crates missing from the scan set");
  assert(m.crates.every(function (n) { return m.perCrate[n]; }), "a crate lacked a scan result");
});

test("[BASELINE-OBSERVE] 109 crates carry >=1 test attribute, 10 carry zero", function () {
  var m = all();
  equal(m.tested.length, 111, "crates with tests");
  equal(m.zero.length, 8, "zero-test crates");
  equal(m.tested.length + m.zero.length, m.crates.length, "partition is not exhaustive");
});

test("[BASELINE-OBSERVE] the zero-test set is exactly the 8 observed crates", function () {
  equal(all().zero.join(","), ZERO.join(","), "zero-test crate list");
});

test("[BASELINE-OBSERVE] botserver is binary-only: no [lib], no src/lib.rs, src/main.rs present", function () {
  var m = all();
  equal(m.libSection, false, "botserver [lib] section");
  equal(m.serverLibFile, false, "botserver src/lib.rs");
  equal(m.serverMainFile, true, "botserver src/main.rs");
});

test("[BASELINE-OBSERVE] test-exemptions.toml covers every zero-test crate", function () {
  equal(all().exemptions.join(","), ZERO.join(","), "exemption list");
  all().exemptions.forEach(function (name) {
    assert(all().zero.indexOf(name) >= 0, "exemption names a crate that is not zero-test: " + name);
  });
});

test("[BASELINE-OBSERVE] 0 zero-test crates have no exemption entry", function () {
  equal(all().unexempted.join(","), UNEXEMPTED.join(","), "unexempted zero-test list");
});

test("[BASELINE-OBSERVE] integration surface: 36 .rs files, 7 CRM-mention, 3 Campaign-mention", function () {
  var i = all().integration;
  equal(i.rsFiles, 36, "integration .rs files");
  equal(i.crmMention.length, 7, "CRM-mention files carrying a test attribute");
  equal(i.campaignMention.length, 3, "Campaign-mention files carrying a test attribute");
});

test("[BASELINE-OBSERVE] CRM/Campaign integration files are named and present", function () {
  var i = all().integration;
  equal(i.crmNamed.length, 3, "CRM-named integration files");
  equal(i.campaignNamed.length, 1, "Campaign-named integration files");
  assert(i.crmNamed.indexOf("bottest/tests/integration/crm_flow.rs") >= 0, "crm_flow.rs missing");
  assert(i.campaignNamed.indexOf("bottest/tests/integration/campaign_flow.rs") >= 0, "campaign_flow.rs missing");
});

test("[BASELINE-OBSERVE] CI runs --bin, documents it, and ships coverage jobs + baseline", function () {
  var m = all();
  equal(m.ciBinCommand, true, "ci.yml cargo test --bin command");
  equal(m.agentsDocumentsBin, true, "AGENTS.md --bin documentation");
  equal(m.ciCoverage, true, "ci.yml coverage jobs");
  equal(m.coverageBaseline, true, "issues/coverage/baseline.json");
});

test("[BASELINE-OBSERVE] no #[ignore] attributes in the two flow tests", function () {
  [ "bottest/tests/integration/crm_flow.rs", "bottest/tests/integration/campaign_flow.rs" ].forEach(function (rel) {
    equal(h.countMatches(h.read(rel), /#\s*\[\s*ignore\b/g), 0, "ignore attributes in " + rel);
  });
});
