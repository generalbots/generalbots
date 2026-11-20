/**
 * Offline verification for #2515 mobile office editing contract.
 * Run with: node botui/ui/suite/tests/webtest-ui-mobile-office.js
 *
 * The REAL shipped mobile IIFEs (docs/modules/19_mobile.js and
 * slides/modules/99_mobile.js) are loaded UNMODIFIED into a Node vm sandbox
 * with stubbed window/document/gbT/localStorage/PointerEvent/visualViewport,
 * a fixed clock and a deterministic uid. Every RUNTIME test executes shipped
 * code; no production logic is reimplemented here.
 *
 * Ordered modules under ./webtest-ui-mobile-office/:
 *   00_harness.js          counters, assertions, shared-vm loader, report
 *   01_dom.js              minimal DOM stub (classList.toggle, selectors, timers)
 *   02_fixture.js          deterministic phone/desktop sandbox factory
 *   10_docs_runtime.js     drawer, caret visibility, tap-to-place-caret
 *   11_slides_runtime.js   drawer, fit-to-width, pinch/pan gesture guards
 *   12_desktop_gating.js   coarse/width gating, graceful degradation
 *   20_static_gaps.js      [STATIC] source assertions + late-swap REGRESSION
 *
 * [STATIC] tests read shipped source only: they document CSS/media-query and
 * touch-target facts, never a fabricated runtime PASS. The REGRESSION executes
 * shipped code with no canvas host at load time and proves the fixed module
 * fits and pans after the real server fragment shape is swapped in.
 */
"use strict";

var harness = require("./webtest-ui-mobile-office/00_harness");

require("./webtest-ui-mobile-office/10_docs_runtime");
require("./webtest-ui-mobile-office/11_slides_runtime");
require("./webtest-ui-mobile-office/12_desktop_gating");
require("./webtest-ui-mobile-office/20_static_gaps");

harness.finish();
