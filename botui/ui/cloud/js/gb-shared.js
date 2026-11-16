/*
 * gb-shared.js — single implementation of the frontend primitives every app
 * used to duplicate (issue #1740).
 *
 * Loaded before app scripts (`/suite/js/gb-shared.js` from the suite shell,
 * `/cloud/js/gb-shared.js` from the cloud pages). Pages that do not load this
 * file keep a local delegate whose fallback keeps them working standalone.
 *
 * Storage keys are intentionally unchanged: `gb-access-token` (canonical,
 * localStorage then sessionStorage) plus the documented legacy fallbacks
 * `token`, `id_token` and `management_token`. Token precedence matches the
 * implementations this kit replaces; do not reorder without checking callers.
 */
(function () {
  "use strict";

  if (window.GBShared && window.GBShared.html && window.GBShared.auth) return;

  var HTML_ESCAPES = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;"
  };

  // Escapes & < > " ' and preserves 0 / false (the `if (!value)` guards that
  // used to swallow them are gone on purpose).
  function escapeHtml(value) {
    return String(value == null ? "" : value).replace(/[&<>"']/g, function (ch) {
      return HTML_ESCAPES[ch];
    });
  }

  function read(storage, key) {
    try {
      return (storage && storage.getItem(key)) || "";
    } catch (error) {
      return "";
    }
  }

  function runtimeToken() {
    try {
      return (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) || "";
    } catch (error) {
      return "";
    }
  }

  // options.legacy     -> token, id_token before gb-access-token (vibe apps)
  // options.management -> management_token between local and session storage
  //                       (tasks autotask API)
  function token(options) {
    var opts = options || {};
    var found = runtimeToken();
    if (found) return found;

    if (opts.legacy) {
      found = read(window.localStorage, "token") || read(window.localStorage, "id_token");
      if (found) return found;
    }

    found = read(window.localStorage, "gb-access-token");
    if (found) return found;

    if (opts.management) {
      found = read(window.localStorage, "management_token");
      if (found) return found;
    }

    return read(window.sessionStorage, "gb-access-token");
  }

  function headers(extra, options) {
    var out = {};
    var key;
    var source = extra || {};
    for (key in source) {
      if (Object.prototype.hasOwnProperty.call(source, key)) out[key] = source[key];
    }
    var found = token(options);
    if (found) out.Authorization = "Bearer " + found;
    return out;
  }

  function activeLocale() {
    try {
      return (window.i18n && typeof window.i18n.getLocale === "function" && window.i18n.getLocale()) ||
        localStorage.getItem("gb-locale") ||
        navigator.language ||
        "en";
    } catch (e) {
      return "en";
    }
  }

  function toDate(value) {
    return value instanceof Date ? value : new Date(value);
  }

  function gbFormatDate(value, opts) {
    var d = toDate(value);
    if (!d || isNaN(d.getTime())) return "";
    var o = opts ? Object.assign({}, opts) : { year: "numeric", month: "2-digit", day: "2-digit" };
    var loc = o.locale || activeLocale();
    delete o.locale;
    try { return new Intl.DateTimeFormat(loc, o).format(d); } catch (e) { return d.toDateString(); }
  }

  function gbFormatTime(value, opts) {
    var d = toDate(value);
    if (!d || isNaN(d.getTime())) return "";
    var o = opts ? Object.assign({}, opts) : { hour: "2-digit", minute: "2-digit" };
    var loc = o.locale || activeLocale();
    delete o.locale;
    try { return new Intl.DateTimeFormat(loc, o).format(d); } catch (e) { return d.toTimeString().slice(0, 5); }
  }

  function gbFormatDateTime(value, opts) {
    var d = toDate(value);
    if (!d || isNaN(d.getTime())) return "";
    var o = opts ? Object.assign({}, opts) : { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" };
    var loc = o.locale || activeLocale();
    delete o.locale;
    try { return new Intl.DateTimeFormat(loc, o).format(d); } catch (e) { return d.toDateString() + " " + d.toTimeString().slice(0, 5); }
  }

  function gbFormatNumber(value, opts, locale) {
    var n = typeof value === "number" ? value : Number(value);
    if (!isFinite(n)) return String(value == null ? "" : value);
    var o = opts ? Object.assign({}, opts) : undefined;
    var loc = locale || (o && o.locale) || activeLocale();
    if (o) delete o.locale;
    try { return new Intl.NumberFormat(loc, o).format(n); } catch (e) { return String(n); }
  }

  var html = { escape: escapeHtml };
  var auth = { token: token, headers: headers };

  window.GBHtml = window.GBHtml || html;
  window.GBShared = { html: html, auth: auth, formatDate: gbFormatDate, formatTime: gbFormatTime, formatDateTime: gbFormatDateTime, formatNumber: gbFormatNumber };
  window.gbFormatDate = window.gbFormatDate || gbFormatDate;
  window.gbFormatTime = window.gbFormatTime || gbFormatTime;
  window.gbFormatDateTime = window.gbFormatDateTime || gbFormatDateTime;
  window.gbFormatNumber = window.gbFormatNumber || gbFormatNumber;
  // auth-service.js owns the rest of window.GBAuth; only add our two helpers.
  window.GBAuth = Object.assign(window.GBAuth || {}, auth);
  // Legacy global used by few remaining call sites; a page-local declaration
  // still hoists over this and wins, which is the pre-existing behaviour.
  if (typeof window.escapeHtml !== "function") window.escapeHtml = escapeHtml;
})();
