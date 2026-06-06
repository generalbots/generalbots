// botui/ui/suite/sheet/modules/00e_i18n.js
// Lightweight i18n system. No dependencies, no build step.
//
// Usage:
//   <span data-i18n="common.save">Save</span>
//   <span data-i18n-title="a11y.modal.close">×</span>
//   <span data-i18n-aria="a11y.grid">grid</span>
//
//   const s = window.I18n.t("common.save");  // "Save" or "Salvar"
//   const formatted = window.I18n.t("modal.matches_found", 5);  // "5 matches found"
//
//   window.I18n.setLocale("pt-BR");
//   window.I18n.loadLocale("pt-BR", { ...strings... });
//
// Locales are loaded from {suite}/locales/{lang}.json via fetch.
// Falls back to en-US for missing keys, then to the key itself.
"use strict";

(function () {
  const DEFAULT_LOCALE = "en-US";
  const STORAGE_KEY = "gb.i18n.locale";

  let currentLocale = DEFAULT_LOCALE;
  let catalogs = {};
  let listeners = [];

  function getStoredLocale() {
    try {
      return localStorage.getItem(STORAGE_KEY) || DEFAULT_LOCALE;
    } catch (e) {
      return DEFAULT_LOCALE;
    }
  }

  function setStoredLocale(locale) {
    try {
      localStorage.setItem(STORAGE_KEY, locale);
    } catch (e) {
      // localStorage may be disabled — non-fatal
    }
  }

  function detectLocaleFromBrowser() {
    const nav = (navigator && (navigator.language || navigator.userLanguage)) || "";
    if (!nav) return DEFAULT_LOCALE;
    const lower = nav.toLowerCase();
    if (lower.startsWith("pt")) return "pt-BR";
    if (lower.startsWith("en")) return "en-US";
    if (lower.startsWith("es")) return "es-ES";
    return DEFAULT_LOCALE;
  }

  function format(template, args) {
    if (typeof template !== "string") return template;
    if (!args || args.length === 0) return template;
    return template.replace(/\{(\d+)\}/g, function (_, idx) {
      const i = parseInt(idx, 10);
      return args[i] !== undefined ? String(args[i]) : "{" + idx + "}";
    });
  }

  function lookup(key, args) {
    const cat = catalogs[currentLocale] || {};
    let val = cat[key];
    if (val === undefined && currentLocale !== DEFAULT_LOCALE) {
      val = (catalogs[DEFAULT_LOCALE] || {})[key];
    }
    if (val === undefined) return key;
    return format(val, args);
  }

  function t(key) {
    const args = Array.prototype.slice.call(arguments, 1);
    return lookup(key, args);
  }

  function loadCatalog(locale, data) {
    if (!data || typeof data !== "object") return;
    const strings = data.strings || data;
    catalogs[locale] = Object.assign({}, catalogs[locale] || {}, strings);
  }

  function applyToDOM() {
    if (!document.querySelector) return;
    const nodes = document.querySelectorAll("[data-i18n]");
    let n = 0;
    for (let i = 0; i < nodes.length; i++) {
      const el = nodes[i];
      const key = el.getAttribute("data-i18n");
      if (!key) continue;
      el.textContent = t(key);
      n++;
    }
    const titleNodes = document.querySelectorAll("[data-i18n-title]");
    for (let i = 0; i < titleNodes.length; i++) {
      const el = titleNodes[i];
      const key = el.getAttribute("data-i18n-title");
      if (!key) continue;
      el.setAttribute("title", t(key));
      n++;
    }
    const ariaNodes = document.querySelectorAll("[data-i18n-aria]");
    for (let i = 0; i < ariaNodes.length; i++) {
      const el = ariaNodes[i];
      const key = el.getAttribute("data-i18n-aria");
      if (!key) continue;
      el.setAttribute("aria-label", t(key));
      n++;
    }
    return n;
  }

  function notifyListeners() {
    for (let i = 0; i < listeners.length; i++) {
      try {
        listeners[i](currentLocale);
      } catch (e) {
        // Listener error must not break i18n
      }
    }
  }

  function setLocale(locale) {
    if (!locale || locale === currentLocale) return;
    currentLocale = locale;
    setStoredLocale(locale);
    notifyListeners();
    applyToDOM();
  }

  function getLocale() {
    return currentLocale;
  }

  function getAvailableLocales() {
    return Object.keys(catalogs);
  }

  function onChange(fn) {
    if (typeof fn !== "function") return function () {};
    listeners.push(fn);
    return function off() {
      const idx = listeners.indexOf(fn);
      if (idx >= 0) listeners.splice(idx, 1);
    };
  }

  function detectLocalePath() {
    // Find which suite we are in by checking for known classes
    if (document.querySelector(".sheet-app")) return "sheet/locales";
    if (document.querySelector(".docs-app")) return "docs/locales";
    if (document.querySelector(".slides-app")) return "slides/locales";
    return "locales";
  }

  function loadLocale(locale, data) {
    // Synchronous variant
    loadCatalog(locale, data);
    if (locale === currentLocale) {
      applyToDOM();
      notifyListeners();
    }
    return true;
  }

  function fetchLocale(locale) {
    if (!window.fetch) {
      return Promise.resolve(false);
    }
    const base = detectLocalePath();
    const url = base + "/" + locale + ".json";
    return fetch(url, { credentials: "same-origin" })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        loadCatalog(locale, data);
        if (locale === currentLocale) {
          applyToDOM();
          notifyListeners();
        }
        return true;
      })
      .catch(function () {
        return false;
      });
  }

  function init() {
    currentLocale = getStoredLocale() || detectLocaleFromBrowser();
    // Load default + current in parallel
    const promises = [];
    if (currentLocale !== DEFAULT_LOCALE) {
      promises.push(fetchLocale(DEFAULT_LOCALE));
    }
    promises.push(fetchLocale(currentLocale));
    return Promise.all(promises).then(function () {
      applyToDOM();
      notifyListeners();
    });
  }

  // Expose
  window.I18n = {
    t: t,
    setLocale: setLocale,
    getLocale: getLocale,
    getAvailableLocales: getAvailableLocales,
    loadLocale: loadLocale,
    fetchLocale: fetchLocale,
    onChange: onChange,
    applyToDOM: applyToDOM,
    init: init,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
