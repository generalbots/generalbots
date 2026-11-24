"use strict";
/* Unified navigation pane (#2485): headings tree (drag to reorder), pages,
 * search results, comments and bookmarks in one surface. Clicking any entry
 * jumps to its target and highlights it. */

(function () {
  var NS = window.DocsNav;
  if (!NS) return;
  var tr = NS.tr;
  var searchMatches = []; // #2457 — search result rows jump to the matched text node.

  function host() {
    return document.getElementById("docs-nav-pane");
  }

  function activeTab() {
    var pane = host();
    var el = pane && pane.querySelector(".gb-nav-tab.is-active");
    return el ? el.getAttribute("data-nav-tab") : "headings";
  }

  function setActiveTab(name) {
    var pane = host();
    if (!pane) return;
    Array.prototype.forEach.call(pane.querySelectorAll(".gb-nav-tab"), function (tab) {
      tab.classList.toggle("is-active", tab.getAttribute("data-nav-tab") === name);
    });
    Array.prototype.forEach.call(pane.querySelectorAll(".gb-nav-section"), function (section) {
      section.hidden = section.getAttribute("data-nav-section") !== name;
    });
    renderSection(name);
  }

  NS.openPane = function () {
    var sidebar = document.getElementById("sidebar-panels");
    var tab = document.querySelector('[data-sidebar-tab="panels"]');
    if (tab && typeof tab.click === "function") tab.click();
    if (sidebar) sidebar.style.display = "flex";
    var files = document.getElementById("sidebar-files");
    if (files) files.style.display = "none";
    NS.refreshPane();
  };

  function build() {
    var pane = host();
    if (!pane || pane.dataset.built) return;
    pane.dataset.built = "1";
    var tabs = [
      ["headings", "docs-nav-headings", "Headings"],
      ["pages", "docs-nav-pages", "Pages"],
      ["results", "docs-nav-results", "Results"],
      ["comments", "docs-nav-comments", "Comments"],
      ["bookmarks", "docs-nav-bookmarks", "Bookmarks"]
    ];
    var html = ['<div class="gb-nav-tabs" role="tablist">'];
    tabs.forEach(function (entry) {
      html.push('<button type="button" class="gb-nav-tab" role="tab" data-nav-tab="' + entry[0] + '">' +
        NS.escapeHtml(tr(entry[1], entry[2])) + "</button>");
    });
    html.push("</div>");
    html.push('<div class="gb-nav-body">');
    tabs.forEach(function (entry) {
      html.push('<div class="gb-nav-section" data-nav-section="' + entry[0] + '" role="tabpanel"></div>');
    });
    html.push('</div><div class="gb-nav-status" id="docs-nav-status" role="status" aria-live="polite"></div>');
    pane.innerHTML = html.join("");
    pane.addEventListener("click", function (event) {
      var tab = event.target.closest(".gb-nav-tab");
      if (tab) {
        setActiveTab(tab.getAttribute("data-nav-tab"));
        return;
      }
      var jump = event.target.closest("[data-nav-jump]");
      if (jump) {
        event.preventDefault();
        NS.jumpTo(resolveJump(jump));
      }
    });
    setActiveTab("headings");
  }

  function resolveJump(row) {
    var article = NS.getArticle();
    // #2457 — search rows carry the match index. Jump to the matched text node
    // itself; the old path used `list[-1]` (no headingIndex) -> undefined, so
    // search results were dead rows.
    var matchIndex = parseInt(row.getAttribute("data-nav-match"), 10);
    if (!isNaN(matchIndex) && searchMatches[matchIndex]) {
      var match = searchMatches[matchIndex];
      // Return an Element (text nodes have no scrollIntoView/flash support).
      var target = (match.node && match.node.parentElement) || match.block;
      return target;
    }
    var id = row.getAttribute("data-nav-target") || "";
    if (id) {
      var node = article && article.querySelector('[id="' + id + '"]');
      if (node) return node;
    }
    var offset = parseInt(row.getAttribute("data-nav-offset"), 10);
    if (!isNaN(offset) && NS.jumpToOffset) return NS.jumpToOffset(offset);
    var index = parseInt(row.getAttribute("data-nav-index"), 10);
    var list = NS.headings();
    return !isNaN(index) ? list[index] : null;
  }

  function emptyRow(key, fallback) {
    return '<div class="gb-nav-empty">' + NS.escapeHtml(tr(key, fallback)) + "</div>";
  }

  function renderHeadings(section) {
    var list = NS.headings();
    if (!list.length) {
      section.innerHTML = emptyRow("docs-nav-empty", "Nothing here yet");
      return;
    }
    var html = ['<div class="gb-nav-hint">' + NS.escapeHtml(tr("docs-nav-drag-hint", "Drag to reorder")) + "</div>"];
    list.forEach(function (heading, index) {
      var level = parseInt(heading.tagName.slice(1), 10) || 1;
      var id = heading.getAttribute("id") || ("sec-" + index);
      html.push('<div class="gb-nav-row gb-nav-heading" draggable="true" data-nav-index="' + index +
        '" data-nav-target="' + NS.escapeHtml(id) + '" data-nav-jump="1" style="padding-left:' +
        ((level - 1) * 12 + 8) + 'px">' +
        '<span class="gb-nav-grip" aria-hidden="true">&#9776;</span>' +
        NS.escapeHtml((heading.textContent || "").trim() || id) + "</div>");
    });
    section.innerHTML = html.join("");
    bindDrag(section, list);
  }

  function bindDrag(section, list) {
    var dragged = null;
    Array.prototype.forEach.call(section.querySelectorAll(".gb-nav-heading"), function (row) {
      row.addEventListener("dragstart", function (event) {
        dragged = row;
        if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
      });
      row.addEventListener("dragover", function (event) {
        event.preventDefault();
        if (dragged && dragged !== row) row.classList.add("gb-nav-drop");
      });
      row.addEventListener("dragleave", function () {
        row.classList.remove("gb-nav-drop");
      });
      row.addEventListener("drop", function (event) {
        event.preventDefault();
        row.classList.remove("gb-nav-drop");
        if (!dragged || dragged === row) return;
        var from = parseInt(dragged.getAttribute("data-nav-index"), 10);
        var to = parseInt(row.getAttribute("data-nav-index"), 10);
        if (isNaN(from) || isNaN(to)) return;
        var article = NS.getArticle();
        var source = list[from];
        var target = list[to];
        if (!article || !source || !target || source === target) return;
        var block = NS.collectSection(source);
        var before = from < to ? NS.collectSection(target).slice(-1)[0].nextElementSibling : target;
        block.forEach(function (node) {
          article.insertBefore(node, before);
        });
        NS.save();
        NS.status(tr("docs-nav-reordered", "Headings reordered"));
        NS.refreshPane();
      });
    });
  }

  function renderPages(section) {
    var article = NS.getArticle();
    if (!article) {
      section.innerHTML = emptyRow("docs-nav-empty", "Nothing here yet");
      return;
    }
    var pageH = parseInt(article.dataset.pageH || "1123", 10) || 1123;
    var pages = Math.max(1, Math.ceil((article.scrollHeight || article.offsetHeight || pageH) / pageH));
    var html = [];
    for (var i = 0; i < pages; i++) {
      html.push('<div class="gb-nav-row" data-nav-offset="' + (i * pageH) + '" data-nav-jump="1">' +
        NS.escapeHtml(tr("docs-nav-page", "Page {$n}", { n: i + 1 })) + "</div>");
    }
    section.innerHTML = html.join("");
  }

  function renderResults(section) {
    // #2457 — clear stale matches on every render so a prior query's rows
    // can never jump to the wrong text after an empty/no-results render.
    searchMatches = [];
    var query = NS.currentQuery || "";
    if (!query) {
      section.innerHTML = emptyRow("docs-nav-search-hint", "Find something first");
      return;
    }
    var matches = NS.findMatches ? NS.findMatches(query, {}) : [];
    if (!matches.length) {
      section.innerHTML = emptyRow("docs-nav-no-results", "No matches");
      return;
    }
    searchMatches = matches.slice(0, 200);
    var html = [];
    searchMatches.forEach(function (match, i) {
      // #2457 — show a real context snippet (and scope) rather than the raw
      // `body:/footnotes:` internal scope path the UAT flagged.
      var blockText = (match.block && match.block.textContent || "").replace(/\s+/g, " ").trim();
      var snippet = blockText.slice(0, 120) || String(match.text || "").slice(0, 90);
      var label = (match.scope ? "[" + match.scope + "] " : "") + snippet;
      html.push('<div class="gb-nav-row" data-nav-match="' + i +
        '" data-nav-target="' + NS.escapeHtml(match.id || "") +
        '" data-nav-index="' + (match.headingIndex == null ? -1 : match.headingIndex) +
        '" data-nav-jump="1">' + NS.escapeHtml(label) + "</div>");
    });
    section.innerHTML = html.join("");
  }

  function renderComments(section) {
    var article = NS.getArticle();
    var nodes = [];
    if (article) {
      try {
        nodes = Array.prototype.slice.call(article.querySelectorAll("[data-gb-comment],.gb-comment,.dc-comment,[data-comment-id]"));
      } catch (error) {
        nodes = [];
      }
    }
    var stored = NS.readStore("comments", []);
    if (!nodes.length && !stored.length) {
      section.innerHTML = emptyRow("docs-nav-no-comments", "No comments yet");
      return;
    }
    var html = [];
    nodes.forEach(function (node) {
      html.push('<div class="gb-nav-row" data-nav-target="' + NS.escapeHtml(node.getAttribute("id") || "") +
        '" data-nav-jump="1">' + NS.escapeHtml((node.textContent || "").slice(0, 90)) + "</div>");
    });
    stored.forEach(function (item) {
      html.push('<div class="gb-nav-row" data-nav-target="' + NS.escapeHtml(item.id || "") +
        '" data-nav-jump="1">' + NS.escapeHtml(String(item.text || "").slice(0, 90)) + "</div>");
    });
    section.innerHTML = html.join("");
  }

  function renderBookmarks(section) {
    var items = NS.readStore("bookmarks", []);
    if (!items.length) {
      section.innerHTML = emptyRow("docs-bm-none", "No bookmarks yet");
      return;
    }
    var html = [];
    items.forEach(function (item) {
      html.push('<div class="gb-nav-row gb-nav-bookmark" data-nav-target="' + NS.escapeHtml(item.id || "") +
        '" data-nav-jump="1"><span class="gb-nav-label">' + NS.escapeHtml(item.name || "") + "</span>" +
        '<button type="button" class="gb-nav-mini" data-bm-goto="' + NS.escapeHtml(item.id || "") + '">' +
        NS.escapeHtml(tr("docs-bm-goto", "Go to")) + "</button>" +
        '<button type="button" class="gb-nav-mini" data-bm-delete="' + NS.escapeHtml(item.id || "") + '">' +
        NS.escapeHtml(tr("docs-bm-delete", "Delete")) + "</button></div>");
    });
    section.innerHTML = html.join("");
  }

  function renderSection(name) {
    var pane = host();
    if (!pane) return;
    var section = pane.querySelector('.gb-nav-section[data-nav-section="' + name + '"]');
    if (!section) return;
    if (name === "headings") renderHeadings(section);
    else if (name === "pages") renderPages(section);
    else if (name === "results") renderResults(section);
    else if (name === "comments") renderComments(section);
    else if (name === "bookmarks") renderBookmarks(section);
  }

  NS.refreshPane = function () {
    build();
    renderSection(activeTab());
  };
  NS.navSetQuery = function (query) {
    NS.currentQuery = query || "";
    build();
    setActiveTab("results");
  };
})();
