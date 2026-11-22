"use strict";
/* Docs KB citation rendering (#2499).
 *
 * Renders permission-filtered retrieval results as accessible citation chips.
 * Every visible label comes from an existing i18n catalogue key via docsT();
 * document/file/heading values are supplied by the backend as data. */
(function (window) {
  function label(key, fallback, params) {
    return typeof docsT === "function" ? docsT(key, fallback, params) : fallback;
  }

  function withParam(text, name, value) {
    return String(text)
      .replace(new RegExp("\\{\\$" + name + "\\}", "g"), String(value))
      .replace(new RegExp("\\{" + name + "\\}", "g"), String(value));
  }

  function locationLabel(source) {
    if (!source) return "";
    var kind = source.kind || "page";
    var locator = source.locator == null ? "" : String(source.locator);
    if (kind === "page") {
      var page = parseInt(locator, 10);
      if (!isFinite(page)) page = source.page || 1;
      return withParam(label("docs-nav-page", "Page {$n}", { n: page }), "n", page);
    }
    if (kind === "slide") {
      var slide = parseInt(locator, 10);
      if (!isFinite(slide)) slide = 1;
      return withParam(label("slides-delivery-slide", "Slide { $n }", { n: slide }), "n", slide);
    }
    if (kind === "cell") {
      return label("sheet-ingest-cell-header", "Cell") + " " + locator;
    }
    if (kind === "section") {
      return label("docs-layout-sections", "Sections") + " › " + locator;
    }
    return label("docs-eq-reference", "Reference") + " " + locator;
  }

  function sourcePage(source) {
    if (!source) return 0;
    var raw = source.page != null ? source.page : source.locator;
    var page = parseInt(raw, 10);
    return isFinite(page) && page > 0 ? page : 0;
  }

  function clauseText(citation) {
    var source = citation.source || {};
    // A page-kind locator already is the page number; the page label below
    // renders it, so it must not be repeated as the clause (#2499).
    if (source.kind !== "page" && source.locator != null && String(source.locator) !== "") {
      return String(source.locator);
    }
    var path = citation.heading_path;
    return path && path.length ? String(path[path.length - 1]) : "";
  }

  // Chip text: file · version · clause, pN — every word from an existing key.
  function citationLabel(citation) {
    if (!citation) return "";
    var parts = [];
    if (citation.file) parts.push(String(citation.file));
    if (citation.version !== undefined && citation.version !== null && citation.version !== "") {
      parts.push(label("label-version", "Version") + ": " + citation.version);
    }
    var clause = clauseText(citation);
    var page = sourcePage(citation.source);
    if (clause) parts.push(clause);
    if (page) {
      var pageLabel = withParam(label("docs-nav-page", "Page {$n}"), "n", page);
      if (clause) {
        parts[parts.length - 1] = parts[parts.length - 1] + ", " + pageLabel;
      } else {
        parts.push(pageLabel);
      }
    }
    return parts.filter(Boolean).join(" · ");
  }

  function renderChip(citation, options) {
    var settings = options || {};
    var text = citationLabel(citation);
    var button = document.createElement("button");
    button.type = "button";
    button.className = "docs-kb-citation";
    button.setAttribute("role", "link");
    button.setAttribute("aria-label", label("action-view", "View") + " " + text);
    button.title = text;
    var tag = document.createElement("span");
    tag.className = "docs-kb-citation-tag";
    tag.textContent = label("docs-eq-reference", "Reference");
    var value = document.createElement("span");
    value.className = "docs-kb-citation-value";
    value.textContent = text;
    button.appendChild(tag);
    button.appendChild(value);
    button.addEventListener("click", function () {
      if (typeof settings.onOpen === "function") {
        settings.onOpen(citation);
        return;
      }
      document.dispatchEvent(new CustomEvent("docs-kb-citation-open", {
        detail: citation,
        bubbles: true
      }));
    });
    return button;
  }

  function renderCitations(container, citations, options) {
    if (!container) return;
    container.textContent = "";
    var results = citations || [];
    if (!results.length) {
      var status = document.createElement("div");
      status.className = "docs-kb-citations-empty";
      status.setAttribute("role", "status");
      status.setAttribute("aria-live", "polite");
      status.textContent = label("docs-nav-empty", "Nothing here yet");
      container.appendChild(status);
      return;
    }
    var list = document.createElement("ol");
    list.className = "docs-kb-citations";
    results.forEach(function (citation) {
      var item = document.createElement("li");
      item.className = "docs-kb-citations-item";
      item.appendChild(renderChip(citation, options));
      list.appendChild(item);
    });
    container.appendChild(list);
  }

  function mount() {
    var containers = document.querySelectorAll("[data-docs-kb-citations]");
    for (var i = 0; i < containers.length; i += 1) {
      renderCitations(containers[i], window.DocsKbCitations.results || []);
    }
  }

  window.DocsKbCitations = {
    label: citationLabel,
    locationLabel: locationLabel,
    sourcePage: sourcePage,
    render: renderCitations,
    renderChip: renderChip,
    mount: mount,
    results: []
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount, { once: true });
  } else {
    mount();
  }
})(window);
