/*
 * General Bots — i18n loader for the login (5000) and cloud (4000) surfaces
 * (#2090).
 *
 * These pages do not load the suite bundle (`/suite/js/i18n.js` assumes the
 * desktop shell and is not served by the login/cloud routers), so this file
 * shares the same API contract with a smaller footprint:
 *
 *   GET /api/i18n/:locale   -> { locale, translations: { key: value } }
 *   GET /api/i18n/locales   -> { locales: [...], default: "en" }
 *
 * It is served from `cloud/js/` on purpose: the login router serves `/js/*`
 * from `login/js` and falls back to `cloud/js`, so one copy covers both
 * surfaces.
 *
 * Supported attributes: data-i18n, data-i18n-html, data-i18n-placeholder,
 * data-i18n-title, data-i18n-aria-label, data-i18n-value and
 * data-i18n-document-title (on <html>).
 */
(function () {
  "use strict";

  var DEFAULT_LOCALE = "en";
  var STORAGE_KEY = "gb-locale";
  var CACHE_PREFIX = "gb-i18n-lite-";
  var CACHE_TTL_MS = 3600000;
  var RTL_LANGUAGES = ["ar", "he", "fa", "ur"];
  var FALLBACK_LOCALES = ["en", "pt-BR", "es", "zh-CN", "fr", "de", "ja", "ko"];

  var ATTRIBUTES = [
    "data-i18n",
    "data-i18n-html",
    "data-i18n-placeholder",
    "data-i18n-title",
    "data-i18n-aria-label",
    "data-i18n-value",
    "data-i18n-label"
  ];
  var SELECTOR = ATTRIBUTES.map(function (name) {
    return "[" + name + "]";
  }).join(", ");

  var supportedLocales = FALLBACK_LOCALES.slice();
  var currentLocale = DEFAULT_LOCALE;
  var translations = {};
  var initialized = false;

  function normalize(locale) {
    if (!locale) return null;
    if (supportedLocales.indexOf(locale) !== -1) return locale;
    var short = String(locale).split("-")[0];
    for (var i = 0; i < supportedLocales.length; i += 1) {
      if (supportedLocales[i].split("-")[0] === short) return supportedLocales[i];
    }
    return null;
  }

  function detectLocale() {
    var stored = null;
    try {
      stored = localStorage.getItem(STORAGE_KEY);
    } catch (e) {
      stored = null;
    }
    return normalize(stored) || normalize(navigator.language) || DEFAULT_LOCALE;
  }

  function readCache(locale) {
    try {
      var raw = localStorage.getItem(CACHE_PREFIX + locale);
      if (!raw) return null;
      var entry = JSON.parse(raw);
      if (Date.now() - entry.timestamp > CACHE_TTL_MS) return null;
      return entry.data || null;
    } catch (e) {
      return null;
    }
  }

  function writeCache(locale, data) {
    try {
      localStorage.setItem(
        CACHE_PREFIX + locale,
        JSON.stringify({ data: data, timestamp: Date.now() })
      );
    } catch (e) {
      /* private mode / quota — translation still works in memory */
    }
  }

  function requestHeaders() {
    return { Accept: "application/json", "Accept-Language": currentLocale };
  }

  function fetchTranslations(locale) {
    return fetch("/api/i18n/" + encodeURIComponent(locale), {
      headers: requestHeaders()
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (payload) {
        return (payload && payload.translations) || null;
      })
      .catch(function () {
        return null;
      });
  }

  function load(locale) {
    var cached = readCache(locale);
    if (cached) {
      translations = cached;
      currentLocale = locale;
      return Promise.resolve(true);
    }
    return fetchTranslations(locale).then(function (data) {
      if (data && Object.keys(data).length > 0) {
        translations = data;
        currentLocale = locale;
        writeCache(locale, data);
        return true;
      }
      if (locale !== DEFAULT_LOCALE) {
        return load(DEFAULT_LOCALE);
      }
      translations = {};
      return false;
    });
  }

  function t(key, params) {
    var text = translations[key];
    if (text === undefined || text === null) return key;
    if (params && typeof params === "object") {
      Object.keys(params).forEach(function (name) {
        text = text
          .split("{" + name + "}").join(String(params[name]))
          .split("{$" + name + "}").join(String(params[name]));
      });
    }
    return text;
  }

  function tOr(key, fallback, params) {
    var value = t(key, params);
    return value === key ? fallback : value;
  }

  function params(element) {
    var raw = element.getAttribute("data-i18n-params");
    if (!raw) return null;
    try {
      return JSON.parse(raw);
    } catch (e) {
      return null;
    }
  }

  function translateElement(element) {
    var map = [
      ["data-i18n", "textContent"],
      ["data-i18n-html", "innerHTML"],
      ["data-i18n-placeholder", "placeholder"],
      ["data-i18n-title", "title"],
      ["data-i18n-aria-label", "aria-label"],
      ["data-i18n-value", "value"],
      // <optgroup label> and <option label> carry visible text in an attribute.
      ["data-i18n-label", "label"]
    ];
    map.forEach(function (pair) {
      var key = element.getAttribute(pair[0]);
      if (!key) return;
      var value = t(key, params(element));
      if (value === key) return;
      if (pair[1] === "textContent") element.textContent = value;
      else if (pair[1] === "innerHTML") element.innerHTML = value;
      else element.setAttribute(pair[1], value);
    });
  }

  function translatePage(root) {
    var scope = root || document;
    if (scope.querySelectorAll) {
      Array.prototype.forEach.call(scope.querySelectorAll(SELECTOR), translateElement);
    }
    if (scope.hasAttribute && scope.matches && scope.matches(SELECTOR)) {
      translateElement(scope);
    }
    var titleKey = document.documentElement.getAttribute("data-i18n-document-title");
    if (titleKey) {
      var title = t(titleKey);
      if (title !== titleKey) document.title = title;
    }
  }

  function applyDocumentLocale(locale) {
    var primary = locale.split("-")[0];
    document.documentElement.setAttribute("lang", locale);
    document.documentElement.setAttribute(
      "dir",
      RTL_LANGUAGES.indexOf(primary) !== -1 ? "rtl" : "ltr"
    );
  }

  function announce(locale) {
    try {
      window.dispatchEvent(new CustomEvent("localeChanged", { detail: { locale: locale } }));
    } catch (e) {
      /* CustomEvent unavailable */
    }
  }

  function setLocale(locale) {
    var target = normalize(locale) || DEFAULT_LOCALE;
    try {
      localStorage.setItem(STORAGE_KEY, target);
    } catch (e) {
      /* storage unavailable — the switch still applies to this page */
    }
    return load(target).then(function () {
      applyDocumentLocale(currentLocale);
      translatePage();
      announce(currentLocale);
      return currentLocale;
    });
  }

  function formatDate(value, options) {
    var date = value instanceof Date ? value : new Date(value);
    if (isNaN(date.getTime())) return "";
    try {
      return new Intl.DateTimeFormat(currentLocale, options || {}).format(date);
    } catch (e) {
      return date.toISOString().slice(0, 10);
    }
  }

  function formatNumber(value, options) {
    try {
      return new Intl.NumberFormat(currentLocale, options || {}).format(value);
    } catch (e) {
      return String(value);
    }
  }

  function formatCurrency(value, currency, options) {
    var opts = options || {};
    opts.style = "currency";
    opts.currency = currency || "USD";
    try {
      return new Intl.NumberFormat(currentLocale, opts).format(value);
    } catch (e) {
      return String(value);
    }
  }

  function onReady(callback) {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", callback);
    } else {
      callback();
    }
  }

  function watchDom() {
    if (!document.body || typeof MutationObserver === "undefined") return;
    var observer = new MutationObserver(function (mutations) {
      mutations.forEach(function (mutation) {
        Array.prototype.forEach.call(mutation.addedNodes, function (node) {
          if (node.nodeType === Node.ELEMENT_NODE) translatePage(node);
        });
      });
    });
    observer.observe(document.body, { childList: true, subtree: true });
  }

  function init() {
    if (initialized) return Promise.resolve(currentLocale);
    initialized = true;
    return fetch("/api/i18n/locales", { headers: requestHeaders() })
      .then(function (response) {
        return response.ok ? response.json() : null;
      })
      .then(function (payload) {
        if (payload && Array.isArray(payload.locales) && payload.locales.length) {
          supportedLocales = payload.locales;
        }
      })
      .catch(function () {
        /* offline — the bundled list stays authoritative */
      })
      .then(function () {
        return load(detectLocale());
      })
      .then(function () {
        applyDocumentLocale(currentLocale);
        onReady(function () {
          translatePage();
          watchDom();
          announce(currentLocale);
        });
        return currentLocale;
      });
  }

  window.i18n = {
    t: t,
    tOr: tOr,
    init: init,
    setLocale: setLocale,
    getLocale: function () {
      return currentLocale;
    },
    getCurrentLocale: function () {
      return currentLocale;
    },
    getAvailableLocales: function () {
      return supportedLocales.slice();
    },
    translatePage: translatePage,
    translateElement: translateElement,
    formatDate: formatDate,
    formatNumber: formatNumber,
    formatCurrency: formatCurrency
  };

  // Same short alias the suite uses, so shared snippets keep working.
  window.gbT = tOr;

  init();
})();
