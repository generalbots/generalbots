"use strict";
/* Sheet advanced module: 10_i18n — the sheet UI translates through the shared
   runtime (window.i18n / botlib/locales) instead of a private dictionary
   (#2091). Keys live in the `sheet-*` / `action-*` namespaces. */

(function () {
  function paramsOf(fallbackOrParams, maybeParams) {
    if (fallbackOrParams && typeof fallbackOrParams === "object") return fallbackOrParams;
    if (maybeParams && typeof maybeParams === "object") return maybeParams;
    return null;
  }

  function t(key, fallbackOrParams, maybeParams) {
    const fallback = typeof fallbackOrParams === "string" ? fallbackOrParams : key;
    const params = paramsOf(fallbackOrParams, maybeParams);
    if (window.i18n && window.i18n.tOr) return window.i18n.tOr(key, fallback, params);
    return fallback;
  }

  function localize(root) {
    if (window.i18n && window.i18n.translatePage) {
      window.i18n.translatePage(root || document);
    }
  }

  function getLocale() {
    if (window.i18n && window.i18n.getLocale) return window.i18n.getLocale();
    return document.documentElement.getAttribute("lang") || "en";
  }

  function load(locale) {
    if (!window.i18n || !window.i18n.setLocale) {
      localize();
      return Promise.resolve();
    }
    return Promise.resolve(window.i18n.setLocale(locale)).then(function () {
      localize();
      applyDynamic();
      window.dispatchEvent(
        new CustomEvent("gb-sheet-locale", { detail: { locale: getLocale() } }),
      );
    });
  }

  function applyDynamic() {
    const nameInput = document.getElementById("sheetName");
    if (nameInput && (!nameInput.value || nameInput.value === "Untitled Spreadsheet")) {
      nameInput.value = t("sheet-untitled", "Untitled Spreadsheet");
    }
    const empty = document.querySelector("#sidebar-files .ss-empty-hint");
    if (empty) {
      empty.textContent = t(
        "sheet-none",
        "No spreadsheets yet. Create one to start.",
      );
    }
  }

  function boot() {
    localize();
    applyDynamic();
  }

  window.SheetI18n = {
    t: t,
    localize: localize,
    load: load,
    getLocale: getLocale,
    applyDynamic: applyDynamic,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
