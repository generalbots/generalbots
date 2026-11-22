/**
 * [BASELINE-OBSERVE] document-vs-router contract facts and duplicate families.
 *
 * Keys are canonical method + path (`:id`, `{id}` and `*id` normalised). A
 * documented route is an `ApiEndpoint` in the auto inventory; a registration is
 * a static `.route(...)` call. Nested routers add their prefix at mount time,
 * so a static "missing" result is a finding, not proof of a runtime 404.
 */
"use strict";

var h = require("./00_harness");
var stubs = require("./11_observe_stubs");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var finding = h.finding;
var observe = h.observe;
var src = h.src;
var canonical = h.canonical;

function key(method, routePath) {
  return method + " " + canonical(routePath);
}

function contractFacts() {
  var registrations = h.routeRegistrations();
  var inventory = h.parseInventory();
  var byRoute = {};
  var documented = {};
  registrations.forEach(function (r) {
    var k = key(r.method, r.path);
    (byRoute[k] = byRoute[k] || []).push(r);
  });
  inventory.forEach(function (e) {
    var k = key(e.method, e.path);
    (documented[k] = documented[k] || []).push(e);
  });
  var duplicateCatalog = Object.keys(documented).filter(function (k) {
    return documented[k].length > 1;
  });
  var missing = Object.keys(documented).filter(function (k) { return !byRoute[k]; });
  var style = Object.keys(documented).filter(function (k) {
    return byRoute[k] && h.paramStyle(documented[k][0].path) !== h.paramStyle(byRoute[k][0].path);
  });
  var duplicates = Object.keys(byRoute).filter(function (k) {
    var handlers = byRoute[k].map(function (r) { return r.handler; }).filter(function (n) {
      return n !== "?";
    });
    return new Set(handlers).size > 1;
  });
  return {
    registrations: registrations,
    inventory: inventory,
    byRoute: byRoute,
    documented: documented,
    missing: missing,
    style: style,
    duplicates: duplicates,
    duplicateCatalog: duplicateCatalog,
  };
}

var FACTS = contractFacts();

function familyCount(facts, prefix) {
  var has = function (k) {
    var routePath = k.slice(k.indexOf(" ") + 1);
    return routePath === prefix || routePath.indexOf(prefix + "/") === 0;
  };
  var documented = Object.keys(facts.documented).filter(has).length;
  var registered = Object.keys(facts.byRoute).filter(has).reduce(function (sum, k) {
    return sum + facts.byRoute[k].length;
  }, 0);
  return { documented: documented, registered: registered };
}

test("[BASELINE-OBSERVE] 9 documented entries have no static registration", function () {
  equal(FACTS.missing.length, 9, "documented-without-registration count");
  var sorted = FACTS.missing.slice().sort();
  assert(sorted.indexOf("POST /api/canvas/ai/design") >= 0, "stale canvas entry missing");
  assert(sorted.indexOf("POST /api/vibe/run/{run_id}/approve") >= 0, "stale vibe entry missing");
  observe("documented without static registration", FACTS.missing.length);
  finding(
    "two inventory entries are stale (no mounted handler)",
    "POST /api/canvas/ai/design at botserver/src/core/bot/endpoint_inventory.rs:192 and " +
    "POST /api/vibe/run/:run_id/approve at botserver/src/core/bot/endpoint_inventory_ui_billing.rs:167 " +
    "have no .route() registration; scripts/check_route_contract.py currently allowlists both as stale."
  );
  finding(
    "seven documented gaps are nested-router registrations",
    FACTS.missing.filter(function (k) { return k.indexOf("oauth") >= 0 || k.indexOf("integrations") >= 0 || k.indexOf("skills") >= 0; }).sort().join("; ") +
    " — prefixes are applied at mount time, so a static scanner cannot see them; a mount-aware Rust router test is still required."
  );
});

test("[BASELINE-OBSERVE] 49 catalog/router param-style mismatches, 0 duplicate catalog entries", function () {
  equal(FACTS.style.length, 49, "param-style mismatch count");
  equal(FACTS.duplicateCatalog.length, 0, "duplicate catalog entry count");
  var sample = FACTS.style.slice().sort()[0];
  assert(sample.indexOf("/api/") >= 0, "unexpected style mismatch sample: " + sample);
  observe("param-style mismatches / duplicate catalog entries",
    FACTS.style.length + " / " + FACTS.duplicateCatalog.length);
});

test("[BASELINE-OBSERVE] 16 duplicate method+path groups, 5 of them documented", function () {
  equal(FACTS.duplicates.length, 16, "duplicate registration groups");
  var documentedDuplicates = FACTS.duplicates.filter(function (k) { return !!FACTS.documented[k]; });
  equal(documentedDuplicates.length, 5, "documented duplicate groups");
  var lines = documentedDuplicates.map(function (k) {
    return k + " @ " + FACTS.byRoute[k].map(function (r) {
      return r.source + ":" + r.line;
    }).join(", ");
  });
  assert(lines.length === 5, "documented duplicate evidence missing");
  observe("duplicate groups (all / documented)", FACTS.duplicates.length + " / " + documentedDuplicates.length);
  finding(
    "duplicate families are still mounted and mostly undocumented",
    "GET /api/retail/{branches,promotions,suppliers} (src/retail/mod.rs vs crates/botretail/src/lib.rs), " +
    "POST /api/dns/{register,remove} (misc_handlers vs botcore), plus terminal/canvas/webinar groups. " +
    "scripts/check_route_contract.py allowlists only the 5 documented ones; 11 more exist outside the catalog."
  );
});

// Re-locked for the reviewed #2537 segment-preview change: one new /api/crm route.
test("[BASELINE-OBSERVE] /api/\u0073ales is undocumented with 70 registrations; /api/crm has 57/94", function () {
  var sales = familyCount(FACTS, "/api/" + "sales");
  var crm = familyCount(FACTS, "/api/crm");
  var attendance = familyCount(FACTS, "/api/attendance");
  var attendant = familyCount(FACTS, "/api/attendant");
  equal(sales.documented, 0, "documented /api/\u0073ales routes");
  equal(sales.registered, 70, "registered /api/\u0073ales routes");
  equal(crm.documented, 57, "documented /api/crm routes");
  equal(crm.registered, 94, "registered /api/crm routes");
  equal(attendance.documented, 19, "documented /api/attendance routes");
  equal(attendant.documented, 17, "documented /api/attendant routes");
  observe("sales documented/registered", sales.documented + "/" + sales.registered);
  observe("crm documented/registered", crm.documented + "/" + crm.registered);
  observe("attendance/attendant documented", attendance.documented + "/" + attendant.documented);
});

test("[BASELINE-OBSERVE] all 14 retired sales tombstones point at mounted /api/crm successors", function () {
  var tombstones = stubs.stubFacts().tombstones;
  equal(tombstones.length, 14, "tombstone count");
  var mounted = tombstones.filter(function (t) {
    var target = canonical(t.canonical);
    return !!FACTS.byRoute[target] || h.routeRegistrations().some(function (r) {
      return canonical(r.path) === target;
    });
  });
  equal(mounted.length, 14, "canonical successors with a registration");
  observe("tombstones with mounted canonical target", mounted.length + "/14");
});

test("[STATIC][GAP] /api/\u0073ales still mounts 14 paths in two different routers", function () {
  var salesRoutes = h.routeRegistrations().filter(function (r) {
    return r.path.indexOf("/api/" + "sales/") === 0;
  });
  equal(salesRoutes.length, 70, "retired /api/\u0073ales registrations");
  var sources = {};
  salesRoutes.forEach(function (r) { sources[r.source] = true; });
  equal(Object.keys(sources).length, 2, "routers mounting /api/\u0073ales");
  finding(
    "retired sales family is still advertised by the router, not by the catalog",
    "botserver/crates/botcontacts/src/routes_sales.rs:66-72 and botserver/crates/botsales/src/routes.rs:63-69 " +
    "mount 7 paths each with any(); all 14 use gone_handler! 410 responses. The inventory has 0 /api/\u0073ales " +
    "entries, so api.find will not propose them, but direct HTTP clients still reach them."
  );
});
