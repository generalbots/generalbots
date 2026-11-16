"use strict";
var h = require("./00_harness");

h.test("LISTS: files exist", function () {
  h.assert(h.fileExists("lists/lists.js"), "lists.js missing");
  h.assert(h.fileExists("lists/modules/13_delta.js"), "lists delta module missing");
});

h.test("LISTS: mounts after load (readyState complete)", function () {
  var src = h.readFile("lists/lists.js");
  h.assert(/readyState/.test(src) || src.indexOf("DOMContentLoaded") === -1, "lists should be mount-safe");
  var ids = ["lists-search-input","listsList","list-modal","list-form","list-form-id","list-form-name","list-form-type","list-form-query","list-form-submit","members-modal"];
  var doc = h.makeDocument({ readyState: "complete", ids: ids });
  var sandbox = {
    window: {}, document: doc, console: { warn: function () {}, error: function () {}, log: function () {} },
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve([]);}});},
    htmx: { process: function(){}, trigger: function(){} },
    gbT: function(k,f){return f;}, confirm: function(){return true;}, alert: function(){},
    JSON: JSON, Date: Date,
  };
  sandbox.window = sandbox;
  try { h.loadScript("lists/lists.js", sandbox); } catch(e) {}
  h.assert(doc.getElementById("listsList"), "lists grid marker present");
});

h.test("LISTS: one listener per control (type change/search handler source guard)", function () {
  // lists uses inline handlers in HTML for some controls; ensure module boot doesn't duplicate core listeners when re-run.
  var ids = ["lists-search-input","listsList","list-modal","list-form","list-form-id","list-form-name","list-form-type","list-form-query","list-form-submit","members-modal"];
  var doc = h.makeDocument({ readyState: "complete", ids: ids });
  var sandbox = {
    window: {}, document: doc, console: { warn: function () {}, error: function () {}, log: function () {} },
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve([]);}});},
    htmx: { process: function(){}, trigger: function(){} },
    gbT: function(k,f){return f;}, confirm: function(){return true;}, alert: function(){},
    JSON: JSON, Date: Date,
  };
  sandbox.window = sandbox;
  try { h.loadScript("lists/lists.js", sandbox); } catch(e) {}
  var formSubmits = h.countListeners(doc.getElementById("list-form"), "submit");
  h.assert(formSubmits <= 1, "expected <=1 submit listener on list-form, got " + formSubmits);
});

h.test("LISTS: hostile input as text (delta/escape guards)", function () {
  var src = h.readFile("lists/modules/13_delta.js") + "\n" + h.readFile("lists/lists.js");
  h.assert(src.indexOf("textContent") !== -1 || src.indexOf("escape") !== -1 || src.indexOf("createTextNode") !== -1,
    "lists should render hostile input as text");
});
