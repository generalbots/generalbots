/**
 * Mobile chat offline harness for issue #2516 — durable, browser-free evidence.
 * Run with: node botui/ui/suite/tests/webtest-ui-chat-mobile.js
 *
 * ONE Node vm sandbox loads the REAL shipped chat message modules:
 *   chat/messages/10_scroll.js       (reader intent, stick-to-bottom, markers)
 *   chat/messages/15_virtualize.js   (windowing, height-preserving placeholders)
 *   chat/messages/20_persist.js      (local transcript persistence)
 *   chat/messages/70_stream.js       (updateStreaming / finalizeStreaming)
 * against a purpose-built minimal DOM with stubbed window/document/gbT/
 * localStorage/fixed clock/fixed uid. Every RUNTIME assertion calls shipped
 * code; nothing about virtualization is reimplemented here.
 *
 * Criteria that are CSS/HTML/device-bound are labelled [STATIC] source
 * assertions — never a fabricated runtime PASS. Observed defects are labelled
 * KNOWN BUG and reported only; no production file is patched.
 *
 * Ordered modules under ./webtest-ui-chat-mobile/:
 *   00_harness.js     sandbox, stubs, counters, report
 *   05_dom.js         minimal DOM + module loader
 *   10_virtualize.js  RUNTIME windowing/placeholder/pin/scroll-restore proofs
 *   20_scroll.js      RUNTIME scroll helpers + #2014 intent + marker integration
 *   30_invariants.js  RUNTIME persistence/streaming + [STATIC] + KNOWN BUGs
 */
"use strict";

var h = require("./webtest-ui-chat-mobile/00_harness");

require("./webtest-ui-chat-mobile/05_dom");
require("./webtest-ui-chat-mobile/10_virtualize");
require("./webtest-ui-chat-mobile/20_scroll");
require("./webtest-ui-chat-mobile/30_invariants");

h.finish();
