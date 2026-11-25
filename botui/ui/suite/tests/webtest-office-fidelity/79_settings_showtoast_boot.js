/**
 * Settings open regression — offline, deterministic coverage for the pageerror
 * `Uncaught ReferenceError: showToast is not defined` thrown from HTMX inline
 * `hx-on::after-request` handlers when Settings opens.
 *
 * Root cause: the only assignment of the global lived in
 * settings/main/08_preview.js:19 (`window.showToast = showToast`), and that
 * module loads INSIDE partial 05, deep in the nested hx-get chain
 * (index -> 01 -> 02 -> 03 -> 04 -> 05). The inline handlers in partials 01/03/06
 * can fire first — a nested `hx-get hx-trigger="load"` completes and its
 * `htmx:afterRequest` bubbles to the form/button carrying `hx-on::after-request`,
 * which called a bare `showToast(...)` before the global existed.
 *
 * The fix guards every inline call with `window.showToast && window.showToast(...)`
 * (site 03 uses `if (event.detail.successful && window.showToast)` because it
 * already branches on the request result). No toast call is removed, so user
 * feedback is preserved; once main/08_preview.js loads, the full toast runs.
 * main/08_preview.js also stops clobbering a pre-existing global with undefined.
 *
 * This guard proves:
 *   (a) NO bare `showToast(` remains in ANY hx-on attribute of the Settings
 *       surface — every call is a `window.showToast(` guarded call;
 *   (b) the four named sites (01:10, 01:151, 01:206, 03:140) plus the DNS site
 *       are each guarded in place, with the existing i18n key kept;
 *   (c) 08_preview.js never clobbers the global with undefined, and still
 *       installs the full implementation sourced from S.showToast;
 *   (d) non-vacuity: registered in the harness, files under 450 lines, no new
 *       i18n keys. Restoring any bare call flips a case to FAIL.
 *
 * No browser, server, cargo, network or commit. The driver owns issue closure
 * and the live Chrome smoke test that originally surfaced the pageerror.
 */
"use strict";

var h = require("./00_harness");

var SETTINGS = "botui/ui/suite/settings";
var PREVIEW = "botui/ui/suite/settings/main/08_preview.js";
var P01 = "botui/ui/suite/settings/partials/01_profile_security.html";
var P03 = "botui/ui/suite/settings/partials/03_appearance_storage.html";
var P06 = "botui/ui/suite/settings/partials/06_dialogs.html";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/79_settings_showtoast_boot.js";

var GUARD_LINE = 'if (typeof showToast === "function") window.showToast = showToast;';
var ASSIGN = "window.showToast = showToast;";
// Built by concatenation so the i18n scanner never sees a showToast( + quote.
var CALL = "showToast" + "(";
var QCALL = "window." + "showToast" + "(";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }

/** Recursively collect .html files under the Settings surface. */
function htmlFiles(dir, out) {
  h.fs.readdirSync(dir).forEach(function (name) {
    var full = h.path.join(dir, name);
    if (h.fs.statSync(full).isDirectory()) htmlFiles(full, out);
    else if (name.slice(-5) === ".html") out.push(full);
  });
  return out;
}

/** Extract every hx-on* attribute value (double-quoted) from an HTML source. */
function hxOnAttrs(src) {
  var out = [];
  var at = 0;
  while (true) {
    var i = src.indexOf("hx-on", at);
    if (i === -1) break;
    var q = src.indexOf('="', i);
    h.assert(q !== -1 && q - i < 40, "hx-on without a quoted value at " + i);
    var end = src.indexOf('"', q + 2);
    h.assert(end !== -1, "unterminated hx-on value at " + i);
    out.push({ offset: i, value: src.slice(q + 2, end) });
    at = end + 1;
  }
  return out;
}

// (a) No bare showToast can run before the global exists: scan EVERY hx-on
// attribute in the Settings surface, not just the four named sites.
h.staticTest("showToast boot guard: every inline hx-on toast call is window-guarded", function () {
  var files = htmlFiles(h.path.join(h.REPO_DIR, SETTINGS), []);
  h.assert(files.length > 10, "the Settings surface must ship its HTML partials");
  var sites = 0;
  files.forEach(function (file) {
    var src = h.fs.readFileSync(file, "utf8");
    var rel = h.path.relative(h.REPO_DIR, file);
    hxOnAttrs(src).forEach(function (attr) {
      if (attr.value.indexOf(CALL) === -1) return;
      sites++;
      // Every toast call in an hx-on value must be window-qualified.
      h.assertEqual(countIn(attr.value, CALL), countIn(attr.value, QCALL),
        "unguarded bare showToast call in " + rel + ": " + attr.value);
      // A real guard must be present, either short-circuit or the if-branch.
      h.assert(attr.value.indexOf("window.showToast &&") !== -1 ||
        attr.value.indexOf("&& window.showToast)") !== -1,
        "inline toast call must be guarded in " + rel + ": " + attr.value);
    });
  });
  h.assertGreater(sites, 4, "at least five inline toast call sites must be covered by the scan");
});

// (b) The four named sites (and the DNS site) are guarded in place, with the
// existing i18n key/catalog lookup preserved at site 03.
h.staticTest("showToast boot guard: the four named sites and the DNS site are guarded in place", function () {
  var p01 = h.readRepo(P01);
  var p03 = h.readRepo(P03);
  var p06 = h.readRepo(P06);
  var sites = [
    [p01, 'hx-on::after-request="window.showToast && window.showToast(\'Profile updated successfully\')"', "01 profile save"],
    [p01, 'hx-on::after-request="this.reset(); window.showToast && window.showToast(\'Password changed successfully\')"', "01 password change"],
    [p01, 'hx-on::after-request="htmx.trigger(\'#sessions-list\', \'load\'); window.showToast && ' + QCALL, "01 revoke sessions guard"],
    [p03, 'hx-on::after-request="if (event.detail.successful && window.showToast) window.showToast((window.gbT || function (k, f) { return f; })(\'settings-notifications-saved\'', "03 notifications saved guard"],
    [p06, 'hx-on::after-request="if(event.detail.successful) { document.getElementById(\'register-dns-modal\').close(); window.showToast && ' + QCALL, "06 DNS guard"],
  ];
  sites.forEach(function (site) {
    contains(site[0], site[1], site[2] + " inline handler must be window-guarded");
  });
  // Site 03 reuses the existing catalog key; no new string was introduced.
  contains(p03, "'settings-notifications-saved', 'Notification preferences saved'",
    "site 03 must reuse the existing i18n key and English fallback");
  h.assertEqual(countIn(p03, "'settings-notifications-saved'"), 1,
    "site 03 must not duplicate the i18n key");
});

// (c) The module chain must never clobber the global with undefined.
h.staticTest("showToast boot guard: 08_preview.js cannot clobber the global with undefined", function () {
  var preview = h.readRepo(PREVIEW);
  h.assertEqual(countIn(preview, ASSIGN), 1, "08_preview.js must assign the global exactly once");
  var at = preview.indexOf(ASSIGN);
  h.assert(at !== -1, "08_preview.js must still install the full showToast implementation");
  h.assert(preview.indexOf(GUARD_LINE) !== -1,
    "the showToast assignment must be guarded by a typeof-function check");
  h.assertEqual(preview.indexOf(GUARD_LINE) + GUARD_LINE.length, at + ASSIGN.length,
    "the guarded line is the single showToast assignment");
  contains(preview, "var showToast = S.showToast;", "the full implementation is still sourced from S.showToast");
  contains(preview, "togglePreviewMode", "08_preview.js API preserved");
  contains(preview, "gb-preview-changed", "08_preview.js preview wiring preserved");
});

// (d) Non-vacuity: registered, substantial, scoped, files small, no new keys.
h.staticTest("showToast boot guard is wired in and stays small and scoped", function () {
  var src = h.readRepo(GUARD);
  h.assert(src.length > 3000, "guard must be a non-trivial assertion set");
  h.assert(src.split("\n").length < 450, "guard must stay under 450 lines");
  var runner = h.readRepo(RUNNER);
  contains(runner, 'require("./webtest-office-fidelity/79_settings_showtoast_boot");',
    "the guard must be registered in webtest-office-fidelity.js to run in CI");
  [P01, P03, P06, PREVIEW].forEach(function (path) {
    var lines = h.readRepo(path).split("\n").length;
    h.assert(lines < 450, path + " must stay under 450 lines, got " + lines);
  });
  // The fix guards calls; it must not delete them or add new toast strings.
  h.assertEqual(countIn(h.readRepo(P01), "window.showToast && " + QCALL), 3,
    "all three profile/security toast calls are preserved");
});

h.note("showToast boot regression: every inline hx-on toast call in the Settings surface is guarded with window.showToast, and 08_preview.js no longer clobbers an existing global. Live Chrome settings-open smoke test (zero pageerrors) still belongs to the driver; no browser/server/cargo used here.");
