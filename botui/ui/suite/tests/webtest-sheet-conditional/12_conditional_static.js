/**
 * Sheet conditional formatting (#2260) — labelled STATIC source assertions.
 * These never claim runtime PASS for behaviour the shipped code does not
 * implement: they document the backend-deferred application path, the
 * missing cash-flow orchestration and where the rendering hooks live.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var condSrc = h.readSuite(h.SOURCES.conditional);
var renderSrc = h.readSuite(h.SOURCES.render);
var both = condSrc + "\n" + renderSrc;
var createSrc = h.readRoot("botserver/crates/botsheet/src/handlers/create.rs");
var cfSrc = h.readRoot("botserver/crates/botsheet/src/handlers/data_ops/conditional_format.rs");

test("[STATIC] producer applies rules through the backend conditional-format endpoint", function () {
  assert(/\/api\/sheet\/conditional-format/.test(condSrc));
  assert(/method:\s*"POST"/.test(condSrc));
  assert(!/bodyInner/.test(condSrc), "producer must not write grid DOM directly");
  assert(!/conditional_formats/.test(condSrc), "producer must not mutate loaded sheet state");
});

test("[STATIC] renderer is read-only over pre-existing conditional_formats", function () {
  assert(/conditional_formats\s*\|\|\s*\[\]/.test(renderSrc));
  assert(/window\.__LOADED_SHEET/.test(renderSrc));
  assert(!/fetch\s*\(/.test(renderSrc), "renderer must not call the network");
  assert(!/XMLHttpRequest/.test(renderSrc));
  assert(/function evalRule\(rule, cellValue\)/.test(renderSrc));
  assert(/rule_type\s*===\s*"colorScale"/.test(renderSrc));
  assert(/between:/.test(renderSrc));
});

test("[STATIC] negative-red / positive-green primitives exist but are never orchestrated", function () {
  assert(/#f44336/.test(condSrc), "red fill constant missing");
  assert(/#4caf50/.test(condSrc), "green fill constant missing");
  assert(/condition:\s*"<"\s*\+\s*value/.test(condSrc));
  assert(/condition:\s*">"\s*\+\s*value/.test(condSrc));
  var redCalls = condSrc.match(/highlightLessThan\s*\(/g) || [];
  var greenCalls = condSrc.match(/highlightGreaterThan\s*\(/g) || [];
  equal(redCalls.length, 1, "only the definition — no automatic lower-bound call");
  equal(greenCalls.length, 1, "only the definition — no automatic upper-bound call");
});

test("[STATIC] these IIFEs expose only rule-add; edit/remove stays backend-deferred", function () {
  assert(/applyRule:\s*applyRule/.test(condSrc));
  assert(!/function\s+(?:removeRule|deleteRule|editRule|updateRule|clearRules)/i.test(both));
  assert(!/method:\s*"(?:DELETE|PUT|PATCH)"/.test(condSrc));
  // Re-running a quick action appends another rule via the backend; neither
  // shipped module dedupes or replaces an existing rule.
});

test("[STATIC] no 12-month cash-flow / forecast generator exists in the shipped modules", function () {
  assert(!/cash[\s_-]?flow/i.test(both), "unexpected cash-flow orchestration in producer/renderer");
  assert(!/forecast/i.test(both), "unexpected forecast orchestration");
  assert(!/balance/i.test(both), "unexpected balance-row orchestration");
  assert(!/month/i.test(both), "unexpected month-column orchestration");
  assert(!/function\s+(?:generate|build|compose)/i.test(both), "unexpected block generator");
});

test("[STATIC] single-call styled creation is backend-deferred, not driven by these IIFEs", function () {
  assert(/pub conditional:\s*Option<Vec<Value>>/.test(createSrc));
  assert(/apply_inline_conditional_rules/.test(createSrc));
  assert(/"negative"\s*\|\s*"negatives"\s*=>\s*"<0"/.test(cfSrc));
  assert(/"positive"\s*\|\s*"positives"\s*=>\s*">=0"/.test(cfSrc));
  assert(!/\/api\/sheet\/(?:create|cells)/.test(condSrc),
    "the producer only posts per-rule, never composes a styled create/write call");
});

test("[STATIC] renderer re-evaluates styles on each grid render (edit-time hook)", function () {
  assert(/function wrapRender\(\)/.test(renderSrc));
  assert(/g\.render\s*=\s*function/.test(renderSrc));
  assert(/applyRules\(\)/.test(renderSrc));
  assert(/conditionalRender\s*=\s*applyRules/.test(renderSrc));
  assert(/validateEdit\s*=\s*validateEdit/.test(renderSrc));
  assert(/validationForCell\s*=\s*validationForCell/.test(renderSrc));
});

test("[STATIC] backend 'positive' maps to >=0 while the frontend primitive uses >0", function () {
  assert(/>=\s*0|>=0/.test(cfSrc));
  assert(/condition:\s*">"\s*\+\s*value/.test(condSrc));
  // Documented inconsistency: zero is green in the backend mapping but not
  // in highlightGreaterThan(0). Kept as a static note, not a runtime claim.
});
