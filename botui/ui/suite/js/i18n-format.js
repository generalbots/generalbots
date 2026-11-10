/* Locale-aware formatters (#2087).
 *
 * Split out of i18n.js to keep both files under the 450-line limit. Loaded
 * right after the canonical loader; uses the active locale from window.i18n.
 */
(function () {
  "use strict";

  function activeLocale() {
    try {
      return (window.i18n && window.i18n.getLocale()) || "en";
    } catch (e) {
      return "en";
    }
  }

  function formatDate(value, options) {
    try {
      return new Intl.DateTimeFormat(activeLocale(), options).format(new Date(value));
    } catch (e) {
      return String(value);
    }
  }

  function formatNumber(value, options) {
    try {
      return new Intl.NumberFormat(activeLocale(), options).format(value);
    } catch (e) {
      return String(value);
    }
  }

  function formatCurrency(value, currency) {
    try {
      return new Intl.NumberFormat(activeLocale(), {
        style: "currency",
        currency: currency || "USD",
      }).format(value);
    } catch (e) {
      return String(value);
    }
  }

  function formatRelativeTime(value, unit) {
    try {
      return new Intl.RelativeTimeFormat(activeLocale(), { numeric: "auto" }).format(value, unit);
    } catch (e) {
      return String(value);
    }
  }

  if (window.i18n) {
    window.i18n.formatDate = formatDate;
    window.i18n.formatNumber = formatNumber;
    window.i18n.formatCurrency = formatCurrency;
    window.i18n.formatRelativeTime = formatRelativeTime;
  }
})();
