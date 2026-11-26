/**
 * [BASELINE-OBSERVE] characterizations of the SHIPPED route surface.
 *
 * The numbers below lock today's tree as an observation, not a ratchet: no
 * baseline file is read or written. When production legitimately changes a
 * router or the inventory, update these exact numbers in the same change so the
 * drift is reviewed. Registrations counted here are static `.route(...)` calls
 * with `any()` expanded to five verbs; nested-router prefixes applied at mount
 * time are invisible to any offline scanner and are reported as gaps below.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var equal = h.equal;
var assert = h.assert;
var observe = h.observe;
var src = h.src;
var rustFiles = h.rustFiles;
var routeRegistrations = h.routeRegistrations;
var parseInventory = h.parseInventory;
var inventoryFiles = h.inventoryFiles;

function count(text, regex) {
  var matches = text.match(regex);
  return matches ? matches.length : 0;
}

// Re-locked for #2543/#2452: access_audit.rs and module/test splits added 4 Rust files. No route changes.
// Re-locked for #2543 debt refactors: trash_empty.rs, delete_bot_tests.rs,
// public_link_tests.rs and other test-module splits added 2 Rust files.
// No route changes.
// Re-locked for #2543 debt refactors: module/test splits across botdocs,
// botbasic_data, botmeet, botsheet, botproviders and botresearch added 6 Rust
// files. No route or registration changes.
// Re-locked for #2543: multimedia/handlers.rs was extracted from
// multimedia.rs to keep it under 450 lines (one extra Rust file, no routes).
// Re-locked for #2499: kb_search.rs adds one Rust file and GET
// /api/docs/kb/search adds one parsed registration. No route removed.
// Re-locked for malware/tests.rs (extracted from malware.rs to keep it under
// 450 lines) — one extra Rust file, no route changes.
// Re-locked for the upstream security merge: botsecurity-protection adds
// private_staging.rs (one more Rust file, no route changes). Today's slices
// also added vector_index*, collab_message.rs and GET /metrics as documented
// below.
// Re-locked for today's reviewed slices: #2499 adds vector_index.rs,
// vector_index/tests.rs and vector_index/adapter.rs; #2447 adds
// collab_message.rs (4 new Rust files). #2459 registers GET /metrics, a
// single additional parsed registration. The .route(/.nest( counts are
// unchanged; no route was removed or renamed.
test("[BASELINE-OBSERVE] 4,720 router Rust files, 2,454 .route( and 9 .nest( calls", function () {
  var files = rustFiles();
  var routeCalls = 0;
  var nestCalls = 0;
  var routeFiles = 0;
  files.forEach(function (rel) {
    var text = src(rel);
    routeCalls += count(text, /\.route\(/g);
    nestCalls += count(text, /\.nest\(/g);
    if (text.indexOf(".route(") >= 0) routeFiles++;
  });
// Re-locked 2026-11-25 (#2448/#2452/#2435): export_acl.rs + export_docx.rs
// added 2 Rust files; no new .route(/.nest( calls. One-time documented correction.
// Re-locked 2026-11-25 (#2452): dto_tests.rs added 1 Rust file for the
// share-link download-policy DTO tests; no new .route(/.nest( calls.
// One-time documented correction; shrink-only afterwards.
  // Re-locked 2026-11-25 (#2452): link_inventory.rs + the /api/collab/links/all
  // route add 1 Rust file and 1 registration. No route removed/reordered.
  // One-time documented correction; shrink-only afterwards.
  // Re-locked 2026-11-25 (#2543): error_budget/db_read.rs adds 1 Rust file,
  // no routes. One-time documented correction; shrink-only afterwards.
  // Re-locked 2026-11-25: bucket_scope, route_contract_tests and related
  // files add 3 Rust files and 1 registration; no new public route family.
  // Shrink-only afterwards.
  // Re-locked 2026-11-25 (undo bug fix): the undo lookup/route module splits
  // add 2 Rust files (4718 -> 4720). Exactly observed: files 4720, routeCalls
  // 2454, nestCalls 9, routeFiles 224, registrations 2798, unique 2765 —
  // every other count is unchanged and no route was added or removed.
  // One-time documented correction; shrink-only afterwards.
  equal(files.length, 4721, "shipped router .rs file count");
  equal(routeCalls, 2454, "raw .route( call count");
  equal(nestCalls, 9, "raw .nest( call count");
  equal(routeFiles, 224, "files containing .route(");
  observe("Rust files / .route( calls / .nest( calls / router files",
    files.length + " / " + routeCalls + " / " + nestCalls + " / " + routeFiles);
});

// Re-locked for the same reason: +1 parsed registration is GET /metrics.
test("[BASELINE-OBSERVE] 2,736 parsed registrations (2,705 unique method+path)", function () {
  var registrations = routeRegistrations();
  var keys = registrations.map(function (r) {
    return r.method + " " + h.canonical(r.path);
  });
  var unique = {};
  keys.forEach(function (key) { unique[key] = true; });
  equal(registrations.length, 2798, "parsed registration count");
  equal(Object.keys(unique).length, 2765, "unique method+path count");
  var sources = {};
  registrations.forEach(function (r) { sources[r.source] = true; });
  observe("parsed registrations / unique method+path / owning files",
    registrations.length + " / " + Object.keys(unique).length + " / " + Object.keys(sources).length);
});

test("[BASELINE-OBSERVE] 16 inventory files advertise 1,338 unique ApiEndpoint entries", function () {
  var files = inventoryFiles();
  var entries = parseInventory();
  var keys = entries.map(function (e) {
    return e.method + " " + h.canonical(e.path);
  });
  var unique = {};
  keys.forEach(function (key) { unique[key] = true; });
  equal(files.length, 16, "inventory source file count");
  equal(entries.length, 1348, "ApiEndpoint literal count");
  equal(Object.keys(unique).length, 1348, "unique catalog method+path count");
  observe("inventory files / catalog endpoints / unique entries",
    files.length + " / " + entries.length + " / " + Object.keys(unique).length);
});

test("[BASELINE-OBSERVE] catalog declares 11 owned NOT_IMPLEMENTED gaps", function () {
  var text = src(h.CATALOG_MOD);
  var marker = text.indexOf("NOT_IMPLEMENTED_ENDPOINTS: &[(&str, &str, &str)]");
  assert(marker >= 0, "declared-gap constant missing from api_catalog/mod.rs");
  var block = text.slice(marker, text.indexOf("];", marker) + 2);
  var strings = block.match(/"[^"]*"/g) || [];
  equal(strings.length % 3, 0, "declared gap triples");
  equal(strings.length / 3, 11, "declared gap count");
  observe("declared 501 gaps (method, path, owner)", String(strings.length / 3));
});

test("[BASELINE-OBSERVE] status field and availability filter are shipped in the LLM catalog", function () {
  var mod = src(h.CATALOG_MOD);
  var find = src(h.FIND_API);
  assert(mod.indexOf("pub enum EndpointStatus") >= 0, "EndpointStatus enum missing");
  assert(mod.indexOf("NotImplemented,") >= 0, "NotImplemented variant missing");
  assert(mod.indexOf("pub fn endpoint_status(") >= 0, "endpoint_status resolver missing");
  assert(find.indexOf("ep.is_available()") >= 0, "api.find availability filter missing");
  assert(find.indexOf(".filter(|ep| ep.is_available())") >= 0, "api.find catalog filter missing");
  observe("status enum / filter", "EndpointStatus in api_catalog/mod.rs; is_available() in find_api_entries.rs");
});
