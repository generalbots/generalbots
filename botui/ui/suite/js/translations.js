/* Global translations loader.
 *
 * Apps that need i18n load this file and call window.gbTranslations.translatePage().
 * Locale dictionaries may be injected per app; without one this stays a no-op
 * so missing translations never break the UI.
 */
(function () {
  "use strict";

  var LOCALE_KEY = "gb-locale";
  var DICT_KEY = "gb-translations";

  function getLocale() {
    return localStorage.getItem(LOCALE_KEY) || navigator.language || "en";
  }

  function getDictionary() {
    try {
      var raw = localStorage.getItem(DICT_KEY);
      if (raw) return JSON.parse(raw);
    } catch (_) { /* corrupted cache is ignored */ }
    return {};
  }

  function cacheDictionary(dict) {
    try {
      localStorage.setItem(DICT_KEY, JSON.stringify(dict));
    } catch (_) { /* storage full is ignored */ }
  }

  function applyDictionary(dict) {
    if (!dict || typeof dict !== "object") return;
    cacheDictionary(dict);
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      var key = el.getAttribute("data-i18n");
      if (key && dict[key]) el.textContent = dict[key];
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach(function (el) {
      var key = el.getAttribute("data-i18n-placeholder");
      if (key && dict[key]) el.setAttribute("placeholder", dict[key]);
    });
    document.querySelectorAll("[data-i18n-title]").forEach(function (el) {
      var key = el.getAttribute("data-i18n-title");
      if (key && dict[key]) el.setAttribute("title", dict[key]);
    });
  }

  function translatePage() {
    var dict = getDictionary();
    if (dict && Object.keys(dict).length > 0) {
      applyDictionary(dict);
      return;
    }
    // No local dictionary yet — fetch from the i18n API (the same source the
    // main i18n.js uses) so translations are never a silent no-op.
    fetch("/api/i18n/" + encodeURIComponent(getLocale()))
      .then(function (r) {
        if (!r.ok) throw new Error("i18n fetch failed");
        return r.json();
      })
      .then(function (data) {
        var dict = data && data.translations ? data.translations : data;
        if (dict && typeof dict === "object") applyDictionary(dict);
      })
      .catch(function (e) {
        console.warn("gbTranslations: failed to load dictionary", e);
      });
  }

  window.gbTranslations = {
    getLocale: getLocale,
    getDictionary: getDictionary,
    translatePage: translatePage,
  };
})();
