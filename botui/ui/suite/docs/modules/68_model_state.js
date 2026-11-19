"use strict";
/* Docs structured document model (#2423), core namespace and local state.
 *
 * Bounded FRONTEND slice only. The projection lives in document-scoped
 * localStorage (the same pattern as the review/forms layers) and is explicitly
 * labelled "Local only — not saved to the server". Nothing here claims a real
 * DOCX/ODT round-trip, server op validation, HTML-body migration, co-editing
 * merge or 100k-word scale — those need a compiler/backend and are deferred. */
(function (window) {
  var NS = (window.DocsModel = window.DocsModel || {});
  var PREFIX = "docs2423:";
  NS.PREFIX = PREFIX;
  NS.SCHEMA_VERSION = 1;

  /* ---- i18n -------------------------------------------------------------- */
  function substitute(text, params) {
    var out = String(text == null ? "" : text);
    if (!params || typeof params !== "object") return out;
    Object.keys(params).forEach(function (name) {
      out = out.replace(new RegExp("\\{\\s*\\$?" + name + "\\s*\\}", "g"), String(params[name]));
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
  NS.localNote = function () { return tr("docs-model-note-local", "Local only — not saved to the server"); };

  /* ---- identity + document-scoped storage -------------------------------- */
  function uid(prefix) {
    var tail = "";
    if (window.crypto && typeof window.crypto.randomUUID === "function") {
      try { tail = window.crypto.randomUUID().replace(/-/g, "").slice(0, 12); } catch (error) { tail = ""; }
    }
    if (!tail) tail = Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
    return (prefix || "id") + "-" + tail;
  }
  NS.uid = uid;

  function article() {
    if (typeof getActiveArticle === "function") {
      var found = getActiveArticle();
      if (found) return found;
    }
    return document.querySelector ? document.querySelector("article[contenteditable]") : null;
  }
  NS.getArticle = article;

  function docId() {
    var active = article();
    if (active && active.dataset && active.dataset.docId) return active.dataset.docId;
    var title = document.getElementById ? document.getElementById("docTitle") : null;
    if (title && title.value) return title.value;
    return "current";
  }
  NS.docId = docId;
  NS.docKey = function (name) { return PREFIX + docId() + ":" + name; };

  function readStore(name, fallback) {
    try {
      var raw = window.localStorage.getItem(NS.docKey(name));
      if (raw == null) return fallback;
      var parsed = JSON.parse(raw);
      return parsed == null ? fallback : parsed;
    } catch (error) { return fallback; }
  }
  NS.readStore = readStore;

  function writeStore(name, value) {
    try {
      window.localStorage.setItem(NS.docKey(name), JSON.stringify(value));
      return true;
    } catch (error) { return false; }
  }
  NS.writeStore = writeStore;

  /* ---- state ------------------------------------------------------------- */
  function defaultState() {
    return {
      doc: null,
      ops: [],
      undo: {},
      redo: {},
      authors: ["You"],
      currentAuthor: "You",
      selection: { blockId: null, offset: 0 }
    };
  }
  NS.defaultState = defaultState;
  NS.state = defaultState();

  function loadState() {
    var stored = readStore("state", null) || {};
    var fresh = defaultState();
    NS.state = {
      doc: stored.doc && typeof stored.doc === "object" ? stored.doc : fresh.doc,
      ops: Array.isArray(stored.ops) ? stored.ops : fresh.ops,
      undo: stored.undo && typeof stored.undo === "object" ? stored.undo : fresh.undo,
      redo: stored.redo && typeof stored.redo === "object" ? stored.redo : fresh.redo,
      authors: Array.isArray(stored.authors) && stored.authors.length ? stored.authors : fresh.authors,
      currentAuthor: typeof stored.currentAuthor === "string" && stored.currentAuthor
        ? stored.currentAuthor : fresh.currentAuthor,
      selection: stored.selection && typeof stored.selection === "object" ? stored.selection : fresh.selection
    };
    return NS.state;
  }
  NS.loadState = loadState;
  NS.persist = function () { return writeStore("state", NS.state); };

  function rememberAuthor(name) {
    var author = name || NS.ME;
    if (NS.state.authors.indexOf(author) < 0) NS.state.authors.push(author);
    if (!NS.state.undo[author]) NS.state.undo[author] = [];
    if (!NS.state.redo[author]) NS.state.redo[author] = [];
    return author;
  }
  NS.ME = "You";
  NS.rememberAuthor = rememberAuthor;
  NS.me = function () { return rememberAuthor(NS.state.currentAuthor); };
  NS.setCurrentAuthor = function (name) { NS.state.currentAuthor = rememberAuthor(name); NS.persist(); };

  /* ---- status region ----------------------------------------------------- */
  NS.status = function (message) {
    var host = document.getElementById ? document.getElementById("docs-model-status") : null;
    if (host) host.textContent = message;
  };
  NS.statusKey = function (key, fallback, params) { NS.status(tr(key, fallback, params)); };

  /* ---- a11y focus trap contract (pure, harness-testable) ----------------- */
  function focusMove(keys, current, event) {
    var ev = event || {};
    if (ev.key === "Escape") return { action: "close" };
    if (ev.key !== "Tab" || !keys || !keys.length) return { action: "none" };
    var index = keys.indexOf(current);
    if (index < 0) index = 0;
    var next = ev.shiftKey ? (index - 1 + keys.length) % keys.length : (index + 1) % keys.length;
    return { action: "focus", index: next, key: keys[next] };
  }
  NS.focusMove = focusMove;
})(typeof window !== "undefined" ? window : globalThis);
