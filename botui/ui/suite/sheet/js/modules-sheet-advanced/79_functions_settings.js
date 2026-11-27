"use strict";
/* Issue #2434 - calculation settings for the local preview. Stored only in
 * workbook-scoped localStorage; nothing here is sent to the server or written
 * into the workbook file. */
(function () {
  var LOCALES = ["en", "pt-BR", "es", "fr", "de", "it", "ru", "ja", "ko", "zh-CN"];
  var SEPARATORS = {
    en: { decimal: ".", group: "," },
    "pt-BR": { decimal: ",", group: "." },
    es: { decimal: ",", group: "." },
    fr: { decimal: ",", group: " " },
    de: { decimal: ",", group: "." },
    it: { decimal: ",", group: "." },
    ru: { decimal: ",", group: " " },
    ja: { decimal: ".", group: "," },
    ko: { decimal: ".", group: "," },
    "zh-CN": { decimal: ".", group: "," },
  };

  function t(key, fallback) {
    if (window.gbT) return window.gbT(key, fallback);
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key, fallback);
    return fallback;
  }

  function sheetId() {
    try {
      if (window.SheetCore && typeof window.SheetCore.currentSheetId === "function") {
        var id = window.SheetCore.currentSheetId();
        if (id) return String(id);
      }
      if (window.__LOADED_SHEET && window.__LOADED_SHEET.id) return String(window.__LOADED_SHEET.id);
    } catch (error) {
      return "current";
    }
    return "current";
  }

  function storageKey() { return "gb-sheet-functions:" + sheetId(); }

  function appLocale() {
    if (window.SheetI18n && window.SheetI18n.getLocale) {
      var locale = window.SheetI18n.getLocale();
      if (LOCALES.indexOf(locale) >= 0) return locale;
    }
    return document.documentElement && document.documentElement.getAttribute("lang") === "pt-BR" ? "pt-BR" : "en";
  }

  function defaults() {
    var locale = appLocale();
    return {
      version: 1,
      localOnly: true,
      mode: "auto",
      iterative: { enabled: false, maxIterations: 100, tolerance: 1e-7 },
      precisionAsDisplayed: false,
      dateSystem: "1900",
      locale: locale,
      separators: { decimal: SEPARATORS[locale].decimal, group: SEPARATORS[locale].group },
    };
  }

  function normalize(raw) {
    var base = defaults();
    if (!raw || typeof raw !== "object") return base;
    base.mode = raw.mode === "manual" ? "manual" : "auto";
    base.precisionAsDisplayed = Boolean(raw.precisionAsDisplayed);
    base.dateSystem = raw.dateSystem === "1904" ? "1904" : "1900";
    if (LOCALES.indexOf(raw.locale) >= 0) {
      base.locale = raw.locale;
      base.separators = { decimal: SEPARATORS[raw.locale].decimal, group: SEPARATORS[raw.locale].group };
    }
    if (raw.iterative && typeof raw.iterative === "object") {
      base.iterative.enabled = Boolean(raw.iterative.enabled);
      var max = Number(raw.iterative.maxIterations);
      base.iterative.maxIterations = Number.isFinite(max) ? Math.min(1000, Math.max(1, Math.floor(max))) : 100;
      var tolerance = Number(raw.iterative.tolerance);
      base.iterative.tolerance = Number.isFinite(tolerance) && tolerance > 0 ? tolerance : 1e-7;
    }
    if (raw.separators && typeof raw.separators === "object") {
      var decimal = String(raw.separators.decimal || "").slice(0, 1);
      var group = String(raw.separators.group || "").slice(0, 1);
      if (decimal && group && decimal !== group) base.separators = { decimal: decimal, group: group };
    }
    return base;
  }

  function state() {
    try {
      var raw = localStorage.getItem(storageKey());
      return raw ? normalize(JSON.parse(raw)) : defaults();
    } catch (error) {
      return defaults();
    }
  }

  function save(next) {
    try {
      localStorage.setItem(storageKey(), JSON.stringify(next));
      return true;
    } catch (error) {
      return false;
    }
  }

  function mutate(fn) {
    var current = state();
    var result = fn(current);
    save(current);
    return result;
  }

  function setLocale(locale) {
    if (LOCALES.indexOf(locale) < 0) return false;
    mutate(function (current) {
      current.locale = locale;
      current.separators = { decimal: SEPARATORS[locale].decimal, group: SEPARATORS[locale].group };
    });
    window.dispatchEvent(new CustomEvent("gb-sheet-functions-locale", { detail: { locale: locale } }));
    if (window.SheetI18n && window.SheetI18n.load) window.SheetI18n.load(locale);
    return true;
  }

  function applyForm(form) {
    var next = normalize({
      mode: form.mode,
      iterative: {
        enabled: form.iterative,
        maxIterations: form.maxIterations,
        tolerance: form.tolerance,
      },
      precisionAsDisplayed: form.precisionAsDisplayed,
      dateSystem: form.dateSystem,
      locale: form.locale,
      separators: { decimal: form.decimal, group: form.group },
    });
    return save(next) ? next : null;
  }

  window.SheetFunctionsSettings = {
    LOCALES: LOCALES, SEPARATORS: SEPARATORS, t: t, sheetId: sheetId,
    storageKey: storageKey, defaults: defaults, normalize: normalize,
    state: state, save: save, mutate: mutate, setLocale: setLocale,
    applyForm: applyForm,
  };
})();
