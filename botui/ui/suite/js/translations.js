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

  function translatePage() {
    var dict = getDictionary();
    if (!dict || typeof dict !== "object") return;
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      var key = el.getAttribute("data-i18n");
      if (key && dict[key]) el.textContent = dict[key];
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach(function (el) {
      var key = el.getAttribute("data-i18n-placeholder");
      if (key && dict[key]) el.setAttribute("placeholder", dict[key]);
    });
  }

  window.gbTranslations = {
    getLocale: getLocale,
    getDictionary: getDictionary,
    translatePage: translatePage,
  };
})();
