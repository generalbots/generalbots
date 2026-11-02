/* Setup wizard i18n.
 *
 * The wizard must render with no server and no network, so it cannot call
 * /api/i18n. Each supported locale is a small dictionary under
 * wizard/i18n/<locale>.js loaded before this file. English is the fallback for
 * any key a dictionary does not define.
 */
(function () {
  "use strict";

  var STORAGE_KEY = "gb-locale";
  var DEFAULT_LOCALE = "en";
  var LOCALES = [
    { id: "en", label: "English" },
    { id: "pt-BR", label: "Português (Brasil)" },
    { id: "es", label: "Español" },
    { id: "fr", label: "Français" },
    { id: "de", label: "Deutsch" },
    { id: "ja", label: "日本語" },
    { id: "ko", label: "한국어" },
    { id: "zh-CN", label: "简体中文" }
  ];

  var dictionaries = window.GBWizardLocales || {};
  var current = DEFAULT_LOCALE;

  function normalize(locale) {
    if (!locale) { return DEFAULT_LOCALE; }
    if (dictionaries[locale]) { return locale; }
    var short = String(locale).split("-")[0];
    for (var i = 0; i < LOCALES.length; i += 1) {
      if (LOCALES[i].id.split("-")[0] === short) { return LOCALES[i].id; }
    }
    return DEFAULT_LOCALE;
  }

  function detect() {
    var stored = null;
    try { stored = localStorage.getItem(STORAGE_KEY); } catch (e) { stored = null; }
    if (stored) { return normalize(stored); }
    return normalize(navigator.language || navigator.userLanguage || DEFAULT_LOCALE);
  }

  function t(key, params) {
    var dict = dictionaries[current] || {};
    var text = dict[key];
    if (text == null) { text = (dictionaries[DEFAULT_LOCALE] || {})[key]; }
    if (text == null) { text = key; }
    if (params && typeof params === "object") {
      Object.keys(params).forEach(function (name) {
        text = text.split("{" + name + "}").join(String(params[name]));
      });
    }
    return text;
  }

  function setLocale(locale) {
    current = normalize(locale);
    try { localStorage.setItem(STORAGE_KEY, current); } catch (e) { /* storage may be unavailable */ }
    document.documentElement.setAttribute("lang", current);
    document.documentElement.setAttribute("dir", current === "ar" || current === "he" ? "rtl" : "ltr");
    window.dispatchEvent(new CustomEvent("wizard:localeChanged", {
      detail: { locale: current }
    }));
  }

  function getLocale() { return current; }

  function populate(select) {
    if (!select) { return; }
    select.innerHTML = LOCALES.map(function (locale) {
      return '<option value="' + locale.id + '">' + locale.label + "</option>";
    }).join("");
    select.value = current;
    select.addEventListener("change", function () { setLocale(select.value); });
  }

  current = detect();
  document.documentElement.setAttribute("lang", current);

  window.WizardI18n = {
    t: t,
    setLocale: setLocale,
    getLocale: getLocale,
    normalize: normalize,
    populate: populate,
    LOCALES: LOCALES
  };
})();
