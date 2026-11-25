/**
 * #2448 — deterministic static coverage for the SHIPPED chat consent preview
 * renderer (botui/ui/suite/chat/modules/40_consent_cards.js) and its i18n key.
 *
 * All assertions are literal substrings of production source / locale
 * catalogs. Nothing is executed, no server, no browser, no cargo, no network.
 *
 * NON-VACUITY: each case first proves the real renderer (or catalog) ships by
 * checking its function/constant markers and that the extracted source span is
 * non-empty. Removing buildPreview, the gb-consent-preview region, either cap,
 * the optional-region guard, or any one locale key makes a case FAIL.
 */
"use strict";

var h = require("./00_harness");

var CARD = "botui/ui/suite/chat/modules/40_consent_cards.js";
var GATE = "botserver/src/core/bot/consent_gate.rs";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

h.staticTest("#2448 consent preview renders a text-only role=status region with mirrored caps and optional guard", function () {
  var src = h.readRepo(CARD);

  // Non-vacuity: the real renderer, its public hook and its caps ship.
  contains(src, "function buildPreview(preview) {", "buildPreview renderer is shipped");
  contains(src, "function buildCard(req) {", "consent card builder is shipped");
  contains(src, "window.GBBuildConsentCard = buildCard;", "card builder hook is shipped");
  contains(src, "var PREVIEW_MAX_ROWS = 10, PREVIEW_VALUE_CHARS = 80;",
    "client preview caps are shipped");

  var preview = span(src, "  function buildPreview(preview) {", "  function buildCard(req) {");

  // (a) The region is a real DOM node carrying role=status / aria-live=polite.
  contains(preview, "document.createElement(\"div\")", "renderer creates DOM nodes");
  contains(preview, "region.className = \"gb-consent-preview\"", "gb-consent-preview region is built");
  contains(preview, "region.setAttribute(\"role\", \"status\")", "region has role=status");
  contains(preview, "region.setAttribute(\"aria-live\", \"polite\")", "region has aria-live=polite");

  // (a) Every row/line is written with textContent only — never innerHTML.
  contains(preview, "title.textContent =", "preview title uses textContent");
  contains(preview, "line.textContent =", "affected-count line uses textContent");
  contains(preview, "row.textContent =", "preview row uses textContent");
  contains(preview, "region.appendChild(row)", "rows are appended to the region");
  h.assert(preview.indexOf("innerHTML") === -1,
    "consent preview must build rows via textContent only, found innerHTML in <" + preview.slice(0, 200) + ">");

  // (b) Client caps mirror the server bounds exactly.
  contains(preview, "preview.entries.slice(0, PREVIEW_MAX_ROWS)", "entries are capped client-side");
  contains(src, "PREVIEW_VALUE_CHARS - 1", "values are truncated client-side");
  var server = h.readRepo(GATE);
  contains(server, "const PREVIEW_MAX_ENTRIES: usize = 10;", "server entry cap is shipped");
  contains(server, "const PREVIEW_VALUE_CHARS: usize = 80;", "server value cap is shipped");

  // (b) Preview is optional: no marker / no rows+count yields no region.
  contains(preview, "if (!preview || typeof preview !== \"object\") return null;",
    "absent preview yields no region");
  contains(preview, "if (!rows.length && (count === undefined || count === null)) return null;",
    "empty preview yields no region");

  // Hook placement: the card inserts the region only when it exists.
  var card = span(src, "  function buildCard(req) {", "window.GBBuildConsentCard = buildCard;");
  contains(card, "var previewRegion = buildPreview(req.preview);", "card builds the optional preview");
  contains(card, "if (previewRegion) card.insertBefore(previewRegion, risk);",
    "card inserts preview only when present");
});

h.staticTest("#2448 chat-consent-affected-count key ships in all 8 catalogs and preview labels use gbT", function () {
  var src = h.readRepo(CARD);

  // Non-vacuity: the shipped renderer really localises its preview labels.
  contains(src, "function buildPreview(preview) {", "buildPreview renderer is shipped");
  contains(src, "function t(k, fb) { return typeof gbT === \"function\" ? gbT(k, fb) : fb; }",
    "renderer routes labels through gbT");
  contains(src, "title.textContent = t(\"chat-preview\", \"Preview\");",
    "preview title label uses gbT");
  contains(src,
    "line.textContent = t(\"chat-consent-affected-count\", \"Affected records\") + \": \" + count;",
    "affected-count label uses gbT");

  // The key must be defined, with a real value, in every shipped catalog.
  var values = [];
  LOCALES.forEach(function (loc) {
    var rel = "botlib/locales/" + loc + "/ui-chat.ftl";
    var catalog = h.readRepo(rel);
    h.assert(catalog.length > 0, rel + " must be a non-empty catalog");
    h.assert(catalog.indexOf("chat-consent-affected-count = ") !== -1,
      rel + " must define chat-consent-affected-count");
    var match = catalog.match(/^chat-consent-affected-count\s*=\s*(\S.*)$/m);
    h.assert(match !== null, rel + " key must have a value on its own line");
    var value = match[1].trim();
    h.assert(value.length > 0, rel + " translated value must not be empty");
    values.push(value);
  });
  h.assertEqual(values.length, 8, "all 8 catalogs must be checked");

  // Non-vacuity: real translations differ; copied/empty stubs cannot pass.
  var distinct = {};
  values.forEach(function (v) { distinct[v] = true; });
  h.assert(Object.keys(distinct).length >= 2,
    "catalogs must carry per-locale translations, found <" + values.join(" | ") + ">");
});

h.note("FOLLOW-UP #2448 criterion 3: the consent preview is rendered and localised in 8 catalogs; " +
  "the explicit confirm/cancel resolution gate remains a tracked gap.");
