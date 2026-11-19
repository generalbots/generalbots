/* Split from gb-shared.js for #2305 (450-line limit). */
(function () {
  "use strict";
  var _I = window.__GBSharedInternal || {};
  var escapeHtml = _I.escapeHtml;
  var storeGet = _I.storeGet;
  var copyText = _I.copyText;
  var printHtml = _I.printHtml;
  var format = _I.format;
  var token = _I.token;
  var headers = _I.headers;
  var isDesktop = _I.isDesktop;
  var download = _I.download;
  var saveBlob = _I.saveBlob;
  var gbFormatDate = _I.gbFormatDate;
  var gbFormatTime = _I.gbFormatTime;
  var gbFormatDateTime = _I.gbFormatDateTime;
  var gbFormatNumber = _I.gbFormatNumber;
  var gbFormatCurrency = _I.gbFormatCurrency;

  // Shared internals for split module (#2305).
  _I = window.__GBSharedInternal = window.__GBSharedInternal || {};
  // Re-export what gb-shared_a.js (if loaded) provided, without referencing
  // undeclared identifiers that would abort this module and break gbLog.
  ["escapeHtml","storeGet","copyText","printHtml","format","read","runtimeToken","token","headers","isDesktop","tauriInvoke","absoluteUrl","notify","anchorDownload","reportSaved","download","saveBlob","gbFormatDate","gbFormatTime","gbFormatDateTime","gbFormatNumber","gbFormatCurrency"].forEach(function (k) {
    if (_I[k] == null) {
      try { if (typeof eval(k) !== "undefined") _I[k] = eval(k); } catch (_e) { window.gbLogError ? gbLogError(_e, "js") : console.error("js", _e); }
    }
  });
  escapeHtml = escapeHtml || _I.escapeHtml;
  storeGet = storeGet || _I.storeGet;
  copyText = copyText || _I.copyText;
  printHtml = printHtml || _I.printHtml;
  format = format || _I.format;
  token = token || _I.token;
  headers = headers || _I.headers;
  isDesktop = isDesktop || _I.isDesktop;
  download = download || _I.download;
  saveBlob = saveBlob || _I.saveBlob;
  gbFormatDate = gbFormatDate || _I.gbFormatDate;
  gbFormatTime = gbFormatTime || _I.gbFormatTime;
  gbFormatDateTime = gbFormatDateTime || _I.gbFormatDateTime;
  gbFormatNumber = gbFormatNumber || _I.gbFormatNumber;
  gbFormatCurrency = gbFormatCurrency || _I.gbFormatCurrency;

  function gbDebugEnabled() {
    try {
      var v = localStorage.getItem("gb-debug");
      if (v == null) v = localStorage.getItem("GB_DEBUG");
      return v === "1" || v === "true" || v === "on" || v === "yes";
    } catch (e) {
      return false;
    }
  }
  var gbLog = {
    debug: function () {
      if (!gbDebugEnabled()) return;
      if (console && console.debug) return console.debug.apply(console, arguments);
      if (console && console.log) return console.log.apply(console, arguments);
    },
    info: function () {
      if (console && console.info) return console.info.apply(console, arguments);
      if (console && console.log) return console.log.apply(console, arguments);
    },
    warn: function () {
      if (console && console.warn) return console.warn.apply(console, arguments);
    },
    error: function () {
      if (console && console.error) return console.error.apply(console, arguments);
    },
    isDebugEnabled: gbDebugEnabled,
  };

  // #2301 — one way to report a failure that was caught. The suite used to
  // have ~350 `catch (e) { window.gbLogError ? gbLogError(e, "error") : console.error("error", e); }` blocks: the error vanished, the UI silently did
  // nothing, and nothing reached the server log. Every one of those now calls
  // this, which logs once per context per 30 s (a failing poll or keystroke
  // handler must not flood the console) and forwards to the client error
  // reporter when it is loaded. Returns the error so `return gbLogError(...)`
  // keeps the caller's flow.
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
    if (!repeated) gbLog.error("[gb] " + where + ": " + message, error);
    try {
      if (window.ErrorReporter && window.ErrorReporter.report) {
        window.ErrorReporter.report(
          error instanceof Error ? error : new Error(message),
          { context: where, repeated: repeated }
        );
      }
    } catch (e) {
      gbLog.debug("gbLogError: reporter unavailable", e);
    }
    return error;
  }

  var html = { escape: escapeHtml };
  var auth = { token: token, headers: headers };
  var downloadKit = { download: download, saveBlob: saveBlob, isDesktop: isDesktop };

  window.GBHtml = window.GBHtml || html;
  window.GBShared = {
    html: html,
    auth: auth,
    download: download,
    saveBlob: saveBlob,
    storeGet: storeGet,
    log: gbLog,
  };
  window.gbLog = gbLog;
  window.gbLogError = window.gbLogError || gbLogError;
  window.gbStoreGet = storeGet;
  window.gbCopyText = copyText;
  window.gbFormat = format;
  window.gbFormatDate = gbFormatDate;
  window.gbFormatTime = gbFormatTime;
  window.gbFormatDateTime = gbFormatDateTime;
  window.gbFormatNumber = gbFormatNumber;
  window.gbFormatCurrency = gbFormatCurrency;
  window.gbPrintHtml = printHtml;
  window.GBShared.copyText = copyText;
  window.GBShared.format = format;
  window.GBShared.printHtml = printHtml;
  window.GBShared.formatDate = gbFormatDate;
  window.GBShared.formatTime = gbFormatTime;
  window.GBShared.formatDateTime = gbFormatDateTime;
  window.GBShared.formatNumber = gbFormatNumber;
  window.GBShared.formatCurrency = gbFormatCurrency;
  window.GBDownload = window.GBDownload || downloadKit;
  // auth-service.js owns the rest of window.GBAuth; only add our two helpers.
  window.GBAuth = Object.assign(window.GBAuth || {}, auth);
  // Legacy global used by few remaining call sites; a page-local declaration
  // still hoists over this and wins, which is the pre-existing behaviour.
  if (typeof window.escapeHtml !== "function") window.escapeHtml = escapeHtml;
})();
