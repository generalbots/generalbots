"use strict";
/* Navigation / find-replace / reusable-content core (#2485).
 *
 * Shared, dependency-free helpers used by the nav pane, the find/replace
 * dialog, bookmarks and the content library. Everything is scoped to the
 * active document article and localStorage so the feature is self-contained
 * in the frontend (no backend/compile gate required). */

(function () {
  var NS = (window.DocsNav = window.DocsNav || {});

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
    return "docs2485:" + id + ":" + name;
  }
  NS.docKey = docKey;

  function readStore(name, fallback) {
    try {
      var raw = window.localStorage.getItem(docKey(name));
      if (raw == null) return fallback;
      var parsed = JSON.parse(raw);
      return parsed == null ? fallback : parsed;
    } catch (error) {
      return fallback;
    }
  }
  NS.readStore = readStore;

  function writeStore(name, value) {
    try {
      window.localStorage.setItem(docKey(name), JSON.stringify(value));
      return true;
    } catch (error) {
      return false;
    }
  }
  NS.writeStore = writeStore;

  var BLOCKED_TAGS = ["script", "iframe", "object", "embed", "link", "meta", "style", "form"];

  /* Schema validation before a bulk replace is applied. A replace built from
   * user regex input must never be able to inject active content or leave a
   * fragment of unbalanced markup in the document. Returns {ok, error}. */
  function validateHtml(html, original) {
    var source = String(html == null ? "" : html);
    var doc = null;
    if (typeof DOMParser !== "undefined") {
      try {
        doc = new DOMParser().parseFromString(source, "text/html");
      } catch (error) {
        doc = null;
      }
    }
    if (doc && doc.querySelector("parsererror")) {
      return { ok: false, error: tr("docs-find-schema-error", "Replacement would corrupt the document.") };
    }
    var scan = doc || (function () {
      var holder = document.createElement("div");
      holder.innerHTML = source;
      return holder;
    })();
    for (var i = 0; i < BLOCKED_TAGS.length; i++) {
      if (scan.querySelector(BLOCKED_TAGS[i])) {
        return { ok: false, error: tr("docs-find-schema-error", "Replacement would corrupt the document.") };
      }
    }
    var attrs = scan.querySelectorAll("*");
    for (var n = 0; n < attrs.length; n++) {
      var node = attrs[n];
      if (!node.attributes) continue;
      for (var a = 0; a < node.attributes.length; a++) {
        var name = node.attributes[a].name.toLowerCase();
        var value = String(node.attributes[a].value || "").toLowerCase();
        if (name.indexOf("on") === 0 || value.indexOf("javascript:") >= 0) {
          return { ok: false, error: tr("docs-find-schema-error", "Replacement would corrupt the document.") };
        }
      }
    }
    if (original && String(original).trim() && !source.trim()) {
      return { ok: false, error: tr("docs-find-schema-error", "Replacement would corrupt the document.") };
    }
    return { ok: true, error: "" };
  }
  NS.validateHtml = validateHtml;

  var SCOPE_SELECTORS = {
    body: "p,h1,h2,h3,h4,h5,h6,li,blockquote,pre,td,th,figcaption",
    headers: ".docs-header-zone,[data-gb-header],header",
    footers: ".docs-footer-zone,[data-gb-footer],footer",
    footnotes: "[data-gb-footnote],.footnote,.dc-footnote,[data-footnote]",
    comments: "[data-gb-comment],.gb-comment,.dc-comment,[data-comment-id]",
    textboxes: "[data-gb-textbox],.gb-textbox,figure",
    embedded: "img[alt],svg[aria-label],[data-gb-embedded]"
  };

  /* A flat index of searchable scopes, covering comments, footnotes, headers,
   * footers and embedded text in addition to the body (#2485 criterion 7). */
  function searchScopes(article) {
    if (!article) return [];
    var scopes = [];
    Object.keys(SCOPE_SELECTORS).forEach(function (name) {
      var nodes = [];
      try {
        nodes = Array.prototype.slice.call(article.querySelectorAll(SCOPE_SELECTORS[name]));
      } catch (error) {
        nodes = [];
      }
      nodes.forEach(function (node) {
        var text = node.textContent || node.getAttribute("alt") || node.getAttribute("aria-label") || "";
        if (String(text).trim()) scopes.push({ scope: name, node: node, text: String(text) });
      });
    });
    if (!scopes.length) scopes.push({ scope: "body", node: article, text: article.textContent || "" });
    return scopes;
  }
  NS.searchScopes = searchScopes;

  function flash(node) {
    if (!node || !node.classList) return;
    node.classList.add("gb-nav-flash");
    window.setTimeout(function () {
      node.classList.remove("gb-nav-flash");
    }, 1400);
  }
  NS.flash = flash;

  function jumpTo(node) {
    if (!node) return false;
    if (typeof window.docsScrollToHeading === "function" && node.tagName && /^H[1-6]$/.test(node.tagName)) {
      window.docsScrollToHeading(node);
      return true;
    }
    if (typeof node.scrollIntoView === "function") {
      node.scrollIntoView({ behavior: "smooth", block: "center" });
    }
    flash(node);
    return true;
  }
  NS.jumpTo = jumpTo;

  function save() {
    var article = getArticle();
    if (!article) return;
    if (typeof scheduleSave === "function") scheduleSave(article.dataset ? article.dataset.docId : null, article.innerHTML);
    if (typeof docsAfterContentLoad === "function") docsAfterContentLoad(article);
  }
  NS.save = save;

  function status(message) {
    var host = document.getElementById("docs-fr-status") || document.getElementById("docs-nav-status");
    if (host) host.textContent = message;
  }
  NS.status = status;

  /* Collect a heading plus its following blocks up to the next heading of the
   * same or a higher level. Used by drag-to-reorder in the headings tree. */
  function collectSection(heading) {
    var article = getArticle();
    if (!article || !heading) return [];
    var level = parseInt(heading.tagName.slice(1), 10) || 6;
    var nodes = [];
    var sibling = heading.nextElementSibling;
    while (sibling) {
      var tag = sibling.tagName || "";
      if (/^H[1-6]$/.test(tag) && (parseInt(tag.slice(1), 10) || 6) <= level) break;
      nodes.push(sibling);
      sibling = sibling.nextElementSibling;
    }
    return [heading].concat(nodes);
  }
  NS.collectSection = collectSection;

  function headings() {
    var article = getArticle();
    if (!article) return [];
    return Array.prototype.slice.call(article.querySelectorAll("h1,h2,h3,h4,h5,h6"));
  }
  NS.headings = headings;
})();
