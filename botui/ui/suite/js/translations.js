/* Compatibility shim (#2087).
 *
 * Settings used to load this file, which kept its own dictionary cache
 * (`gb-translations`) and its own locale read while the shell loaded
 * `/suite/js/i18n.js` with a different cache (`gb-i18n-cache-*`). Two loaders
 * meant two caches, two locale sources and pages that disagreed about the
 * active language.
 *
 * The canonical loader is `i18n.js` (window.i18n / window.gbT). This file now
 * only exists for callers that still load it: it delegates to the canonical
 * loader, injecting it when the page did not already.
 */
(function () {
  "use strict";

  var CANONICAL_SRC = "/suite/js/i18n.js";
  var pending = [];
  var loading = false;

  function canonicalReady() {
    return !!(window.i18n && typeof window.i18n.translatePage === "function");
  }

  function runWhenReady(callback) {
    if (canonicalReady()) {
      callback();
      return;
    }
    pending.push(callback);
    if (loading) return;
    loading = true;
    var script = document.createElement("script");
    script.src = CANONICAL_SRC;
    script.onload = function () {
      loading = false;
      var queue = pending.slice();
      pending.length = 0;
      queue.forEach(function (cb) {
        try {
          cb();
        } catch (e) {
          console.warn("gbTranslations: delegated call failed", e);
        }
      });
    };
    script.onerror = function () {
      loading = false;
      console.warn("gbTranslations: could not load the canonical i18n loader");
    };
    document.head.appendChild(script);
  }

  function getLocale() {
    if (window.i18n && typeof window.i18n.getLocale === "function") {
      return window.i18n.getLocale();
    }
    try {
      return localStorage.getItem("gb-locale") || navigator.language || "en";
    } catch (e) {
      return "en";
    }
  }

  function translatePage() {
    runWhenReady(function () {
      window.i18n.translatePage();
    });
  }

  function getDictionary() {
    if (window.i18n && typeof window.i18n.getDictionary === "function") {
      return window.i18n.getDictionary();
    }
    return {};
  }

  window.gbTranslations = {
    getLocale: getLocale,
    getDictionary: getDictionary,
    translatePage: translatePage,
  };
})();
