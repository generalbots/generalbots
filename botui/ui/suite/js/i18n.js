(function () {
  "use strict";

  const DEFAULT_LOCALE = "en";

  // Languages rendered right-to-left; the document dir attribute follows.
  const RTL_LANGUAGES = ["ar", "he", "fa", "ur"];
  const STORAGE_KEY = "gb-locale";
  // #2087 — key carries the server build version; catalogue-only edits keep it.
  // #2923/#2935-smoke — explicit revision discards stale version-keyed entries exactly once.
  let cacheBuild = "dev";
  const CATALOG_REVISION = "r3";
  const CACHE_TTL_MS = 3600000;
  // Server list wins; the bundled one is only a first-paint fallback.
  let supportedLocales = ["en", "pt-BR", "es", "zh-CN", "fr", "de", "ja", "ko"];

  const MINIMAL_FALLBACK = {
    "label-loading": "Loading...",
    "status-error": "Error",
    "action-retry": "Retry",
  };

  let currentLocale = DEFAULT_LOCALE;
  let translations = {};
  let isInitialized = false;
  // #2542 — English is the last resort before the raw key; gaps logged once.
  let englishFallback = {};
  let fallbackLocale = DEFAULT_LOCALE;
  const missingKeysReported = new Set();
  // #2921 — early gbT() misses are held until the catalogue arrives.
  const earlyKeys = new Set();

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
    return `gb-i18n-cache-${locale}-${cacheBuild}-${CATALOG_REVISION}`;
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

  // #2921 — re-resolve early misses; only still-absent keys really warn.
  function reportMissing(key) {
    if (missingKeysReported.has(key)) return;
    missingKeysReported.add(key);
    console.warn(
      `i18n: Missing translation key: ${key} in ${currentLocale}` +
        (fallbackLocale !== currentLocale ? ` (English fallback has no entry either)` : ""),
    );
  }
  function resolveEarlyKeys() {
    earlyKeys.forEach((key) => {
      if (translations[key] === undefined && englishFallback[key] === undefined &&
          MINIMAL_FALLBACK[key] === undefined) reportMissing(key);
    });
    earlyKeys.clear();
  }

  function t(key, params) {
    let text = translations[key];
    if (text === undefined) text = englishFallback[key];
    if (text === undefined) text = MINIMAL_FALLBACK[key];
    if (text === undefined) {
      text = key;
      if (!isInitialized) earlyKeys.add(key);
      else reportMissing(key);
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

  // Authored fallback: t() returns the key on a miss, so DOM-building callers
  // use this helper to avoid leaking identifiers.
  function tOr(key, fallback, params) {
    const value = t(key, params);
    if (value !== key) return value;
    // Catalogue miss: substitute params into the authored fallback ourselves
    // so literal {param} tokens never leak (#2503,#2548). Null/undefined must
    // not render as words; other fallbacks stringify as before.
    if (fallback === null || fallback === undefined) return fallback;
    if (!params || typeof params !== "object") return String(fallback);
    // One function-replacer pass: {p}/{ $p } share a pattern, $&/$'/$$ in
    // values stay literal, inserted text is not rescanned (no double
    // substitution), and unmatched tokens are left as-is.
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
    // #2087 — best-effort server persistence; local switch still applies.
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
    // Keep the full BCP-47 tag on <html>; `dir` needs only the primary tag.
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

    // #2933 — capture the language the shell was rendered with, before the
    // overwrite below: the boot announce re-renders ~20 imperative surfaces.
    var renderedLang = document.documentElement.getAttribute("lang");

    // #2087 — build version keys the cache; both fetches are best-effort.
    let productVersion = null;
    try {
      const product = window.productConfig || (await (await fetch("/api/product")).json());
      if (product && product.version) {
        productVersion = String(product.version);
      }
    } catch (e) {
      /* offline: no build version is available */
    }
    if (productVersion) {
      cacheBuild = productVersion;
    } else {
      // #2921 — discard stale unversioned "dev" entries once; a versioned
      // boot keeps its cache untouched.
      cacheBuild = "dev";
      clearCache();
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

    // #2087 — without a local choice, prefer the server-persisted locale.
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
    resolveEarlyKeys();
    gbLog.debug(`i18n: Initialization complete, current locale: ${currentLocale}`);

    // Late-binding consumers (chips, rail) render their own DOM; announce
    // readiness after translatePage so DOMContentLoaded listeners are set.
    function announceLocale() {
      // No user action happened: every listener would render the same copy it
      // already did at boot. setLocale() still announces real switches.
      if (sameRenderedLocale(currentLocale, renderedLang)) return;
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

  // #2933 — compare primary tags only (pt-BR vs pt changes no copy).
  function sameRenderedLocale(locale, renderedLang) {
    if (!renderedLang) return false;
    return locale.split("-")[0].toLowerCase() === renderedLang.split("-")[0].toLowerCase();
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

  // Short alias for call sites that build their own DOM strings.
  window.gbT = tOr;

  init().catch((e) => console.error("i18n: Initialization failed", e));
})();
