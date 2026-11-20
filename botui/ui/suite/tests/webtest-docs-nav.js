/**
 * Offline verification for #2485 docs navigation pane, bookmarks, quick parts
 * and autotext (the NAV slice; find/replace is covered by webtest-docs-find).
 * Run with: node botui/ui/suite/tests/webtest-docs-nav.js
 *
 * The suite lives in ordered modules under ./webtest-docs-nav/:
 *   00_harness.js       counters, assertions, FakeNode/FakeDOM/vm loader, report
 *   10_nav_runtime.js   runtime invariants of the real shipped 19/20/22/23/24 IIFEs
 *   11_nav_deferred.js  more runtime, labelled [STATIC] and KNOWN BUG checks
 *
 * Every production module is loaded as its real, unmodified IIFE into ONE
 * shared Node vm sandbox with stubbed window/document/gbT/storage/clock/uid.
 * No production logic is reimplemented here: fixtures only feed the shipped
 * helpers deterministic inputs. DOM sentinels are code-like tokens (angle
 * brackets), never user-facing English literals (i18n gate #2096).
 */
"use strict";

var harness = require("./webtest-docs-nav/00_harness");

require("./webtest-docs-nav/10_nav_runtime");
require("./webtest-docs-nav/11_nav_deferred");

harness.finish();
