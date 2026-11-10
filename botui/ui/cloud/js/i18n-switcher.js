/*
 * General Bots — compact language switcher for the login (5000) and cloud
 * (4000) surfaces (#2090).
 *
 * Mount points are declared in markup so a page never hardcodes the widget:
 *
 *   <span data-i18n-switcher></span>
 *
 * The widget lists the supported locales by their native name, persists the
 * choice (localStorage `gb-locale`, written by i18n-lite.js) and re-renders the
 * page in place — no reload, no navigation.
 */
(function () {
  "use strict";

  var NATIVE_NAMES = {
    en: "English",
    "pt-BR": "Português (Brasil)",
    es: "Español",
    fr: "Français",
    de: "Deutsch",
    ja: "日本語",
    ko: "한국어",
    "zh-CN": "中文"
  };

  var SELECT_STYLE = [
    "font:inherit",
    "font-size:.7rem",
    "padding:2px 6px",
    "border-radius:6px",
    "border:1px solid currentColor",
    "background:transparent",
    "color:inherit",
    "cursor:pointer",
    "opacity:.75"
  ].join(";");

  function nameFor(locale) {
    return NATIVE_NAMES[locale] || locale;
  }

  function mount(container) {
    if (!container || container.getAttribute("data-i18n-switcher-ready") === "1") return;
    container.setAttribute("data-i18n-switcher-ready", "1");

    var api = window.i18n;
    var locales = api && api.getAvailableLocales ? api.getAvailableLocales() : ["en"];
    var current = api && api.getLocale ? api.getLocale() : "en";

    var select = document.createElement("select");
    select.className = "gb-locale-switcher";
    select.setAttribute("style", SELECT_STYLE);
    select.setAttribute("aria-label", "Language");
    select.setAttribute("data-i18n-aria-label", "language-switcher-label");

    locales.forEach(function (locale) {
      var option = document.createElement("option");
      option.value = locale;
      option.textContent = nameFor(locale);
      if (locale === current) option.selected = true;
      select.appendChild(option);
    });

    select.addEventListener("change", function () {
      if (api && api.setLocale) api.setLocale(select.value);
    });

    // Keep the control in sync when the locale changes from another surface
    // (settings page in the same tab, another widget instance).
    window.addEventListener("localeChanged", function (event) {
      var locale = event && event.detail && event.detail.locale;
      if (locale && select.value !== locale) select.value = locale;
    });

    container.textContent = "";
    container.appendChild(select);
  }

  function mountAll() {
    // Pages that do not declare a mount point get the switcher in the cloud
    // top bar automatically (every .mgmt-* page has one), so adding the widget
    // never requires touching markup twice.
    if (!document.querySelector("[data-i18n-switcher]")) {
      var actions = document.querySelector(".mgmt-topbar-actions");
      if (actions) {
        var slot = document.createElement("span");
        slot.setAttribute("data-i18n-switcher", "");
        actions.appendChild(slot);
      }
    }
    Array.prototype.forEach.call(
      document.querySelectorAll("[data-i18n-switcher]"),
      mount
    );
  }

  function start() {
    if (!document.body) return;
    mountAll();
    if (typeof MutationObserver !== "undefined") {
      new MutationObserver(function () {
        mountAll();
      }).observe(document.body, { childList: true, subtree: true });
    }
    window.addEventListener("localeChanged", function () {
      mountAll();
    });
  }

  window.GBLocaleSwitcher = { mount: mount, mountAll: mountAll };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
