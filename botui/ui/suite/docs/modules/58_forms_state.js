"use strict";
/* Docs templates / content controls / mail merge / protection (#2427).
 *
 * Local-only browser layer. Every object lives in document-scoped localStorage
 * (the same pattern as the language/tables/structure/review slices). Nothing
 * here talks to the server, claims real DOCX content-control round-trip,
 * cryptographic signing, irreversible redaction or server-enforced permissions.
 * Each surface is labelled "Local only — not saved to the server". */
(function (window) {
  var NS = (window.DocsForms = window.DocsForms || {});
  var PREFIX = "docs2427:";
  NS.PREFIX = PREFIX;

  /* ---- i18n -------------------------------------------------------------- */
  function substitute(text, params) {
    var out = String(text == null ? "" : text);
    if (!params || typeof params !== "object") return out;
    Object.keys(params).forEach(function (name) {
      var re = new RegExp("\\{\\s*\\$?" + name + "\\s*\\}", "g");
      out = out.replace(re, String(params[name]));
    });
    return out;
  }
  NS.substitute = substitute;

  function tr(key, fallback, params) {
    var value = fallback || key;
    if (typeof docsT === "function") value = docsT(key, fallback, params);
    else if (window.gbT) value = window.gbT(key, fallback, params);
    if (value === fallback && params) value = substitute(fallback, params);
    return value;
  }
  NS.tr = tr;

  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }
  NS.esc = esc;

  NS.localNote = function () { return tr("docs-forms-note-local", "Local only — not saved to the server"); };
  NS.previewNote = function () { return tr("docs-forms-note-preview", "Preview only — no real signature or redaction"); };

  /* ---- identity + storage ------------------------------------------------ */
  function uid(prefix) {
    var tail = "";
    if (window.crypto && typeof window.crypto.randomUUID === "function") {
      try { tail = window.crypto.randomUUID().slice(0, 8); } catch (error) { tail = ""; }
    }
    if (!tail) tail = Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
    return (prefix || "id") + "-" + tail;
  }
  NS.uid = uid;

  function article() {
    if (typeof getActiveArticle === "function") {
      var active = getActiveArticle();
      if (active) return active;
    }
    return document.querySelector ? document.querySelector("article[contenteditable]") : null;
  }
  NS.getArticle = article;

  function docId() {
    var a = article();
    if (a && a.dataset && a.dataset.docId) return a.dataset.docId;
    var title = document.getElementById ? document.getElementById("docTitle") : null;
    if (title && title.value) return title.value;
    return "current";
  }
  NS.docId = docId;

  function docKey(name) { return PREFIX + docId() + ":" + name; }
  NS.docKey = docKey;

  function readStore(name, fallback) {
    try {
      var raw = window.localStorage.getItem(docKey(name));
      if (raw == null) return fallback;
      var parsed = JSON.parse(raw);
      return parsed == null ? fallback : parsed;
    } catch (error) { return fallback; }
  }
  NS.readStore = readStore;

  function writeStore(name, value) {
    try {
      window.localStorage.setItem(docKey(name), JSON.stringify(value));
      return true;
    } catch (error) { return false; }
  }
  NS.writeStore = writeStore;

  /* ---- state ------------------------------------------------------------- */
  function defaultState() {
    return {
      selectedTemplate: "",
      controls: [],
      csvText: "",
      mapping: {},
      recordIndex: 0,
      merged: "",
      signature: null,
      redactions: [],
      protect: { enabled: false, passwordPreview: "" }
    };
  }
  NS.defaultState = defaultState;
  NS.state = defaultState();

  function loadState() {
    var fresh = defaultState();
    var stored = readStore("state", null) || {};
    var protect = stored.protect || {};
    NS.state = {
      selectedTemplate: typeof stored.selectedTemplate === "string" ? stored.selectedTemplate : fresh.selectedTemplate,
      controls: Array.isArray(stored.controls) ? stored.controls : fresh.controls,
      csvText: typeof stored.csvText === "string" ? stored.csvText : fresh.csvText,
      mapping: stored.mapping && typeof stored.mapping === "object" ? stored.mapping : fresh.mapping,
      recordIndex: typeof stored.recordIndex === "number" ? stored.recordIndex : fresh.recordIndex,
      merged: typeof stored.merged === "string" ? stored.merged : fresh.merged,
      signature: stored.signature && typeof stored.signature === "object" ? stored.signature : fresh.signature,
      redactions: Array.isArray(stored.redactions) ? stored.redactions : fresh.redactions,
      protect: {
        enabled: protect.enabled === true,
        passwordPreview: typeof protect.passwordPreview === "string" ? protect.passwordPreview : ""
      }
    };
    return NS.state;
  }
  NS.loadState = loadState;

  function persist() { return writeStore("state", NS.state); }
  NS.persist = persist;

  /* ---- status region ----------------------------------------------------- */
  function ensureStatus() {
    var host = document.getElementById ? document.getElementById("docs-forms-status") : null;
    if (host) return host;
    host = document.createElement("div");
    host.id = "docs-forms-status";
    host.className = "gb-forms-global-status";
    host.setAttribute("role", "status");
    host.setAttribute("aria-live", "polite");
    var bar = document.querySelector ? document.querySelector(".docs-status-bar") : null;
    (bar || document.body).appendChild(host);
    return host;
  }
  NS.ensureStatus = ensureStatus;

  function status(message) {
    var host = ensureStatus();
    if (host) host.textContent = message;
  }
  NS.status = status;
  NS.statusKey = function (key, fallback, params) { status(tr(key, fallback, params)); };

  /* ---- a11y focus trap (pure, harness-testable) -------------------------- */
  function focusMove(keys, current, event) {
    var ev = event || {};
    if (ev.key === "Escape") return { action: "close" };
    if (ev.key !== "Tab" || !keys || !keys.length) return { action: "none" };
    var index = keys.indexOf(current);
    if (index < 0) index = 0;
    var next = ev.shiftKey
      ? (index - 1 + keys.length) % keys.length
      : (index + 1) % keys.length;
    return { action: "focus", index: next, key: keys[next] };
  }
  NS.focusMove = focusMove;
})(window);
