/**
 * #2529 render scope -- [BASELINE-OBSERVE].
 *
 * The presentation view is ALREADY single-slide; the thumbnail rail and the
 * sorter panel still render every slide server-side into one fragment. Both
 * facts are locked so neither a false "full deck render" claim nor a silent
 * regression of the already-windowed view can pass unnoticed.
 */
"use strict";

var h = require("./00_harness");
var s = require("./01_sources");

var test = h.test;
var assert = h.assert;

test("[BASELINE-OBSERVE] #2529: presentation view renders exactly one requested slide", function () {
  var src = s.rust("sidebars");
  assert(s.has(src, "pub fn render_presentation_view(pres: &Presentation, requested: usize)"),
    "view renderer takes a requested slide index");
  assert(s.has(src, "pres.slides.get(current)"), "only one slide is selected");
  assert(s.has(src, "current_slide.map(render_slide)"), "only the selected slide is rendered");
  assert(!s.has(src, "for (index, slide) in pres.slides.iter()"), "no all-slide loop in the view renderer");
});

test("[BASELINE-OBSERVE] #2529: thumbnail rail still renders every slide into one fragment", function () {
  var src = s.rust("getBridge");
  assert(s.has(src, "fn thumbnails_html(pres: &Presentation)"), "thumbnail fragment builder exists");
  assert(s.has(src, "for (index, slide) in pres.slides.iter().enumerate()"), "loop covers all slides");
  assert(s.has(src, "html.push_str(&format!("), "HTML for each slide is appended to one string");
  assert(s.has(src, "Html<String>"), "a single HTML string is returned");
});

test("[BASELINE-OBSERVE] #2529: sorter panel also iterates the full slide list", function () {
  var src = s.rust("panels");
  assert(s.has(src, "for (i, slide) in slides.iter().enumerate()"), "sorter renders all slides");
});

test("[BASELINE-OBSERVE] #2529: an edit swaps one slide canvas plus the whole thumbnail rail", function () {
  var src = s.readModule("04_events.js");
  assert(s.has(src, "window.refreshSlideView = function (slideIndex)"), "refresh entry point exists");
  assert(s.has(src, "\"&slide=\" + encodeURIComponent(index)"), "canvas request is slide-scoped");
  assert(s.has(src, "/suite/slides/fragments/slide-thumbnails?id="), "full rail is re-requested");
  assert(s.count(src, "swap: \"innerHTML\"") >= 2, "both fragments are innerHTML swaps");
});

test("[BASELINE-OBSERVE] #2529: no lazy-loading or windowing marker is present in shipped modules", function () {
  var js = s.allJs();
  ["IntersectionObserver", "loading=\"lazy\"", "virtualiz", "windowed", "progressive-render"].forEach(function (marker) {
    assert(!s.has(js, marker), "no shipped windowing marker " + marker);
  });
});
