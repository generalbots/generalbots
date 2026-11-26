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

  // #2301 — a caught failure has to leave a trace. Cloud pages used empty
  // `catch (e) { window.gbLogError ? gbLogError(e, "gbFormatNumber") : console.error("gbFormatNumber", e); }` blocks too; this logs once per context per 30 s and posts
  // to the client error endpoint when the page is signed in.
  var reportedAt = {};
  var REPORT_WINDOW_MS = 30000;

  function gbLogError(error, context) {
    var where = context || "unknown";
    var message = error && error.message ? error.message : String(error);
    var now = Date.now();
    var repeated = false;
    try {
      repeated = reportedAt[where] && now - reportedAt[where] < REPORT_WINDOW_MS;
      reportedAt[where] = now;
    } catch (e) {
      repeated = false;
    }
    if (!repeated && typeof console !== "undefined" && console.error) {
      console.error("[gb] " + where + ": " + message, error);
    }
    if (repeated) return error;
    try {
      var body = JSON.stringify({
        errors: [{
          type: (error && error.name) || "Error",
          message: message,
          stack: error && error.stack,
          url: window.location.href,
          timestamp: new Date().toISOString(),
          context: { context: where }
        }]
      });
      fetch("/api/client-errors", {
        method: "POST",
        headers: headers({ "Content-Type": "application/json" }),
        body: body
      }).catch(function (sendError) {
        if (console && console.warn) console.warn("[gb] error report failed", sendError);
      });
    } catch (e) {
      if (console && console.debug) console.debug("[gb] error report unavailable", e);
    }
    return error;
  }

  // ---------------------------------------------------------------------
  // #2297 — in-page dialogs. Native alert()/confirm()/prompt() block the whole
  // tab, ignore the app's theme and language, and are indistinguishable from a
  // hostile popup. One overlay replaces all three. `alert` stays
  // fire-and-forget; `confirm`/`prompt` return Promises, so callers await them.
  // ---------------------------------------------------------------------
  var DIALOG_CSS_ID = "gb-dialog-css";
  var DIALOG_ID = "gb-dialog";

  function dialogFallback(key, text) {
    return typeof window.gbT === "function" ? window.gbT(key, text) : text;
  }

  function ensureDialogCss() {
    if (document.getElementById(DIALOG_CSS_ID)) return;
    var style = document.createElement("style");
    style.id = DIALOG_CSS_ID;
    style.textContent = [
      "#" + DIALOG_ID + "{position:fixed;inset:0;z-index:100000;display:flex;",
      "align-items:center;justify-content:center;background:color-mix(in srgb, var(--palette-000000) 45%, transparent)}",
      "#" + DIALOG_ID + " .gb-dialog-box{background:var(--card,var(--id-surface-solid));",
      "color:var(--text,var(--palette-000000));border:1px solid var(--border,var(--palette-d1d5db));",
      "border-radius:12px;max-width:min(420px,92vw);width:100%;padding:1.25rem;",
      "box-shadow:0 18px 48px color-mix(in srgb, var(--palette-000000) 28%, transparent)}",
      "#" + DIALOG_ID + " h2{margin:0 0 .6rem;font-size:1rem;font-weight:700}",
      "#" + DIALOG_ID + " p{margin:0 0 1rem;font-size:.9rem;line-height:1.45;",
      "white-space:pre-wrap;word-break:break-word}",
      "#" + DIALOG_ID + " input{width:100%;box-sizing:border-box;padding:.55rem .7rem;",
      "border:1px solid var(--border,var(--palette-d1d5db));border-radius:8px;margin-bottom:1rem;font:inherit}",
      "#" + DIALOG_ID + " .gb-dialog-actions{display:flex;gap:.5rem;justify-content:flex-end}",
      "#" + DIALOG_ID + " button{font:inherit;padding:.5rem 1rem;border-radius:8px;cursor:pointer;",
      "border:1px solid var(--border,var(--palette-d1d5db));background:transparent;color:inherit}",
      "#" + DIALOG_ID + " button.primary{background:var(--accent,var(--palette-3b82f6));border-color:transparent;color:var(--id-surface-solid)}"
    ].join("");
    document.head.appendChild(style);
  }

  function openDialog(opts) {
    return new Promise(function (resolve) {
      ensureDialogCss();
      var previous = document.getElementById(DIALOG_ID);
      if (previous) previous.remove();
      var overlay = document.createElement("div");
      overlay.id = DIALOG_ID;
      overlay.setAttribute("role", "dialog");
      overlay.setAttribute("aria-modal", "true");

      var box = document.createElement("div");
      box.className = "gb-dialog-box";
      var heading = document.createElement("h2");
      heading.textContent = opts.title;
      var text = document.createElement("p");
      text.textContent = opts.message;
      box.appendChild(heading);
      box.appendChild(text);

      var field = null;
      if (opts.kind === "prompt") {
        field = document.createElement("input");
        field.type = "text";
        field.value = opts.defaultValue || "";
        field.setAttribute("aria-label", opts.title);
        box.appendChild(field);
      }

      var actions = document.createElement("div");
      actions.className = "gb-dialog-actions";
      var cancel = null;
      if (opts.kind !== "alert") {
        cancel = document.createElement("button");
        cancel.type = "button";
        cancel.textContent = dialogFallback("modal-cancel", "Cancel");
        actions.appendChild(cancel);
      }
      var ok = document.createElement("button");
      ok.type = "button";
      ok.className = "primary";
      ok.textContent = dialogFallback("modal-ok", "OK");
      actions.appendChild(ok);
      box.appendChild(actions);
      overlay.appendChild(box);

      function finish(value) {
        document.removeEventListener("keydown", onKey, true);
        overlay.remove();
        resolve(value);
      }
      function onKey(event) {
        if (event.key === "Escape") {
          event.preventDefault();
          finish(opts.kind === "alert" ? true : null);
        } else if (event.key === "Enter" && (opts.kind !== "prompt" || event.target === field)) {
          event.preventDefault();
          accept();
        }
      }
      function accept() {
        if (opts.kind === "prompt") finish(field ? field.value : "");
        else finish(true);
      }
      ok.addEventListener("click", accept);
      if (cancel) cancel.addEventListener("click", function () { finish(null); });
      overlay.addEventListener("click", function (event) {
        if (event.target === overlay && opts.kind !== "alert") finish(null);
      });
      document.addEventListener("keydown", onKey, true);

      document.body.appendChild(overlay);
      if (field) { field.focus(); field.select(); } else { ok.focus(); }
    });
  }

  function gbAlert(message, title) {
    return openDialog({
      kind: "alert",
      title: title || dialogFallback("modal-alert-title", "Notice"),
      message: String(message == null ? "" : message)
    });
  }

  function gbConfirm(message, title) {
    return openDialog({
      kind: "confirm",
      title: title || dialogFallback("modal-confirm-title", "Confirm"),
      message: String(message == null ? "" : message)
    }).then(function (value) { return value === true; });
  }

  function gbPrompt(message, defaultValue, title) {
    return openDialog({
      kind: "prompt",
      title: title || dialogFallback("modal-input-title", "Input"),
      message: String(message == null ? "" : message),
      defaultValue: defaultValue == null ? "" : String(defaultValue)
    });
  }

  var html = { escape: escapeHtml };
  var auth = { token: token, headers: headers };

  window.GBHtml = window.GBHtml || html;
  window.GBShared = { html: html, auth: auth, formatDate: gbFormatDate, formatTime: gbFormatTime, formatDateTime: gbFormatDateTime, formatNumber: gbFormatNumber };
  window.gbFormatDate = window.gbFormatDate || gbFormatDate;
  window.gbFormatTime = window.gbFormatTime || gbFormatTime;
  window.gbFormatDateTime = window.gbFormatDateTime || gbFormatDateTime;
  window.gbFormatNumber = window.gbFormatNumber || gbFormatNumber;
  // #2297 — shared in-page dialogs for every cloud page.
  window.gbAlert = window.gbAlert || gbAlert;
  window.gbConfirm = window.gbConfirm || gbConfirm;
  window.gbPrompt = window.gbPrompt || gbPrompt;
  window.gbLogError = window.gbLogError || gbLogError;
  // auth-service.js owns the rest of window.GBAuth; only add our two helpers.
  window.GBAuth = Object.assign(window.GBAuth || {}, auth);
  // Legacy global used by few remaining call sites; a page-local declaration
  // still hoists over this and wins, which is the pre-existing behaviour.
  if (typeof window.escapeHtml !== "function") window.escapeHtml = escapeHtml;
})();
