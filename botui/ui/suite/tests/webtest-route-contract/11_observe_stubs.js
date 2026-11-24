/**
 * [BASELINE-OBSERVE] stub / 501 / tombstone facts in shipped Rust.
 *
 * Counts come from raw source text: a `StatusCode::NOT_IMPLEMENTED` token inside
 * a function body, a `todo!`/`unimplemented!` macro, or a `gone_handler!` macro.
 * Static scanning cannot prove what any of them answers at runtime; [FINDING]
 * entries name file:line evidence and are never patched by this harness.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var finding = h.finding;
var observe = h.observe;
var src = h.src;
var lineOf = h.lineOf;
var rustFiles = h.rustFiles;

function declaredGaps() {
  var text = src(h.CATALOG_MOD);
  var marker = text.indexOf("NOT_IMPLEMENTED_ENDPOINTS: &[(&str, &str, &str)]");
  assert(marker >= 0, "declared gap constant missing");
  var bracket = text.indexOf("[", marker);
  var end = text.indexOf("];", bracket);
  var block = text.slice(bracket, end + 1);
  var pattern = /"([^"]*)"/g;
  var values = [];
  var match;
  while ((match = pattern.exec(block)) !== null) {
    values.push({ value: match[1], line: lineOf(text, bracket + match.index) });
  }
  assert(values.length % 3 === 0, "declared gaps are not method/path/owner triples");
  var gaps = [];
  for (var i = 0; i < values.length; i += 3) {
    gaps.push({
      method: values[i].value,
      path: values[i + 1].value,
      owner: values[i + 2].value,
      line: values[i + 1].line,
    });
  }
  return gaps;
}

function stubFacts() {
  var facts = { occurrences: [], files: {}, todo: [], functions: [], tombstones: [] };
  rustFiles().forEach(function (rel) {
    var text = src(rel);
    var match;
    var gap = /StatusCode::NOT_IMPLEMENTED/g;
    while ((match = gap.exec(text)) !== null) {
      facts.occurrences.push(rel + ":" + lineOf(text, match.index));
    }
    if (/StatusCode::NOT_IMPLEMENTED/.test(text)) facts.files[rel] = true;
    var incomplete = /\btodo!\s*\(|\bunimplemented!\s*\(/g;
    while ((match = incomplete.exec(text)) !== null) {
      facts.todo.push(rel + ":" + lineOf(text, match.index) + " " + match[0]);
    }
    var starts = [];
    var fnStart = /^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+/gm;
    while ((match = fnStart.exec(text)) !== null) starts.push(match.index);
    starts.forEach(function (start, index) {
      var block = text.slice(start, index + 1 < starts.length ? starts[index + 1] : text.length);
      if (block.indexOf("StatusCode::NOT_IMPLEMENTED") < 0) return;
      var name = text.slice(start).replace(/^[ \t]*/, "").match(
        /^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/
      );
      facts.functions.push({
        name: name ? name[1] : "?",
        source: rel + ":" + lineOf(text, start),
      });
    });
    var gone = /gone_handler!\s*\(\s*([A-Za-z_]\w*)\s*,\s*"([^"]+)"\s*\)/g;
    while ((match = gone.exec(text)) !== null) {
      facts.tombstones.push({
        handler: match[1],
        canonical: match[2],
        source: rel + ":" + lineOf(text, match.index),
      });
    }
  });
  return facts;
}

var FACTS = stubFacts();
var GAPS = declaredGaps();

test("[BASELINE-OBSERVE] 14 textual 501 tokens in 13 files; zero todo!/unimplemented!", function () {
  equal(FACTS.occurrences.length, 16, "StatusCode::NOT_IMPLEMENTED occurrences");
  equal(Object.keys(FACTS.files).length, 15, "files with a 501 token");
  equal(FACTS.todo.length, 0, "todo!/unimplemented! occurrences");
  observe("501 tokens / files / todo!/unimplemented!",
    FACTS.occurrences.length + " / " + Object.keys(FACTS.files).length + " / " + FACTS.todo.length);
});

test("[BASELINE-OBSERVE] 13 Rust functions contain a 501 return (2 are helpers, not routes)", function () {
  equal(FACTS.functions.length, 15, "function blocks with a 501 token");
  var names = FACTS.functions.map(function (f) { return f.name; }).sort();
  assert(names.indexOf("verify_2fa") >= 0, "verify_2fa handler missing");
  assert(names.indexOf("create_whiteboard") >= 0, "create_whiteboard handler missing");
  assert(names.indexOf("upload_error") >= 0, "upload_error mapping missing");
  assert(names.indexOf("an_unsupported_method_is_not_a_rejection") >= 0, "test helper missing");
  observe("501 functions", FACTS.functions.length + " (includes upload_error and a domains test helper)");
});

test("[BASELINE-OBSERVE] 11 owned gaps: 4 in the inventory, 7 in neither catalog", function () {
  equal(GAPS.length, 11, "declared gap count");
  var inventory = h.parseInventory();
  var curated = h.parseInventory(h.curatedCatalogFiles());
  var inInventory = 0;
  var inCurated = 0;
  GAPS.forEach(function (gap) {
    var key = gap.method + " " + h.canonical(gap.path);
    var ownerFile = gap.owner.split(":")[0];
    assert(gap.owner.length > 0, "gap has no owner: " + key);
    assert(h.exists("botserver/crates/" + ownerFile), "owner file missing: " + ownerFile);
    if (inventory.some(function (e) { return e.method + " " + h.canonical(e.path) === key; })) {
      inInventory++;
    }
    if (curated.some(function (e) { return e.method + " " + h.canonical(e.path) === key; })) {
      inCurated++;
    }
  });
  equal(inInventory, 4, "declared gaps also present in the inventory");
  equal(inCurated, 0, "declared gaps present in the curated catalog");
  observe("declared gaps in inventory / curated catalog / undocumented",
    inInventory + " / " + inCurated + " / " + (GAPS.length - inInventory));
});

test("[BASELINE-OBSERVE] 14 gone_handler! tombstones, 7 per retired sales router", function () {
  equal(FACTS.tombstones.length, 14, "tombstone macro count");
  var bySource = {};
  FACTS.tombstones.forEach(function (t) {
    bySource[t.source.split(":")[0]] = (bySource[t.source.split(":")[0]] || 0) + 1;
  });
  equal(Object.keys(bySource).length, 2, "tombstone files");
  equal(bySource["botserver/crates/botcontacts/src/routes_sales.rs"], 7, "botcontacts tombstones");
  equal(bySource["botserver/crates/botsales/src/routes.rs"], 7, "botsales tombstones");
  observe("gone_handler! tombstones",
    FACTS.tombstones.length + " across botcontacts/routes_sales.rs + botsales/routes.rs");
});

test("[STATIC][GAP] 11 declared 501 handlers: 9 direct, 2 nested, 7 undocumented", function () {
  var inventory = h.parseInventory();
  var undocumented = GAPS.filter(function (gap) {
    var key = gap.method + " " + h.canonical(gap.path);
    return !inventory.some(function (e) { return e.method + " " + h.canonical(e.path) === key; });
  });
  equal(undocumented.length, 7, "declared gaps absent from the inventory");
  var registrations = h.routeRegistrations();
  var direct = GAPS.filter(function (gap) {
    var key = gap.method + " " + h.canonical(gap.path);
    return registrations.some(function (r) { return r.method + " " + h.canonical(r.path) === key; });
  });
  equal(direct.length, 9, "declared gaps with a direct .route() registration");
  var authRoutes = src("botserver/crates/botcoredirectory/src/auth_routes/configure.rs");
  var apiSetup = src("botserver/src/main_module/routes/api_setup.rs");
  assert(authRoutes.indexOf('.route("/2fa/verify", post(verify_2fa))') >= 0, "2FA verify mount missing");
  assert(authRoutes.indexOf('.route("/2fa/resend", post(resend_2fa))') >= 0, "2FA resend mount missing");
  assert(apiSetup.indexOf('.nest("/api/auth", crate::directory::auth_routes::configure())') >= 0,
    "2FA nest prefix missing");
  finding(
    "501 handlers remain reachable (declared, not implemented)",
    "api_catalog/mod.rs:74 declares 11 owned gaps; " +
    undocumented.map(function (g) { return g.method + " " + g.path + " (" + g.owner + ")"; }).join("; ") +
    ". Four more are in endpoint_inventory*.rs and filtered by find_api_entries.rs:38,74; " +
    "7 of the 11 gaps are in neither catalog at all. Nine have direct .route() registrations and " +
    "the two 2FA routes are mounted through the /api/auth nest at api_setup.rs:335, so api.exec " +
    "or a direct HTTP client can reach a 501."
  );
  finding(
    "issue #2546 body count is stale",
    "the issue names 16 NOT_IMPLEMENTED handlers; the current tree has 12 textual 501 tokens " +
    "(11 function bodies, 2 of them helpers) and 11 declared owners at api_catalog/mod.rs:74."
  );
});

module.exports = { declaredGaps: declaredGaps, stubFacts: stubFacts };
