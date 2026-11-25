/**
 * #2923 — deterministic static coverage for the Slides/Sheet AI endpoints.
 *
 * Cases (literal source assertions, no execution, no server/browser):
 *   (a) neither handler ships the old keyword-to-hints table or its canned
 *       "cannot modify" / "only explains how to use" strings;
 *   (b) both handlers reach a real configured LLM: Slides mirrors Docs'
 *       llm_complete contract through the botlib HTTP client, Sheet reuses its
 *       existing call_llm path;
 *   (c) both return typed action/data for allowlisted verbs and an explicit
 *       typed 503 ai_unavailable envelope otherwise — never a plausible
 *       instruction reply on failure;
 *   (d) the DTO fields and both botbook AI sections match those statuses, with
 *       the fabricated slide_count/generation_time_ms/operation payloads gone;
 *   (e) offline unit tests in each crate cover typed actions, unlisted verbs,
 *       typed unavailability and the absence of the hint table.
 *
 * NON-VACUITY: every case first proves the real source spans are non-empty;
 * scoped absence checks apply only to those spans. Restoring a keyword table,
 * dropping an LLM call, widening the verb allowlist, returning instructions on
 * failure, deleting a test, reverting a botbook section or removing this
 * module's require line makes the matching case FAIL. GB_OFFICE_AI_PROBE_ROOT
 * can point at a /tmp copy for the mutation probe without touching the repo.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var SLIDES = "botserver/crates/botslides/src/handlers/ai.rs";
var SLIDES_TESTS = "botserver/crates/botslides/src/handlers/ai_tests.rs";
var SLIDES_DTO = "botserver/crates/botslides/src/types/requests.rs";
var SHEET = "botserver/crates/botsheet/src/handlers/ai.rs";
var SHEET_TESTS = "botserver/crates/botsheet/src/handlers/ai_tests.rs";
var SHEET_DTO = "botserver/crates/botsheet-core/src/types/requests_feature.rs";
var SLIDES_DOC = "botbook/src/08-rest-api-tools/slides-api.md";
var SHEET_DOC = "botbook/src/08-rest-api-tools/sheets-api.md";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";

function readSrc(rel) {
  var probe = process.env.GB_OFFICE_AI_PROBE_ROOT;
  if (probe) {
    var candidate = path.join(probe, rel);
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(
    src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 160)
  );
}
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function section(src, heading) {
  var start = src.indexOf("### " + heading);
  h.assert(start !== -1, "missing documentation section <" + heading + ">");
  var end = src.indexOf("\n---", start);
  h.assert(end !== -1, "unterminated documentation section <" + heading + ">");
  var body = src.slice(start, end);
  h.assert(body.length > 600, "non-vacuity: section <" + heading + "> must be substantial");
  return body;
}
function verbsIn(src) {
  var block = span(src, "const ALLOWED_ACTIONS: &[&str] = &[", "];");
  return block.match(/"(?:slides|sheet)\.[a-z.]+"/g) || [];
}

h.staticTest("#2923 Slides AI handler has no keyword-to-hints table", function () {
  var src = readSrc(SLIDES);
  h.assert(src.length > 5000, "non-vacuity: Slides handler must be non-trivial");
  absent(src, "fn ai_reply_for", "keyword table removed from Slides");
  [
    "This chat endpoint",
    "cannot modify the",
    "cannot insert",
    "only explains how to use the editor",
    "Use the + button",
    "the slide insert menu"
  ].forEach(function (hint) {
    absent(src, hint, "canned Slides hint removed");
  });
});

h.staticTest("#2923 Sheet AI handler has no keyword-to-hints table", function () {
  var src = readSrc(SHEET);
  h.assert(src.length > 5000, "non-vacuity: Sheet handler must be non-trivial");
  absent(src, "fn ai_reply_for", "keyword table removed from Sheet");
  [
    "This chat endpoint",
    "cannot modify the",
    "cannot insert",
    "cannot sort data",
    "only explains how to use the editor",
    "AutoSum in the toolbar",
    "number-format control"
  ].forEach(function (hint) {
    absent(src, hint, "canned Sheet hint removed");
  });
});

h.staticTest("#2923 Slides AI calls a configured LLM and returns typed verbs", function () {
  var src = readSrc(SLIDES);
  var complete = span(src, "fn llm_config()", "fn system_prompt(");
  contains(complete, "LLM_URL", "Docs-compatible URL is required");
  contains(complete, "LLM_KEY", "Docs-compatible key is required");
  contains(complete, "LLM_MODEL", "model override is supported");
  contains(complete, "async fn llm_complete(", "LLM completion helper ships");
  contains(complete, "post_authorized", "Bearer key is sent with the completion request");
  contains(complete, "Unavailable::NotConfigured", "missing config is an explicit error");
  contains(complete, "Unavailable::Provider", "provider failures are explicit errors");
  contains(complete, "choices", "the completion envelope is read");
  var handler = span(src, "pub async fn handle_slides_ai", "#[cfg(test)]");
  contains(handler, "llm_complete(&system_prompt(), &user_prompt(&req)).await", "handler calls LLM");
  contains(handler, "Json(parse_llm_answer(&content, &req)).into_response()", "LLM answer is returned");
  contains(handler, "unavailable_response(reason)", "failure is an explicit 503");
  var verbs = verbsIn(src);
  h.assert(verbs.length >= 9, "Slides verb allowlist is complete, got " + verbs.length);
  contains(src, '"slides.slide.add"');
  contains(src, '"slides.transition.set"');
});

h.staticTest("#2923 Sheet AI reuses its existing call_llm path and typed verbs", function () {
  var src = readSrc(SHEET);
  var handler = span(src, "pub async fn handle_sheet_ai", "pub async fn handle_evaluate_ai_prompt");
  contains(handler, "llm_available()", "configuration is checked before calling out");
  contains(handler, "call_llm(&prompt).await", "existing Sheet LLM path is reused");
  contains(handler, "build_prompt(&req)", "command/selection/active cell reach the LLM");
  contains(handler, "parse_llm_answer(&content, &req)", "LLM answer is parsed into the typed DTO");
  contains(handler, "unavailable_response(Unavailable::NotConfigured)", "missing config is 503");
  contains(handler, "unavailable_response(Unavailable::Provider)", "provider error is 503");
  var available = span(src, "fn llm_available()", "fn build_prompt(");
  contains(available, '"BOT_AI_PROMPT_URL"', "existing Sheet provider variable is used");
  var verbs = verbsIn(src);
  h.assert(verbs.length >= 10, "Sheet verb allowlist is complete, got " + verbs.length);
  contains(src, '"sheet.formula.set"');
  contains(src, '"sheet.pivot.create"');
});

h.staticTest("#2923 unavailability is a typed 503 with no instruction reply", function () {
  [SLIDES, SHEET].forEach(function (file) {
    var src = readSrc(file);
    var body = span(src, "fn unavailable_body(", "fn unavailable_response(");
    contains(body, '"error": "ai_unavailable"', "typed error code");
    contains(body, '"reason": reason.reason()', "typed reason");
    absent(body, "response", "no plausible instruction text in the error body");
    var response = span(src, "fn unavailable_response(", "pub async fn handle_");
    contains(response, "StatusCode::SERVICE_UNAVAILABLE", "503 status");
    contains(response, "Json(unavailable_body(reason))", "typed JSON body");
  });
});

h.staticTest("#2923 typed verbs are allowlisted and context is injected", function () {
  [SLIDES, SHEET].forEach(function (file) {
    var src = readSrc(file);
    var parsed = span(src, "fn parse_llm_answer(", "fn unavailable_body(");
    contains(parsed, "ALLOWED_ACTIONS.contains", "action is validated against the allowlist");
    contains(parsed, "serde_json::Value::Object", "data must be a typed object");
    contains(parsed, "data = None", "stripped actions carry no data");
  });
  var slides = readSrc(SLIDES);
  var sheet = readSrc(SHEET);
  contains(span(slides, "fn parse_llm_answer(", "fn unavailable_body("), '"presentation_id"');
  contains(span(sheet, "fn parse_llm_answer(", "fn unavailable_body("), '"file_id"');
});

h.staticTest("#2923 DTO fields match the handlers", function () {
  var slides = readSrc(SLIDES_DTO);
  var slidesReq = span(slides, "pub struct SlidesAiRequest {", "}");
  contains(slidesReq, '#[serde(alias = "prompt")]', "editor form alias is declared");
  contains(slidesReq, "pub command: String,", "required command field");
  contains(slidesReq, "pub slide_index: Option<usize>,", "optional slide index");
  contains(slidesReq, "pub presentation_id: Option<String>,", "optional deck id");
  var slidesResp = span(slides, "pub struct SlidesAiResponse {", "}");
  contains(slidesResp, "pub response: String,");
  contains(slidesResp, "pub action: Option<String>,");
  contains(slidesResp, "pub data: Option<serde_json::Value>,");

  var sheet = readSrc(SHEET_DTO);
  var sheetReq = span(sheet, "pub struct SheetAiRequest {", "}");
  ["command", "selection", "active_cell", "sheet_id"].forEach(function (field) {
    contains(sheetReq, "pub " + field + ":", "Sheet DTO field " + field);
  });
  var sheetResp = span(sheet, "pub struct SheetAiResponse {", "}");
  ["response", "action", "data"].forEach(function (field) {
    contains(sheetResp, "pub " + field + ":", "Sheet response field " + field);
  });
});

h.staticTest("#2923 botbook AI sections describe the real DTO and statuses", function () {
  var slidesDoc = section(readSrc(SLIDES_DOC), "AI Assistant");
  contains(slidesDoc, "POST /api/slides/ai");
  contains(slidesDoc, "SlidesAiRequest");
  contains(slidesDoc, "SlidesAiResponse");
  contains(slidesDoc, "503");
  contains(slidesDoc, "ai_unavailable");
  contains(slidesDoc, "not_configured");
  ["LLM_URL", "LLM_KEY", "LLM_MODEL", "slide_index", "presentation_id", "slides.slide.add"].forEach(
    function (token) {
      contains(slidesDoc, token, "Slides doc token " + token);
    }
  );
  ["slide_count", "generation_time_ms", "style", "thumbnail_url"].forEach(function (stale) {
    absent(slidesDoc, stale, "fabricated Slides request/response field removed");
  });

  var sheetDoc = section(readSrc(SHEET_DOC), "AI Assist");
  contains(sheetDoc, "POST /api/sheet/ai");
  contains(sheetDoc, "SheetAiRequest");
  contains(sheetDoc, "SheetAiResponse");
  contains(sheetDoc, "503");
  contains(sheetDoc, "ai_unavailable");
  contains(sheetDoc, "not_configured");
  ["BOT_AI_PROMPT_URL", "BOT_AI_PROMPT_MODEL", "command", "selection", "active_cell", "sheet_id", "sheet.formula.set"].forEach(
    function (token) {
      contains(sheetDoc, token, "Sheet doc token " + token);
    }
  );
  ["sheet_tab", "generated_at", "operation", "formulas_generated"].forEach(function (stale) {
    absent(sheetDoc, stale, "fabricated Sheet request/response field removed");
  });
});

h.staticTest("#2923 offline unit tests cover typed actions and unavailability", function () {
  [SLIDES_TESTS, SHEET_TESTS].forEach(function (file) {
    var src = readSrc(file);
    h.assert(src.length > 2500, "non-vacuity: " + file + " must be substantial");
    h.assert((src.match(/#\[test\]/g) || []).length >= 6, file + " needs at least six tests");
    [
      "typed_action_is_extracted",
      "unlisted_action_is_never_executable",
      "unavailability_is_typed_without_instructions",
      "handler_has_no_keyword_hint_table"
    ].forEach(function (name) {
      contains(src, name, file + " must test " + name);
    });
    contains(src, 'include_str!("ai.rs")', file + " asserts its handler is hint-free");
  });
  contains(readSrc(SLIDES_TESTS), "prompts_carry_every_allowed_verb_and_the_request_context");
  contains(readSrc(SHEET_TESTS), "every_action_command_reaches_the_llm_prompt");
});

h.staticTest("#2923 static guard is wired into the fidelity runner", function () {
  contains(readSrc(RUNNER), 'require("./webtest-office-fidelity/72_ai_endpoints")');
});
