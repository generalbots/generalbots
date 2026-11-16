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
  var _I = window.__GBSharedInternal = window.__GBSharedInternal || {};
  _I.escapeHtml = escapeHtml;
  _I.storeGet = storeGet;
  _I.copyText = copyText;
  _I.printHtml = printHtml;
  _I.format = format;
  _I.read = read;
  _I.runtimeToken = runtimeToken;
  _I.token = token;
  _I.headers = headers;
  _I.isDesktop = isDesktop;
  _I.tauriInvoke = tauriInvoke;
  _I.absoluteUrl = absoluteUrl;
  _I.notify = notify;
  _I.anchorDownload = anchorDownload;
  _I.reportSaved = reportSaved;
  _I.download = download;
  _I.saveBlob = saveBlob;
  _I.gbFormatDate = gbFormatDate;
  _I.gbFormatTime = gbFormatTime;
  _I.gbFormatDateTime = gbFormatDateTime;
  _I.gbFormatNumber = gbFormatNumber;
  _I.gbFormatCurrency = gbFormatCurrency;

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
