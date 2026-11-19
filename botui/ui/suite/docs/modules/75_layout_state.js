"use strict";
/* Docs deterministic layout/pagination preview (#2424), local state.
 *
 * Bounded FRONTEND slice only. The layout projection lives in ONE
 * document-scoped localStorage key `docs-layout:<docId>` (the same pattern as
 * the model/review/forms layers) and is explicitly labelled
 * "Local only — preview, not the PDF renderer". Nothing here claims real PDF
 * export parity, tagged PDF/PDF-UA, a 100-page benchmark, print CSS
 * byte-comparison or CI page-image comparison — those need a compiler, a real
 * renderer and a backend and are deferred. */
(function (window) {
  var NS = (window.DocsLayout = window.DocsLayout || {});
  NS.PREFIX = "docs-layout:";
  NS.VERSION = 1;

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
    return tr("docs-layout-note-local", "Local only — preview, not the PDF renderer");
  };

  function defaultState() {
    return {
      version: NS.VERSION,
      setup: NS.defaultSetup ? NS.defaultSetup() : null,
      sections: NS.defaultSections ? NS.defaultSections() : [],
      lines: [],
      fields: { title: "", author: "", date: "" },
      currentPage: 0
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
      setup: stored.setup && typeof stored.setup === "object" ? stored.setup : fresh.setup,
      sections: Array.isArray(stored.sections) && stored.sections.length ? stored.sections : fresh.sections,
      lines: Array.isArray(stored.lines) ? stored.lines : fresh.lines,
      fields: stored.fields && typeof stored.fields === "object" ? stored.fields : fresh.fields,
      currentPage: Math.max(0, Number(stored.currentPage) || 0)
    };
    return NS.state;
  }
  NS.loadState = loadState;
  NS.persist = function () { return writeStore(NS.state); };

  /* ---- status region (writes need role="status" aria-live="polite") ------- */
  NS.status = function (message) {
    var host = document.getElementById ? document.getElementById("docs-layout-status") : null;
    if (host) host.textContent = message;
  };
  NS.statusKey = function (key, fallback, params) { NS.status(tr(key, fallback, params)); };

  /* ---- a11y focus-trap contract (pure, harness-testable) ------------------ */
  function focusMove(keys, current, event) {
    var ev = event || {};
    if (ev.key === "Escape") return { action: "close" };
    if (ev.key !== "Tab" || !keys || !keys.length) return { action: "none" };
    var index = keys.indexOf(current);
    /* Focus escaped the dialog: bring it back to the first (or last for
     * Shift+Tab) focusable instead of skipping one. */
    if (index < 0) {
      var edge = ev.shiftKey ? keys.length - 1 : 0;
      return { action: "focus", index: edge, key: keys[edge] };
    }
    var next = ev.shiftKey ? (index - 1 + keys.length) % keys.length : (index + 1) % keys.length;
    return { action: "focus", index: next, key: keys[next] };
  }
  NS.focusMove = focusMove;
})(typeof window !== "undefined" ? window : globalThis);
