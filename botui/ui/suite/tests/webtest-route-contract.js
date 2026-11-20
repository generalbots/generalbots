/**
 * Offline static audit for #2546 — route contract truthfulness.
 * Run with: node botui/ui/suite/tests/webtest-route-contract.js
 *
 * Scans shipped botserver Rust router registrations, the LLM endpoint inventory
 * and the api command catalog. [BASELINE-OBSERVE] locks today's tree, [STATIC]
 * labels criteria this offline process cannot prove, and [FINDING] reports real
 * bugs with file:line evidence. No router is built, no HTTP request is made and
 * no runtime/browser/CI PASS is fabricated.
 *
 * Ordered modules under ./webtest-route-contract/:
 *   00_harness.js         scan helpers, cached reads, report
 *   10_observe_surface.js [BASELINE-OBSERVE] routers, inventory, catalog status
 *   11_observe_stubs.js   [BASELINE-OBSERVE] 501/todo/tombstone handlers
 *   12_observe_contract.js [BASELINE-OBSERVE] gaps, duplicates, families
 *   20_criteria.js        [STATIC] acceptance criteria and deferred runtime gates
 */
"use strict";

var harness = require("./webtest-route-contract/00_harness");

require("./webtest-route-contract/10_observe_surface");
require("./webtest-route-contract/11_observe_stubs");
require("./webtest-route-contract/12_observe_contract");
require("./webtest-route-contract/20_criteria");

harness.finish();
