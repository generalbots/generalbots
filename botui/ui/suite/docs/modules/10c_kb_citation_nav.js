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
  var NAV_EVENT = "docs-kb-citation-nav";
  var POPOVER_ID = "docs-kb-citation-source";
  var popoverNode = null;
  var lastAnchor = null;

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

  function unpack(detail) {
    if (!detail) return { citation: null, anchor: null };
    if (detail.citation && typeof detail.citation === "object") {
      return { citation: detail.citation, anchor: detail.anchor || null };
    }
    return { citation: detail, anchor: null };
  }

  function isEditable(node) {
    if (!node || !node.tagName) return false;
    var tag = String(node.tagName).toLowerCase();
    return tag === "input" || tag === "textarea" || tag === "select" ||
      node.isContentEditable === true;
  }

  function chipAt(index) {
    var api = lib();
    var chips = api && typeof api.getChips === "function" ? api.getChips() : [];
    for (var i = 0; i < chips.length; i += 1) {
      if (parseInt(chips[i].getAttribute("data-docs-kb-citation-index"), 10) === index) {
        return chips[i];
      }
    }
    return null;
  }

  function ensurePopover() {
    if (popoverNode && popoverNode.parentNode) return popoverNode;
    var panel = document.createElement("div");
    panel.id = POPOVER_ID;
    panel.className = "docs-kb-citation-source";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", tr("docs-references", "References"));
    panel.style.position = "fixed";
    panel.style.zIndex = "var(--z-system, 1000)";
    panel.style.width = "min(360px, calc(100vw - 24px))";
    panel.style.maxHeight = "60vh";
    panel.style.overflow = "auto";
    panel.style.padding = "12px 14px";
    panel.style.border = "1px solid var(--palette-334155)";
    panel.style.borderRadius = "8px";
    panel.style.background = "var(--palette-0f172a)";
    panel.style.color = "var(--palette-e2e8f0)";
    panel.style.boxShadow = "0 10px 30px rgba(0,0,0,0.45)";
    panel.style.font = "inherit";

    var head = document.createElement("div");
    head.className = "docs-kb-citation-source-head";
    head.style.display = "flex";
    head.style.alignItems = "center";
    head.style.gap = "8px";
    head.style.marginBottom = "8px";

    var ref = document.createElement("strong");
    ref.className = "docs-kb-citation-source-ref";
    ref.style.flex = "1";
    head.appendChild(ref);

    var close = document.createElement("button");
    close.type = "button";
    close.className = "docs-kb-citation-source-close";
    close.textContent = tr("action-close", "Close");
    close.setAttribute("aria-label", tr("action-close", "Close"));
    close.style.cursor = "pointer";
    close.addEventListener("click", hideSource);
    head.appendChild(close);
    panel.appendChild(head);

    var quote = document.createElement("blockquote");
    quote.className = "docs-kb-citation-source-quote";
    quote.style.margin = "0";
    quote.style.paddingLeft = "10px";
    quote.style.borderLeft = "2px solid var(--palette-3b82f6)";
    quote.style.fontSize = "13px";
    quote.style.whiteSpace = "pre-wrap";
    panel.appendChild(quote);

    document.body.appendChild(panel);
    popoverNode = panel;
    return panel;
  }

  function hideSource() {
    if (popoverNode && popoverNode.parentNode) {
      popoverNode.parentNode.removeChild(popoverNode);
    }
    popoverNode = null;
    lastAnchor = null;
  }

  function positionPopover(anchor) {
    var panel = ensurePopover();
    var width = 360;
    var rect = anchor && typeof anchor.getBoundingClientRect === "function"
      ? anchor.getBoundingClientRect() : null;
    var viewW = window.innerWidth || 1024;
    var viewH = window.innerHeight || 768;
    var left = Math.max(8, viewW - width - 16);
    var top = 72;
    if (rect) {
      left = Math.min(Math.max(8, rect.left), Math.max(8, viewW - width - 8));
      top = rect.bottom + 8;
      if (top + 180 > viewH && rect.top > 180) top = Math.max(8, rect.top - 188);
    }
    panel.style.left = Math.round(left) + "px";
    panel.style.top = Math.round(top) + "px";
  }

  function showSource(citation, anchor) {
    var panel = ensurePopover();
    var api = lib();
    var ref = api && typeof api.label === "function"
      ? api.label(citation)
      : (citation && citation.file) || "";
    var quote = api && typeof api.quoteOf === "function" ? api.quoteOf(citation) : "";
    var title = panel.querySelector(".docs-kb-citation-source-ref");
    var body = panel.querySelector(".docs-kb-citation-source-quote");
    if (title) title.textContent = ref || tr("docs-eq-reference", "Reference");
    if (body) body.textContent = quote || tr("docs-nav-empty", "Nothing here yet");
    positionPopover(anchor);
    lastAnchor = anchor || null;
    var close = panel.querySelector(".docs-kb-citation-source-close");
    if (close && typeof close.focus === "function") close.focus();
  }

  function setActiveFor(citation, anchor) {
    var api = lib();
    if (!api || typeof api.setActive !== "function") return;
    var index = -1;
    if (anchor && typeof anchor.getAttribute === "function") {
      var raw = anchor.getAttribute("data-docs-kb-citation-index");
      if (raw != null) index = parseInt(raw, 10);
    }
    if (!(index >= 0)) {
      var results = api.results || [];
      index = results.indexOf(citation);
    }
    if (index >= 0) api.setActive(index);
  }

  function applyTarget(target, citation) {
    if (!target) {
      announce(citation, false);
      return false;
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
    return true;
  }

  function onOpen(event) {
    var detail = unpack(event && event.detail);
    var citation = detail.citation;
    if (!citation) return;
    setActiveFor(citation, detail.anchor);
    showSource(citation, detail.anchor);
    applyTarget(locate(citation), citation);
  }

  function navigate(step) {
    var api = lib();
    var results = (api && api.results) || [];
    var total = results.length;
    if (total < 2 || (step !== 1 && step !== -1)) return false;
    var current = api && typeof api.activeIndex === "number" ? api.activeIndex : -1;
    var start = current >= 0 && current < total ? current : (step > 0 ? -1 : 0);
    var next = ((start + step) % total + total) % total;
    if (api && typeof api.setActive === "function") api.setActive(next);
    var citation = results[next];
    showSource(citation, chipAt(next));
    var navApi = window.DocsKbCitationNav;
    var target = navApi && typeof navApi.locate === "function"
      ? navApi.locate(citation) : locate(citation);
    applyTarget(target, citation);
    return true;
  }

  function onNav(event) {
    var detail = event && event.detail;
    if (detail) navigate(detail.step);
  }

  function onKeyDown(event) {
    if (!event) return;
    if (event.key === "Escape") {
      if (popoverNode) {
        var anchor = lastAnchor;
        hideSource();
        if (anchor && typeof anchor.focus === "function") anchor.focus();
      }
      return;
    }
    if (event.altKey && event.key === "ArrowRight") { navigate(1); return; }
    if (event.altKey && event.key === "ArrowLeft") { navigate(-1); return; }
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if (event.key === "]") {
      if (!isEditable(event.target)) navigate(1);
      return;
    }
    if (event.key === "[" && !isEditable(event.target)) navigate(-1);
  }

  document.addEventListener(EVENT, onOpen);
  document.addEventListener(NAV_EVENT, onNav);
  document.addEventListener("keydown", onKeyDown);

  window.DocsKbCitationNav = {
    locate: locate,
    event: EVENT,
    navigate: navigate,
    next: function () { return navigate(1); },
    prev: function () { return navigate(-1); },
    showSource: showSource,
    hideSource: hideSource
  };
})(window);
