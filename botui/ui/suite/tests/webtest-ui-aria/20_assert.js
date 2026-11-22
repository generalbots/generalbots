/**
 * Suite UI a11y static audit — characterization locks + criteria (#2331).
 * Run with: node botui/ui/suite/tests/webtest-ui-aria.js
 *
 * [BASELINE-OBSERVE] locks the numbers measured on the current tree. It is a
 * characterization, not a ratchet: no baseline/guard file is read or written.
 * A production change must update these constants deliberately, in review.
 *
 * [STATIC] checks the announcement plumbing that the shipped sources really
 * contain. Remaining acceptance criteria are registered with `gap()` as
 * [STATIC][GAP] lines: they are documented, never claimed as PASS.
 */
"use strict";

var h = require("./00_harness");
var metrics = require("./10_scan").metrics;

var EXPECTED = {
  filesScanned: 1565,
  htmlFiles: 214,
  jsFiles: 1351,
  vendorFiles: 290,
  topSegments: 94,
  ariaLiveMentions: 708,
  ariaLiveFiles: 242,
  ariaLiveDeclarations: 663,
  livePolite: 593,
  liveAssertive: 69,
  liveOff: 1,
  roleStatus: 530,
  roleAlert: 106,
  roleLog: 12,
  roleStatusAlertFiles: 243,
  dataI18nAttrs: 5845,
  htmlWithI18n: 203,
  gbTCalls: 377, // +10: #2511 localized the translate modal's 10 raw labels via gbT
  jsWithGbt: 121,
  dynamicWrites: 4229,
  writesInRegionFiles: 1210,
  writesOutsideRegionFiles: 3019,
  filesWithWritesNoRegion: 605,
};

Object.keys(EXPECTED).forEach(function (key) {
  var locked = EXPECTED[key];
  h.test("[BASELINE-OBSERVE] " + key + " == " + locked, function () {
    h.assert(metrics[key] === locked,
      key + " is " + metrics[key] + ", characterized as " + locked);
  });
});

h.test("[BASELINE-OBSERVE] live-region declarations did not decrease", function () {
  h.assert(metrics.ariaLiveDeclarations >= EXPECTED.ariaLiveDeclarations,
    "live-region declarations fell from " + EXPECTED.ariaLiveDeclarations +
    " to " + metrics.ariaLiveDeclarations);
});

h.test("[STATIC] GBToasts declares polite and assertive announcers", function () {
  var src = h.readSource("js/gb-toasts.js");
  h.assert(src.indexOf("gb-toast-live-polite") !== -1, "no polite announcer id");
  h.assert(src.indexOf("gb-toast-live-assertive") !== -1, "no assertive announcer id");
  h.assert(src.indexOf('setAttribute("role", "status")') !== -1, "no status role");
  h.assert(src.indexOf('setAttribute("role", "alert")') !== -1, "no alert role");
});

h.test("[STATIC] app-lifecycle declares the generic polite status region", function () {
  var src = h.readSource("js/app-lifecycle.js");
  h.assert(src.indexOf("gb-sr-live") !== -1, "no generic status region id");
  h.assert(src.indexOf('setAttribute("role", "status")') !== -1, "no status role");
  h.assert(src.indexOf('setAttribute("aria-live", "polite")') !== -1, "no polite live value");
});

h.test("[STATIC] chat stream has a one-shot announcement hook", function () {
  var src = h.readSource("chat/messages/70_stream.js");
  var calls = src.split("gbChatAnnounce(").length - 1;
  h.assert(calls >= 3, "expected the announce hook and its call sites, saw " + calls);
  h.assert(src.indexOf("chat-response-ready") !== -1, "no completion announcement key");
  h.assert(src.indexOf("chat-response-started") !== -1, "no start announcement key");
});

h.test("[STATIC] collab announcement infrastructure and call sites are shipped", function () {
  var helper = h.readSource("js/collab-a11y.js");
  h.assert(helper.indexOf("gb-aria-live") !== -1, "no collab live region id");
  h.assert(helper.indexOf("GBCollabA11y") !== -1, "no collab announce helper");
  var docs = h.readSource("docs/modules/07_presence.js");
  var slides = h.readSource("slides/modules/05_presence.js");
  h.assert(docs.indexOf("GBCollabA11y.announce") !== -1, "no docs presence announcement");
  h.assert(slides.indexOf("GBCollabA11y.announce") !== -1, "no slides presence announcement");
});

h.test("[OBSERVE] every characterized metric is a finite number", function () {
  Object.keys(EXPECTED).forEach(function (key) {
    h.assert(Number.isFinite(metrics[key]), key + " is not a finite number");
  });
});

h.gap("AT-SPOTCHECK",
  "NVDA/VoiceOver spot-check for chat, toasts and a collab presence event",
  "requires a real screen reader; the offline static harness cannot run one");

h.gap("PER-APP-OUTCOMES",
  "save/upload/send success and failure announced in every active .product app",
  "needs browser UAT per active app; async outcomes are runtime events");

var chatLogDeclared = h.readTree().some(function (file) {
  var t = file.text;
  return t.indexOf('id="messages"') !== -1 &&
    /\brole\s*=\s*(["'])log\1/.test(t) &&
    /\baria-live\s*=\s*(["'])polite\1/.test(t);
});
if (!chatLogDeclared) {
  h.gap("CHAT-LOG-REGION",
    "chat #messages role=log with aria-live=polite declared in shipped sources",
    "no static declaration found in the scanned tree; issue comments cite CDP AX-tree evidence only, so re-verify in the browser");
} else {
  h.test("[STATIC] chat #messages declares role=log with aria-live=polite", function () {
    h.assert(chatLogDeclared, "chat messages region missing role=log/aria-live=polite declaration");
  });
}

h.gap("PRESENCE-RUNTIME",
  "collab join/leave/edit announcements observed on a live session",
  "static helper and call sites exist; delivery and throttling need browser/AT verification");

h.gap("UNANNOUNCED-WRITES-PROXY",
  "dynamic writes outside a file that declares an announced region: " +
    metrics.writesOutsideRegionFiles + " of " + metrics.dynamicWrites,
  "file-level upper bound: a write may be announced by a cross-file helper such as GBToasts; not proof of silence");
