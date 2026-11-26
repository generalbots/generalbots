/**
 * Offline self-test for #2955 — KYC/Retail `TypeError: t is not a function`
 * and the Retail stock 404. Run with:
 *
 *   node botui/ui/suite/tests/webtest-kyc-retail-appkit.js
 *
 * [STATIC]   the i18n helper is never shadowed in the shared app kit, and every
 *            /api/kyc/* or /api/retail/* path the two apps call is registered
 *            in the router that is actually mounted by botserver;
 * [RUNTIME]  GBAppKit.apiCall is loaded in a VM and driven through a 404, a
 *            network rejection and malformed JSON — each must return null and
 *            use the i18n helper instead of throwing a TypeError;
 * [LIVE-GATE] a rebuilt binary + authenticated browser must still confirm no
 *            console exceptions. This test never claims that runtime PASS.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var ROOT = path.resolve(__dirname, "..", "..", "..", "..");
var KIT = path.join(ROOT, "botui", "ui", "suite", "js", "gb-app-kit.js");
var RETAIL_APP = path.join(ROOT, "botui", "ui", "suite", "retail", "retail.js");
var KYC_APP = path.join(ROOT, "botui", "ui", "suite", "kyc", "kyc.js");
var RETAIL_ROUTER = path.join(ROOT, "botserver", "crates", "botretail", "src", "lib.rs");
var RETAIL_STUB = path.join(ROOT, "botserver", "src", "retail", "mod.rs");
var RETAIL_MERGE = path.join(ROOT, "botserver", "src", "main_module", "routes",
  "feature_merge_platform", "commerce.rs");
var KYC_ROUTER = path.join(ROOT, "botserver", "crates", "botkyc", "src", "routes.rs");
var KYC_MERGE = path.join(ROOT, "botserver", "src", "main_module", "routes",
  "feature_merge_office", "api_surfaces.rs");

var PASS = 0;
var FAIL = 0;

function test(name, fn) {
  try { fn(); PASS++; console.log("[PASS] " + name); }
  catch (error) { FAIL++; console.log("[FAIL] " + name + " :: " + error.message); }
}
async function asyncTest(name, fn) {
  try { await fn(); PASS++; console.log("[PASS] " + name); }
  catch (error) { FAIL++; console.log("[FAIL] " + name + " :: " + error.message); }
}
function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}
function read(file) {
  assert(fs.existsSync(file), "missing shipped source: " + file);
  return fs.readFileSync(file, "utf8");
}
function matchAll(text, regex) {
  var found = [];
  var m;
  while ((m = regex.exec(text)) !== null) found.push(m[1]);
  return found;
}
function normalizeRoute(p) {
  return p.replace(/\/:.*$/, "").replace(/\/+$/, "");
}
function fakeElement() {
  return {
    id: "", textContent: "", style: {}, hidden: false, tabIndex: 0,
    classList: { add: function () {}, remove: function () {}, toggle: function () {} },
    setAttribute: function () {}, appendChild: function () {}, remove: function () {},
    addEventListener: function () {}
  };
}
/** Load the shipped kit in a VM with inert DOM and an i18n spy. */
function loadKit(fetchImpl) {
  var calls = { i18n: [], feedback: [] };
  var element = fakeElement();
  var sandbox = {
    console: console,
    setTimeout: function () {},
    fetch: fetchImpl,
    document: {
      getElementById: function () { return element; },
      createElement: function () { return fakeElement(); },
      body: { appendChild: function () {} }
    },
    localStorage: { getItem: function () { return null; } },
    sessionStorage: { getItem: function () { return null; } },
    URL: { createObjectURL: function () { return "blob:test"; }, revokeObjectURL: function () {} },
    gbT: function (key, fallback) { calls.i18n.push(key); return fallback; }
  };
  sandbox.window = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(KIT, "utf8"), sandbox, { filename: KIT });
  // Capture the feedback text without depending on DOM internals.
  Object.defineProperty(element, "textContent", {
    get: function () { return calls.feedback[calls.feedback.length - 1] || ""; },
    set: function (v) { calls.feedback.push(v); }
  });
  assert(typeof sandbox.GBAppKit === "object" && sandbox.GBAppKit !== null,
    "gb-app-kit.js did not publish GBAppKit");
  return { kit: sandbox.GBAppKit, calls: calls };
}
function response(ok, status, body) {
  return Promise.resolve({
    ok: ok,
    status: status,
    text: function () { return Promise.resolve(body); }
  });
}

var kitSource = read(KIT);
var retailSource = read(RETAIL_APP);
var kycSource = read(KYC_APP);
var retailRouter = read(RETAIL_ROUTER);
var retailStub = read(RETAIL_STUB);
var retailMerge = read(RETAIL_MERGE);
var kycRouter = read(KYC_ROUTER);
var kycMerge = read(KYC_MERGE);

// ---------------------------------------------------------------------------
// [STATIC] 1. The i18n helper `t` is not shadowed in the shared skeleton.
// ---------------------------------------------------------------------------
test("gb-app-kit.js declares no local variable named t", function () {
  assert(!/\b(?:var|let|const)\s+t\b/.test(kitSource),
    "a local `t` still shadows the module-level i18n helper");
  assert(/function\s+t\s*\(key,\s*fallback\)/.test(kitSource),
    "the module-level i18n helper `t(key, fallback)` was removed");
  assert(/showFeedback\(appId,\s*t\("app-kit-api-error"/.test(kitSource),
    "apiCall's error path no longer uses the i18n helper");
});

// ---------------------------------------------------------------------------
// [STATIC] 2. Route contracts — every path the apps call is really mounted.
// ---------------------------------------------------------------------------
var mountedRetail = matchAll(retailRouter, /\.route\(\s*"(\/api\/retail\/[^"]+)"/g)
  .map(normalizeRoute);
var calledRetail = matchAll(retailSource, /(\/api\/retail\/[a-z-]+)/g);
var stubRetail = matchAll(retailStub, /\.route\(\s*"(\/api\/retail\/[^"]+)"/g)
  .map(normalizeRoute);

test("botretail registers every /api/retail/* path called by retail.js", function () {
  var missing = calledRetail.filter(function (p) { return mountedRetail.indexOf(p) === -1; });
  assert(missing.length === 0, "unmounted Retail paths: " + missing.join(", "));
  assert(mountedRetail.indexOf("/api/retail/stock") !== -1,
    "/api/retail/stock is not registered in the mounted botretail router");
  assert(mountedRetail.indexOf("/api/retail/top-products") !== -1,
    "/api/retail/top-products is not registered in the mounted botretail router");
});

test("the unmounted stub's GET paths are all covered by the live router", function () {
  var missing = stubRetail.filter(function (p) { return mountedRetail.indexOf(p) === -1; });
  assert(missing.length === 0,
    "stub-only paths (404 on open): " + missing.join(", "));
});

test("botretail::configure() is merged under the retail feature gate", function () {
  assert(/#\[cfg\(feature = "retail"\)\][\s\S]*?botretail::configure\(\)/.test(retailMerge),
    "botretail::configure() is not merged under #[cfg(feature = \"retail\")]");
});

var mountedKyc = matchAll(kycRouter, /\.route\(\s*"(\/api\/kyc\/[^"]+)"/g)
  .map(normalizeRoute);
var calledKyc = matchAll(kycSource, /(\/api\/kyc\/[a-z-]+)/g);

test("botkyc registers every /api/kyc/* path called by kyc.js", function () {
  var missing = calledKyc.filter(function (p) { return mountedKyc.indexOf(p) === -1; });
  assert(missing.length === 0, "unmounted KYC paths: " + missing.join(", "));
});

test("botkyc::configure() is actually merged by botserver", function () {
  assert(kycMerge.indexOf("botkyc::configure()") !== -1,
    "botkyc::configure() is not merged into the API router");
});

// ---------------------------------------------------------------------------
// [RUNTIME] 3. apiCall's error paths return null and never throw a TypeError.
// ---------------------------------------------------------------------------
async function main() {
  await asyncTest("apiCall: 404 returns null and uses gbT (no TypeError)", async function () {
    var loaded = loadKit(function () { return response(false, 404, ""); });
    var result = await loaded.kit.apiCall("retail", "/api/retail/stock");
    assert(result === null, "a failed request must resolve to null");
    assert(loaded.calls.i18n.indexOf("app-kit-api-error") !== -1,
      "the i18n helper was not called on the error path");
    assert(/API error/.test(loaded.calls.feedback.join(" ")),
      "no feedback was shown for the failed request");
  });

  await asyncTest("apiCall: network rejection returns null (no TypeError)", async function () {
    var loaded = loadKit(function () { return Promise.reject(new Error("offline")); });
    var result = await loaded.kit.apiCall("kyc", "/api/kyc/verifications");
    assert(result === null, "a rejected fetch must resolve to null");
    assert(loaded.calls.i18n.indexOf("app-kit-api-error") !== -1,
      "the i18n helper was not called after a network rejection");
  });

  await asyncTest("apiCall: malformed JSON returns null (no TypeError)", async function () {
    var loaded = loadKit(function () { return response(true, 200, "not-json"); });
    var result = await loaded.kit.apiCall("retail", "/api/retail/stock");
    assert(result === null, "a malformed body must resolve to null");
    assert(loaded.calls.i18n.indexOf("app-kit-api-error") !== -1,
      "the i18n helper was not called after a parse failure");
  });

  await asyncTest("apiCall: 200 + valid JSON still parses normally", async function () {
    var loaded = loadKit(function () {
      return response(true, 200, '[{"sku":"A-1"}]');
    });
    var result = await loaded.kit.apiCall("retail", "/api/retail/stock");
    assert(Array.isArray(result) && result[0].sku === "A-1",
      "a valid body was not parsed into an array");
    assert(loaded.calls.i18n.length === 0, "i18n helper called on the success path");
  });

  console.log("[LIVE-GATE] rebuilt binary + authenticated browser must still confirm:");
  console.log("  GET /api/retail/stock -> 200 and a JSON array (no 404)");
  console.log("  GET /api/retail/top-products -> 200 and a JSON array (no 404)");
  console.log("  KYC and Retail open with zero uncaught `t is not a function` exceptions");
  console.log("kyc/retail app-kit self-test (offline): " + PASS + " passed, " + FAIL + " failed");
  process.exitCode = FAIL === 0 ? 0 : 1;
}

main().catch(function (error) {
  console.error("[FAIL] harness: " + error.message);
  process.exitCode = 1;
});
