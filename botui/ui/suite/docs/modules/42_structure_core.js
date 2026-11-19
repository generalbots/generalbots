"use strict";
/* Docs long-document structure (#2426), core namespace.
 *
 * A local-only authoring layer: numbering definitions, cross-reference fields,
 * captions and the Document Structure dialog. The server has no schema for
 * these objects yet, so configuration lives in localStorage and the structured
 * data lives in data attributes on the document itself. Nothing here invents a
 * page number or a citation lookup result.
 *
 * Every document write goes through commitArticle()/commitNode(): the changes
 * are prepared on a clone and committed with one execCommand("insertHTML") so
 * each edit is a single native undo step and neighbouring markup is untouched.
 */
(function (window) {
  var NS = (window.DocsStructure = window.DocsStructure || {});
  var LEVELS = 6;
  NS.LEVELS = LEVELS;

  function nav() { return window.DocsNav || {}; }

  function tr(key, fallback, params) {
    var n = nav();
    if (typeof n.tr === "function") return n.tr(key, fallback, params);
    if (typeof docsT === "function") return docsT(key, fallback, params);
    if (window.gbT) return window.gbT(key, fallback, params);
    return fallback || key;
  }
  NS.tr = tr;

  function esc(value) {
    var n = nav();
    if (typeof n.escapeHtml === "function") return n.escapeHtml(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }
  NS.esc = esc;

  function article() {
    var n = nav();
    if (typeof n.getArticle === "function") return n.getArticle();
    return document.querySelector("article[contenteditable]");
  }
  NS.getArticle = article;

  function status(message) {
    var host = document.getElementById("docs-structure-status");
    if (host) { host.textContent = message; return; }
    var n = nav();
    if (typeof n.status === "function") n.status(message);
  }
  NS.status = status;
  NS.statusKey = function (key, fallback) { status(tr(key, fallback)); };

  function save() {
    var a = article();
    if (!a) return;
    var n = nav();
    if (typeof n.save === "function") { n.save(); return; }
    if (typeof scheduleSave === "function") {
      scheduleSave(a.getAttribute ? a.getAttribute("data-doc-id") : null, a.innerHTML);
    }
  }
  NS.save = save;

  function uid(prefix) {
    var tail;
    if (typeof crypto !== "undefined" && crypto && typeof crypto.randomUUID === "function") {
      try { tail = crypto.randomUUID().slice(0, 8); } catch (error) { tail = ""; }
    }
    if (!tail) tail = Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
    return prefix + "-" + tail;
  }
  NS.uid = uid;

  function defaultConfig() {
    var levels = [];
    for (var i = 1; i <= LEVELS; i++) {
      levels.push({ level: i, scheme: "decimal", continuePrevious: false });
    }
    return { levels: levels, restartPerSection: true };
  }
  NS.defaultConfig = defaultConfig;

  function config() {
    var n = nav();
    var stored = typeof n.readStore === "function" ? n.readStore("structureConfig", null) : null;
    if (!stored || !stored.levels || stored.levels.length !== LEVELS) return defaultConfig();
    var levels = [];
    for (var i = 0; i < LEVELS; i++) {
      var row = stored.levels[i] || {};
      levels.push({
        level: i + 1,
        scheme: row.scheme === "legal" ? "legal" : "decimal",
        continuePrevious: row.continuePrevious === true,
      });
    }
    return { levels: levels, restartPerSection: stored.restartPerSection !== false };
  }
  NS.config = config;

  NS.setConfig = function (next) {
    var n = nav();
    if (typeof n.writeStore !== "function") return false;
    return n.writeStore("structureConfig", next);
  };

  function selection() { return window.getSelection ? window.getSelection() : null; }
  NS.selection = selection;

  function caretInside(a) {
    var sel = selection();
    if (!sel || !sel.rangeCount || !a) return false;
    var range = sel.getRangeAt(0);
    var node = range.startContainer;
    if (node === a) return true;
    while (node && node !== a) node = node.parentNode;
    return node === a;
  }
  NS.caretInside = caretInside;

  function ownerDoc(a) { return (a && a.ownerDocument) || document; }

  /* Commit a whole-article mutation as one undoable insertion. */
  NS.commitArticle = function (mutate) {
    var a = article();
    if (!a || typeof a.cloneNode !== "function") return false;
    var clone = a.cloneNode(true);
    mutate(clone);
    var html = clone.innerHTML;
    var doc = ownerDoc(a);
    var sel = selection();
    var ok = false;
    try {
      if (sel && doc.createRange && document.execCommand) {
        var saved = [];
        for (var i = 0; i < sel.rangeCount; i++) saved.push(sel.getRangeAt(i).cloneRange());
        a.focus();
        var range = doc.createRange();
        range.selectNodeContents(a);
        sel.removeAllRanges();
        sel.addRange(range);
        ok = !!document.execCommand("insertHTML", false, html);
        sel.removeAllRanges();
        for (var j = 0; j < saved.length; j++) sel.addRange(saved[j]);
      }
    } catch (error) { ok = false; }
    if (!ok) {
      try { a.innerHTML = html; } catch (fallbackError) { return false; }
    }
    save();
    return true;
  };

  function firstNode(html, doc) {
    var holder = doc.createElement("div");
    holder.innerHTML = html;
    return holder.firstChild || null;
  }
  NS.firstNode = firstNode;

  /* Commit one field/caption at the caret (or replace one node) in one step. */
  NS.commitNode = function (html, target) {
    var a = article();
    if (!a || typeof a.cloneNode !== "function") return false;
    var doc = ownerDoc(a);
    var sel = selection();
    var range = null;
    try {
      if (target && doc.createRange) {
        range = doc.createRange();
        range.selectNode(target);
      } else if (sel && sel.rangeCount && caretInside(a)) {
        range = sel.getRangeAt(0).cloneRange();
      } else if (doc.createRange) {
        range = doc.createRange();
        range.selectNodeContents(a);
        range.collapse(false);
      }
    } catch (error) { range = null; }
    if (!range) return false;
    var ok = false;
    try {
      if (sel && document.execCommand) {
        sel.removeAllRanges();
        sel.addRange(range);
        a.focus();
        ok = !!document.execCommand("insertHTML", false, html);
      }
    } catch (commitError) { ok = false; }
    if (!ok) {
      try {
        if (target && target.parentNode) target.parentNode.removeChild(target);
        range.deleteContents();
        var node = firstNode(html, doc);
        if (node) range.insertNode(node);
      } catch (fallbackError) { return false; }
    }
    save();
    return true;
  };
})(window);
