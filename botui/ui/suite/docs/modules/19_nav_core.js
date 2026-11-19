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

  /* Accessible in-app prompt/confirm. Native window.prompt/confirm block the
   * UI, cannot be themed and break the surrounding dialog's focus contract.
   * Each call renders one focus-trapped dialog and resolves on submit/cancel. */
  function ask(kind, message, value) {
    var host = document.getElementById("docs-modal-container");
    if (!host || typeof Promise === "undefined") return Promise.resolve(null);
    var restore = document.activeElement;
    var isPrompt = kind === "prompt";
    var okLabel = NS.escapeHtml(tr("docs-dialog-ok", "OK"));
    var cancelLabel = NS.escapeHtml(tr("docs-dialog-cancel", "Cancel"));
    host.innerHTML = [
      '<div class="docs-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:var(--z-modal-backdrop);">',
      '<div class="gb-qp-dialog" role="dialog" aria-modal="true" aria-labelledby="gb-ask-title" style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:480px;max-width:92vw;padding:20px;color:#f8fafc;">',
      '<h3 id="gb-ask-title" style="margin:0 0 12px;font-size:16px;">' + NS.escapeHtml(message) + "</h3>",
      isPrompt
        ? '<input id="gb-ask-input" type="text" aria-label="' + NS.escapeHtml(message) +
          '" style="display:block;width:100%;margin-bottom:12px;padding:8px;box-sizing:border-box;" />'
        : "",
      '<div style="display:flex;justify-content:flex-end;gap:8px;">',
      '<button type="button" id="gb-ask-cancel" class="gb-fr-btn">' + cancelLabel + "</button>",
      '<button type="button" id="gb-ask-ok" class="gb-fr-btn gb-fr-primary">' + okLabel + "</button>",
      "</div></div></div>"
    ].join("");
    return new Promise(function (resolve) {
      var input = document.getElementById("gb-ask-input");
      var cancel = document.getElementById("gb-ask-cancel");
      var ok = document.getElementById("gb-ask-ok");
      var focusables = input ? [input, cancel, ok] : [cancel, ok];
      var done = function (result) {
        document.removeEventListener("keydown", onKey, true);
        host.innerHTML = "";
        if (restore && restore.focus) restore.focus();
        resolve(result);
      };
      var onKey = function (event) {
        if (event.key === "Escape") {
          event.preventDefault();
          done(null);
        } else if (event.key === "Tab") {
          var index = focusables.indexOf(document.activeElement);
          if (index < 0) return;
          var next = event.shiftKey
            ? (index - 1 + focusables.length) % focusables.length
            : (index + 1) % focusables.length;
          event.preventDefault();
          focusables[next].focus();
        }
      };
      document.addEventListener("keydown", onKey, true);
      cancel.addEventListener("click", function () { done(null); });
      ok.addEventListener("click", function () {
        done(isPrompt ? (input ? input.value : "") : true);
      });
      if (input) {
        input.value = value == null ? "" : String(value);
        input.focus();
        input.select();
      } else {
        cancel.focus();
      }
    });
  }
  NS.ask = ask;
  NS.dialogPrompt = function (message, value) { return ask("prompt", message, value); };
  NS.dialogConfirm = function (message) {
    return ask("confirm", message).then(function (result) { return result === true; });
  };
})();
