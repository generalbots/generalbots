"use strict";
/* Docs KB citation navigation (#2499).
 *
 * Consumer of the `docs-kb-citation-open` event emitted by 10b_kb_citations.js.
 * Resolves the cited page/section/clause inside the open document, scrolls to
 * it and flashes it. When the target is not part of the current document an
 * accessible status message is announced instead. Every visible word comes
 * from an existing i18n key; server-derived values are written only through
 * textContent (never innerHTML). */
(function (window) {
  var EVENT = "docs-kb-citation-open";

  function tr(key, fallback, params) {
    if (typeof docsT === "function") return docsT(key, fallback, params);
    if (window.gbT) return window.gbT(key, fallback, params);
    return fallback || key;
  }

  function lib() {
    return window.DocsKbCitations || null;
  }

  function pageOf(citation) {
    var api = lib();
    if (api && typeof api.sourcePage === "function") return api.sourcePage(citation.source);
    var raw = citation.source && citation.source.page != null
      ? citation.source.page
      : citation.source && citation.source.locator;
    var value = parseInt(raw, 10);
    return isFinite(value) && value > 0 ? value : 0;
  }

  function where(citation) {
    var api = lib();
    if (api && typeof api.locationLabel === "function") return api.locationLabel(citation.source);
    return "";
  }

  function activeArticle() {
    var article = document.querySelector("#docs-content article[contenteditable]");
    if (article) return article;
    var standalone = document.getElementById("doc-content");
    return standalone && standalone.isContentEditable ? standalone : null;
  }

  function scrollHost(element) {
    var node = element && element.parentElement;
    while (node && node !== document.body && node !== document.documentElement) {
      var overflow = window.getComputedStyle(node).overflowY;
      if ((overflow === "auto" || overflow === "scroll") && node.scrollHeight > node.clientHeight + 4) {
        return node;
      }
      node = node.parentElement;
    }
    return document.getElementById("docs-content") || node;
  }

  function flash(node) {
    if (!node || !node.classList) return;
    node.classList.add("docs-kb-citation-flash");
    window.setTimeout(function () {
      node.classList.remove("docs-kb-citation-flash");
    }, 1600);
  }

  function scrollToNode(node) {
    if (!node) return false;
    var host = scrollHost(node);
    if (host && typeof host.scrollTo === "function") {
      var box = node.getBoundingClientRect();
      var hostBox = host.getBoundingClientRect();
      host.scrollTo({
        top: Math.max(0, host.scrollTop + box.top - hostBox.top - 16),
        behavior: "smooth"
      });
    } else if (typeof node.scrollIntoView === "function") {
      node.scrollIntoView({ behavior: "smooth", block: "center" });
    }
    flash(node);
    return true;
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

  /* A section resolves to a real heading: by id (the anchors 14_toc_anchors.js
   * assigns), by its slug, or by its visible text. */
  function findHeading(article, citation) {
    var source = citation.source || {};
    var locator = source.locator == null ? "" : String(source.locator);
    if (typeof window.docsAssignHeadingIds === "function") window.docsAssignHeadingIds(article);
    var headings = Array.prototype.slice.call(article.querySelectorAll("h1,h2,h3,h4,h5,h6"));
    var index;
    if (locator) {
      var direct = document.getElementById(locator);
      if (direct && article.contains(direct)) return direct;
      var expected = "sec-" + slugify(locator);
      for (index = 0; index < headings.length; index += 1) {
        if (headings[index].id === locator ||
            headings[index].id === expected ||
            headings[index].getAttribute("data-gb-toc-anchor") === locator) {
          return headings[index];
        }
      }
    }
    var path = citation.heading_path;
    var wanted = locator || (path && path.length ? String(path[path.length - 1]) : "");
    if (!wanted) return null;
    var needle = wanted.toLowerCase();
    for (index = 0; index < headings.length; index += 1) {
      if ((headings[index].textContent || "").toLowerCase().indexOf(needle) >= 0) return headings[index];
    }
    return null;
  }

  /* A clause (or slide/cell/reference) resolves to the first document scope
   * containing its locator text, reusing the shared nav index. */
  function findClause(article, citation) {
    var heading = findHeading(article, citation);
    if (heading) return heading;
    var source = citation.source || {};
    var path = citation.heading_path;
    var wanted = source.locator == null ? "" : String(source.locator);
    if (!wanted && path && path.length) wanted = String(path[path.length - 1]);
    if (!wanted) return null;
    var needle = wanted.toLowerCase();
    var scopes = window.DocsNav && typeof window.DocsNav.searchScopes === "function"
      ? window.DocsNav.searchScopes(article)
      : [];
    for (var i = 0; i < scopes.length; i += 1) {
      if ((scopes[i].text || "").toLowerCase().indexOf(needle) >= 0) return scopes[i].node;
    }
    return null;
  }

  function pageTarget(article, page) {
    var thumbs = document.querySelectorAll("#docs-content [data-page-index]");
    for (var i = 0; i < thumbs.length; i += 1) {
      if (parseInt(thumbs[i].getAttribute("data-page-index"), 10) === page - 1) {
        return { node: thumbs[i] };
      }
    }
    var breaks = article.querySelectorAll(
      ".docs-page-break, hr[data-page-break='true'], .gb-doc-page-break, [data-gb-page-break]"
    );
    if (page - 1 <= breaks.length) {
      if (page === 1) return { node: article.firstElementChild || article };
      return breaks[page - 2] ? { node: breaks[page - 2] } : null;
    }
    var pageH = parseInt(article.dataset.pageH || "1123", 10);
    if (!pageH || pageH < 200) pageH = 1123;
    var total = Math.max(1, Math.ceil((article.scrollHeight || article.offsetHeight || pageH) / pageH));
    if (page > total) return null;
    return { top: article.offsetTop + (page - 1) * pageH, host: scrollHost(article) };
  }

  function locate(citation) {
    var article = activeArticle();
    if (!article) return null;
    var kind = (citation.source && citation.source.kind) || "page";
    if (kind === "page") {
      var page = pageOf(citation);
      return page ? pageTarget(article, page) : null;
    }
    if (kind === "section") {
      var heading = findHeading(article, citation);
      return heading ? { node: heading } : null;
    }
    var node = findClause(article, citation);
    return node ? { node: node } : null;
  }

  function statusRegion() {
    var region = document.getElementById("docs-kb-citation-status");
    if (region) return region;
    var host = document.querySelector("[data-docs-kb-citations]") ||
      document.getElementById("docs-content") ||
      document.body;
    region = document.createElement("div");
    region.id = "docs-kb-citation-status";
    region.className = "docs-kb-citation-status";
    region.setAttribute("role", "status");
    region.setAttribute("aria-live", "polite");
    if (host.parentNode) host.parentNode.insertBefore(region, host.nextSibling);
    else host.appendChild(region);
    return region;
  }

  function announce(citation, found) {
    var region = statusRegion();
    if (!region) return;
    var position = where(citation);
    var text = found
      ? tr("action-view", "View") + (position ? " " + position : "")
      : tr("docs-nav-empty", "Nothing here yet") + (position ? " · " + position : "");
    region.textContent = text;
  }

  function onOpen(event) {
    var citation = event && event.detail;
    if (!citation) return;
    var target = locate(citation);
    if (!target) {
      announce(citation, false);
      return;
    }
    if (target.node) {
      scrollToNode(target.node);
    } else if (target.top != null) {
      var host = target.host;
      if (host && typeof host.scrollTo === "function") {
        host.scrollTo({ top: Math.max(0, target.top - 16), behavior: "smooth" });
      } else if (host) {
        host.scrollTop = Math.max(0, target.top - 16);
      }
      flash(activeArticle());
    }
    announce(citation, true);
  }

  document.addEventListener(EVENT, onOpen);

  window.DocsKbCitationNav = {
    locate: locate,
    event: EVENT
  };
})(window);
