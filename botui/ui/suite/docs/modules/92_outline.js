"use strict";
/* Docs document outline / navigation panel (#2472, #2485).
 *
 * Self-contained, offline frontend addition to Docs. It derives a heading tree
 * from the live `article.docs-doc-view`, gives every heading a stable id,
 * renders an indented, clickable outline, scrolls the editor to the selected
 * heading and keeps the active row in sync with scrolling.
 *
 * It is inert when the Docs window/article is absent, uses pointer events for
 * activation (keyboard as an accessibility fallback), re-derives on
 * `gb:deep-link` / HTMX swaps so a re-targeted window never keeps a stale tree,
 * and adds no new i18n catalog keys (existing keys + English gbT fallbacks).
 * No backend, no build step, no CDN. */

(function () {
  if (window.__docsOutlineLoaded) return;
  window.__docsOutlineLoaded = true;

  var PANEL_ID = "docs-outline-panel";
  var STYLE_ID = "docs-outline-style";
  var HEADINGS = "h1,h2,h3,h4,h5,h6";
  var ACTIVE_CLASS = "is-active";

  var refs = { article: null, scope: null, host: null, panel: null, list: null, empty: null };
  var headings = [];        // [{ id, level, text, el }]
  var rows = [];            // [{ id, el }]
  var scrollHost = null;    // recomputed on every refresh
  var activeId = null;
  var lastSignature = "";
  var observer = null;
  var scrollTicking = false;
  var refreshTicking = false;
  var suppressScrollUntil = 0;
  var bound = false;

  /* English fallback always wins if no catalogue is installed, so no raw key
   * can ever leak into the UI. */
  function tr(key, fallback) {
    if (typeof window.gbT === "function") return window.gbT(key, fallback);
    if (typeof docsT === "function") return docsT(key, fallback);
    return fallback || key;
  }

  function getArticle() {
    var app = document.getElementById("docs-app");
    var root = app || document;
    var article = root.querySelector("article.docs-doc-view[contenteditable]") ||
      root.querySelector("article.docs-doc-view");
    if (article) return article;
    var all = document.querySelectorAll("article.docs-doc-view[contenteditable], article.docs-doc-view");
    return all.length ? all[0] : null;
  }

  function getScope(article) {
    if (!article) return null;
    if (typeof article.closest === "function") {
      var app = article.closest(".gb-app");
      if (app) return app;
    }
    return document.getElementById("docs-app") || article.parentElement || null;
  }

  function panelHost() {
    var scope = refs.scope;
    if (!scope) return null;
    var sidebar = scope.querySelector("#sidebar-panels");
    if (sidebar) return sidebar;
    var nav = scope.querySelector("#docs-nav-pane");
    if (nav && nav.parentElement) return nav.parentElement;
    return scope;
  }

  function injectStyle() {
    if (document.getElementById(STYLE_ID)) return;
    var style = document.createElement("style");
    style.id = STYLE_ID;
    style.textContent = [
      "#" + PANEL_ID + "{margin:6px 0 8px;padding:6px;border:1px solid var(--palette-334155,#334155);" +
        "border-radius:4px;background:var(--palette-1e293b,var(--palette-1e293b));}",
      "#" + PANEL_ID + " .gb-outline-header{display:flex;align-items:center;padding:2px 4px 6px;" +
        "color:var(--palette-f8fafc,var(--palette-f8fafc));font-size:12px;font-weight:600;}",
      "#" + PANEL_ID + " .gb-outline-tree{display:flex;flex-direction:column;max-height:280px;overflow-y:auto;}",
      "#" + PANEL_ID + " .gb-outline-item{display:block;width:100%;text-align:left;padding:4px 6px;" +
        "border:0;border-radius:3px;background:transparent;color:var(--palette-cbd5e1,var(--palette-cbd5e1));" +
        "font:inherit;font-size:12px;line-height:1.4;cursor:pointer;overflow:hidden;" +
        "text-overflow:ellipsis;white-space:nowrap;}",
      "#" + PANEL_ID + " .gb-outline-item:hover{background:var(--palette-334155,#334155);" +
        "color:var(--palette-f8fafc,var(--palette-f8fafc));}",
      "#" + PANEL_ID + " .gb-outline-item." + ACTIVE_CLASS + "{background:var(--palette-0f766e);color:var(--id-surface-solid);}",
      "#" + PANEL_ID + " .gb-outline-item:focus-visible{outline:2px solid var(--palette-3b82f6);outline-offset:-2px;}",
      "#" + PANEL_ID + " .gb-outline-empty{padding:6px 4px;color:var(--palette-94a3b8,var(--palette-94a3b8));" +
        "font-size:11px;font-style:italic;}"
    ].join("\n");
    (document.head || document.documentElement).appendChild(style);
  }

  function ensurePanel() {
    if (!refs.scope) return false;
    injectStyle();
    var panel = document.getElementById(PANEL_ID);
    if (panel && panel.isConnected !== false) {
      refs.panel = panel;
      refs.list = panel.querySelector(".gb-outline-tree");
      refs.empty = panel.querySelector(".gb-outline-empty");
      return !!refs.list;
    }
    var host = panelHost();
    if (!host) return false;

    panel = document.createElement("section");
    panel.id = PANEL_ID;
    panel.className = "gb-outline-panel";
    panel.setAttribute("aria-label", tr("docs-outline", "Outline"));
    panel.setAttribute("data-i18n-aria-label", "docs-outline");

    var header = document.createElement("div");
    header.className = "gb-outline-header";
    var title = document.createElement("span");
    title.className = "gb-outline-title";
    title.setAttribute("data-i18n", "docs-outline");
    title.textContent = tr("docs-outline", "Outline");
    header.appendChild(title);
    panel.appendChild(header);

    var list = document.createElement("nav");
    list.className = "gb-outline-tree";
    list.setAttribute("role", "tree");
    list.setAttribute("aria-label", tr("docs-nav-headings", "Headings"));
    list.setAttribute("data-i18n-aria-label", "docs-nav-headings");
    panel.appendChild(list);

    var empty = document.createElement("div");
    empty.className = "gb-outline-empty";
    empty.setAttribute("data-i18n", "docs-toc-empty");
    empty.textContent = tr("docs-toc-empty", "No headings found.");
    panel.appendChild(empty);

    host.insertBefore(panel, host.firstChild);
    list.addEventListener("pointerdown", onPointerDown);
    list.addEventListener("keydown", onKeyDown);
    refs.panel = panel;
    refs.list = list;
    refs.empty = empty;
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

  /* Reuse an authored id when possible; otherwise mint a deterministic
   * `sec-<slug>` id (the same convention as 14_toc_anchors.js) so the outline,
   * the TOC and the PDF export agree on destinations. */
  function stableId(el, text, used) {
    var id = (el.getAttribute("id") || "").trim();
    if (id && !used[id]) {
      used[id] = true;
      return id;
    }
    var base = "sec-" + (slugify(text) || "section");
    var candidate = base;
    var suffix = 2;
    while (used[candidate] || document.getElementById(candidate)) {
      candidate = base + "-" + suffix;
      suffix += 1;
    }
    el.setAttribute("id", candidate);
    used[candidate] = true;
    return candidate;
  }

  function deriveHeadings() {
    var article = refs.article;
    if (!article) return [];
    if (typeof window.docsAssignHeadingIds === "function") {
      try { window.docsAssignHeadingIds(article); } catch (error) { /* fall through */ }
    }
    var used = Object.create(null);
    Array.prototype.forEach.call(article.querySelectorAll("[id]"), function (node) {
      if (!/^H[1-6]$/.test(node.tagName)) used[node.getAttribute("id")] = true;
    });
    var list = [];
    Array.prototype.forEach.call(article.querySelectorAll(HEADINGS), function (el) {
      var text = (el.textContent || "").replace(/\s+/g, " ").trim();
      var id = stableId(el, text, used);
      list.push({
        id: id,
        level: parseInt(el.tagName.charAt(1), 10) || 1,
        text: text || id,
        el: el
      });
    });
    return list;
  }

  function render() {
    if (!refs.list) return;
    var fragment = document.createDocumentFragment();
    rows = [];
    headings.forEach(function (item) {
      var button = document.createElement("button");
      button.type = "button";
      button.className = "gb-outline-item";
      button.setAttribute("role", "treeitem");
      button.setAttribute("data-outline-id", item.id);
      button.setAttribute("aria-level", String(item.level));
      button.setAttribute("aria-selected", "false");
      button.tabIndex = 0;
      button.style.paddingLeft = (6 + (item.level - 1) * 12) + "px";
      var label = document.createElement("span");
      label.className = "gb-outline-label";
      label.textContent = item.text;
      button.appendChild(label);
      fragment.appendChild(button);
      rows.push({ id: item.id, el: button });
    });
    refs.list.textContent = "";
    refs.list.appendChild(fragment);
    if (refs.empty) refs.empty.hidden = headings.length > 0;
    refs.list.hidden = headings.length === 0;
  }

  function activate(id) {
    activeId = id || null;
    rows.forEach(function (row) {
      var on = row.id === activeId;
      row.el.classList.toggle(ACTIVE_CLASS, on);
      row.el.setAttribute("aria-selected", on ? "true" : "false");
    });
  }

  function findHeading(id) {
    if (!refs.article || !id) return null;
    var found = null;
    Array.prototype.forEach.call(refs.article.querySelectorAll("[id]"), function (node) {
      if (!found && node.getAttribute("id") === id) found = node;
    });
    return found;
  }

  function scrollHostOf(element) {
    var node = element.parentElement;
    while (node && node !== document.body && node !== document.documentElement) {
      var overflow = window.getComputedStyle ? window.getComputedStyle(node).overflowY : "";
      if ((overflow === "auto" || overflow === "scroll") && node.scrollHeight > node.clientHeight + 4) {
        return node;
      }
      node = node.parentElement;
    }
    var fallback = document.getElementById("docs-content");
    if (fallback && refs.scope && typeof refs.scope.contains === "function" && !refs.scope.contains(fallback)) {
      return null;
    }
    return fallback || null;
  }

  function scrollToHeading(element) {
    if (!element) return false;
    var host = scrollHost || scrollHostOf(element);
    if (host && typeof host.getBoundingClientRect === "function") {
      var top = (host.scrollTop || 0) +
        (element.getBoundingClientRect().top - host.getBoundingClientRect().top) - 16;
      top = Math.max(0, top);
      if (typeof host.scrollTo === "function") host.scrollTo({ top: top, behavior: "smooth" });
      else host.scrollTop = top;
    } else if (typeof element.scrollIntoView === "function") {
      element.scrollIntoView({ behavior: "smooth", block: "start" });
    }
    suppressScrollUntil = Date.now() + 400;
    activate(element.getAttribute("id"));
    return true;
  }

  function onPointerDown(event) {
    if (event.isPrimary === false) return;
    if (event.pointerType === "mouse" && event.button !== 0) return;
    var item = event.target && event.target.closest ? event.target.closest(".gb-outline-item") : null;
    if (!item || !refs.list || !refs.list.contains(item)) return;
    event.preventDefault();
    event.stopPropagation();
    scrollToHeading(findHeading(item.getAttribute("data-outline-id")));
  }

  function onKeyDown(event) {
    if (event.key !== "Enter" && event.key !== " " && event.key !== "Spacebar") return;
    var item = event.target && event.target.closest ? event.target.closest(".gb-outline-item") : null;
    if (!item) return;
    event.preventDefault();
    event.stopPropagation();
    scrollToHeading(findHeading(item.getAttribute("data-outline-id")));
  }

  function syncActiveOnScroll() {
    if (!refs.panel || !rows.length) return;
    if (Date.now() < suppressScrollUntil) return;
    var host = scrollHost || refs.article;
    if (!host) return;
    var base = typeof host.getBoundingClientRect === "function" ? host.getBoundingClientRect().top : 0;
    var threshold = base + 24;
    var current = rows[0].id;
    for (var i = 0; i < headings.length; i++) {
      var element = headings[i].el;
      if (!element || typeof element.getBoundingClientRect !== "function") continue;
      if (element.getBoundingClientRect().top <= threshold) current = headings[i].id;
      else break;
    }
    var scrollHeight = host.scrollHeight || 0;
    var scrollTop = host.scrollTop || 0;
    var clientHeight = host.clientHeight || 0;
    if (scrollHeight && clientHeight && scrollHeight - scrollTop - clientHeight < 6) {
      current = headings[headings.length - 1].id;
    }
    if (current !== activeId) activate(current);
  }

  function onScroll() {
    if (!refs.article || !rows.length || scrollTicking) return;
    scrollTicking = true;
    var run = function () { scrollTicking = false; syncActiveOnScroll(); };
    if (typeof window.requestAnimationFrame === "function") window.requestAnimationFrame(run);
    else run();
  }

  function disconnectObserver() {
    if (!observer) return;
    try { observer.disconnect(); } catch (error) { /* best effort */ }
    observer = null;
  }

  function observe(article) {
    if (observer && observer.__docsOutlineArticle === article) return;
    disconnectObserver();
    if (!article || typeof MutationObserver === "undefined") return;
    var next = new MutationObserver(function () { scheduleRefresh(); });
    next.__docsOutlineArticle = article;
    try {
      next.observe(article, { childList: true, subtree: true, characterData: true });
      observer = next;
    } catch (error) { /* observation is best effort */ }
  }

  function scheduleRefresh() {
    if (refreshTicking) return;
    refreshTicking = true;
    var run = function () { refreshTicking = false; refresh(); };
    if (typeof window.requestAnimationFrame === "function") window.requestAnimationFrame(run);
    else run();
  }

  function refresh() {
    var article = getArticle();
    if (!article) {
      disconnectObserver();
      refs.article = null;
      refs.scope = null;
      headings = [];
      rows = [];
      return false;
    }
    refs.article = article;
    refs.scope = getScope(article);
    if (!ensurePanel()) return false;

    var next = deriveHeadings();
    var signature = next.map(function (item) {
      return item.id + ":" + item.level + ":" + item.text;
    }).join("|");
    if (signature !== lastSignature) {
      lastSignature = signature;
      headings = next;
      render();
      activeId = null;
    } else {
      headings = next;
    }
    scrollHost = scrollHostOf(article);
    observe(article);
    syncActiveOnScroll();
    return true;
  }

  function init() {
    if (bound) { refresh(); return; }
    bound = true;
    document.addEventListener("scroll", onScroll, true);
    document.addEventListener("gb:deep-link", function () {
      lastSignature = "";
      refresh();
    });
    document.addEventListener("htmx:afterSwap", function (event) {
      var target = event && event.target;
      var id = target && target.id ? String(target.id) : "";
      if (!target || id.indexOf("docs") !== -1 ||
        (typeof target.closest === "function" && target.closest(".docs-app"))) {
        refresh();
      }
    });
    refresh();
  }

  /* Small read-only surface, also handy for offline vm stub tests. */
  window.DocsOutline = {
    refresh: refresh,
    items: function () {
      return headings.map(function (item) {
        return { id: item.id, level: item.level, text: item.text };
      });
    }
  };

  init();
})();
