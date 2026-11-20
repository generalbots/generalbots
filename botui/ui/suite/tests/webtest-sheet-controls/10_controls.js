/**
 * Sheets enterprise controls — pure-helper invariant tests (#2437).
 * Runtime checks call the exported helpers of the shipped IIFE modules loaded
 * into the sandbox. Source-bound behaviour is labelled STATIC and never
 * presented as a runtime result. Production logic is not modified here.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var c = h.loadControls();

function reset() { c.reset(); }
function unique(list) { return list.filter(function (v, i) { return list.indexOf(v) === i; }); }

var ACL_VISIBLE = "sheet-ctrl-acl-visible";
var ACL_INVISIBLE = "sheet-ctrl-acl-invisible";
var ACL_NOMATCH = "sheet-ctrl-acl-nomatch";
var EXPORT_ALLOWED = "sheet-ctrl-class-export-allowed";
var EXPORT_DENIED = "sheet-ctrl-class-export-denied";
var EXPORT_APPROVAL = "sheet-ctrl-class-export-approval";
var SNAP_MISSING = "sheet-ctrl-snap-missing";
var SNAP_MATCH = "sheet-ctrl-snap-match";
var SNAP_TAMPERED = "sheet-ctrl-snap-tampered";

// ── API surface ───────────────────────────────────────────────────────
test("all eight shipped control namespaces are exported", function () {
  [
    ["core", c.core], ["protection", c.protection], ["inputs", c.inputs],
    ["acl", c.acl], ["classification", c.classification], ["links", c.links],
    ["audit", c.audit], ["snapshot", c.snapshot],
  ].forEach(function (entry) {
    assert(entry[1] && typeof entry[1] === "object", entry[0] + " namespace missing");
  });
});

test("each feature module exposes its documented pure helpers", function () {
  var surface = {
    protection: ["inRange", "isAllowed", "protect", "unprotect", "setAllowance", "statusText"],
    inputs: ["validRef", "validate", "mark", "apply", "remove", "highlight"],
    acl: ["roles", "add", "preview", "remove"],
    classification: ["levelLabel", "bannerText", "watermarkText", "exportStatusKey", "canExport", "set"],
    links: ["normalizeHost", "validHost", "addHost", "isApproved", "addLink", "refreshState"],
    audit: ["recordEdit", "recordEvent", "prune", "setRetention", "clear", "visible"],
    snapshot: ["payload", "has", "create", "verify", "statusKey"],
  };
  Object.keys(surface).forEach(function (module) {
    surface[module].forEach(function (name) {
      assert(typeof c[module][name] === "function", module + "." + name + " is not a function");
    });
  });
});

// ── Core state + hash ─────────────────────────────────────────────────
test("empty state carries version, local-only marker and one-year audit default", function () {
  reset();
  var s = c.core.emptyState();
  assert(s.version === 1 && s.localOnly === true, "empty state metadata wrong");
  assert(s.audit.retentionDays === 365, "default retention is not one year");
  assert(s.classification.level === "internal" && s.classification.exportPolicy === "allow",
    "default classification wrong");
  assert(s.snapshot === null, "default snapshot should be empty");
});

test("local preview hash is deterministic, 8-hex and input-sensitive", function () {
  var a = c.core.hash("payroll");
  assert(a === c.core.hash("payroll"), "hash not deterministic");
  assert(a !== c.core.hash("payrolL"), "hash did not change with input");
  assert(/^[0-9a-f]{8}$/.test(a), "hash is not 8 lowercase hex digits");
  assert(c.core.hash(null) === c.core.hash(""), "null did not normalize to empty string");
});

test("storage key is workbook-scoped under the controls prefix", function () {
  assert(c.core.storageKey().indexOf("gb-sheet-controls:") === 0, "wrong storage key prefix");
});

// ── Protection semantics ──────────────────────────────────────────────
test("default protection allows selection but blocks editing allowances", function () {
  reset();
  var a = c.core.emptyState().protection.allowances;
  assert(a.selectLocked === true && a.selectUnlocked === true, "select allowances should be on");
  ["format", "insertDeleteRows", "sort", "filter", "usePivot"].forEach(function (key) {
    assert(a[key] === false, key + " should be off by default");
  });
});

test("inRange resolves A1 bounds, reversed bounds and sheet-qualified refs", function () {
  assert(c.protection.inRange("A5", "A1:C10") === true, "inside cell rejected");
  assert(c.protection.inRange("D5", "A1:C10") === false, "outside cell accepted");
  assert(c.protection.inRange("B2", "C10:A1") === true, "reversed range bounds not normalized");
  assert(c.protection.inRange("Sheet2!B2", "Sheet1!A1:C10") === true, "sheet prefix not stripped");
  assert(c.protection.inRange("A5", "A1") === false, "single-cell range should not match");
  assert(c.protection.inRange("A5", "not-a-range") === false, "malformed range accepted");
});

test("protect requires a password and persists scope, target and hash", function () {
  reset();
  var missing = c.protection.protect("range", "A1:C10", "");
  assert(missing.ok === false && missing.key === "sheet-ctrl-prot-need-password",
    "empty password was accepted");
  var result = c.protection.protect("range", "A1:C10", "pw");
  assert(result.ok === true, "protect failed");
  var p = c.core.state().protection;
  assert(p.locked === true && p.scope === "range" && p.target === "A1:C10", "protection not persisted");
  assert(p.hash === c.core.hash("pw"), "password hash not stored");
});

test("unprotect rejects a wrong password and clears state on the right one", function () {
  reset();
  c.protection.protect("sheet", "", "pw");
  assert(c.protection.unprotect("nope") === false, "wrong password unlocked protection");
  assert(c.core.state().protection.locked === true, "wrong password mutated lock state");
  assert(c.protection.unprotect("pw") === true, "correct password did not unlock");
  var p = c.core.state().protection;
  assert(p.locked === false && p.hash === "", "unprotect did not clear lock and hash");
});

test("locked range gates every toolbar action by its allowance", function () {
  reset();
  c.protection.protect("range", "A1:C10", "pw");
  ["format", "insertRows", "deleteRows", "sort", "filter", "pivot"].forEach(function (action) {
    assert(c.protection.isAllowed(action) === false, action + " was allowed while locked");
  });
  assert(c.protection.isAllowed("selectLocked") === true, "locked-cell selection was blocked");
  assert(c.protection.isAllowed("selectUnlocked") === true, "unlocked-cell selection was blocked");
  assert(c.protection.setAllowance("sort", true) === true, "setAllowance(sort) failed");
  assert(c.protection.isAllowed("sort") === true, "sort allowance did not enable sorting");
  assert(c.protection.isAllowed("filter") === false, "enabling sort leaked into filter");
});

test("unknown allowance keys are rejected and status text reflects lock state", function () {
  reset();
  assert(c.protection.setAllowance("bogus", true) === false, "unknown allowance accepted");
  var open = c.protection.statusText();
  c.protection.protect("sheet", "", "pw");
  var locked = c.protection.statusText();
  assert(open !== locked, "status text did not change on protect");
  assert(typeof locked === "string" && locked.length > 0, "status text empty");
});

// ── Unlocked input ranges ─────────────────────────────────────────────
test("validRef accepts absolute and bounded ranges only", function () {
  assert(c.inputs.validRef("A1") === true, "A1 rejected");
  assert(c.inputs.validRef("$A$1") === true, "absolute ref rejected");
  assert(c.inputs.validRef("A1:B2") === true, "bounded range rejected");
  assert(c.inputs.validRef("A1:B2:C3") === false, "over-long range accepted");
  assert(c.inputs.validRef("not-a-cell") === false, "garbage ref accepted");
  assert(c.inputs.validRef("") === false, "empty ref accepted");
});

test("validate enforces number, email, date and required rules", function () {
  assert(c.inputs.validate("number", "123") === true, "integer rejected");
  assert(c.inputs.validate("number", "1,234") === true, "grouped number rejected");
  assert(c.inputs.validate("number", "abc") === false, "non-number accepted");
  assert(c.inputs.validate("number", "") === false, "empty number accepted");
  assert(c.inputs.validate("email", "a@example.com") === true, "email rejected");
  assert(c.inputs.validate("email", "a.example.com") === false, "bad email accepted");
  assert(c.inputs.validate("date", "2026-01-15") === true, "ISO date rejected");
  assert(c.inputs.validate("date", "garbage") === false, "bad date accepted");
  assert(c.inputs.validate("required", "") === false, "empty required value accepted");
  assert(c.inputs.validate("any", "") === true, "any rule rejected a value");
});

test("mark normalizes refs, rejects invalid refs and never duplicates", function () {
  reset();
  assert(c.inputs.mark("bad ref", "number") === false, "invalid ref marked");
  assert(c.inputs.mark(" b2 ", "number") === true, "valid ref not marked");
  c.inputs.mark("B2", "email");
  var entries = c.core.state().inputs;
  assert(entries.length === 1 && entries[0].ref === "B2" && entries[0].rule === "email",
    "re-mark did not replace the entry");
});

test("apply maps an invalid rule to an error and records a valid edit in audit", function () {
  reset();
  assert(c.inputs.apply("Z9", "1").key === "sheet-ctrl-input-needs", "unmarked ref accepted");
  c.inputs.mark("b2", "number");
  assert(c.inputs.apply("B2", "abc").ok === false, "invalid value accepted");
  var ok = c.inputs.apply("B2", "42");
  assert(ok.ok === true && ok.ref === "B2" && ok.rule === "number", "valid value rejected");
  var entries = c.audit.visible();
  assert(entries.length === 1 && entries[0].ref === "B2" && entries[0].after === "42",
    "cross-module audit entry missing");
});

test("highlight toggles the marker class on DOM cells and remove undoes it", function () {
  reset();
  c.inputs.mark("B2", "number");
  var original = c.document.querySelectorAll;
  function cell(ref, className) {
    return {
      getAttribute: function (name) { return name === "data-ref" ? ref : null; },
      className: className,
    };
  }
  var target = cell("b2", "");
  var other = cell("A1", "grid-cell");
  c.document.querySelectorAll = function () { return [target, other]; };
  try {
    var added = c.inputs.highlight();
    assert(added === 1, "expected one class addition");
    assert(target.className.indexOf("ss-ctrl-input-cell") !== -1, "marker class not added");
    assert(other.className === "grid-cell", "unrelated cell was modified");
    c.inputs.remove("B2");
    assert(target.className.indexOf("ss-ctrl-input-cell") === -1, "marker class not removed");
  } finally {
    c.document.querySelectorAll = original;
  }
});

// ── Range-level ACL ───────────────────────────────────────────────────
test("roles splits and trims comma/semicolon lists", function () {
  var parsed = c.acl.roles(" admin , finance; auditor ");
  assert(parsed.join(",") === "admin,finance,auditor", "roles parsed incorrectly");
  assert(c.acl.roles("").length === 0, "empty role list produced entries");
});

test("ACL grants readable roles and denies hidden roles for a range", function () {
  reset();
  assert(c.acl.add(" payroll ", "finance", "intern") === true, "add failed");
  assert(c.acl.preview("payroll", "finance") === ACL_VISIBLE, "readable role not granted");
  assert(c.acl.preview("payroll", "intern") === ACL_INVISIBLE, "hidden role not denied");
  assert(c.acl.preview("payroll", "sales") === ACL_NOMATCH, "unlisted role should not match");
  assert(c.acl.preview("unknown", "finance") === ACL_NOMATCH, "unknown range should not match");
});

test("hidden role wins when listed in both ACL columns", function () {
  reset();
  c.acl.add("payroll", "finance,intern", "intern");
  assert(c.acl.preview("payroll", "intern") === ACL_INVISIBLE, "deny did not take precedence");
  // add()/preview() normalize ranges to uppercase; remove() compares its raw
  // argument, so callers must pass the stored (uppercase) range form.
  c.acl.remove("PAYROLL");
  assert(c.acl.preview("payroll", "finance") === ACL_NOMATCH, "remove did not drop the rule");
});

test("ACL remove normalization asymmetry is observable (shipped behaviour)", function () {
  reset();
  c.acl.add("payroll", "finance", "");
  c.acl.remove("payroll");
  assert(c.acl.preview("payroll", "finance") === ACL_VISIBLE,
    "observed: lowercase remove leaves the uppercase rule in place (#2437 bug)");
});

// ── Classification / export policy ────────────────────────────────────
test("classification set validates level and export policy", function () {
  reset();
  assert(c.classification.set("bogus", "allow") === false, "invalid level accepted");
  assert(c.classification.set("confidential", "bogus") === false, "invalid policy accepted");
  assert(c.classification.set("confidential", "deny") === true, "valid classification rejected");
  assert(c.core.state().classification.level === "confidential", "level not persisted");
});

test("export gate follows the policy key, not the classification level", function () {
  reset();
  c.classification.set("confidential", "deny");
  assert(c.classification.exportStatusKey() === EXPORT_DENIED, "deny policy not reported");
  assert(c.classification.canExport() === false, "deny policy still exported");
  c.classification.set("confidential", "approve");
  assert(c.classification.exportStatusKey() === EXPORT_APPROVAL, "approval policy not reported");
  assert(c.classification.canExport() === false, "approval policy auto-exported");
  c.classification.set("public", "allow");
  assert(c.classification.exportStatusKey() === EXPORT_ALLOWED, "allow policy not reported");
  assert(c.classification.canExport() === true, "allow policy blocked export");
});

test("banner and watermark react to the active classification level", function () {
  reset();
  c.classification.set("public", "allow");
  var publicBanner = c.classification.bannerText();
  var publicWatermark = c.classification.watermarkText();
  var publicLabel = c.classification.levelLabel();
  c.classification.set("restricted", "allow");
  assert(c.classification.bannerText() !== publicBanner, "banner ignored the level");
  assert(c.classification.watermarkText() !== publicWatermark, "watermark ignored the level");
  assert(c.classification.levelLabel() !== publicLabel, "level label did not change");
});

// ── External-link governance ──────────────────────────────────────────
test("normalizeHost strips scheme, port, path and case", function () {
  assert(c.links.normalizeHost(" HTTPS://Data.Example.COM:8443/path?q=1 ") === "data.example.com",
    "host normalization wrong");
  assert(c.links.normalizeHost("example.com/") === "example.com", "trailing slash not stripped");
});

test("validHost rejects localhost, malformed and dot-edge hosts", function () {
  assert(c.links.validHost("data.example.com") === true, "valid host rejected");
  assert(c.links.validHost("localhost") === false, "localhost accepted");
  assert(c.links.validHost("a..example.com") === false, "double-dot host accepted");
  assert(c.links.validHost(".example.com") === false, "leading-dot host accepted");
  assert(c.links.validHost("bad") === false, "single-label host accepted");
});

test("addLink marks approval from the allowlist and rejects invalid hosts", function () {
  reset();
  assert(c.links.addLink("https://bad/p").ok === false, "invalid link host accepted");
  var blocked = c.links.addLink("https://evil.example.com/feed");
  assert(blocked.ok === true && blocked.approved === false, "unapproved link not blocked");
  assert(c.links.addHost("data.example.com") === true, "allowlist add failed");
  assert(c.links.isApproved("https://data.example.com:443/x") === true, "approval check failed");
  var ok = c.links.addLink("https://data.example.com/x");
  assert(ok.ok === true && ok.approved === true, "approved link not recognized");
  assert(c.links.addHost("DATA.EXAMPLE.COM") === true, "host normalization on add failed");
  assert(c.core.state().links.allowlist.length === 1, "duplicate host was added twice");
});

test("removing a host revokes previously approved links (fail closed)", function () {
  reset();
  c.links.addHost("data.example.com");
  c.links.addLink("https://data.example.com/x");
  assert(c.links.removeHost("data.example.com") === true, "removeHost failed");
  assert(c.core.state().links.current[0].approved === false, "existing link stayed approved");
  assert(c.links.isApproved("data.example.com") === false, "host remained on allowlist");
});

// ── Audit trail ───────────────────────────────────────────────────────
test("recordEdit captures who, when, before/after and formula flag", function () {
  reset();
  assert(c.audit.recordEdit("A1", "old", "new", true, "alice") === true, "recordEdit failed");
  var entry = c.audit.visible()[0];
  assert(entry.kind === "edit" && entry.ref === "A1", "edit identity wrong");
  assert(entry.before === "old" && entry.after === "new", "before/after not captured");
  assert(entry.who === "alice" && entry.formulaChanged === true, "who/formula flag wrong");
  assert(typeof entry.when === "string" && entry.id.length > 0, "when/id missing");
});

test("recordEvent stores a detail key and visible() filters by cell", function () {
  reset();
  c.audit.recordEdit("A1", "old", "new", false, "alice");
  c.audit.recordEvent("export", "sheet-ctrl-class-export");
  assert(c.audit.visible().length === 2, "both entries should be visible");
  var exportEntry = c.audit.visible().filter(function (e) { return e.kind === "export"; })[0];
  assert(exportEntry && exportEntry.detailKey === "sheet-ctrl-class-export", "export event not recorded");
  var filtered = c.audit.visible("a1");
  assert(filtered.length === 1 && filtered[0].ref === "A1", "ref filter failed");
});

test("setRetention validates input and prune drops entries past the window", function () {
  reset();
  assert(c.audit.setRetention(0) === false, "zero retention accepted");
  assert(c.audit.setRetention(365) === true, "valid retention rejected");
  var old = new Date(Date.now() - 400 * 86400000).toISOString();
  var recent = new Date(Date.now() - 10 * 86400000).toISOString();
  c.seed(function (state) {
    state.audit.entries = [
      { id: "old", kind: "edit", when: old, ref: "OLD1" },
      { id: "recent", kind: "edit", when: recent, ref: "NEW1" },
    ];
  });
  assert(c.audit.prune() === 1, "expected one pruned entry");
  var refs = c.audit.visible().map(function (e) { return e.ref; });
  assert(refs.indexOf("OLD1") === -1 && refs.indexOf("NEW1") !== -1, "prune kept the wrong entry");
});

test("clear empties the local trail", function () {
  reset();
  c.audit.recordEdit("A1", "", "1", false, "alice");
  assert(c.audit.clear() === true && c.audit.visible().length === 0, "clear did not empty the trail");
});

// ── Snapshot preview ──────────────────────────────────────────────────
test("snapshot create produces a verifiable, deterministic local hash", function () {
  reset();
  assert(c.snapshot.has() === false, "snapshot present before create");
  assert(c.snapshot.verify() === false, "empty snapshot verified");
  assert(c.snapshot.statusKey() === SNAP_MISSING, "missing status key wrong");
  var hash = c.snapshot.create();
  assert(/^[0-9a-f]{8}$/.test(hash), "snapshot hash format wrong");
  assert(c.snapshot.has() === true && c.snapshot.verify() === true, "created snapshot not verified");
  assert(c.snapshot.statusKey() === SNAP_MATCH, "matching status key wrong");
  assert(c.core.state().snapshot.hash === hash, "stored hash differs from returned hash");
});

test("snapshot detects tampering in protection and ACL payload members", function () {
  reset();
  c.snapshot.create();
  c.protection.setAllowance("sort", true);
  assert(c.snapshot.verify() === false, "protection change was not detected");
  assert(c.snapshot.statusKey() === SNAP_TAMPERED, "tampered status key wrong");
  c.snapshot.create();
  c.acl.add("A1:B2", "finance", "");
  assert(c.snapshot.verify() === false, "ACL change was not included in the payload");
});

test("snapshot is stable for identical state and rebuilds on demand", function () {
  reset();
  var first = c.snapshot.create();
  var second = c.snapshot.create();
  assert(first === second, "snapshot hash changed without a state change");
  c.core.mutate(function (state) { state.classification.level = "restricted"; });
  assert(c.snapshot.verify() === false, "classification change was not detected");
  c.snapshot.create();
  assert(c.snapshot.verify() === true, "rebuilt snapshot did not verify");
});

// ── STATIC source checks (source inspection, not runtime results) ─────
test("STATIC: all eight modules register a window.SheetControls namespace", function () {
  h.CONTROL_MODULES.forEach(function (fileName) {
    assert(h.moduleSource(fileName).indexOf("window.SheetControls") !== -1,
      fileName + " does not register a controls namespace");
  });
});

test("STATIC: no network primitives in the local-only control modules", function () {
  var forbidden = ["fetch(", "XMLHttpRequest", "https://", "http://", "WebSocket", "sendBeacon"];
  h.CONTROL_MODULES.forEach(function (fileName) {
    var source = h.moduleSource(fileName);
    forbidden.forEach(function (token) {
      assert(source.indexOf(token) === -1, fileName + " contains network primitive " + token);
    });
  });
});

test("STATIC: every sheet-ctrl key used in source exists in the en catalog", function () {
  var catalog = h.readFile("../../../botlib/locales/en/sheet-controls.ftl");
  var known = {};
  (catalog.match(/^sheet-ctrl-[a-z0-9-]+(?=\s*=)/gm) || []).forEach(function (key) { known[key] = true; });
  var used = {};
  h.CONTROL_MODULES.forEach(function (fileName) {
    var matches = h.moduleSource(fileName).match(/sheet-ctrl-[a-z0-9-]+/g) || [];
    matches.forEach(function (key) { used[key] = true; });
  });
  var keys = Object.keys(used);
  assert(keys.length > 0, "no catalog keys found in source");
  unique(keys).forEach(function (key) {
    assert(known[key] === true, key + " missing from en catalog");
    assert(key.indexOf("auto-") === -1, "machine-generated key used: " + key);
  });
});
