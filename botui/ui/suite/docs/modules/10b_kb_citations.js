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

  function citationLabel(citation) {
    if (!citation) return "";
    var parts = [citation.file || "", locationLabel(citation.source)];
    if (citation.version !== undefined && citation.version !== null && citation.version !== "") {
      parts.push(label("label-version", "Version") + ": " + citation.version);
    }
    var heading = citation.heading_path && citation.heading_path.length
      ? citation.heading_path[citation.heading_path.length - 1] : "";
    if (heading) parts.push(String(heading));
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
