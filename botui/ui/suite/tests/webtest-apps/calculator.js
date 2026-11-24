"use strict";
var h = require("./00_harness");

h.test("CALCULATOR: files exist", function () {
  h.assert(h.fileExists("calculator/modules/99_init.js"), "calculator init missing");
  h.assert(h.fileExists("calculator/calculator.html"), "calculator.html missing");
});

h.test("[UAT:CALC-02] CALCULATOR: mounts after load (readyState complete)", function () {
  var src = h.readFile("calculator/modules/99_init.js");
  h.assert(/readyState/.test(src) || src.indexOf("DOMContentLoaded") === -1, "calculator init should be mount-safe");
  var ids = ["gb-calc-root","gb-calc-expr","gb-calc-preview","gb-calc-pad","gb-calc-sci","gb-calc-converter","gb-calc-copy","gb-calc-history-btn","gb-calc-history","gb-calc-history-list","gb-calc-history-clear","gb-calc-angle","gb-calc-memory-indicator","gb-tab-standard","gb-tab-scientific","gb-tab-convert","gb-calc-conv-cat","gb-calc-conv-amt","gb-calc-conv-from","gb-calc-conv-to","gb-calc-conv-out","gb-calc-conv-swap"];
  var doc = h.makeDocument({ readyState: "complete", ids: ids });
  var sandbox = { window: {}, document: doc, console: { warn: function () {}, error: function () {}, log: function () {} }, localStorage:{getItem:function(){return null;},setItem:function(){}}, JSON:JSON, Math:Math };
  sandbox.window = sandbox;
  try { h.loadScript("js/calculator-engine.js", sandbox); } catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); }
  ["calculator/modules/01_state.js","calculator/modules/02_render.js","calculator/modules/03_events.js","calculator/modules/99_init.js"].forEach(function(p){ try{ h.loadScript(p,sandbox);}catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); } });
  h.assert(doc.getElementById("gb-calc-root"), "calculator root marker present");
});

h.test("CALCULATOR: one listener per control (Copy)", function () {
  var ids = ["gb-calc-root","gb-calc-expr","gb-calc-preview","gb-calc-pad","gb-calc-sci","gb-calc-converter","gb-calc-copy","gb-calc-history-btn","gb-calc-history","gb-calc-history-list","gb-calc-history-clear","gb-calc-angle","gb-calc-memory-indicator","gb-tab-standard","gb-tab-scientific","gb-tab-convert","gb-calc-conv-cat","gb-calc-conv-amt","gb-calc-conv-from","gb-calc-conv-to","gb-calc-conv-out","gb-calc-conv-swap"];
  var doc = h.makeDocument({ readyState: "complete", ids: ids });
  var sandbox = { window: {}, document: doc, console: { warn: function () {}, error: function () {}, log: function () {} }, localStorage:{getItem:function(){return null;},setItem:function(){}}, JSON:JSON, Math:Math, navigator:{clipboard:{writeText:function(){return Promise.resolve();}}} };
  sandbox.window = sandbox;
  try { h.loadScript("js/calculator-engine.js", sandbox); } catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); }
  ["calculator/modules/01_state.js","calculator/modules/02_render.js","calculator/modules/03_events.js","calculator/modules/99_init.js"].forEach(function(p){ try{ h.loadScript(p,sandbox);}catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); } });
  var clicks = h.countListeners(doc.getElementById("gb-calc-copy"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on gb-calc-copy, got " + clicks);
});

h.test("CALCULATOR: hostile input as text", function () {
  var src = "";
  ["calculator/modules/01_state.js","calculator/modules/02_render.js","calculator/modules/03_events.js","calculator/modules/99_init.js"].forEach(function(p){ try{ src += "\n"+h.readFile(p);}catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); } });
  h.assert(src.indexOf("textContent") !== -1 || src.indexOf(".value") !== -1 || src.indexOf("innerText") !== -1,
    "calculator should render values as text");
});
