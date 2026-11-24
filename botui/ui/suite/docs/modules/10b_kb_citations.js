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

  /* The retrieval layer ships one bounded snippet with every result. Accept
   * the common field names so the source panel works across producers. */
  function quoteOf(citation) {
    if (!citation) return "";
    var candidates = [
      citation.quote,
      citation.snippet,
      citation.excerpt,
      citation.passage,
      citation.text,
      citation.content,
      citation.source && citation.source.quote,
      citation.source && citation.source.snippet,
      citation.chunk && citation.chunk.content
    ];
    for (var i = 0; i < candidates.length; i += 1) {
      var value = candidates[i];
      if (typeof value === "string" && value.trim()) return value.trim();
    }
    return "";
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
    if (typeof settings.index === "number" && settings.index >= 0) {
      button.setAttribute("data-docs-kb-citation-index", String(settings.index));
    }
    button.addEventListener("click", function () {
      var api = window.DocsKbCitations;
      if (api && typeof api.setActive === "function" && typeof settings.index === "number") {
        api.setActive(settings.index);
      }
      if (typeof settings.onOpen === "function") {
        settings.onOpen(citation);
        return;
      }
      document.dispatchEvent(new CustomEvent("docs-kb-citation-open", {
        detail: { citation: citation, anchor: button },
        bubbles: true
      }));
    });
    return button;
  }

  function navButton(key, fallback, step) {
    var button = document.createElement("button");
    button.type = "button";
    button.className = "docs-kb-citation-nav-btn";
    var text = label(key, fallback);
    button.textContent = text;
    button.setAttribute("aria-label", text);
    button.style.padding = "4px 10px";
    button.style.marginRight = "6px";
    button.style.cursor = "pointer";
    button.style.border = "1px solid var(--palette-334155)";
    button.style.borderRadius = "4px";
    button.style.background = "var(--palette-0f172a)";
    button.style.color = "var(--palette-e2e8f0)";
    button.style.font = "inherit";
    button.addEventListener("click", function () {
      document.dispatchEvent(new CustomEvent("docs-kb-citation-nav", {
        detail: { step: step },
        bubbles: true
      }));
    });
    return button;
  }

  function renderCitations(container, citations, options) {
    if (!container) return;
    var settings = options || {};
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
    results.forEach(function (citation, index) {
      var item = document.createElement("li");
      item.className = "docs-kb-citations-item";
      item.appendChild(renderChip(citation, { index: index, onOpen: settings.onOpen }));
      list.appendChild(item);
    });
    container.appendChild(list);
    if (results.length > 1) {
      var nav = document.createElement("div");
      nav.className = "docs-kb-citation-nav";
      nav.setAttribute("role", "group");
      nav.setAttribute("aria-label", label("docs-references", "References"));
      nav.style.marginTop = "8px";
      nav.appendChild(navButton("action-previous", "Previous", -1));
      nav.appendChild(navButton("action-next", "Next", 1));
      container.appendChild(nav);
    }
  }

  function getChips() {
    return Array.prototype.slice.call(
      document.querySelectorAll(".docs-kb-citation[data-docs-kb-citation-index]")
    );
  }

  function setActive(index) {
    var value = typeof index === "number" && isFinite(index) ? index : -1;
    window.DocsKbCitations.activeIndex = value;
    var chips = getChips();
    for (var i = 0; i < chips.length; i += 1) {
      var chip = chips[i];
      var active = parseInt(chip.getAttribute("data-docs-kb-citation-index"), 10) === value;
      chip.classList.toggle("docs-kb-citation-active", active);
      if (active) {
        chip.setAttribute("aria-current", "true");
        chip.style.borderColor = "var(--palette-3b82f6)";
        chip.style.boxShadow = "inset 0 0 0 1px var(--palette-3b82f6)";
      } else {
        chip.removeAttribute("aria-current");
        chip.style.borderColor = "";
        chip.style.boxShadow = "";
      }
    }
  }

  function mount() {
    window.DocsKbCitations.activeIndex = -1;
    var containers = document.querySelectorAll("[data-docs-kb-citations]");
    for (var i = 0; i < containers.length; i += 1) {
      renderCitations(containers[i], window.DocsKbCitations.results || []);
    }
  }

  window.DocsKbCitations = {
    label: citationLabel,
    locationLabel: locationLabel,
    sourcePage: sourcePage,
    quoteOf: quoteOf,
    render: renderCitations,
    renderChip: renderChip,
    getChips: getChips,
    setActive: setActive,
    mount: mount,
    results: [],
    activeIndex: -1
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount, { once: true });
  } else {
    mount();
  }
})(window);
