"use strict";
/* Docs large .docx load path (#2533), core namespace + document-scoped state.
 *
 * Bounded FRONTEND slice only. All state lives in ONE document-scoped key
 * `docs-large:<docId>` and is explicitly labelled "Local only — preview, not
 * the server load path". Nothing here claims a real streaming DOCX import,
 * real server body limits, gzip/brotli transport, server-side ETag/If-Match
 * enforcement, async export or measured browser/server numbers — those need a
 * compiler/backend and are DEFERRED. */
(function (window) {
  var NS = (window.DocsLarge = window.DocsLarge || {});
  NS.PREFIX = "docs-large:";
  NS.VERSION = 1;
  NS.SIZES_MB = [5, 10, 20];
  NS.DEFAULT_THRESHOLD_MB = 10;
  NS.CEILING_BYTES = 2 * 1024 * 1024;

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

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }
  NS.el = el;

  function i18n(node, key, fallback, params) {
    node.setAttribute("data-i18n", key);
    if (params) node.setAttribute("data-i18n-params", JSON.stringify(params));
    node.textContent = tr(key, fallback, params);
    return node;
  }
  NS.i18n = i18n;

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
  NS.storeKey = function () { return NS.PREFIX + docId(); };

  function readStore() {
    try {
      var raw = window.localStorage.getItem(NS.storeKey());
      if (raw == null) return null;
      var parsed = JSON.parse(raw);
      return parsed && typeof parsed === "object" ? parsed : null;
    } catch (error) { return null; }
  }
  NS.readStore = readStore;

  function writeStore(value) {
    try {
      window.localStorage.setItem(NS.storeKey(), JSON.stringify(value));
      return true;
    } catch (error) { return false; }
  }
  NS.writeStore = writeStore;

  NS.localNote = function () {
    return tr("docs-large-note-local", "Local only — preview, not the server load path");
  };
  NS.deferredNote = function () {
    return tr("docs-large-deferred",
      "Deferred: streaming import, server body limits, gzip/brotli transport, server ETag/If-Match enforcement, async export and measured baseline.");
  };

  function defaultState() {
    return {
      version: NS.VERSION,
      sizeMb: NS.SIZES_MB[0],
      mode: "automatic",
      etag: "v1",
      etagSent: "v1",
      queue: [],
      editingEnabled: false,
      fullBytes: 0
    };
  }
  NS.defaultState = defaultState;
  NS.state = defaultState();

  function loadState() {
    var stored = readStore();
    var fresh = defaultState();
    if (!stored) { NS.state = fresh; return NS.state; }
    NS.state = {
      version: NS.VERSION,
      sizeMb: NS.SIZES_MB.indexOf(Number(stored.sizeMb)) >= 0 ? Number(stored.sizeMb) : fresh.sizeMb,
      mode: stored.mode === "manual" ? "manual" : "automatic",
      etag: typeof stored.etag === "string" && stored.etag ? stored.etag : fresh.etag,
      etagSent: typeof stored.etagSent === "string" && stored.etagSent ? stored.etagSent : fresh.etagSent,
      queue: Array.isArray(stored.queue) ? stored.queue : [],
      editingEnabled: !!stored.editingEnabled,
      fullBytes: Math.max(0, Number(stored.fullBytes) || 0)
    };
    return NS.state;
  }
  NS.loadState = loadState;
  NS.persist = function () { return writeStore(NS.state); };

  function bumpEtag(etag) {
    var match = /^v(\d+)$/.exec(String(etag || ""));
    return match ? "v" + (Number(match[1]) + 1) : "v2";
  }
  NS.bumpEtag = bumpEtag;

  /* Writes always target a role="status" aria-live="polite" region. */
  NS.status = function (message) {
    var host = document.getElementById ? document.getElementById("docs-large-status") : null;
    if (host) host.textContent = message;
  };
  NS.statusKey = function (key, fallback, params) { NS.status(tr(key, fallback, params)); };

  /* Pure focus-trap contract (harness-testable). */
  function focusMove(keys, current, event) {
    var ev = event || {};
    if (ev.key === "Escape") return { action: "close" };
    if (ev.key !== "Tab" || !keys || !keys.length) return { action: "none" };
    var index = keys.indexOf(current);
    if (index < 0) return { action: "focus", index: ev.shiftKey ? keys.length - 1 : 0, key: keys[ev.shiftKey ? keys.length - 1 : 0] };
    var next = ev.shiftKey ? (index - 1 + keys.length) % keys.length : (index + 1) % keys.length;
    return { action: "focus", index: next, key: keys[next] };
  }
  NS.focusMove = focusMove;
})(typeof window !== "undefined" ? window : globalThis);
