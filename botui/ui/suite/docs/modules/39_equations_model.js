"use strict";
/* Docs equation editor (#2483) — document model and single-step writes.
 *
 * Equations live in the article as <span class="docs-eq" contenteditable=false>
 * wrapping native MathML. LaTeX source, placement and numbering metadata are
 * data attributes, so equations stay searchable and re-editable. Every
 * insertion, edit and deletion is committed through one execCommand("insertHTML")
 * over the exact target range: one native undo step, no surrounding markup
 * corruption. The outerHTML path is only a fallback for engines/stubs without
 * execCommand. */
(function (window) {
  var NS = (window.DocsEquations = window.DocsEquations || {});
  var EQ_CLASS = "docs-eq";
  var REF_CLASS = "docs-eq-ref";

  NS.getArticle = function () {
    var nav = window.DocsNav || {};
    if (typeof nav.getArticle === "function") return nav.getArticle();
    return document.querySelector("article[contenteditable]") || document.querySelector("article");
  };
  function article() { return NS.getArticle(); }
  function tr(key, fallback) { return NS.tr(key, fallback); }
  function esc(value) { return NS.esc(value); }

  function save() {
    var a = article();
    if (!a) return;
    try { a.dispatchEvent(new Event("input", { bubbles: true })); } catch (error) { /* stub */ }
    if (typeof scheduleSave === "function") {
      var id = a.getAttribute ? (a.getAttribute("data-doc-id") || "current") : "current";
      scheduleSave(id, a.innerHTML);
    }
  }
  NS.save = save;

  var seq = 0;
  function uid() {
    seq += 1;
    return "eq" + Date.now().toString(36) + seq.toString(36);
  }
  NS.uid = uid;

  function status(key, fallback) {
    var message = tr(key, fallback);
    if (typeof NS.announce === "function") NS.announce(message);
    var nav = window.DocsNav || {};
    if (typeof nav.status === "function") nav.status(message);
  }
  NS.statusKey = status;

  function nodes(selector) {
    var a = article();
    if (!a || !a.querySelectorAll) return [];
    return Array.prototype.slice.call(a.querySelectorAll(selector));
  }
  NS.equationNodes = function () { return nodes("." + EQ_CLASS); };
  NS.referenceNodes = function () { return nodes("." + REF_CLASS); };

  function attr(node, name) { return node && node.getAttribute ? (node.getAttribute(name) || "") : ""; }
  function read(node) {
    return {
      id: attr(node, "data-docs-eq-id"),
      src: attr(node, "data-docs-eq-src"),
      description: attr(node, "data-docs-eq-desc"),
      kind: attr(node, "data-docs-eq-kind") === "display" ? "display" : "inline",
      numbered: attr(node, "data-docs-eq-numbered") === "1",
      number: attr(node, "data-docs-eq-num") || "",
      node: node
    };
  }
  NS.read = read;

  function equationHtml(record) {
    var display = record.kind === "display";
    var math = NS.latexToMathml(record.src, display, null);
    var label = NS.describe(record.src);
    var attrs = ' class="' + EQ_CLASS + (display ? " docs-eq-display" : " docs-eq-inline") + '"' +
      ' contenteditable="false" role="math" tabindex="0"' +
      ' data-docs-eq-id="' + esc(record.id) + '"' +
      ' data-docs-eq-src="' + esc(record.src) + '"' +
      ' data-docs-eq-kind="' + (display ? "display" : "inline") + '"' +
      ' data-docs-eq-numbered="' + (record.numbered && display ? "1" : "0") + '"' +
      ' data-docs-eq-num="' + esc(record.number || "") + '"' +
      ' data-docs-eq-desc="' + esc(record.description || "") + '"' +
      ' aria-label="' + esc(label) + '"';
    var number = display && record.numbered
      ? '<span class="docs-eq-num" aria-hidden="true">(' + esc(record.number || "") + ")</span>"
      : "";
    return "<span" + attrs + ">" + math + number + "</span>";
  }
  NS.equationHtml = equationHtml;

  function selection() {
    return window.getSelection ? window.getSelection() : null;
  }
  function caretInside(a) {
    var sel = selection();
    if (!sel || !sel.rangeCount || !a) return false;
    var range = sel.getRangeAt(0);
    var node = range.startContainer;
    if (!node) return false;
    if (node === a) return true;
    while (node && node !== a) node = node.parentNode;
    return node === a;
  }
  NS.caretInside = caretInside;

  function commit(range, html, fallbackText) {
    var sel = selection();
    var ok = false;
    try {
      if (sel && document.execCommand) {
        sel.removeAllRanges();
        sel.addRange(range);
        ok = !!document.execCommand("insertHTML", false, html);
      }
    } catch (error) { ok = false; }
    if (!ok) {
      try {
        range.deleteContents();
        range.insertNode(document.createTextNode(fallbackText || ""));
      } catch (fallbackError) { /* detached range: nothing to commit */ }
    }
    save();
    return ok;
  }
  NS.commit = commit;

  function commitCaret(html, fallbackText) {
    var a = article();
    var sel = selection();
    if (!caretInside(a) || !sel || !sel.rangeCount) {
      status("docs-eq-status-no-selection", "Place the cursor in the document first");
      return false;
    }
    commit(sel.getRangeAt(0), html, fallbackText);
    return true;
  }

  function replaceNode(node, html, fallbackText) {
    if (!node || !node.parentNode || !document.createRange) return false;
    var range = document.createRange();
    try { range.selectNode(node); } catch (error) { return false; }
    var ok = false;
    var sel = selection();
    try {
      if (sel && document.execCommand) {
        sel.removeAllRanges();
        sel.addRange(range);
        ok = !!document.execCommand("insertHTML", false, html);
      }
    } catch (commitError) { ok = false; }
    if (!ok) {
      try { node.outerHTML = html; } catch (outerError) { return false; }
    }
    save();
    return ok;
  }
  NS.replaceNode = replaceNode;

  function apply(config, target) {
    var src = String(config.src || "").trim();
    if (!src) {
      status("docs-eq-status-empty-source", "Enter a LaTeX expression");
      return null;
    }
    if (src.length > NS.LIMITS.input) src = src.slice(0, NS.LIMITS.input);
    var record = {
      id: target ? read(target).id : uid(),
      src: src,
      description: String(config.description || "").trim(),
      kind: config.kind === "display" ? "display" : "inline",
      numbered: !!config.numbered && config.kind === "display",
      number: target ? read(target).number : ""
    };
    var html = equationHtml(record);
    if (target) replaceNode(target, html, src);
    else if (!commitCaret(html, src)) return null;
    renumber();
    status(target ? "docs-eq-status-updated" : "docs-eq-status-inserted",
      target ? "Equation updated" : "Equation inserted");
    return record.id;
  }
  NS.apply = apply;

  function remove(id) {
    var list = NS.equationNodes();
    for (var i = 0; i < list.length; i += 1) {
      if (read(list[i]).id !== id) continue;
      replaceNode(list[i], "", "");
      renumber();
      status("docs-eq-status-deleted", "Equation deleted");
      return true;
    }
    status("docs-eq-status-not-found", "Equation not found");
    return false;
  }
  NS.remove = remove;

  function renumber() {
    var a = article();
    if (!a) return 0;
    var marks = a.querySelectorAll ? a.querySelectorAll("h1,h2,h3,h4,h5,h6,." + EQ_CLASS) : [];
    var counters = [0, 0, 0, 0, 0, 0];
    var section = "";
    var perSection = {};
    var numbered = 0;
    for (var i = 0; i < marks.length; i += 1) {
      var el = marks[i];
      var tag = String(el.tagName || "").toLowerCase();
      if (/^h[1-6]$/.test(tag)) {
        var level = parseInt(tag.charAt(1), 10) - 1;
        counters[level] += 1;
        for (var deeper = level + 1; deeper < counters.length; deeper += 1) counters[deeper] = 0;
        section = counters.slice(0, level + 1).join(".");
        continue;
      }
      if (!el.getAttribute || el.getAttribute("data-docs-eq-kind") !== "display") continue;
      if (el.getAttribute("data-docs-eq-numbered") !== "1") continue;
      var key = section || "0";
      perSection[key] = (perSection[key] || 0) + 1;
      var value = section ? section + "." + perSection[key] : String(perSection[key]);
      if (el.getAttribute("data-docs-eq-num") !== value) el.setAttribute("data-docs-eq-num", value);
      var label = el.querySelector ? el.querySelector(".docs-eq-num") : null;
      if (label && label.textContent !== "(" + value + ")") label.textContent = "(" + value + ")";
      numbered += 1;
    }
    updateReferences();
    return numbered;
  }
  NS.renumber = renumber;

  function findEquation(id) {
    var list = NS.equationNodes();
    for (var i = 0; i < list.length; i += 1) if (read(list[i]).id === id) return list[i];
    return null;
  }
  NS.findEquation = findEquation;

  function updateReferences() {
    var refs = NS.referenceNodes();
    for (var i = 0; i < refs.length; i += 1) {
      var ref = refs[i];
      var target = findEquation(attr(ref, "data-docs-eq-ref-id"));
      var text = tr("docs-eq-term", "Equation") + " " + (target ? (read(target).number || "?") : "?");
      if (ref.textContent !== text) ref.textContent = text;
    }
    return refs.length;
  }
  NS.updateReferences = updateReferences;

  function insertReference(id) {
    var target = findEquation(id);
    if (!target) {
      status("docs-eq-status-not-found", "Equation not found");
      return false;
    }
    var number = read(target).number || "?";
    var text = tr("docs-eq-term", "Equation") + " " + number;
    var html = '<span class="' + REF_CLASS + '" contenteditable="false" data-docs-eq-ref-id="' +
      esc(id) + '">' + esc(text) + "</span>";
    if (!commitCaret(html, text)) return false;
    updateReferences();
    status("docs-eq-status-ref-inserted", "Reference inserted");
    return true;
  }
  NS.insertReference = insertReference;

  function records(query) {
    var all = NS.equationNodes().map(read);
    var q = String(query || "").trim().toLowerCase();
    if (!q) return all;
    return all.filter(function (item) {
      return (item.src + " " + item.description + " " + item.number).toLowerCase().indexOf(q) >= 0;
    });
  }
  NS.records = records;

  NS.numberedRecords = function () {
    return NS.records().filter(function (item) { return item.kind === "display" && item.numbered; });
  };
})(window);
