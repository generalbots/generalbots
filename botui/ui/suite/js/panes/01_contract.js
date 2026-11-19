/* General Bots — shared list+detail pane behaviour (#2415).
 *
 * Loaded by each opted-in app after gb-panes.css. It is intentionally
 * generic: apps declare their surfaces with data-gb-panes-* attributes and
 * this module supplies the Back control, the labelled resize handle, the
 * polite status region and the compact-mode state restore. No app-specific
 * selectors or user-facing strings live here.
 */
(function () {
  "use strict";

  var COMPACT = "(max-width: 767px)";
  var MEDIUM = "(min-width: 768px) and (max-width: 1024px)";
  var initialized = new WeakSet();
  var saved = new WeakMap();

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function all(selector, scope) {
    try {
      return Array.prototype.slice.call((scope || document).querySelectorAll(selector));
    } catch (e) {
      return [];
    }
  }

  function one(selector, scope) {
    try {
      return (scope || document).querySelector(selector);
    } catch (e) {
      return null;
    }
  }

  function compact() {
    return window.matchMedia && window.matchMedia(COMPACT).matches;
  }

  function rootOf(element) {
    return element.closest(".gb-panes");
  }

  function visiblePrimary(root) {
    var panes = all(".gb-pane-primary", root);
    for (var i = 0; i < panes.length; i += 1) {
      if (panes[i].getClientRects().length) return panes[i];
    }
    return panes[0] || null;
  }

  /* --- control creation ------------------------------------------------- */

  function addBack(detail) {
    var back = one(".gb-pane-back", detail);
    if (back) return back;
    back = document.createElement("button");
    back.type = "button";
    back.className = "gb-pane-back";
    back.setAttribute("data-i18n-aria-label", "panes-back-aria");
    back.setAttribute("aria-label", t("panes-back-aria", "Back to list"));
    back.innerHTML =
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">' +
      '<line x1="19" y1="12" x2="5" y2="12"></line>' +
      '<polyline points="12 19 5 12 12 5"></polyline></svg>' +
      '<span data-i18n="panes-back">' + t("panes-back", "Back") + "</span>";
    var first = detail.firstChild;
    if (first) detail.insertBefore(back, first);
    else detail.appendChild(back);
    return back;
  }

  function addResize(root, primary, detail, selector) {
    var handle = selector ? one(selector, root) : null;
    if (!handle) {
      handle = document.createElement("div");
      handle.className = "gb-panes-resize";
      primary.parentNode.insertBefore(handle, detail);
    }
    handle.classList.add("gb-panes-resize");
    handle.setAttribute("role", "separator");
    handle.setAttribute("tabindex", "0");
    handle.setAttribute("aria-orientation", "vertical");
    handle.setAttribute("data-i18n-aria-label", "panes-resize-handle");
    handle.setAttribute("aria-label", t("panes-resize-handle", "Resize panes"));
    var size = Math.round(parseFloat(root.style.getPropertyValue("--gb-pane-primary-size")) || 42);
    handle.setAttribute("aria-valuemin", "30");
    handle.setAttribute("aria-valuemax", "70");
    handle.setAttribute("aria-valuenow", String(size));
    bindResize(root, handle);
    return handle;
  }

  function addStatus(root) {
    var status = one(".gb-panes-status", root);
    if (status) return status;
    status = document.createElement("div");
    status.className = "gb-panes-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("aria-atomic", "true");
    root.appendChild(status);
    return status;
  }

  function announce(root, key, fallback) {
    var status = one(".gb-panes-status", root);
    if (status) status.textContent = t(key, fallback);
  }

  /* --- state capture / restore ----------------------------------------- */

  function scrollContainer(primary) {
    if (primary.scrollHeight > primary.clientHeight) return primary;
    var candidates = all("*", primary);
    for (var i = 0; i < candidates.length; i += 1) {
      var el = candidates[i];
      var overflow = window.getComputedStyle(el).overflowY;
      if ((overflow === "auto" || overflow === "scroll") && el.scrollHeight > el.clientHeight) {
        return el;
      }
    }
    return primary;
  }

  function activeFilter(root) {
    var node = one("[data-filter].active, [aria-pressed='true']", root);
    if (node) {
      return { kind: "click", selector: "[data-filter='" + node.getAttribute("data-filter") + "']" };
    }
    var select = one("select[data-filter], select.grid-filter, select[name]", root);
    if (select) {
      return { kind: "value", selector: "select[data-filter], select.grid-filter, select[name]", value: select.value };
    }
    return null;
  }

  function saveState(root, primary, rows) {
    saved.set(root, {
      primary: primary,
      scroll: scrollContainer(primary).scrollTop,
      filter: activeFilter(root),
      selected: rows
        .map(function (row, index) {
          return row.classList.contains("selected") || row.getAttribute("aria-selected") === "true" ? index : -1;
        })
        .filter(function (index) {
          return index >= 0;
        })
    });
  }

  function restoreState(root, rows) {
    var state = saved.get(root);
    if (!state) return;
    var primary = state.primary || visiblePrimary(root);
    if (!primary) return;
    window.requestAnimationFrame(function () {
      window.requestAnimationFrame(function () {
        scrollContainer(primary).scrollTop = state.scroll;
        if (state.filter) {
          var target = one(state.filter.selector, root);
          if (target) {
            if (state.filter.kind === "value" && target.value !== state.filter.value) {
              target.value = state.filter.value;
              target.dispatchEvent(new Event("change", { bubbles: true }));
            } else if (state.filter.kind === "click" && !target.classList.contains("active")) {
              target.click();
            }
          }
        }
        state.selected.forEach(function (index) {
          if (rows[index]) {
            rows[index].classList.add("selected");
            rows[index].setAttribute("aria-selected", "true");
          }
        });
      });
    });
  }

  /* --- pane transitions ------------------------------------------------- */

  function openDetail(root) {
    if (!compact() || root.classList.contains("is-detail-open")) return;
    var primary = visiblePrimary(root);
    var detail = one(".gb-pane-detail", root);
    if (!primary || !detail) return;
    var rows = all(root.getAttribute("data-gb-panes-rows") || "", primary);
    saveState(root, primary, rows);
    root.classList.add("is-detail-open");
    detail.classList.add("open", "active");
    announce(root, "panes-detail-open", "Detail opened");
    root.dispatchEvent(new CustomEvent("gbpane:open", { bubbles: true }));
    var back = one(".gb-pane-back", detail);
    if (back) back.focus();
  }

  function closeDetail(root) {
    if (!root.classList.contains("is-detail-open")) return;
    var primary = visiblePrimary(root);
    var detail = one(".gb-pane-detail", root);
    var rows = primary ? all(root.getAttribute("data-gb-panes-rows") || "", primary) : [];
    root.classList.remove("is-detail-open");
    if (detail) detail.classList.remove("open", "active", "show", "visible");
    restoreState(root, rows);
    announce(root, "panes-detail-closed", "Back to list");
    root.dispatchEvent(new CustomEvent("gbpane:close", { bubbles: true }));
    if (primary) primary.focus && primary.focus();
  }

  /* --- resize handle ---------------------------------------------------- */

  function setSize(root, handle, percent) {
    var clamped = Math.max(30, Math.min(70, percent));
    root.style.setProperty("--gb-pane-primary-size", clamped + "%");
    handle.setAttribute("aria-valuenow", String(Math.round(clamped)));
  }

  function bindResize(root, handle) {
    handle.addEventListener("keydown", function (event) {
      var current = Number(handle.getAttribute("aria-valuenow")) || 42;
      if (event.key === "ArrowLeft") setSize(root, handle, current - 2);
      else if (event.key === "ArrowRight") setSize(root, handle, current + 2);
      else if (event.key === "Home") setSize(root, handle, 30);
      else if (event.key === "End") setSize(root, handle, 70);
      else return;
      event.preventDefault();
    });

    handle.addEventListener("pointerdown", function (event) {
      if (!window.matchMedia || !window.matchMedia(MEDIUM).matches) return;
      var rect = root.getBoundingClientRect();
      if (!rect.width) return;
      handle.setPointerCapture(event.pointerId);
      function move(moveEvent) {
        setSize(root, handle, ((moveEvent.clientX - rect.left) / rect.width) * 100);
      }
      function up() {
        handle.removeEventListener("pointermove", move);
        handle.removeEventListener("pointerup", up);
        handle.removeEventListener("pointercancel", up);
      }
      handle.addEventListener("pointermove", move);
      handle.addEventListener("pointerup", up);
      handle.addEventListener("pointercancel", up);
    });
  }

  /* --- wiring ----------------------------------------------------------- */

  function init(root) {
    if (initialized.has(root)) return;

    var primarySelector = root.getAttribute("data-gb-panes-primary") || ".gb-pane-primary";
    var detailSelector = root.getAttribute("data-gb-panes-detail") || ".gb-pane-detail";
    var primaries = all(primarySelector, root);
    var detail = one(detailSelector, root) || one(detailSelector);
    if (!primaries.length || !detail) return;
    initialized.add(root);

    primaries.forEach(function (primary) {
      primary.classList.add("gb-pane", "gb-pane-primary");
      if (!primary.hasAttribute("tabindex")) primary.setAttribute("tabindex", "0");
    });
    detail.classList.add("gb-pane", "gb-pane-detail");

    addBack(detail);
    addStatus(root);
    if (!root.hasAttribute("data-gb-panes-detail-mode")) {
      addResize(root, primaries[0], detail, root.getAttribute("data-gb-panes-resize"));
    }
  }

  function initAll() {
    all("[data-gb-panes]").forEach(init);
  }

  document.addEventListener("click", function (event) {
    var back = event.target.closest && event.target.closest(".gb-pane-back");
    if (back) {
      var rootForBack = rootOf(back);
      if (rootForBack) closeDetail(rootForBack);
      return;
    }
    var root = rootOf(event.target);
    if (!root || !compact() || root.classList.contains("is-detail-open")) return;
    var selector = root.getAttribute("data-gb-panes-rows");
    if (!selector) return;
    if (event.target.closest("button, input, a, select, textarea, [role='button']")) return;
    var primaries = all(".gb-pane-primary", root);
    var row = null;
    try {
      row = event.target.closest(selector);
    } catch (e) {
      return;
    }
    if (row && primaries.some(function (primary) { return primary.contains(row); })) {
      openDetail(root);
    }
  });

  if (window.matchMedia) {
    var query = window.matchMedia(COMPACT);
    var onChange = function (event) {
      if (!event.matches) all(".gb-panes.is-detail-open").forEach(closeDetail);
    };
    if (query.addEventListener) query.addEventListener("change", onChange);
    else if (query.addListener) query.addListener(onChange);
  }

  document.addEventListener("DOMContentLoaded", initAll);
  document.addEventListener("htmx:afterSwap", initAll);
  document.addEventListener("htmx:afterRequest", initAll);
  if (document.readyState !== "loading") initAll();

  window.GBPanes = { init: init, initAll: initAll, open: openDetail, close: closeDetail };
})();
