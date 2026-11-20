/**
 * Slides delivery mode — Offline Test Suite (#2432).
 * Run with: node botui/ui/suite/tests/webtest-slides-delivery.js
 *
 * The eight real shipped modules (51-58) are loaded into one VM sandbox with
 * stubbed window/document/gbT. Pure helpers are exercised at runtime; DOM-only
 * render paths are covered by labelled STATIC source checks in 10_delivery.js.
 * Dual-screen sync, speech captions, PDF export and media recording are not
 * claimed here — they remain CI/browser work for the issue.
 */
"use strict";

var harness = require("./webtest-slides-delivery/00_harness");

require("./webtest-slides-delivery/10_delivery");

harness.finish();
