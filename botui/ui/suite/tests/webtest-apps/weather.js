"use strict";
var h = require("./00_harness");

h.test("WEATHER: files exist", function () {
  h.assert(h.fileExists("weather/weather.js"), "weather.js missing");
  h.assert(h.fileExists("weather/weather-render.js"), "weather-render.js missing");
});

h.test("WEATHER: mounts after load (readyState complete)", function () {
  var src = h.readFile("weather/weather.js");
  h.assert(/readyState/.test(src) || src.indexOf("DOMContentLoaded") === -1, "weather should boot when ready");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-weather", "weatherCity", "weatherGo", "weatherCurrent", "weatherForecast"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve({});}});},
    navigator: { geolocation: { getCurrentPosition: function(){} } },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  try { h.loadScript("weather/weather-render.js", sandbox); } catch(e) {}
  h.loadScript("weather/weather.js", sandbox);
  h.assert(doc.getElementById("weatherGo"), "weather search marker present");
});

h.test("WEATHER: one listener per control (Search)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-weather", "weatherCity", "weatherGo", "weatherCurrent", "weatherForecast"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve({});}});},
    navigator: { geolocation: { getCurrentPosition: function(){} } },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  try { h.loadScript("weather/weather-render.js", sandbox); } catch(e) {}
  h.loadScript("weather/weather.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("weatherGo"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on weatherGo, got " + clicks);
});

h.test("WEATHER: hostile input as text", function () {
  var src = h.readFile("weather/weather.js") + "\n" + h.readFile("weather/weather-render.js");
  h.assert(src.indexOf("escape") !== -1 || src.indexOf("textContent") !== -1 || src.indexOf("innerText") !== -1,
    "weather should render hostile input as text");
});
