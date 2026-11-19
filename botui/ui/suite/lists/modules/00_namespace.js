"use strict";
/* #2537 - shared namespace for the structured segment builder.
   Plain script (not a module) so it works from the HTMX-injected lists
   fragment without a bundler. Every new string goes through gbT(). */
(function (global) {
  var ns = global.GBSegments || {};
  global.GBSegments = ns;

  ns.t = function (key, fallback, params) {
    if (typeof global.gbT === "function") return global.gbT(key, fallback, params);
    return fallback;
  };

  ns.escapeHtml = function (value) {
    if (value == null) return "";
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  };

  ns.FOCUSABLE = 'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  ns.byId = function (id) { return document.getElementById(id); };

  ns.clear = function (node) {
    if (!node) return;
    while (node.firstChild) node.removeChild(node.firstChild);
  };

  // A single, authoritative debounce helper so preview calls never stack.
  ns.debounce = function (fn, delay) {
    var timer = null;
    return function () {
      var args = Array.prototype.slice.call(arguments);
      if (timer) clearTimeout(timer);
      timer = setTimeout(function () { timer = null; fn.apply(null, args); }, delay);
    };
  };
})(window);
