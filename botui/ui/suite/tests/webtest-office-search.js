/**
 * Offline verification for #2505 — unified office search and discovery.
 * Run with: node botui/ui/suite/tests/webtest-office-search.js
 *
 * The REAL shipped browser modules (js/command-palette.js and js/gb-search.js)
 * are read verbatim and executed UNMODIFIED in one Node vm sandbox per fixture.
 * window/document/gbT/localStorage/fetch/timers are stubbed and the clock is
 * fixed; every RUNTIME and KNOWN BUG test therefore executes production code.
 * No production logic is reimplemented here.
 *
 * Ordered modules under ./webtest-office-search/:
 *   00_harness.js        counters, assertions, source readers, vm loader, report
 *   01_dom.js            minimal DOM stub with innerHTML parsing and events
 *   02_sandbox.js        SyncThenable chain + deterministic shipped-code fixture
 *   10_command_palette.js  real palette: Ctrl+K, ranking, deep links, keyboard
 *   11_universal_search.js real search bar: debounce, API, deep link, 401
 *   20_static_gaps.js    labelled [STATIC] gaps + KNOWN BUG (reported only)
 *
 * [STATIC] tests read shipped source/catalogs only. They document the many
 * #2505 criteria that are NOT shipped and never claim a fabricated runtime PASS.
 * KNOWN BUG tests execute the shipped code and assert the observed defect; the
 * fix belongs to production, not to this harness.
 */
"use strict";

var harness = require("./webtest-office-search/00_harness");

require("./webtest-office-search/10_command_palette");
require("./webtest-office-search/11_universal_search");
require("./webtest-office-search/20_static_gaps");

harness.finish();
