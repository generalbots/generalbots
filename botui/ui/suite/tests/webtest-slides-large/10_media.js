/**
 * #2529 media storage characterisation -- [BASELINE-OBSERVE].
 *
 * Every assertion executes shipped-source scans and locks the OBSERVED state.
 * A change to any of these facts is a regression in the measured baseline and
 * must be acknowledged by updating this harness, not by editing a guard.
 */
"use strict";

var h = require("./00_harness");
var s = require("./01_sources");

var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;

test("[BASELINE-OBSERVE] #2529: imported picture bytes become a base64 data URL inside the deck element", function () {
  var src = s.rust("import");
  assert(s.has(src, "data:{mime};base64,"), "import builds a data: URL from media bytes");
  assert(s.has(src, "base64::engine::general_purpose::STANDARD.encode(&bytes)"),
    "base64 engine encodes the raw zip media bytes");
  assert(s.has(src, "element_type: \"image\".to_string()"), "media is an element of the deck model");
  assert(s.has(src, "parse_pic_element("), "picture parsing is the media entry point");
});

test("[BASELINE-OBSERVE] #2529: the media import path uploads nothing and stores no object reference", function () {
  var src = s.rust("import");
  ["upload", "put_object", "media_url", "object_ref", "source_path: Some"].forEach(function (marker) {
    assert(!s.has(src, marker), "import.rs has no " + marker + " marker");
  });
  // The only "Drive" mention is a comment; there is no Drive call in the path.
  var driveMentions = s.count(src, "Drive");
  assert(driveMentions <= 1, "no Drive write calls in import.rs (comments only)");
});

test("[BASELINE-OBSERVE] #2529: load routes return the whole serialized deck including inline media", function () {
  var routes = s.rust("routes");
  var storage = s.rust("storage");
  assert(s.has(routes, "/api/slides/load"), "full-deck load route exists");
  assert(s.has(routes, "/api/slides/:id"), "by-id load route exists");
  assert(s.has(storage, "serde_json::from_slice(&bytes)"), "whole JSON body is parsed into Presentation");
  assert(s.has(storage, "let presentation: Presentation ="), "load yields the full Presentation model");
});

test("[BASELINE-OBSERVE] #2529: Drive persistence writes one whole pretty-printed deck object", function () {
  var storage = s.rust("storage");
  assert(s.has(storage, "serde_json::to_string_pretty(presentation)"), "full deck serialized on save");
  assert(s.has(storage, ".put_object(\"gbo\", &path, content.into_bytes()"), "one object write per save");
  assert(s.has(storage, "{}/{}.json"), "deck stored as a single JSON object per presentation");
  equal(s.count(storage, "put_object(") >= 2, true, "media and deck each use whole-object writes");
});
