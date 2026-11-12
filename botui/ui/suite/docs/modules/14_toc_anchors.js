"use strict";
/* DOC-04 (#2252) — real in-document anchors for the Docs table of contents.
 *
 * The server TOC/outline panels list headings but link to `#pos-<offset>`,
 * an id that exists nowhere in the document, so a click could only be faked by
 * guessing the nearest `<hN>` from an HTML character offset. Headings also had
 * no id, which is why the exported PDF carried no internal destinations.
 *
 * This module:
 *   1. gives every heading a stable `sec-<slug>` id when a document is
 *      rendered/loaded (ids are saved with the content, so the PDF exporter
 *      can turn them into destinations);
 *   2. rewrites the sidebar TOC/outline entries to those real anchors and
 *      scrolls `#docs-content` to the heading element on click;
 *   3. can insert an in-document TOC block whose entries are anchor links
 *      (that block is what the PDF export renders as clickable TOC lines).
 */

(function () {
  function activeArticle() {
    var article = document.querySelector("#docs-content article[contenteditable]");
    if (article) return article;
    var standalone = document.getElementById("doc-content");
    return standalone && standalone.isContentEditable ? standalone : null;
  }

  function headingsOf(article) {
    if (!article) return [];
    return Array.prototype.slice.call(article.querySelectorAll("h1,h2,h3,h4,h5,h6"));
  }

  function slugify(text) {
    return String(text || "")
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 48);
  }

  function persist(article) {
    if (typeof scheduleSave !== "function" || !article) return;
    scheduleSave(article.dataset ? article.dataset.docId : null, article.innerHTML);
  }

  /* Stable ids: derived from the heading text, so the same document produces
     the same anchors after a reload (the id is also persisted in the HTML). */
  function assignHeadingIds(article) {
    var list = headingsOf(article);
    if (!list.length) return list;
    var used = Object.create(null);
    var added = false;
    list.forEach(function (heading) {
      var id = (heading.getAttribute("id") || "").trim();
      if (!id) {
        var slug = slugify(heading.textContent) || "section";
        var candidate = "sec-" + slug;
        var suffix = 2;
        while (used[candidate]) {
          candidate = "sec-" + slug + "-" + suffix;
          suffix += 1;
        }
        heading.setAttribute("id", candidate);
        id = candidate;
        added = true;
      }
      used[id] = true;
      heading.setAttribute("data-gb-toc-anchor", id);
    });
    if (added) persist(article);
    return list;
  }

  /* `#docs-content` is the editor scroll container; the article itself only
     scrolls on very old shells, so walk up to the first scrollable ancestor. */
  function scrollHostFor(element) {
    var node = element.parentElement;
    while (node && node !== document.body && node !== document.documentElement) {
      var overflow = window.getComputedStyle(node).overflowY;
      if ((overflow === "auto" || overflow === "scroll") && node.scrollHeight > node.clientHeight + 4) {
        return node;
      }
      node = node.parentElement;
    }
    return document.getElementById("docs-content") || node;
  }

  function flash(heading) {
    var previous = heading.style.outline;
    heading.style.outline = "2px solid #3b82f6";
    heading.style.outlineOffset = "2px";
    setTimeout(function () {
      heading.style.outline = previous;
      heading.style.outlineOffset = "";
    }, 1200);
  }

  function scrollToHeading(heading) {
    if (!heading) return false;
    var host = scrollHostFor(heading);
    if (!host) {
      heading.scrollIntoView({ behavior: "smooth", block: "start" });
      flash(heading);
      return true;
    }
    var box = heading.getBoundingClientRect();
    var hostBox = host.getBoundingClientRect();
    var top = host.scrollTop + (box.top - hostBox.top) - 12;
    if (typeof host.scrollTo === "function") {
      host.scrollTo({ top: Math.max(0, top), behavior: "smooth" });
    } else {
      host.scrollTop = Math.max(0, top);
    }
    flash(heading);
    return true;
  }

  /* Resolves a sidebar entry to a heading element: by real anchor id first,
     then by the entry's ordinal position (the server emits TOC/outline items
     in document order), which is exact even for `#pos-<offset>` links. */
  function resolveHeading(link) {
    var article = activeArticle();
    if (!article) return null;
    var list = assignHeadingIds(article);
    var anchor = link.getAttribute("data-gb-anchor") || link.getAttribute("href") || "";
    if (anchor.charAt(0) === "#") anchor = anchor.slice(1);
    if (anchor && anchor.indexOf("pos-") !== 0) {
      var found = document.getElementById(anchor);
      if (found && article.contains(found)) return found;
    }
    var row = link.closest(".dc-toc-entry, .dc-outline-item, .gb-doc-toc-entry");
    if (!row || !row.parentNode) return null;
    var rows = Array.prototype.filter.call(row.parentNode.children, function (node) {
      return node.classList && (
        node.classList.contains("dc-toc-entry") ||
        node.classList.contains("dc-outline-item") ||
        node.classList.contains("gb-doc-toc-entry")
      );
    });
    var index = rows.indexOf(row);
    if (index >= 0 && list[index]) return list[index];
    return null;
  }

  function tocTitle() {
    if (typeof docsT === "function") return docsT("docs-toc-title", "Sumário");
    return "Sumário";
  }

  function insertTocIntoDocument() {
    var article = activeArticle();
    if (!article) return false;
    var list = assignHeadingIds(article);
    if (!list.length) return false;
    Array.prototype.forEach.call(article.querySelectorAll("[data-gb-toc]"), function (node) {
      node.remove();
    });
    // One top-level <p> per entry: the PDF block parser turns each paragraph
    // into its own line, so every TOC entry becomes a link annotation (#2252).
    var fragment = document.createDocumentFragment();
    var title = document.createElement("p");
    title.className = "gb-doc-toc-title";
    title.setAttribute("data-gb-toc", "1");
    title.style.cssText =
      "margin:0 0 6px;padding:8px 12px 0;font-weight:700;border-top:1px solid #cbd5e1;";
    title.textContent = tocTitle();
    fragment.appendChild(title);
    list.forEach(function (heading) {
      var level = parseInt(heading.tagName.slice(1), 10) || 1;
      var row = document.createElement("p");
      row.className = "gb-doc-toc-entry";
      row.setAttribute("data-gb-toc", "1");
      row.style.cssText = "margin:2px 0;padding-left:" + (12 + (level - 1) * 16) + "px;";
      var link = document.createElement("a");
      link.setAttribute("href", "#" + heading.getAttribute("id"));
      link.textContent = (heading.textContent || "").trim() || heading.getAttribute("id");
      link.style.cssText = "color:#1d4ed8;text-decoration:none;";
      row.appendChild(link);
      fragment.appendChild(row);
    });
    var spacer = document.createElement("p");
    spacer.setAttribute("data-gb-toc", "1");
    spacer.innerHTML = "<br>";
    fragment.appendChild(spacer);
    article.insertBefore(fragment, article.firstChild);
    persist(article);
    if (typeof docsAfterContentLoad === "function") docsAfterContentLoad(article);
    return true;
  }

  function addInsertButton(panel) {
    if (!panel || panel.querySelector("#gb-toc-insert")) return;
    var header = panel.querySelector(".dc-panel-header");
    if (!header) return;
    var button = document.createElement("button");
    button.type = "button";
    button.id = "gb-toc-insert";
    button.className = "btn-icon";
    button.textContent = "Inserir no documento";
    button.title = "Insert a table of contents with links to every heading";
    button.style.cssText =
      "background:#0f766e;color:white;border:none;padding:4px 10px;border-radius:4px;" +
      "cursor:pointer;font-size:12px;margin-left:6px;";
    button.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      insertTocIntoDocument();
    });
    header.appendChild(button);
  }

  /* Points every sidebar entry at the heading element it describes. */
  function enhancePanels() {
    var panel = document.getElementById("docs-panel-body");
    if (!panel) return;
    var toc = panel.querySelector("#toc-panel, #outline-panel");
    var article = activeArticle();
    if (!toc || !article) return;
    var list = assignHeadingIds(article);
    var links = toc.querySelectorAll(".dc-toc-entry a, .dc-outline-item a");
    for (var index = 0; index < links.length; index++) {
      var heading = list[index];
      if (!heading) continue;
      var id = heading.getAttribute("id");
      links[index].setAttribute("href", "#" + id);
      links[index].setAttribute("data-gb-anchor", id);
      links[index].title = (heading.textContent || "").trim();
    }
    if (toc.id === "toc-panel") addInsertButton(toc);
  }

  /* The TOC/outline fragments are built from the content the caller posts; the
     toolbar buttons post only `{"id":"current"}`, so the panel used to render
     "no headings". Post the live document HTML with the request (#2252). */
  function requestPath(detail) {
    if (detail && detail.path) return detail.path;
    var elt = detail && detail.elt;
    return (elt && (elt.getAttribute("hx-post") || elt.getAttribute("hx-get"))) || "";
  }

  document.addEventListener("htmx:configRequest", function (event) {
    var path = requestPath(event.detail || {});
    if (!/\/suite\/docs\/fragments\/(toc|outline)$/.test(path)) return;
    var article = activeArticle();
    if (!article) return;
    assignHeadingIds(article);
    var parameters = (event.detail || {}).parameters;
    if (!parameters) return;
    parameters.content = article.innerHTML;
    var docId = article.dataset && article.dataset.docId;
    if (docId && (!parameters.id || parameters.id === "current")) parameters.id = docId;
  });

  document.addEventListener("htmx:afterSwap", function (event) {
    if (event.target && event.target.id === "docs-panel-body") enhancePanels();
    if (event.target && event.target.id === "docs-content") {
      assignHeadingIds(activeArticle());
    }
  });

  document.addEventListener("click", function (event) {
    var target = event.target;
    var link = target && target.closest
      ? target.closest(".dc-toc-entry a, .dc-outline-item a, .gb-doc-toc-entry a")
      : null;
    if (!link) return;
    var heading = resolveHeading(link);
    if (!heading) return;
    event.preventDefault();
    event.stopPropagation();
    scrollToHeading(heading);
  }, true);

  window.docsAssignHeadingIds = assignHeadingIds;
  window.docsScrollToHeading = scrollToHeading;
  window.docsInsertTocIntoDocument = insertTocIntoDocument;

  function boot() {
    assignHeadingIds(activeArticle());
    enhancePanels();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
