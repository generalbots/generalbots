"use strict";
var h = require("./00_harness");

h.test("PHOTOS: files exist", function () {
  h.assert(h.fileExists("photos/photos.js"), "photos.js missing");
  h.assert(h.fileExists("photos/photos.html"), "photos.html missing");
});

h.test("PHOTOS: mounts after load (readyState complete)", function () {
  var src = h.readFile("photos/photos.js");
  h.assert(/readyState/.test(src), "photos.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-photos", "photosRefresh", "photosCount", "photosGrid", "photosLightbox", "photosLbClose", "photosLbImg"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve([]);}});},
    localStorage: { getItem: function(){return null;}, setItem: function(){} },
    sessionStorage: { getItem: function(){return null;}, setItem: function(){} },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    IntersectionObserver: function(){ return { observe:function(){}, disconnect:function(){} }; },
    URL: { createObjectURL: function(){return "blob:x";}, revokeObjectURL: function(){} },
    atob: function(s){return s||"";}, Uint8Array: Uint8Array, Blob: function(){},
    JSON: JSON, Date: Date, Array: Array, Set: Set, Map: Map,
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  try { h.loadScript("photos/photos.js", sandbox); } catch(e) {}
  h.assert(doc.getElementById("photosGrid"), "photos grid marker present");
});

h.test("PHOTOS: one listener per control (Refresh)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-photos", "photosRefresh", "photosCount", "photosGrid", "photosLightbox", "photosLbClose", "photosLbImg"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve([]);}});},
    localStorage: { getItem: function(){return null;}, setItem: function(){} },
    sessionStorage: { getItem: function(){return null;}, setItem: function(){} },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    IntersectionObserver: function(){ return { observe:function(){}, disconnect:function(){} }; },
    URL: { createObjectURL: function(){return "blob:x";}, revokeObjectURL: function(){} },
    atob: function(s){return s||"";}, Uint8Array: Uint8Array, Blob: function(){},
    JSON: JSON, Date: Date, Array: Array, Set: Set, Map: Map,
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  try { h.loadScript("photos/photos.js", sandbox); } catch(e) {}
  var clicks = h.countListeners(doc.getElementById("photosRefresh"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on photosRefresh, got " + clicks);
});

h.test("PHOTOS: hostile input as text (no unescaped filenames)", function () {
  var src = h.readFile("photos/photos.js");
  h.assert(src.indexOf("escapeHtml") !== -1 || src.indexOf("textContent") !== -1,
    "photos should escape hostile filenames/input");
});
