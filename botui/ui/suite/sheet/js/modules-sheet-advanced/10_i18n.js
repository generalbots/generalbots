"use strict";
/* Sheet advanced module: 10_i18n — localize static sheet UI strings from sheet/locales */

(function () {
  let strings = {};

  function detectLocale() {
    const stored = localStorage.getItem("gb-sheet-locale");
    if (stored) return stored;
    const lang = (navigator.language || "en").toLowerCase();
    if (lang.indexOf("pt") === 0) return "pt-BR";
    if (lang.indexOf("es") === 0) return "es";
    return "en";
  }

  function t(key, params) {
    let text = strings[key] || key;
    if (params && typeof params === "object") {
      Object.keys(params).forEach(function (p) {
        text = text.replace(new RegExp("\\{" + p + "\\}", "g"), params[p]);
      });
    }
    return text;
  }

  function localize(root) {
    const container = root || document;
    container.querySelectorAll("[data-i18n]").forEach(function (el) {
      el.textContent = t(el.getAttribute("data-i18n"));
    });
    container.querySelectorAll("[data-i18n-placeholder]").forEach(function (el) {
      el.setAttribute("placeholder", t(el.getAttribute("data-i18n-placeholder")));
    });
    container.querySelectorAll("[data-i18n-title]").forEach(function (el) {
      el.setAttribute("title", t(el.getAttribute("data-i18n-title")));
    });
  }

  function applyDynamic() {
    const nameInput = document.getElementById("sheetName");
    if (nameInput && (!nameInput.value || nameInput.value === "Untitled Spreadsheet")) {
      nameInput.value = t("file.untitled_sheet");
    }
    const empty = document.querySelector("#sidebar-files .ss-empty-hint");
    if (empty) empty.textContent = t("sidebar.none");
  }

  function load(locale) {
    return fetch("/suite/sheet/locales/" + locale + ".json")
      .then(function (r) {
        if (!r.ok) throw new Error("locale not found");
        return r.json();
      })
      .then(function (data) {
        strings = (data && data.strings) || {};
        localStorage.setItem("gb-sheet-locale", locale);
        localize();
        applyDynamic();
        window.dispatchEvent(new CustomEvent("gb-sheet-locale", { detail: { locale: locale } }));
      })
      .catch(function () {
        if (locale !== "en") return load("en");
        strings = {};
        localize();
        applyDynamic();
      });
  }

  window.SheetI18n = { t: t, localize: localize, load: load, getLocale: detectLocale };

  load(detectLocale());
})();