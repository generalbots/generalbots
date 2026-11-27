/*
 * Security bulletin catalogue loader (#2716).
 *
 * Contract: every user-visible string in the bulletin pages is a data-i18n
 * key. Catalogues live in ../lang/<locale>.json next to this script's parent
 * directory, so pages may be mounted at any depth. PT-BR is the authoritative
 * locale; `en` is the second locale; the remaining catalogues may lag, but a
 * missing key always falls back through en -> pt-BR and never renders a
 * `[key]` placeholder. When no catalogue loads at all, the inline PT-BR text
 * authored in the HTML remains on screen.
 */
(function () {
  "use strict";

  var supported = ["pt-BR", "en", "es", "fr", "de", "it", "ru", "ja", "ko", "zh-CN"];
  var authoritative = "pt-BR";
  var fallback = "en";
  var storageKey = "gb-security-locale";

  function catalogBase() {
    var script = document.currentScript || document.querySelector("script[data-i18n-loader]");
    if (script && script.src) {
      return new URL("lang/", script.src);
    }
    return new URL("../lang/", document.baseURI);
  }

  function normalize(tag) {
    if (!tag) {
      return authoritative;
    }
    var value = String(tag).toLowerCase();
    if (value === "pt" || value.indexOf("pt-") === 0) {
      return "pt-BR";
    }
    for (var i = 0; i < supported.length; i += 1) {
      var candidate = supported[i].toLowerCase();
      if (value === candidate || value.indexOf(candidate + "-") === 0) {
        return supported[i];
      }
    }
    return authoritative;
  }

  function preferredLocale() {
    var fromQuery = new URLSearchParams(window.location.search).get("lang");
    if (fromQuery) {
      return normalize(fromQuery);
    }
    try {
      var stored = window.localStorage.getItem(storageKey);
      if (stored) {
        return normalize(stored);
      }
    } catch {
      /* storage disabled: fall back to the browser locale */
    }
    return normalize(window.navigator.language);
  }

  function fetchCatalog(locale, base) {
    return window.fetch(new URL(locale + ".json", base).toString())
      .then(function (response) {
        if (!response.ok) {
          throw new Error("catalogue unavailable: " + locale);
        }
        return response.json();
      })
      .catch(function () {
        return {};
      });
  }

  function applyAttributes(attribute, target, catalog) {
    var nodes = document.querySelectorAll("[" + attribute + "]");
    for (var i = 0; i < nodes.length; i += 1) {
      var value = catalog[nodes[i].getAttribute(attribute)];
      if (typeof value === "string" && value.length > 0 && value.charAt(0) !== "[") {
        nodes[i].setAttribute(target, value);
      }
    }
  }

  function markActive(locale) {
    var links = document.querySelectorAll("[data-locale]");
    for (var i = 0; i < links.length; i += 1) {
      if (links[i].getAttribute("data-locale") === locale) {
        links[i].setAttribute("aria-current", "true");
      } else {
        links[i].removeAttribute("aria-current");
      }
    }
  }

  function apply(catalog, locale) {
    var root = document.documentElement;
    root.lang = locale;
    root.setAttribute("data-locale", locale);
    root.setAttribute("data-authoritative-locale", authoritative);

    var nodes = document.querySelectorAll("[data-i18n]");
    for (var i = 0; i < nodes.length; i += 1) {
      var value = catalog[nodes[i].getAttribute("data-i18n")];
      if (typeof value === "string" && value.length > 0 && value.charAt(0) !== "[") {
        nodes[i].textContent = value;
      }
    }

    applyAttributes("data-i18n-aria-label", "aria-label", catalog);
    applyAttributes("data-i18n-title", "title", catalog);
    applyAttributes("data-i18n-placeholder", "placeholder", catalog);
    markActive(locale);
  }

  function start() {
    var locale = preferredLocale();
    try {
      window.localStorage.setItem(storageKey, locale);
    } catch {
      /* locale still works for this page through the ?lang= query string */
    }

    var base = catalogBase();
    var chain = [locale, fallback, authoritative].filter(function (item, index, all) {
      return all.indexOf(item) === index;
    });

    Promise.all(chain.map(function (item) {
      return fetchCatalog(item, base);
    })).then(function (catalogs) {
      var merged = {};
      for (var i = catalogs.length - 1; i >= 0; i -= 1) {
        Object.keys(catalogs[i]).forEach(function (key) {
          merged[key] = catalogs[i][key];
        });
      }
      apply(merged, locale);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
  } else {
    start();
  }
})();
