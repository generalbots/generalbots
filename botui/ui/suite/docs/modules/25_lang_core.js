"use strict";
/* Docs language tools core (#2484).
 *
 * Local-first namespace shared by the language modules below. All checks run
 * in the browser: no document text is ever sent to a server. A single-step
 * undo snapshot makes every accepted suggestion reviewable and reversible. */

(function () {
  var NS = (window.DocsLang = window.DocsLang || {});

  var GLOBAL = "docs2484:";

  var state = {
    language: "en",
    autoformat: false,
    aiOptIn: false,
    domains: { legal: false, medical: false, financial: false, public: false },
    ignoredOnce: Object.create(null),
    ignoredAllCache: [],
    undoSnapshot: null,
    lastFocus: null,
    keyHandler: null,
    activeTab: "spelling",
    suggestions: [],
    thesaurus: [],
    report: null
  };
  NS.state = state;

  function tr(key, fallback, params) {
    if (typeof docsT === "function") return docsT(key, fallback, params);
    if (window.gbT) return window.gbT(key, fallback, params);
    return fallback || key;
  }
  NS.tr = tr;

  function getArticle() {
    if (typeof getActiveArticle === "function") return getActiveArticle();
    return document.querySelector("article[contenteditable]");
  }
  NS.getArticle = getArticle;

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }
  NS.escapeHtml = escapeHtml;

  function docKey(name) {
    var article = getArticle();
    var id = article && article.dataset ? article.dataset.docId || "current" : "current";
    return GLOBAL + id + ":" + name;
  }
  NS.docKey = docKey;

  function readGlobal(name, fallback) {
    try {
      var raw = window.localStorage.getItem(GLOBAL + name);
      if (raw == null) return fallback;
      var parsed = JSON.parse(raw);
      return parsed == null ? fallback : parsed;
    } catch (error) {
      return fallback;
    }
  }
  NS.readGlobal = readGlobal;

  function writeGlobal(name, value) {
    try {
      window.localStorage.setItem(GLOBAL + name, JSON.stringify(value));
      return true;
    } catch (error) {
      return false;
    }
  }
  NS.writeGlobal = writeGlobal;

  function readDoc(name, fallback) {
    try {
      var raw = window.localStorage.getItem(docKey(name));
      if (raw == null) return fallback;
      var parsed = JSON.parse(raw);
      return parsed == null ? fallback : parsed;
    } catch (error) {
      return fallback;
    }
  }
  NS.readDoc = readDoc;

  function writeDoc(name, value) {
    try {
      window.localStorage.setItem(docKey(name), JSON.stringify(value));
      return true;
    } catch (error) {
      return false;
    }
  }
  NS.writeDoc = writeDoc;

  function loadState() {
    state.language = readDoc("language", "en") || "en";
    state.autoformat = readGlobal("autoformat", false) === true;
    state.aiOptIn = readGlobal("ai-opt-in", false) === true;
    var domains = readGlobal("domains", {});
    state.domains = {
      legal: !!(domains && domains.legal),
      medical: !!(domains && domains.medical),
      financial: !!(domains && domains.financial),
      public: !!(domains && domains.public)
    };
  }
  NS.loadState = loadState;

  function setLanguage(value) {
    state.language = value || "en";
    writeDoc("language", state.language);
  }
  NS.setLanguage = setLanguage;

  function customWords() {
    var words = readGlobal("custom-words", []);
    return Array.isArray(words) ? words.filter(function (w) { return typeof w === "string" && w; }) : [];
  }
  NS.customWords = customWords;

  function addCustomWord(word) {
    var clean = String(word || "").trim().toLowerCase();
    if (!clean) return false;
    var words = customWords();
    if (words.indexOf(clean) >= 0) return false;
    words.push(clean);
    writeGlobal("custom-words", words);
    return true;
  }
  NS.addCustomWord = addCustomWord;

  function removeCustomWord(word) {
    var clean = String(word || "").trim().toLowerCase();
    var words = customWords().filter(function (w) { return w !== clean; });
    writeGlobal("custom-words", words);
    return true;
  }
  NS.removeCustomWord = removeCustomWord;

  function glossary() {
    var list = readGlobal("glossary", []);
    return Array.isArray(list) ? list.filter(function (e) {
      return e && typeof e.term === "string" && typeof e.preferred === "string" && e.term && e.preferred;
    }) : [];
  }
  NS.glossary = glossary;

  function addGlossaryTerm(term, preferred) {
    var t = String(term || "").trim().toLowerCase();
    var p = String(preferred || "").trim();
    if (!t || !p) return false;
    var list = glossary();
    for (var i = 0; i < list.length; i++) {
      if (list[i].term.toLowerCase() === t) return false;
    }
    list.push({ term: t, preferred: p });
    writeGlobal("glossary", list);
    return true;
  }
  NS.addGlossaryTerm = addGlossaryTerm;

  function removeGlossaryTerm(term) {
    var t = String(term || "").trim().toLowerCase();
    var list = glossary().filter(function (e) { return e.term.toLowerCase() !== t; });
    writeGlobal("glossary", list);
    return true;
  }
  NS.removeGlossaryTerm = removeGlossaryTerm;

  function status(message) {
    var host = document.getElementById("docs-lang-status");
    if (host) host.textContent = message || "";
    if (window.DocsNav && typeof window.DocsNav.status === "function") {
      window.DocsNav.status(message || "");
    }
  }
  NS.status = status;

  function save() {
    var article = getArticle();
    if (!article) return;
    if (window.DocsNav && typeof window.DocsNav.save === "function") {
      window.DocsNav.save();
      return;
    }
    if (typeof scheduleSave === "function") {
      scheduleSave(article.dataset ? article.dataset.docId : null, article.innerHTML);
    }
  }
  NS.save = save;

  // Map a global plain-text offset onto the article's visible text nodes.
  function locate(root, offset) {
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null, false);
    var remaining = Math.max(0, offset);
    var node;
    var lastNode = null;
    while ((node = walker.nextNode())) {
      lastNode = node;
      var length = (node.nodeValue || "").length;
      if (remaining <= length) return { node: node, offset: remaining };
      remaining -= length;
    }
    if (lastNode) return { node: lastNode, offset: (lastNode.nodeValue || "").length };
    return { node: root, offset: 0 };
  }
  NS.locate = locate;

  // Replace [start,end) across text nodes without touching element structure.
  function replaceRange(root, start, end, replacement) {
    if (!root || end < start) return false;
    var a = locate(root, start);
    var b = locate(root, end);
    var range = document.createRange();
    try {
      range.setStart(a.node, a.offset);
      range.setEnd(b.node, b.offset);
      range.deleteContents();
      if (replacement) range.insertNode(document.createTextNode(String(replacement)));
      return true;
    } catch (error) {
      return false;
    }
  }
  NS.replaceRange = replaceRange;

  function validResult(clone, original) {
    if (!window.DocsNav || typeof window.DocsNav.validateHtml !== "function") return true;
    return window.DocsNav.validateHtml(clone.innerHTML, original).ok === true;
  }
  NS.validResult = validResult;

  // Apply one reviewable change on a clone first; only touch the live article
  // after the resulting markup passes the shared schema guard.
  function applyChange(item) {
    var article = getArticle();
    if (!article || !item) return false;
    var original = article.innerHTML;
    var clone = article.cloneNode(true);
    var offset = typeof item.start === "number" ? item.start : -1;
    var end = typeof item.end === "number" ? item.end : offset;
    if (offset < 0 || end < offset) return false;
    if (!NS.replaceRange(clone, offset, end, item.replacement == null ? "" : item.replacement)) return false;
    if (!NS.validResult(clone, original)) {
      status(tr("docs-find-schema-error", "Replacement would corrupt the document"));
      return false;
    }
    state.undoSnapshot = original;
    if (!NS.replaceRange(article, offset, end, item.replacement == null ? "" : item.replacement)) return false;
    save();
    status(tr("docs-lang-applied", "Suggestion applied."));
    return true;
  }
  NS.applyChange = applyChange;

  function undo() {
    var article = getArticle();
    if (!article || state.undoSnapshot == null) {
      status(tr("docs-lang-nothing-to-undo", "Nothing to undo."));
      return false;
    }
    article.innerHTML = state.undoSnapshot;
    state.undoSnapshot = null;
    save();
    status(tr("docs-lang-undone", "Change undone."));
    return true;
  }
  NS.undo = undo;
})();
