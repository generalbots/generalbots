/**
 * #2529 size claims, honest findings and [STATIC] scope documentation.
 *
 * [STATIC] assertions record acceptance criteria that are NOT shipped. They
 * assert absence in shipped source and deliberately make no runtime claim.
 * Genuine discrepancies between the issue text and the observed code are
 * reported with note() -- they are never patched here.
 */
"use strict";

var h = require("./00_harness");
var s = require("./01_sources");

var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var note = h.note;

test("[BASELINE-OBSERVE] #2529: import cap is 50 MB and save/element routes carry the same body_limit", function () {
  var policy = s.rust("uploadPolicy");
  var transport = s.rust("uploadTransport");
  var routes = s.rust("routes");
  assert(s.has(policy, "static SLIDES_IMPORT_LIMITS: UploadLimits"), "slides import class exists");
  assert(s.has(policy, "max_bytes: 50 * 1024 * 1024"), "cap is 50 MB");
  assert(s.has(transport, "DefaultBodyLimit::max(max_bytes(class))"), "body_limit resolves to that cap");
  var limits = s.count(routes, "body_limit(upload_policy::UploadClass::PresentationImport)");
  assert(limits >= 4, "save, element/add, element/update and import all have a body_limit");
  note("FINDING (reported, not patched): the issue's 'no body limit / axum 2 MB default' claim is stale -- routes.rs already applies a 50 MB PresentationImport body_limit to save/element/import.");
});

test("[BASELINE-OBSERVE] #2529: the only shipped size claim is a 1 MB offline image insert", function () {
  var insert = s.readModule("11_insert_dialog.js");
  assert(s.has(insert, "slides-insert-file-too-large"), "insert dialog uses the size-limit key");
  assert(s.has(insert, "1 MB"), "claim is 1 MB for offline insert");
  s.LOCALES.forEach(function (locale) {
    var text = h.readRepo("botlib/locales/" + locale + "/slides-insert.ftl");
    assert(/^slides-insert-file-too-large\s*=/m.test(text), locale + " defines the real catalog key");
  });
  equal(s.LOCALES.length, 8, "eight locale catalogs checked");
  note("FINDING (reported, not patched): no measured baseline table exists in the repository for 10/25/50 MB or 50/150 slides; acceptance criterion 1 remains unmet and needs CI/Browser evidence.");
});

test("[STATIC] #2529: media-out-of-model and lazy migration are not shipped", function () {
  var combined = s.rust("import") + s.rust("storage") + s.allJs();
  ["media_url", "object_ref", "store_media", "migrate_media", "lazy_migrate"].forEach(function (marker) {
    assert(!s.has(combined, marker), "criterion marker absent: " + marker);
  });
  note("STATIC NOT SHIPPED: upload each media asset to Drive, store a stable URL/id in the deck, and lazily migrate existing decks. Only a local vm/source check is possible offline; reduced deck-JSON size must be measured in CI.");
});

test("[STATIC] #2529: element-scoped server persistence and a deck manifest are not shipped", function () {
  var combined = s.rust("handlers") + s.rust("elementOps") + s.rust("storage");
  ["deck_manifest", "patch_element", "per_element_write", "element_scoped_store"].forEach(function (marker) {
    assert(!s.has(combined, marker), "criterion marker absent: " + marker);
  });
  note("STATIC NOT SHIPPED: one move writes one element. Observed request is element-scoped but the server performs load_pres!/save_pres! of the whole deck. Payload proportionality needs a browser/CDP round-trip measurement.");
});

test("[STATIC] #2529: windowed thumbnails and lazy loading are not shipped", function () {
  var combined = s.allJs() + s.rust("getBridge") + s.rust("panels");
  ["IntersectionObserver", "lazy_thumb", "thumbnail_window", "slide_window"].forEach(function (marker) {
    assert(!s.has(combined, marker), "criterion marker absent: " + marker);
  });
  note("STATIC NOT SHIPPED: active slide plus a small thumbnail window with lazy loading. The view is already single-slide; the thumbnail rail and sorter still render every slide. First-slide paint must be measured in a browser.");
});

test("[STATIC] #2529: element-delta sync, import progress/cancel and degradation are not shipped", function () {
  var combined = s.allJs() + s.rust("routes") + s.rust("storage");
  ["element_delta", "delta_sync", "importProgress", "cancelImport", "abortImport", "slowDevice"].forEach(function (marker) {
    assert(!s.has(combined, marker), "criterion marker absent: " + marker);
  });
  note("STATIC NOT SHIPPED: element deltas for collaboration, import progress/cancel, slow-device degradation preview, explicit deck-JSON 413 message and compressed transport. These need browser/CI verification and are not claimed as runtime PASS here.");
});

note("FINDING (reported, not patched): the collaboration wire already sends an element-scoped JSON payload (03_canvas.js slide_update with element_id); the O(deck) cost is the server-side Drive read-modify-write, not the broadcast.");
