"use strict";
/* Docs enterprise review workflow (#2425), core namespace and state.
 *
 * Local-only browser layer. Every review object lives in document-scoped
 * localStorage (the same pattern as docs2484/docs2485/docs2426): a stable
 * document id prefixes each key. Nothing in this slice talks to the server,
 * sends notifications, or claims DOCX w:ins/w:del persistence — those are
 * explicitly deferred. The UI is labelled "Local only — not saved to the
 * server" so a reviewer is never misled about durability. */
(function (window) {
  var NS = (window.DocsReview = window.DocsReview || {});
  var PREFIX = "docs2425:";
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
  NS.localNote = function () { return tr("docs-review-note-local", "Local only — not saved to the server"); };

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
      var a = getActiveArticle();
      if (a) return a;
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

  /* ---- default state ---------------------------------------------------- */
  function defaultState() {
    return {
      changes: [],
      threads: [],
      redlines: [],
      viewMode: "all",
      suggestionMode: false,
      protectMode: false,
      signOff: "pending",
      reviewStartedAt: Date.now(),
      role: "editor",
      currentUser: "You",
      undoStack: []
    };
  }
  NS.defaultState = defaultState;
  NS.state = defaultState();

  NS.VIEW_MODES = ["all", "simple", "none", "original", "final"];
  NS.ROLES = ["owner", "editor", "commenter", "viewer"];
  NS.ME = "You";

  function loadState() {
    var fresh = defaultState();
    var stored = readStore("state", null) || {};
    NS.state = {
      changes: Array.isArray(stored.changes) ? stored.changes : fresh.changes,
      threads: Array.isArray(stored.threads) ? stored.threads : fresh.threads,
      redlines: Array.isArray(stored.redlines) ? stored.redlines : fresh.redlines,
      viewMode: NS.VIEW_MODES.indexOf(stored.viewMode) >= 0 ? stored.viewMode : fresh.viewMode,
      suggestionMode: stored.suggestionMode === true,
      protectMode: stored.protectMode === true,
      signOff: typeof stored.signOff === "string" ? stored.signOff : fresh.signOff,
      reviewStartedAt: typeof stored.reviewStartedAt === "number" ? stored.reviewStartedAt : fresh.reviewStartedAt,
      role: NS.ROLES.indexOf(stored.role) >= 0 ? stored.role : fresh.role,
      currentUser: typeof stored.currentUser === "string" && stored.currentUser
        ? stored.currentUser : fresh.currentUser,
      undoStack: Array.isArray(stored.undoStack) ? stored.undoStack.slice(-20) : []
    };
    return NS.state;
  }
  NS.loadState = loadState;

  function persist() {
    return writeStore("state", NS.state);
  }
  NS.persist = persist;

  function setCurrentUser(name) {
    NS.state.currentUser = name || NS.ME;
    persist();
  }
  NS.setCurrentUser = setCurrentUser;

  function me() { return NS.state.currentUser || NS.ME; }
  NS.me = me;

  /* ---- status region ---------------------------------------------------- */
  function status(message) {
    var host = document.getElementById ? document.getElementById("docs-review-status") : null;
    if (host) { host.textContent = message; }
  }
  NS.status = status;
  NS.statusKey = function (key, fallback, params) { status(tr(key, fallback, params)); };

  /* ---- a11y focus trap (pure, harness-testable) ------------------------- */
  /* Given the ordered list of focusable keys, the currently focused key and a
   * keyboard event, decide what the dialog shell must do. Kept DOM-free so the
   * contract (Esc closes, Tab wraps in both directions) can be proven offline. */
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
