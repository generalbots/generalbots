/**
 * [STATIC] acceptance-criteria observations. A test here PASSES when it has
 * honestly located the shipped evidence or labelled the gap; it never claims a
 * runtime, browser or LLM result. [DEFERRED] criteria need a real Router, a
 * server and CI, which this read-only Node process deliberately does not build.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var finding = h.finding;
var observe = h.observe;
var src = h.src;
var exists = h.exists;

var CATALOG_TEST = "botserver/src/core/bot/endpoint_inventory/endpoint_inventory_tests.rs";
var SERVER = "botserver/src/main_module/server/run_axum_server.rs";
var GUARD = "scripts/check_route_contract.py";

test("[STATIC][PARTIAL] A1 Rust contract tests exist, but none builds run_axum_server's Router", function () {
  var tests = src(CATALOG_TEST);
  var mod = src(h.CATALOG_MOD);
  assert(tests.indexOf("fn documented_source_files_still_exist()") >= 0, "source-pointer test missing");
  assert(tests.indexOf("fn no_duplicate_method_and_path()") >= 0, "duplicate catalog test missing");
  assert(mod.indexOf("mod route_contract_tests") >= 0, "api_catalog status tests missing");
  assert(src(SERVER).indexOf("pub async fn run_axum_server(") >= 0, "real router builder missing");
  assert(tests.indexOf("Router::new()") < 0 && mod.indexOf("Router::new()") < 0,
    "a Router builder unexpectedly appears in the static contract tests");
  observe("Rust contract tests", "source pointers + catalog uniqueness + status labels; no Router instance");
  finding(
    "A1 real-router assertion is not shipped",
    "the acceptance criterion asks for a test that builds the same Router as run_axum_server.rs:5. " +
    "The shipped tests (endpoint_inventory_tests.rs and api_catalog/mod.rs:306) check source pointers, " +
    "uniqueness and status labels only; 9 documented paths have no static registration."
  );
});

test("[STATIC][OBSERVED] A2 Python guard implements missing/duplicate/style checks with allowlists", function () {
  var guard = src(GUARD);
  assert(guard.indexOf("no matching router registration") >= 0, "missing-route check missing");
  assert(guard.indexOf("duplicate handler for") >= 0, "duplicate-handler check missing");
  assert(guard.indexOf("param-style mismatch") >= 0, "param-style check missing");
  assert(guard.indexOf("KNOWN_MISSING") >= 0 && guard.indexOf("KNOWN_STYLE") >= 0 &&
    guard.indexOf("KNOWN_DUPLICATES") >= 0, "guard allowlists missing");
  assert(guard.indexOf("self-test: all 7 checks passed") >= 0, "guard self-test missing");
  observe("Python guard", "3 failure modes + 3 allowlists + 7 self-checks; not the real-router test");
});

test("[STATIC][PARTIAL] A3 declared gaps are filtered from api.find, but still mounted", function () {
  var find = src(h.FIND_API);
  assert(find.indexOf("EndpointStatus::NotImplemented") >= 0, "501 status filter missing from discovery");
  assert(find.indexOf("ep.is_available()") >= 0, "availability filter missing from the catalog");
  assert(find.indexOf("endpoint_status(c.method, &c.path)") >= 0, "derived-command 501 filter missing");
  observe("501 filtering", "declared gaps excluded from api.find and derived commands; routers unchanged");
  finding(
    "A3 'never planned' cannot be established offline",
    "the filter is present in find_api_entries.rs:38,70-75, but which endpoints an LLM actually plans " +
    "is a runtime/LLM criterion; api.exec can still call a mounted 501 path directly."
  );
});

test("[STATIC][GAP] A4 CI runs the Python guards and all binary tests, not the filtered route_contract command", function () {
  var ci = src(h.CI_WORKFLOW);
  assert(ci.indexOf("python3 scripts/check_route_contract.py") >= 0, "route contract guard absent from CI");
  assert(ci.indexOf("python3 scripts/check_sales_route_contract.py --check") >= 0, "sales guard absent from CI");
  assert(ci.indexOf("run: cargo test -p botserver --bin botserver") >= 0, "binary test job absent from CI");
  assert(ci.indexOf("cargo test -p botserver --bin botserver route_contract") < 0,
    "filtered route_contract command unexpectedly present");
  observe("CI", "Python guards + full binary test job; no `route_contract` filtered invocation");
  finding(
    "A4 filtered CI command is absent",
    ".github/workflows/ci.yml:290-300 runs the Python guards and line 1139 runs the whole binary test " +
    "suite. The acceptance criterion names `cargo test -p botserver --bin botserver route_contract`; " +
    "that exact filter is not present."
  );
});

test("[STATIC][OBSERVED] A5 catalog status field is consumed by the manifest builder", function () {
  var mod = src(h.CATALOG_MOD);
  assert(mod.indexOf("pub enum EndpointStatus") >= 0, "status enum missing");
  assert(mod.indexOf("pub struct CatalogEndpoint") >= 0, "catalog entry wrapper missing");
  assert(mod.indexOf("fn from_endpoint(") >= 0 && mod.indexOf("endpoint_status(") >= 0,
    "catalog status derivation missing");
  assert(src(h.FIND_API).indexOf('"status": ep.status') >= 0, "manifest status field missing");
  observe("A5", "EndpointStatus -> CatalogEndpoint -> api.find JSON status");
});

test("[STATIC][DEFERRED] runtime 404/501, LLM planning and browser evidence are out of scope", function () {
  finding(
    "runtime criteria remain open",
    "no HTTP request is made here: 404 for the 2 stale catalog entries, 501 for the 11 declared gaps, " +
    "410 for the 14 retired sales routes and the LLM's api.find/api.exec behaviour need a running " +
    "botserver plus a browser or CI run. No runtime PASS is claimed."
  );
  observe("deferred", "router response codes, LLM planning, browser tabs, CI result");
});

test("[STATIC][SELF] harness is read-only, owns two paths, has no baseline and stays under 450 lines", function () {
  assert(!exists("botui/ui/suite/tests/webtest-route-contract/baseline.json"), "harness owns a baseline file");
  var files = ["botui/ui/suite/tests/webtest-route-contract.js"].concat(
    h.walk("botui/ui/suite/tests/webtest-route-contract", ".js")
  );
  equal(files.length, 6, "owned file count");
  var writeApi = "write" + "FileSync";
  var appendApi = "append" + "FileSync";
  files.forEach(function (rel) {
    var text = h.readRepo(rel);
    assert(text.split("\n").length < 450, "file over 450 lines: " + rel);
    assert(text.indexOf(writeApi) < 0 && text.indexOf(appendApi) < 0,
      "harness writes a file: " + rel);
  });
  observe("harness", "6 files, read-only, no baseline/ratchet file, all <450 lines");
});

test("[STATIC][SELF] scanner source avoids gbT concatenation and literal i18n attribute selectors", function () {
  var gbt = "gb" + "T(";
  var attr = "data-i18n" + "=";
  h.walk("botui/ui/suite/tests/webtest-route-contract", ".js").forEach(function (rel) {
    var text = h.readRepo(rel);
    assert(text.indexOf(gbt) < 0, "gbT call in harness source: " + rel);
    assert(text.indexOf(attr + '"') < 0, "literal i18n attribute selector in harness source: " + rel);
  });
  observe("i18n scanner safety", "no concatenated gbT key and no literal data-i18n attribute selector");
});
