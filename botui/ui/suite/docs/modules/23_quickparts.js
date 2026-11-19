"use strict";
/* Content library / quick parts / autotext (#2485): reusable blocks with
 * variables, insert-as-copy or insert-and-link (updates propagate), and
 * keyboard autotext shortcuts such as /liability. Org-scoped RBAC/versioning
 * beyond this per-document cache is a backend concern (see issue report). */

(function () {
  var NS = window.DocsNav;
  if (!NS) return;
  var tr = NS.tr;

  var BUILTINS = [
    {
      id: "qp-liability",
      name: "Liability clause",
      shortcut: "/liability",
      content: "<p>The parties agree that liability shall be limited to {{amount}}.</p>",
      version: 1
    },
    {
      id: "qp-signature",
      name: "Signature block",
      shortcut: "/signature",
      content: "<p>Sincerely,<br/>{{name}}<br/>{{title}}</p>",
      version: 1
    }
  ];

  function all() {
    var stored = NS.readStore("quickparts", null);
    return Array.isArray(stored) ? stored : BUILTINS.map(function (p) {
      return JSON.parse(JSON.stringify(p));
    });
  }
  function store(parts) {
    NS.writeStore("quickparts", parts);
  }

  NS.quickParts = all;

  function mintId() {
    return "qp-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8);
  }

  function variablesOf(content) {
    var found = [];
    var re = /\{\{\s*([\w.-]+)\s*\}\}/g;
    var match;
    while ((match = re.exec(String(content || ""))) !== null) {
      if (found.indexOf(match[1]) < 0) found.push(match[1]);
    }
    return found;
  }
  NS.quickPartVariables = variablesOf;

  function fillVariables(content, values) {
    return String(content || "").replace(/\{\{\s*([\w.-]+)\s*\}\}/g, function (allText, name) {
      return Object.prototype.hasOwnProperty.call(values || {}, name) ? values[name] : allText;
    });
  }
  NS.fillQuickPartVariables = fillVariables;

  /* Style reconciliation: a linked insert must adopt the current document's
   * block styling instead of dragging the library's inline styles along. */
  function reconcile(html) {
    var holder = document.createElement("div");
    holder.innerHTML = String(html || "");
    Array.prototype.forEach.call(holder.querySelectorAll("*"), function (node) {
      if (node.hasAttribute("style")) node.removeAttribute("style");
      Array.prototype.forEach.call(node.attributes, function (attr) {
        if (attr.name.indexOf("on") === 0) node.removeAttribute(attr.name);
      });
    });
    return holder.innerHTML;
  }
  NS.reconcileQuickPartStyles = reconcile;

  function insertNode(node) {
    var article = NS.getArticle();
    if (!article || !node) return false;
    article.focus();
    var selection = window.getSelection();
    var range = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
    if (range && article.contains(range.startContainer)) {
      range.deleteContents();
      range.insertNode(node);
    } else {
      article.appendChild(node);
    }
    return true;
  }

  NS.insertQuickPart = function (id, asLink, values) {
    var part = all().filter(function (item) { return item.id === id; })[0];
    if (!part) {
      NS.status(tr("docs-qp-empty", "No quick parts yet"));
      return false;
    }
    var html = reconcile(fillVariables(part.content, values || {}));
    var node = document.createElement("div");
    node.className = asLink ? "gb-quickpart gb-quickpart-linked" : "gb-quickpart";
    if (asLink) {
      node.setAttribute("data-gb-quickpart", part.id);
      node.setAttribute("data-gb-version", String(part.version || 1));
    }
    node.innerHTML = html;
    if (!insertNode(node)) return false;
    NS.save();
    NS.status(asLink
      ? tr("docs-qp-inserted-linked", "Linked quick part inserted")
      : tr("docs-qp-inserted-copy", "Quick part copy inserted"));
    if (NS.refreshPane) NS.refreshPane();
    return true;
  };

  NS.updateQuickPart = function (id, content, propagate) {
    var parts = all();
    var part = parts.filter(function (item) { return item.id === id; })[0];
    if (!part) return false;
    part.content = content;
    part.version = (parseInt(part.version, 10) || 1) + 1;
    store(parts);
    var article = NS.getArticle();
    var linked = article ? article.querySelectorAll('[data-gb-quickpart="' + id + '"]') : [];
    if (propagate && linked.length) {
      var html = reconcile(fillVariables(part.content, {}));
      Array.prototype.forEach.call(linked, function (node) {
        node.innerHTML = html;
        node.setAttribute("data-gb-version", String(part.version));
      });
      NS.save();
    }
    NS.status(tr("docs-qp-updated", "Quick part updated"));
    return true;
  };

  NS.propagateLinked = function () {
    var article = NS.getArticle();
    if (!article) return 0;
    var parts = all();
    var count = 0;
    Array.prototype.forEach.call(article.querySelectorAll("[data-gb-quickpart]"), function (node) {
      var id = node.getAttribute("data-gb-quickpart");
      var part = parts.filter(function (item) { return item.id === id; })[0];
      if (!part) return;
      var current = parseInt(node.getAttribute("data-gb-version"), 10) || 0;
      if (current >= (parseInt(part.version, 10) || 1)) return;
      node.innerHTML = reconcile(fillVariables(part.content, {}));
      node.setAttribute("data-gb-version", String(part.version));
      count++;
    });
    if (count) {
      NS.save();
      NS.status(tr("docs-qp-propagated", "Updated {$n} linked parts", { n: count }));
    }
    return count;
  };

  /* Autotext: called from the editor keydown handler when Space/Enter is
   * pressed. `text` is the text before the caret; a trailing /shortcut that
   * matches the library is replaced with the expanded content. */
  NS.expandAutotext = function (text, node, offset) {
    var prefix = String(text || "");
    var match = /(\/[\w-]+)$/.exec(prefix);
    if (!match) return false;
    var token = match[1];
    var part = all().filter(function (item) { return item.shortcut === token; })[0];
    if (!part) return false;
    var before = prefix.slice(0, prefix.length - token.length);
    node.data = before + fillVariables(part.content, {}).replace(/<[^>]+>/g, "");
    if (typeof node.parentElement !== "undefined" && node.parentElement) {
      var selection = window.getSelection();
      if (selection && typeof document.createRange === "function") {
        var range = document.createRange();
        range.setStart(node, node.data.length);
        range.collapse(true);
        selection.removeAllRanges();
        selection.addRange(range);
      }
    }
    NS.status(tr("docs-at-expanded", "Autotext expanded: {$name}", { name: token }));
    return true;
  };

  function handleKeydown(event) {
    if (event.key !== " " && event.key !== "Enter") return;
    var target = event.target || {};
    if (!target.isContentEditable && !(target.closest && target.closest("#docs-content"))) return;
    var selection = window.getSelection();
    if (!selection || !selection.rangeCount) return;
    var range = selection.getRangeAt(0);
    var node = range.startContainer;
    if (!node || node.nodeType !== 3) return;
    var text = node.textContent.slice(0, range.startOffset);
    if (NS.expandAutotext(text, node, range.startOffset)) {
      event.preventDefault();
      NS.save();
    }
  }

  NS.openContentLibrary = function () {
    var host = document.getElementById("docs-modal-container");
    if (!host) return;
    var restore = document.activeElement;
    var parts = all();
    var rows = parts.map(function (part) {
      return '<div class="gb-qp-row" data-qp-id="' + NS.escapeHtml(part.id) + '">' +
        '<span class="gb-qp-name">' + NS.escapeHtml(part.name || part.id) + "</span> " +
        '<span class="gb-qp-shortcut">' + NS.escapeHtml(part.shortcut || "") + "</span> " +
        '<button type="button" class="gb-fr-btn" data-qp-copy="' + NS.escapeHtml(part.id) + '">' +
        NS.escapeHtml(tr("docs-qp-copy", "Insert a copy")) + "</button> " +
        '<button type="button" class="gb-fr-btn gb-fr-primary" data-qp-link="' + NS.escapeHtml(part.id) + '">' +
        NS.escapeHtml(tr("docs-qp-link", "Insert and link")) + "</button> " +
        '<button type="button" class="gb-fr-btn" data-qp-edit="' + NS.escapeHtml(part.id) + '">' +
        NS.escapeHtml(tr("docs-qp-edit", "Edit")) + "</button></div>";
    }).join("");
    if (!rows) rows = '<div class="gb-nav-empty">' + NS.escapeHtml(tr("docs-qp-empty", "No quick parts yet")) + "</div>";
    host.innerHTML = [
      '<div class="docs-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
      '<div class="gb-qp-dialog" role="dialog" aria-modal="true" aria-labelledby="gb-qp-title" style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:560px;max-width:92vw;padding:20px;color:#f8fafc;">',
      '<h3 id="gb-qp-title" style="margin:0 0 12px;font-size:16px;">' + NS.escapeHtml(tr("docs-qp-library", "Content library")) + "</h3>",
      '<div class="gb-qp-list">' + rows + "</div>",
      '<div style="display:flex;justify-content:flex-end;margin-top:12px;"><button type="button" id="gb-qp-close" class="gb-fr-btn">' +
      NS.escapeHtml(tr("docs-find-close", "Close")) + "</button></div></div></div>"
    ].join("");
    var onKey = function (event) {
      if (event.key === "Escape") {
        document.removeEventListener("keydown", onKey, true);
        host.innerHTML = "";
        if (restore && restore.focus) restore.focus();
      }
    };
    document.addEventListener("keydown", onKey, true);
    host.querySelector("#gb-qp-close").addEventListener("click", function () {
      document.removeEventListener("keydown", onKey, true);
      host.innerHTML = "";
      if (restore && restore.focus) restore.focus();
    });
    host.addEventListener("click", function (event) {
      var edit = event.target.closest("[data-qp-edit]");
      if (edit) {
        var editId = edit.getAttribute("data-qp-edit");
        var editable = all().filter(function (item) { return item.id === editId; })[0];
        if (!editable) return;
        var next = window.prompt(tr("docs-qp-new-content", "New content"), editable.content);
        if (next == null || !String(next).trim()) return;
        var article = NS.getArticle();
        var linked = article ? article.querySelectorAll('[data-gb-quickpart="' + editId + '"]').length : 0;
        var propagate = linked > 0 && window.confirm(tr(
          "docs-qp-propagate-ask",
          "This linked part is used {$n} times. Propagate the update?",
          { n: linked }
        ));
        NS.updateQuickPart(editId, String(next), propagate);
        return;
      }
      var copy = event.target.closest("[data-qp-copy]");
      var link = event.target.closest("[data-qp-link]");
      if (!copy && !link) return;
      var id = (copy || link).getAttribute(copy ? "data-qp-copy" : "data-qp-link");
      var part = all().filter(function (item) { return item.id === id; })[0];
      var values = {};
      if (part) variablesOf(part.content).forEach(function (name) {
        values[name] = window.prompt(tr("docs-qp-variable", "Value for {$name}", { name: name }), "");
      });
      NS.insertQuickPart(id, !!link, values);
    });
  };

  var bound = false;
  NS.bindQuickPartKeys = function () {
    if (bound) return;
    bound = true;
    document.addEventListener("keydown", handleKeydown, true);
  };
})();
