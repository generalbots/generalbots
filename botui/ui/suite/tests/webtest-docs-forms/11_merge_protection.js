/**
 * #2427 offline verification of the shipped mail-merge, dialog-shell,
 * signature/redaction, protection and boot modules (61..67). Pure helpers are
 * called directly; DOM-bound helpers run against the shared fake DOM. STATIC
 * checks document scope that the shipped code explicitly defers to a server or
 * a future implementation — they never fabricate a runtime result.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;
var loadForms = h.loadForms;
var makeArticle = h.makeArticle;

// ── Mail merge (61) ────────────────────────────────────────────────────

var CSV = "name,email,note\r\nAda,ada@example.test,\"hello, world\"\r\n" +
  "Bob,bob@example.test,\"line \"\"quoted\"\"\"\r\n\r\n";

test("RUNTIME #2427: parseCSV handles quoted cells, escapes, CRLF and blank tail", function () {
  var parsed = loadForms().NS.merge.parseCSV(CSV);
  assert(parsed.headers.join("|") === "name|email|note", "headers parsed and trimmed");
  assert(parsed.rows.length === 2, "trailing blank lines are ignored");
  assert(parsed.rows[0][2] === "hello, world", "comma inside quotes preserved");
  assert(parsed.rows[1][2] === 'line "quoted"', "doubled quotes unescaped");
  assert(loadForms().NS.merge.parseCSV("").headers.length === 0, "empty input has no headers");
});

test("RUNTIME #2427: toRecords maps headers to cells with empty defaults", function () {
  var M = loadForms().NS.merge;
  var records = M.toRecords(M.parseCSV(CSV));
  assert(records[0].name === "Ada" && records[1].email === "bob@example.test", "header-to-cell mapping");
  var short = M.toRecords(M.parseCSV("a,b\n1\n"));
  assert(short[0].a === "1" && short[0].b === "", "missing cells become empty strings");
});

test("RUNTIME #2427: autoMap matches field labels case-insensitively", function () {
  var f = loadForms();
  var C = f.NS.controls;
  var fields = [C.create("text", { id: "f1", label: "Name" }), C.create("text", { id: "f2", label: "Email" })];
  var mapping = f.NS.merge.autoMap(["name", "email", "note"], fields);
  assert(mapping.name === "f1" && mapping.email === "f2", "matched columns");
  assert(mapping.note === "", "unmatched column maps to empty");
});

test("RUNTIME #2427: previewRecord clamps navigation and handles an empty source", function () {
  var preview = loadForms().NS.merge.previewRecord;
  var records = [{ a: 1 }, { a: 2 }];
  assert(preview(records, 9).index === 1, "high index clamped to last record");
  assert(preview(records, -4).index === 0, "negative index clamped to first");
  assert(preview(records, 1).record.a === 2, "record returned at position");
  var empty = preview([], 0);
  assert(empty.index === 0 && empty.record === null, "empty source has no record");
});

test("[UAT:BIZ-03] RUNTIME #2427: merge.run reports per-record missing required fields", function () {
  var f = loadForms();
  var C = f.NS.controls;
  var fields = [
    C.create("text", { id: "f1", label: "Name" }),
    C.create("text", { id: "f2", label: "Email" }),
    C.create("text", { id: "f3", label: "Note", required: true }),
  ];
  var records = f.NS.merge.toRecords(f.NS.merge.parseCSV(CSV));
  var mapping = { name: "f1", email: "f2", note: "f3" };
  var outcome = f.NS.merge.run(records, fields, mapping);
  assert(outcome.count === 2 && outcome.missing.length === 0, "no missing required values");
  assert(outcome.text.indexOf("Name: Ada") >= 0 && outcome.text.indexOf("Note: hello, world") >= 0, "combined preview lines");
  assert((outcome.text.match(/Name:/g) || []).length === 2, "one block per record");
  records[1].note = "";
  var withGap = f.NS.merge.run(records, fields, mapping);
  assert(withGap.missing.length === 1, "one failing record reported");
  assert(withGap.missing[0].index === 1 && withGap.missing[0].label === "Note", "failure carries index and label");
});

test("RUNTIME #2427: merge.load stores CSV, resets navigation and auto-maps", function () {
  var f = loadForms();
  makeArticle(f, "doc-M");
  f.NS.state.controls = [f.NS.controls.create("text", { id: "f1", label: "Name" })];
  f.NS.state.recordIndex = 7;
  var result = f.NS.merge.load(CSV);
  assert(result.records.length === 2 && result.parsed.headers.length === 3, "parsed result returned");
  assert(f.NS.state.csvText === CSV && f.NS.state.recordIndex === 0, "source stored and index reset");
  assert(f.NS.state.mapping.name === "f1", "mapping auto-derived");
  assert(f.window.localStorage.getItem(f.NS.docKey("state")) !== null, "state persisted");
});

// ── Dialog shell (62, 63) ─────────────────────────────────────────────

test("RUNTIME #2427: dialog primitives emit data-i18n metadata, not baked text", function () {
  var f = loadForms();
  var D = f.NS.dialog;
  var el = D.el("div", "<cls>");
  assert(el.tagName === "DIV" && el.className === "<cls>", "element factory");
  var node = f.doc.createElement("span");
  D.i18n(node, "some.key", "<fallback>");
  assert(node.getAttribute("data-i18n") === "some.key" && node.textContent === "<fallback>", "i18n helper");
  var button = D.button("some.key", "<close>", true);
  assert(button.getAttribute("type") === "button", "button is non-submitting");
  assert(button.getAttribute("data-i18n") === "some.key" && button.className.indexOf("gb-forms-btn-primary") >= 0, "primary class + i18n");
  var input = D.input("textarea", "some.key", "<hint>", "<value>");
  assert(input.getAttribute("data-i18n-placeholder") === "some.key" && input.placeholder === "<hint>", "translated placeholder");
  assert(input.value === "<value>", "initial value");
  var field = D.field("some.key", "<label>", input);
  assert(field.children.indexOf(input) >= 0 && field.getAttribute("data-i18n") === "some.key", "field wraps input");
});

test("RUNTIME #2427: dialog shell is accessible, focus-trapped and closable", function () {
  var f = loadForms();
  var D = f.NS.dialog;
  var shell = D.shell({ titleKey: "k", titleFallback: "<title>", preview: true });
  assert(shell.root.getAttribute("role") === "dialog" && shell.root.getAttribute("aria-modal") === "true", "dialog role");
  var heading = shell.root.querySelector(".gb-forms-dialog-title");
  assert(heading && shell.root.getAttribute("aria-labelledby") === heading.id, "aria-labelledby points at the title");
  assert(heading.textContent === "<title>", "title resolved through the translator");
  assert(shell.status.getAttribute("role") === "status" && shell.status.getAttribute("aria-live") === "polite", "status region");
  assert(shell.root.querySelectorAll(".gb-forms-local-note").length === 1, "local note rendered");
  assert(shell.root.querySelectorAll(".gb-forms-preview-note").length === 1, "preview note rendered");
  assert(shell.backdrop.parentNode === f.doc.body, "backdrop mounted");
  var other = D.button("k2", "<other>");
  shell.body.appendChild(other);
  var buttons = shell.root.querySelectorAll("button");
  var closeButton = buttons[buttons.length - 1];
  f.doc.activeElement = other;
  f.fire("keydown", { key: "Tab", preventDefault: function () {} });
  assert(f.doc.activeElement === closeButton, "Tab advances focus to the next control");
  f.fire("keydown", { key: "Escape", preventDefault: function () {} });
  assert(shell.backdrop.parentNode === null, "Escape removes the backdrop");
});

test("RUNTIME #2427: template gallery dialog renders real template cards", function () {
  var f = loadForms();
  var close = f.NS.dialogs.openTemplates();
  var cards = f.doc.body.querySelectorAll("[data-form-template]");
  assert(cards.length === 5, "one card per shipped template");
  assert(cards[0].getAttribute("data-form-template") === "blank", "deterministic first card");
  close();
  assert(f.doc.body.querySelectorAll("[data-form-template]").length === 0, "close tears down the dialog");
});

// ── Signature, redaction and protection (65, 66) ──────────────────────

test("RUNTIME #2427: signature preview escapes the signer and stays labelled", function () {
  var html = loadForms().NS.security.signaturePreview("<signer>");
  assert(html.indexOf("gb-forms-signature") >= 0, "signature marker");
  assert(html.indexOf("&lt;signer&gt;") >= 0, "signer name escaped");
  assert(html.indexOf("gb-forms-preview-note") >= 0, "preview disclaimer retained");
});

test("RUNTIME #2427: redaction preview retains original text (not real redaction)", function () {
  var S = loadForms().NS.security;
  var html = S.redactionPreview("secret");
  assert(html.indexOf('data-form-original="secret"') >= 0, "original text remains in the DOM marker");
  assert(html.indexOf("gb-forms-redaction") >= 0, "redaction marker present");
  var bars = />([█]+)</.exec(html);
  assert(bars && bars[1].length === 6, "one block character per input character");
  var capped = />([█]+)</.exec(S.redactionPreview("abcdefghijklmnopqrstuvwxyz"));
  assert(capped && capped[1].length === 16, "bar length is capped at 16");
});

test("RUNTIME #2427: protection enable/disable never persists the password", function () {
  var f = loadForms();
  makeArticle(f, "doc-P");
  f.NS.protection.enable("<secret-password>");
  assert(f.NS.protection.enabled() === true, "enabled flag set");
  assert(f.NS.state.protect.passwordPreview === "set", "password replaced by a set marker");
  var stored = f.window.localStorage.getItem(f.NS.docKey("state"));
  assert(stored.indexOf("<secret-password>") === -1, "raw password absent from persisted state");
  f.NS.protection.disable();
  assert(f.NS.protection.enabled() === false && f.NS.state.protect.passwordPreview === "", "disable clears state");
});

test("RUNTIME #2427: shouldBlock is a pure form/comment gating contract", function () {
  var block = loadForms().NS.protection.shouldBlock;
  assert(block(true, {}) === true, "enabled blocks ordinary edits");
  assert(block(true, { isForm: true }) === false, "form edits allowed");
  assert(block(true, { isComment: true }) === false, "comment edits allowed");
  assert(block(true, { isForm: true, isComment: true }) === false, "both exemptions allowed");
  assert(block(false, {}) === false, "disabled never blocks");
});

test("RUNTIME #2427: contextOf classifies form and comment targets via closest", function () {
  var f = loadForms();
  var P = f.NS.protection;
  var form = f.doc.createElement("span");
  form.setAttribute("data-gb-form-control", "1");
  assert(P.contextOf(form).isForm === true, "form control recognised");
  var comment = f.doc.createElement("span");
  comment.className = "gb-comment";
  assert(P.contextOf(comment).isComment === true, "comment region recognised");
  var plain = f.doc.createElement("span");
  var ctx = P.contextOf(plain);
  assert(ctx.isForm === false && ctx.isComment === false, "plain content has no exemption");
});

test("RUNTIME #2427: guard prevents ordinary edits and announces through status", function () {
  var f = loadForms();
  makeArticle(f, "doc-G");
  var P = f.NS.protection;
  P.enable("<pw>");
  var prevented = false;
  P.guard({ target: f.doc.createElement("span"), preventDefault: function () { prevented = true; } });
  assert(prevented === true, "ordinary edit prevented");
  var status = f.doc.getElementById("docs-forms-status");
  assert(status && status.getAttribute("role") === "status" && status.getAttribute("aria-live") === "polite", "status region created");
  assert(status.textContent.indexOf("protected") >= 0, "block announced");
  var form = f.doc.createElement("span");
  form.setAttribute("data-gb-form-control", "1");
  var formPrevented = false;
  P.guard({ target: form, preventDefault: function () { formPrevented = true; } });
  assert(formPrevented === false, "form-region edit allowed while protected");
});

test("RUNTIME #2427: protect banner is inserted before the article and removed again", function () {
  var f = loadForms();
  var article = makeArticle(f, "doc-B");
  var P = f.NS.protection;
  assert(P.applyBanner() === false, "no banner while protection is off");
  P.enable("<pw>");
  var banner = f.doc.getElementById("docs-forms-protect-banner");
  assert(banner && banner.getAttribute("role") === "status" && banner.getAttribute("aria-live") === "polite", "accessible banner");
  var children = article.parentNode.children;
  assert(children.indexOf(banner) === children.indexOf(article) - 1, "banner immediately precedes article");
  P.disable();
  assert(f.doc.getElementById("docs-forms-protect-banner") === null, "banner removed on disable");
});

// ── Boot wiring (67) ───────────────────────────────────────────────────

test("RUNTIME #2427: boot installs guards and delegates every forms action", function () {
  var f = loadForms();
  var toggle = f.doc.createElement("button");
  toggle.id = "docsFormsProtectBtn";
  f.doc.body.appendChild(toggle);
  assert(!f.doc._listeners.click, "boot deferred while the document is loading");
  f.fire("DOMContentLoaded", {});
  var listeners = f.doc._listeners;
  assert(listeners.click && listeners.click.length === 1, "click delegation installed");
  assert(listeners.beforeinput && listeners.beforeinput.length === 1, "beforeinput guard installed");
  assert(listeners.paste && listeners.paste.length === 1, "paste guard installed");
  assert(toggle.getAttribute("aria-pressed") === "false", "toggle initial aria-pressed");
  var expected = {
    templates: "openTemplates", controls: "openControls", merge: "openMerge",
    sign: "openSign", protect: "openProtect",
  };
  Object.keys(expected).forEach(function (action) {
    var opened = null;
    f.NS.dialogs[expected[action]] = function () { opened = action; };
    var trigger = f.doc.createElement("button");
    trigger.setAttribute("data-gb-forms-open", action);
    f.fire("click", { target: trigger, preventDefault: function () {}, stopPropagation: function () {} });
    assert(opened === action, action + " must delegate to " + expected[action]);
  });
});

// ── Clearly-labelled STATIC guards for deferred / source-level scope ───

test("STATIC #2427: production comments disclaim backend-only capabilities", function () {
  var state = readFile("docs/modules/58_forms_state.js");
  assert(state.indexOf("content-control round-trip") >= 0, "no real DOCX sdt claim");
  assert(state.indexOf("cryptographic signing, irreversible redaction") >= 0, "no signing/redaction claim");
  var templates = readFile("docs/modules/59_forms_templates.js");
  assert(templates.indexOf("no org library, versioning") >= 0, "no versioned template library claim");
  var controls = readFile("docs/modules/60_forms_controls.js");
  assert(controls.indexOf("no DOCX w:sdt round-trip") >= 0, "no DOCX round-trip claim");
  var merge = readFile("docs/modules/61_forms_merge.js");
  assert(merge.indexOf("no per-record DOCX/PDF generation") >= 0, "no per-record generation claim");
  var security = readFile("docs/modules/65_forms_dialog_security.js");
  assert(security.indexOf("no PAdES signing") >= 0 && security.indexOf("no byte removal") >= 0, "no PAdES / removal claim");
  var protection = readFile("docs/modules/66_forms_protection.js");
  assert(protection.indexOf("password is never persisted") >= 0, "password handling contract");
});

test("STATIC #2427: shipped forms modules expose no network or eval surface", function () {
  h.FORMS_MODULES.forEach(function (rel) {
    var src = readFile(rel);
    ["fetch(", "XMLHttpRequest", "WebSocket", "eval(", "new Function("].forEach(function (token) {
      assert(src.indexOf(token) === -1, rel + " must not contain " + token);
    });
  });
});

test("STATIC #2427: i18n and live status regions follow the shipped pattern", function () {
  var shell = readFile("docs/modules/62_forms_dialog_shell.js");
  assert(shell.indexOf('setAttribute("data-i18n", key)') >= 0, "dialog text uses data-i18n");
  assert(shell.indexOf("node.textContent = NS.tr(key, fallback)") >= 0, "dialog text uses NS.tr");
  [58, 62, 66].forEach(function (n) {
    var src = readFile("docs/modules/" + n + "_forms_" + (n === 66 ? "protection" : n === 62 ? "dialog_shell" : "state") + ".js");
    assert(src.indexOf('setAttribute("role", "status")') >= 0, n + " has a status role");
    assert(src.indexOf('setAttribute("aria-live", "polite")') >= 0, n + " has a polite live region");
  });
  var init = readFile("docs/modules/67_forms_init.js");
  ["templates", "controls", "merge", "sign", "protect"].forEach(function (action) {
    assert(init.indexOf('=== "' + action + '"') >= 0, "init must wire " + action);
  });
});

test("STATIC #2427: all ten shipped modules exist and stay under 450 lines", function () {
  h.FORMS_MODULES.forEach(function (rel) {
    assert(h.fileExists(rel), rel + " must exist");
    var lines = readFile(rel).split("\n").length;
    assert(lines < 450, rel + " is " + lines + " lines (limit 450)");
  });
});
