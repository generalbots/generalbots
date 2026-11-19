/**
 * gb-states.js — shared loading / empty / error states for data-dependent
 * regions of the suite (#2328).
 *
 * A blank region is ambiguous: the user cannot tell "still loading" from
 * "broken" from "nothing here yet". This module gives every list, grid, table
 * and panel the same three states, reusing the suite's existing component
 * classes (`.skeleton`, `.skeleton-text`, `.skeleton-card`, `.empty-state`,
 * `.gb-empty`) — it declares no styles of its own.
 *
 * Two ways to use it:
 *
 * 1. Declarative (HTMX regions) — opt in on the container that receives the
 *    fragment. The hooks below then take care of the whole lifecycle:
 *
 *      <tbody id="deals" hx-get="/api/ui/crm/deals" hx-trigger="load"
 *             hx-swap="innerHTML"
 *             data-gb-states="table"
 *             data-gb-empty-icon="🤝"
 *             data-gb-empty-title="crm-deals-empty-title"
 *             data-gb-empty-title-fallback="No deals yet"
 *             data-gb-empty-desc="crm-deals-empty-desc"
 *             data-gb-empty-desc-fallback="Deals you create appear here."
 *             data-gb-empty-action="crm-deals-empty-action"
 *             data-gb-empty-action-fallback="New deal"
 *             data-gb-empty-action-href="#new-deal">
 *
 * 2. Imperative (JS-built lists) — call the renderers directly:
 *
 *      gbStates.skeleton(host, "list");
 *      gbStates.empty(host, { titleKey: "x", titleFallback: "…" });
 *      gbStates.error(host, { retry: function () { load(); } });
 *      gbStates.clear(host);
 *
 * Every user-facing string goes through gbT(key, fallback).
 */
(function () {
  "use strict";

  var SKELETON_SHAPES = {
    list: { rows: 4, lines: 2, card: false },
    table: { rows: 5, lines: 1, card: false },
    cards: { rows: 3, lines: 2, card: true },
    detail: { rows: 5, lines: 3, card: false },
    single: { rows: 1, lines: 3, card: false },
  };

  function t(key, fallback) {
    if (!key) return fallback || "";
    if (typeof window.gbT === "function") return window.gbT(key, fallback || "");
    return fallback || "";
  }

  function el(tag, className) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    return node;
  }

  function attr(host, name) {
    if (!host || !host.getAttribute) return "";
    return host.getAttribute(name) || "";
  }

  function hasContent(host) {
    if (!host) return false;
    if (host.querySelector("svg, canvas, img, video, iframe, input, select, textarea")) {
      return true;
    }
    return host.textContent.trim() !== "";
  }

  function clear(host) {
    if (!host) return;
    var block = host.querySelector("[data-gb-states-block]");
    if (block && block.parentNode) block.parentNode.removeChild(block);
    var cell = host.querySelector("[data-gb-states-cell]");
    if (cell && cell.parentNode) cell.parentNode.removeChild(cell);
    host.removeAttribute("aria-busy");
  }

  /* A table body cannot hold a block element: mount the state inside the
   * single cell that spans the row, so the markup stays valid. */
  function mount(host, node) {
    var tag = host.tagName ? host.tagName.toUpperCase() : "";
    if (tag === "TBODY" || tag === "THEAD") {
      var row = el("tr", null);
      row.setAttribute("data-gb-states-cell", "1");
      var cell = el("td", null);
      cell.setAttribute("colspan", attr(host, "data-gb-colspan") || "8");
      cell.appendChild(node);
      row.appendChild(cell);
      host.appendChild(row);
      return row;
    }
    if (tag === "TR") {
      var only = el("td", null);
      only.setAttribute("data-gb-states-cell", "1");
      only.setAttribute("colspan", attr(host, "data-gb-colspan") || "8");
      only.appendChild(node);
      host.appendChild(only);
      return only;
    }
    host.appendChild(node);
    return node;
  }

  /* ---------------------------------------------------------------- skeleton */

  function skeleton(host, shape) {
    if (!host) return null;
    var config = SKELETON_SHAPES[shape] || SKELETON_SHAPES.list;
    clear(host);
    var wrap = el("div", "gb-states-loading");
    wrap.setAttribute("data-gb-states-block", "loading");
    wrap.setAttribute("aria-busy", "true");
    wrap.setAttribute("role", "status");
    wrap.setAttribute("aria-live", "polite");

    var label = el("span", "sr-only");
    label.textContent = t("gb-state-loading", "Loading…");
    wrap.appendChild(label);

    for (var i = 0; i < config.rows; i++) {
      var row;
      if (config.card) {
        row = el("div", "skeleton skeleton-card");
      } else {
        row = el("div", "gb-states-skeleton-row");
        for (var j = 0; j < config.lines; j++) {
          row.appendChild(el("div", "skeleton skeleton-text"));
        }
      }
      row.setAttribute("aria-hidden", "true");
      wrap.appendChild(row);
    }

    mount(host, wrap);
    host.setAttribute("aria-busy", "true");
    return wrap;
  }

  /* ------------------------------------------------------------------- empty */

  function actionNode(spec) {
    var label = t(spec.actionKey, spec.actionFallback);
    if (!label) return null;
    var node;
    if (spec.actionHref) {
      node = el("a", "empty-state-action");
      node.setAttribute("href", spec.actionHref);
    } else {
      node = el("button", "empty-state-action");
      node.setAttribute("type", "button");
      if (spec.onAction) node.addEventListener("click", spec.onAction);
    }
    node.textContent = label;
    return node;
  }

  function panel(host, kind, spec) {
    if (!host) return null;
    clear(host);
    var wrap = el("div", "empty-state gb-empty gb-states-" + kind);
    wrap.setAttribute("data-gb-states-block", kind);
    if (kind === "error") {
      wrap.setAttribute("role", "alert");
      wrap.setAttribute("aria-live", "assertive");
    } else {
      wrap.setAttribute("role", "status");
      wrap.setAttribute("aria-live", "polite");
    }

    if (spec.icon) {
      var icon = el("span", "empty-state-icon");
      icon.setAttribute("aria-hidden", "true");
      icon.textContent = spec.icon;
      wrap.appendChild(icon);
    }

    var titleKey = kind === "error" ? spec.errorTitleKey : spec.titleKey;
    var titleFallback = kind === "error"
      ? spec.errorTitleFallback
      : spec.titleFallback;
    if (!titleKey && !titleFallback) {
      titleKey = kind === "error" ? "gb-state-error-title" : "gb-state-empty-title";
      titleFallback = kind === "error" ? "Something went wrong" : "Nothing here yet";
    }
    var title = el("p", "empty-state-title");
    title.textContent = t(titleKey, titleFallback);
    wrap.appendChild(title);

    var descKey = kind === "error" ? spec.errorDescKey : spec.descKey;
    var descFallback = kind === "error" ? spec.errorDescFallback : spec.descFallback;
    if (!descKey && !descFallback) {
      descKey = kind === "error" ? "gb-state-error-desc" : "gb-state-empty-desc";
      descFallback = kind === "error"
        ? "This content could not be loaded. Try again."
        : "There is nothing to show here yet.";
    }
    var desc = el("p", "empty-state-message");
    desc.textContent = t(descKey, descFallback);
    wrap.appendChild(desc);

    var action = null;
    if (kind === "error") {
      if (spec.retry) {
        action = el("button", "empty-state-action");
        action.setAttribute("type", "button");
        action.textContent = t("gb-state-retry", "Try again");
        action.addEventListener("click", function () {
          gbStates.skeleton(host, spec.shape || "list");
          spec.retry();
        });
      }
    } else {
      action = actionNode(spec);
    }
    if (action) wrap.appendChild(action);

    mount(host, wrap);
    host.removeAttribute("aria-busy");
    return wrap;
  }

  function empty(host, spec) {
    return panel(host, "empty", spec || {});
  }

  function error(host, spec) {
    return panel(host, "error", spec || {});
  }

  /* -------------------------------------------------- declarative attributes */

  function specFrom(host) {
    return {
      icon: attr(host, "data-gb-empty-icon"),
      titleKey: attr(host, "data-gb-empty-title"),
      titleFallback: attr(host, "data-gb-empty-title-fallback"),
      descKey: attr(host, "data-gb-empty-desc"),
      descFallback: attr(host, "data-gb-empty-desc-fallback"),
      actionKey: attr(host, "data-gb-empty-action"),
      actionFallback: attr(host, "data-gb-empty-action-fallback"),
      actionHref: attr(host, "data-gb-empty-action-href"),
      actionTarget: attr(host, "data-gb-empty-action-target"),
      errorTitleKey: attr(host, "data-gb-error-title"),
      errorTitleFallback: attr(host, "data-gb-error-title-fallback"),
      errorDescKey: attr(host, "data-gb-error-desc"),
      errorDescFallback: attr(host, "data-gb-error-desc-fallback"),
    };
  }

  function targetFor(detail) {
    if (!detail) return null;
    if (detail.target && detail.target.getAttribute) return detail.target;
    var elt = detail.elt;
    if (!elt || !elt.getAttribute) return null;
    var explicit = elt.getAttribute("hx-target");
    if (!explicit) return elt;
    if (!window.htmx || !window.htmx.find) return elt;
    try {
      return window.htmx.find(explicit) || elt;
    } catch (e) {
      return elt;
    }
  }

  function optedIn(host) {
    if (!host || !host.getAttribute) return false;
    return host.hasAttribute("data-gb-states") ||
      host.hasAttribute("data-gb-empty-title");
  }

  function scheduleSkeleton(host, source) {
    if (!optedIn(host)) return;
    if (hasContent(host)) return;
    if (host.__gbSkeletonTimer) clearTimeout(host.__gbSkeletonTimer);
    host.__gbSkeletonTimer = setTimeout(function () {
      host.__gbSkeletonTimer = null;
      if (hasContent(host)) return;
      skeleton(host, attr(host, "data-gb-states") || "list");
    }, 180);
    host.__gbSource = source;
  }

  function cancelSkeleton(host) {
    if (host && host.__gbSkeletonTimer) {
      clearTimeout(host.__gbSkeletonTimer);
      host.__gbSkeletonTimer = null;
    }
  }

  function retryFor(host) {
    var source = host && host.__gbSource;
    return function () {
      if (!source || !source.getAttribute) {
        if (host) error(host, { retry: retryFor(host) });
        return;
      }
      var trigger = source.getAttribute("hx-trigger") || "";
      if (window.htmx && trigger.indexOf("load") === 0) {
        window.htmx.trigger(source, "load");
      } else if (typeof source.click === "function") {
        source.click();
      } else if (window.htmx && source.getAttribute("hx-get")) {
        window.htmx.trigger(source, "load");
      }
    };
  }

  function bindHtmx() {
    if (window.__gbStatesBound) return;
    if (!window.htmx || typeof document === "undefined") return;
    window.__gbStatesBound = true;

    document.addEventListener("htmx:beforeRequest", function (evt) {
      var host = targetFor(evt.detail);
      if (!optedIn(host)) return;
      scheduleSkeleton(host, evt.detail && evt.detail.elt);
    });

    document.addEventListener("htmx:afterSwap", function (evt) {
      var host = targetFor(evt.detail);
      if (!optedIn(host)) return;
      cancelSkeleton(host);
      clear(host);
      if (hasContent(host)) return;
      var spec = specFrom(host);
      spec.onAction = actionRunner(host, spec);
      empty(host, spec);
    });

    document.addEventListener("htmx:responseError", onFailure);
    document.addEventListener("htmx:sendError", onFailure);
    document.addEventListener("htmx:timeout", onFailure);

    function onFailure(evt) {
      var host = targetFor(evt.detail);
      if (!optedIn(host)) return;
      cancelSkeleton(host);
      var spec = specFrom(host);
      spec.shape = attr(host, "data-gb-states") || "list";
      spec.retry = retryFor(host);
      error(host, spec);
    }
  }

  function actionRunner(host, spec) {
    if (spec.actionHref) return null;
    var targetId = spec.actionTarget;
    if (!targetId) return null;
    return function () {
      var node = document.getElementById(targetId.replace(/^#/, ""));
      if (!node) return;
      if (typeof node.focus === "function") node.focus();
      if (typeof node.click === "function" && node.tagName === "BUTTON") node.click();
    };
  }

  var gbStates = {
    skeleton: skeleton,
    empty: empty,
    error: error,
    clear: clear,
    specFrom: specFrom,
    hasContent: hasContent,
  };

  window.gbStates = gbStates;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bindHtmx);
  } else {
    bindHtmx();
  }
})();
