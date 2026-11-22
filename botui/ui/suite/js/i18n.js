(function () {
  "use strict";

  const DEFAULT_LOCALE = "en";

  // Languages rendered right-to-left; the document dir attribute follows.
  const RTL_LANGUAGES = ["ar", "he", "fa", "ur"];
  const STORAGE_KEY = "gb-locale";
  // #2087 — the cache key carries the server build version instead of a manual
  // constant, so a deploy that changes the catalogue invalidates every client
  // cache without anyone remembering to bump a number.
  let cacheBuild = "dev";
  const CACHE_TTL_MS = 3600000;
  // Server-provided locale list (the catalogue is the source of truth); the
  // bundled list is only a first-paint fallback before /api/i18n/locales.
  let supportedLocales = ["en", "pt-BR", "es", "zh-CN", "fr", "de", "ja", "ko"];

  const MINIMAL_FALLBACK = {
    "label-loading": "Loading...",
    "status-error": "Error",
    "action-retry": "Retry",
  };

  let currentLocale = DEFAULT_LOCALE;
  let translations = {};
  let isInitialized = false;
  // #2542 — the English catalogue is the last resort before the raw key: a
  // string the active locale does not ship renders in English instead of an
  // identifier. `fallbackLocale` names the catalogue that fills the gaps, and
  // every gap is logged once so it can be translated.
  let englishFallback = {};
  let fallbackLocale = DEFAULT_LOCALE;
  const missingKeysReported = new Set();

  function detectBrowserLocale() {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return stored;
    }

    const browserLang =
      navigator.language || navigator.userLanguage || DEFAULT_LOCALE;
    const shortLang = browserLang.split("-")[0];

    if (supportedLocales.includes(browserLang)) {
      return browserLang;
    }

    const match = supportedLocales.find((loc) => loc.startsWith(shortLang));
    return match || DEFAULT_LOCALE;
  }

  function getCacheKey(locale) {
    return `gb-i18n-cache-${locale}-${cacheBuild}`;
  }

  function getCachedTranslations(locale) {
    try {
      const cached = localStorage.getItem(getCacheKey(locale));
      if (cached) {
        const { data, timestamp } = JSON.parse(cached);
        const age = Date.now() - timestamp;
        gbLog.debug(`i18n: Cache check for ${locale}: age=${Math.round(age/1000)}s, valid=${age < CACHE_TTL_MS}, keys=${Object.keys(data || {}).length}`);
        if (age < CACHE_TTL_MS) {
          return data;
        }
      }
    } catch (e) {
      console.warn("i18n: Failed to read cache", e);
    }
    return null;
  }

  function setCachedTranslations(locale, data) {
    try {
      localStorage.setItem(
        getCacheKey(locale),
        JSON.stringify({
          data,
          timestamp: Date.now(),
        }),
      );
    } catch (e) {
      console.warn("i18n: Failed to write cache", e);
    }
  }

  async function fetchTranslations(locale) {
    try {
      gbLog.debug(`i18n: Fetching translations for locale: ${locale}`);
      const response = await fetch(`/api/i18n/${locale}`, {
        headers: { Accept: "application/json" },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const result = await response.json();
      gbLog.debug(`i18n: Loaded ${Object.keys(result.translations || {}).length} translations for ${locale}`);
      return result.translations || {};
    } catch (e) {
      console.error(`i18n: Failed to fetch translations for ${locale}`, e);
      return null;
    }
  }

  async function loadTranslations(locale) {
    gbLog.debug(`i18n: loadTranslations called for locale: ${locale}`);
    const cached = getCachedTranslations(locale);
    if (cached) {
      gbLog.debug(`i18n: Using cached translations for ${locale}`);
      translations = cached;
      currentLocale = locale;
      return true;
    }

    gbLog.debug(`i18n: Cache miss, fetching from API for ${locale}`);
    const fetched = await fetchTranslations(locale);
    if (fetched && Object.keys(fetched).length > 0) {
      translations = fetched;
      currentLocale = locale;
      setCachedTranslations(locale, fetched);
      if (locale !== DEFAULT_LOCALE) {
        await loadFallbackTranslations();
      }
      return true;
    }

    if (locale !== DEFAULT_LOCALE) {
      console.warn(`i18n: Falling back to ${DEFAULT_LOCALE}`);
      return loadTranslations(DEFAULT_LOCALE);
    }

    console.warn(`i18n: No translations found, using minimal fallback`);
    translations = MINIMAL_FALLBACK;
    return false;
  }

  // Loads (and caches) the English catalogue underneath the active locale.
  async function loadFallbackTranslations() {
    const cached = getCachedTranslations(DEFAULT_LOCALE);
    if (cached) {
      englishFallback = cached;
      return;
    }
    const fetched = await fetchTranslations(DEFAULT_LOCALE);
    if (fetched && Object.keys(fetched).length > 0) {
      englishFallback = fetched;
      setCachedTranslations(DEFAULT_LOCALE, fetched);
    }
  }

  function t(key, params) {
    let text = translations[key];
    if (text === undefined) text = englishFallback[key];
    if (text === undefined) text = MINIMAL_FALLBACK[key];
    if (text === undefined) {
      text = key;
      if (!missingKeysReported.has(key)) {
        missingKeysReported.add(key);
        console.warn(
          `i18n: Missing translation key: ${key} in ${currentLocale}` +
            (fallbackLocale !== currentLocale ? ` (English fallback has no entry either)` : ""),
        );
      }
    }

    if (params && typeof params === "object") {
      Object.keys(params).forEach((param) => {
        text = text.replace(
          new RegExp(`\\{\\s*\\$?${param}\\s*\\}`, "g"),
          params[param],
        );
        text = text.replace(
          new RegExp(`\\{\\s*${param}\\s*\\}`, "g"),
          params[param],
        );
      });
    }

    return text;
  }

  // Shared with i18n-elements.js (element/page translation).
  window.__gbI18nShared = { t: t };

  // Translation with an authored fallback for dynamically built strings.
  // `t()` intentionally returns the key when the catalogue is missing it; UI
  // code that renders its own DOM should use this helper so a missing
  // translation never leaks an identifier into the interface.
  function tOr(key, fallback, params) {
    const value = t(key, params);
    if (value !== key) return value;
    // Catalogue miss (e.g. backend down before i18n loads): t() returns the
    // bare key, so substitute params into the authored fallback ourselves —
    // otherwise literal {param} tokens leak into the DOM (#2503,#2548).
    // A missing null/undefined fallback must never render the literal words
    // "null"/"undefined"; other non-string fallbacks stringify as before.
    if (fallback === null || fallback === undefined) return fallback;
    if (!params || typeof params !== "object") return String(fallback);
    // One pass with a function replacer:
    //  - {p} and { $p } share one pattern and any param name is accepted;
    //  - a function replacement never interprets $&, $', $$ inside a value
    //    (real callers pass e.g. { amount: '$' + total.toFixed(2) });
    //  - inserted text is not re-scanned, so a value like "{q}" cannot be
    //    substituted again by a later param (no double substitution);
    //  - params with no matching key are left literal.
    return String(fallback).replace(
      /\{\s*\$?([^{}]+?)\s*\}/g,
      (token, name) =>
        Object.prototype.hasOwnProperty.call(params, name)
          ? String(params[name])
          : token,
    );
  }


  async function setLocale(locale) {
    if (locale === currentLocale && isInitialized) {
      return;
    }

    localStorage.setItem(STORAGE_KEY, locale);
    // #2087 — persist the choice server-side so other devices and
    // server-rendered messages follow it. Best effort: an unauthenticated
    // visitor still gets the local switch.
    try {
      fetch("/api/i18n/preference", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ locale: locale }),
      }).catch(function (err) {
        /* Best-effort locale persistence; local preference is already applied. */
        console.debug("[gb-i18n] Locale persistence failed:", err);
      });
    } catch (e) {
      /* persistence is optional */
    }
    await loadTranslations(locale);
    window.__gbI18nShared.translatePage();

    const lang = locale.split("-")[0];
    // Keep the full BCP-47 tag on <html> so font/line-breaking and assistive
    // technology can distinguish e.g. pt-BR from pt-PT; `dir` only needs the
    // primary language.
    document.documentElement.setAttribute("lang", locale);
    document.documentElement.setAttribute("dir", RTL_LANGUAGES.includes(lang) ? "rtl" : "ltr");

    window.dispatchEvent(
      new CustomEvent("localeChanged", {
        detail: { locale: currentLocale },
      }),
    );
    window.dispatchEvent(
      new CustomEvent("i18n:localeChanged", {
        detail: { locale: currentLocale },
      }),
    );
  }

  function setupBodyListeners() {
    if (!document.body) {
      return;
    }

    document.body.addEventListener("htmx:afterSwap", (event) => {
      window.__gbI18nShared.translatePage(event.detail.target);
    });

    document.body.addEventListener("htmx:afterSettle", (event) => {
      window.__gbI18nShared.translatePage(event.detail.target);
    });

    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            if (
              node.hasAttribute &&
              (node.hasAttribute("data-i18n") ||
                node.hasAttribute("data-i18n-html") ||
                node.hasAttribute("data-i18n-placeholder") ||
                node.hasAttribute("data-i18n-title") ||
                node.hasAttribute("data-i18n-aria-label") ||
                node.hasAttribute("data-i18n-value"))
            ) {
              window.__gbI18nShared.translateElement(node);
            }
            if (node.querySelectorAll) {
              window.__gbI18nShared.translatePage(node);
            }
          }
        });
      });
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  }

  async function init() {
    gbLog.debug("i18n: Initialization started");
    if (isInitialized) {
      gbLog.debug("i18n: Already initialized, skipping");
      return;
    }

    // #2087 — the build version keys the cache and the catalogue owns the
    // locale list; both are best-effort (offline boot keeps the fallbacks).
    try {
      const product = window.productConfig || (await (await fetch("/api/product")).json());
      if (product && product.version) {
        cacheBuild = String(product.version);
      }
    } catch (e) {
      /* offline: keep the dev cache key */
    }
    try {
      const response = await fetch("/api/i18n/locales");
      if (response.ok) {
        const data = await response.json();
        if (data && Array.isArray(data.locales) && data.locales.length > 0) {
          supportedLocales = data.locales;
        }
      }
    } catch (e) {
      /* offline: keep the bundled list */
    }

    // #2087 — when this device has no explicit choice, prefer the locale the
    // user persisted server-side (settings, another device, a server-rendered
    // email) before falling back to the browser language.
    let locale = null;
    try {
      if (!localStorage.getItem(STORAGE_KEY)) {
        const pref = await fetch("/api/i18n/preference", {
          headers: { Accept: "application/json" },
        });
        if (pref.ok) {
          const data = await pref.json();
          if (data && data.locale) {
            locale = data.locale;
          }
        }
      }
    } catch (e) {
      /* unauthenticated or offline — the browser language wins */
    }
    if (!locale) {
      locale = detectBrowserLocale();
    }
    gbLog.debug(`i18n: Detected locale: ${locale}`);
    await loadTranslations(locale);

    const lang = currentLocale.split("-")[0];
    document.documentElement.setAttribute("lang", currentLocale);
    document.documentElement.setAttribute("dir", RTL_LANGUAGES.includes(lang) ? "rtl" : "ltr");

    isInitialized = true;
    gbLog.debug(`i18n: Initialization complete, current locale: ${currentLocale}`);

    // Late-binding consumers (sidebar sections, generated chips) render their
    // own DOM and cannot be reached by data-i18n attributes. Announce that
    // the catalogue is ready AFTER the DOM translation pass so listeners that
    // boot on DOMContentLoaded are already registered.
    function announceLocale() {
      try {
        window.dispatchEvent(new CustomEvent("localeChanged", {
          detail: { locale: currentLocale },
        }));
      } catch (e) { /* CustomEvent unavailable */ }
    }

    if (document.readyState === "loading") {
      (function(){ var __cb = () => {
        window.__gbI18nShared.translatePage();
        setupBodyListeners();
        setTimeout(announceLocale, 0);
      }; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
    } else {
      window.__gbI18nShared.translatePage();
      setupBodyListeners();
      setTimeout(announceLocale, 0);
    }
  }

  async function getAvailableLocales() {
    try {
      const response = await fetch("/api/i18n/locales");
      if (response.ok) {
        const data = await response.json();
        return data.locales || ["en"];
      }
    } catch (e) {
      console.warn("i18n: Failed to fetch available locales", e);
    }
    return ["en", "pt-BR", "es", "zh-CN", "fr", "de", "ja", "ko"];
  }

  function getCurrentLocale() {
    return currentLocale;
  }

  function clearCache() {
    const keys = Object.keys(localStorage);
    keys.forEach((key) => {
      if (key.startsWith("gb-i18n-cache-")) {
        localStorage.removeItem(key);
      }
    });
    gbLog.debug("i18n: Cleared all translation caches");
  }

  window.i18n = {
    t,
    tOr,
    init,
    setLocale,
    getLocale: getCurrentLocale,
    getCurrentLocale,
    getFallbackLocale: () => fallbackLocale,
    getAvailableLocales,
    clearCache,
  };

  // Short alias for call sites that build DOM strings outside a translated
  // element (window chrome, launcher, spotlights, toasts).
  window.gbT = tOr;

  init().catch((e) => console.error("i18n: Initialization failed", e));
})();
