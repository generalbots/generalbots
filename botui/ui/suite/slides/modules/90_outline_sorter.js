"use strict";
/* Slides — outline / sorter panel (issue #2473, workstream #2433).
 *
 * Self-contained, plain-script enhancement layered on the rendered thumbnail
 * rail. It lists the deck's slides with stable ids, reorders them with pointer
 * drag (mouse or touch) or with up/down buttons (the reliable fallback), mirrors
 * the resulting order back into the real .sl-thumb DOM and keeps the currently
 * selected slide selected across rebuilds and HTMX window re-targeting.
 *
 * It is a strict no-op when the Slides app is not in the document. No server
 * writes and no new i18n catalog keys: visible labels reuse existing keys via
 * data-i18n / gbT with English fallbacks.
 */
(function () {
  var PANEL_ID = "sl-outline-sorter-panel";
  var TOGGLE_ID = "sl-outline-sorter-toggle";
  var ROW = "sl-outline-row";
  var SEL_KEY = "__slOutlineSelected";

  var knownRoots = [];
  var drag = null;
  var renderPending = false;

  function t(key, fallback, params) {
    var O = window.GBSlidesOutline;
    if (O && typeof O.t === "function") return O.t(key, fallback, params);
    if (typeof window.gbT === "function") return window.gbT(key, fallback, params);
    return fallback;
  }

  /* The Slides app lives inside a desktop window titled window-body-slides-<n>.
   * HTMX may swap that window at any time, so always re-resolve the live root. */
  function scope() {
    var app = document.getElementById("slides-app");
    if (app && document.body && document.body.contains(app)) return app;
    var windows = document.querySelectorAll('[id^="window-body-slides"]');
    for (var i = 0; i < windows.length; i += 1) {
      var inner = windows[i].querySelector("#slides-app");
      if (inner) return inner;
    }
    return null;
  }
  function thumbsHost(root) { return root ? root.querySelector("#sidebar-thumbs") : null; }
  function panelHost(root) {
    var host = root.querySelector("#slides-tools-panel-host");
    if (!host) {
      host = document.createElement("div");
      host.className = "slides-tools-panel-host";
      host.id = "slides-tools-panel-host";
      root.appendChild(host);
    }
    return host;
  }

  /* A stable id is derived from the slide's server index (data-slide /
   * data-index). If the fragment supplies neither, we stamp a persistent one
   * on the node so it survives later panel rebuilds and reordering. */
  function stableId(node, position) {
    var id = node.getAttribute("data-outline-id");
    if (!id) {
      var key = node.getAttribute("data-slide") || node.getAttribute("data-index");
      if (key === null || key === undefined || key === "") key = "p" + position;
      id = "slide:" + key;
      node.setAttribute("data-outline-id", id);
    }
    return id;
  }
  function thumbLabel(node, position) {
    var num = node.querySelector(".sl-thumb-num");
    var raw = num ? (num.textContent || "") : "";
    var match = String(raw).match(/\S+/);
    var prefix = match && /[^\d]/.test(match[0]) ? match[0].replace(/\d.*$/, "").trim() : "";
    var title = node.querySelector("h1,h2,h3,[data-type='title'] p");
    var text = title ? (title.textContent || "").trim() : "";
    if (!text) {
      var el = node.querySelector(".sl-element [contenteditable], .sl-element p");
      text = el ? (el.textContent || "").trim() : "";
    }
    if (!text && prefix) return prefix + " " + (position + 1);
    return text || t("slides-outline-untitled", "Slide {n}", { n: position + 1 });
  }
  function slides(root) {
    var host = thumbsHost(root);
    var result = [];
    if (!host) return result;
    var nodes = host.querySelectorAll(".sl-thumb");
    for (var i = 0; i < nodes.length; i += 1) {
      var node = nodes[i];
      result.push({
        id: stableId(node, i),
        key: String(node.getAttribute("data-index") || node.getAttribute("data-slide") || i),
        node: node,
        title: thumbLabel(node, i)
      });
    }
    return result;
  }
  function currentKey(root) {
    var host = thumbsHost(root);
    var active = host ? host.querySelector(".sl-thumb.active") : null;
    if (active) return stableId(active, 0);
    var canvas = root.querySelector("#slide-canvas") || document.getElementById("slide-canvas");
    if (canvas) {
      var id = canvas.getAttribute("data-slide-id");
      if (id !== null && id !== undefined && id !== "") return "slide:" + id;
    }
    return null;
  }
  function rows(list) {
    return list ? Array.prototype.slice.call(list.querySelectorAll("." + ROW)) : [];
  }
  function markSelected(list, id) {
    rows(list).forEach(function (row) {
      var on = row.getAttribute("data-outline-id") === id;
      row.classList.toggle("is-selected", on);
      row.setAttribute("aria-selected", on ? "true" : "false");
    });
  }
  function renumber(list) {
    rows(list).forEach(function (row, index) {
      var num = row.querySelector(".sl-outline-num");
      if (num) num.textContent = String(index + 1);
    });
  }
  function button(cls, glyph, label) {
    var node = document.createElement("button");
    node.type = "button";
    node.className = "btn-icon " + cls;
    node.title = label;
    node.setAttribute("aria-label", label);
    node.textContent = glyph;
    return node;
  }
  function renderRows(root, list) {
    list.textContent = "";
    var current = root[SEL_KEY] || currentKey(root);
    slides(root).forEach(function (slide, index) {
      var row = document.createElement("div");
      row.className = ROW + " is-slide";
      row.setAttribute("data-outline-id", slide.id);
      row.setAttribute("data-index", slide.key);
      row.setAttribute("role", "option");
      row.__root = root;

      var grip = document.createElement("span");
      grip.className = "slides-outline-grip";
      grip.setAttribute("aria-hidden", "true");
      grip.textContent = "⋮⋮";

      var num = document.createElement("span");
      num.className = "sl-outline-num";
      num.textContent = String(index + 1);

      var label = document.createElement("span");
      label.className = "slides-outline-text";
      label.textContent = slide.title;

      var actions = document.createElement("span");
      actions.className = "slides-outline-row-actions";
      var moveLabel = t("action-move", "Move");
      actions.appendChild(button("sl-outline-up", "↑", moveLabel + " ↑"));
      actions.appendChild(button("sl-outline-down", "↓", moveLabel + " ↓"));

      row.appendChild(grip);
      row.appendChild(num);
      row.appendChild(label);
      row.appendChild(actions);
      list.appendChild(row);
    });
    markSelected(list, current);
  }
  function build(root) {
    var existing = document.getElementById(PANEL_ID);
    if (existing && existing.parentNode) existing.parentNode.removeChild(existing);

    var panel = document.createElement("div");
    panel.className = "slides-tools-panel";
    panel.id = PANEL_ID;
    panel.setAttribute("role", "region");
    panel.setAttribute("data-i18n-aria-label", "slides-sorter");
    panel.setAttribute("aria-label", t("slides-sorter", "Slide sorter"));

    var head = document.createElement("div");
    head.className = "slides-tools-panel-head";
    var heading = document.createElement("h2");
    heading.setAttribute("data-i18n", "slides-sorter");
    heading.textContent = t("slides-sorter", "Slide sorter");
    var close = button("sl-outline-close", "×", t("action-close", "Close"));
    head.appendChild(heading);
    head.appendChild(close);

    var list = document.createElement("div");
    list.className = "slides-outline-list";
    list.setAttribute("role", "listbox");

    panel.appendChild(head);
    panel.appendChild(list);
    panelHost(root).appendChild(panel);
    renderRows(root, list);
    return panel;
  }
  function closePanel() {
    var root = scope();
    var panel = root ? root.querySelector("#" + PANEL_ID) : document.getElementById(PANEL_ID);
    if (panel && panel.parentNode) panel.parentNode.removeChild(panel);
  }
  function openPanel() {
    var root = scope();
    if (root) build(root);
  }
  function togglePanel() {
    var root = scope();
    if (!root) return;
    if (root.querySelector("#" + PANEL_ID)) closePanel();
    else openPanel();
  }

  /* Mirror the panel's order into the thumbnail rail. Appending each node in
   * sequence moves it to the end, so the rail ends up in exactly panel order. */
  function commit(root) {
    var panel = root.querySelector("#" + PANEL_ID);
    var host = thumbsHost(root);
    if (!panel || !host) return;
    var order = rows(panel).map(function (row) { return row.getAttribute("data-outline-id"); });
    var byId = {};
    var thumbs = host.querySelectorAll(".sl-thumb");
    for (var i = 0; i < thumbs.length; i += 1) byId[stableId(thumbs[i], i)] = thumbs[i];
    // The server fragment wraps the thumbnails in an inner flex <div>, so a
    // thumb's parentNode is usually NOT #sidebar-thumbs. Reorder inside the
    // actual container instead of requiring a direct child (#2473 UAT).
    var container = thumbs.length ? thumbs[0].parentNode : null;
    if (!container || typeof container.appendChild !== "function") return;
    order.forEach(function (id) {
      if (byId[id] && container.contains(byId[id])) container.appendChild(byId[id]);
    });
    Array.prototype.slice.call(container.querySelectorAll(".sl-thumb")).forEach(function (node, index) {
      var n = String(index + 1);
      node.setAttribute("data-index", String(index));
      node.setAttribute("data-slide", String(index));
      // Legacy local markup used .sl-thumb-num; the server fragment emits a
      // plain "Slide N" label div. Update whichever is present.
      var num = node.querySelector(".sl-thumb-num");
      if (num) {
        num.textContent = /slide/i.test(num.textContent || "")
          ? t("slides-outline-untitled", "Slide {n}", { n: n }) : String(n);
        return;
      }
      var labels = node.querySelectorAll("div");
      for (var j = 0; j < labels.length; j += 1) {
        if (/^\s*slide\s+\d+\s*$/i.test(labels[j].textContent || "")) {
          labels[j].textContent = t("slides-outline-untitled", "Slide {n}", { n: n });
          break;
        }
      }
    });
  }
  function moveRow(row, delta) {
    if (!row || !row.parentNode) return;
    var list = row.parentNode;
    var all = rows(list);
    var from = all.indexOf(row);
    var to = from + delta;
    if (from < 0 || to < 0 || to >= all.length) return;
    list.insertBefore(row, delta > 0 ? all[to].nextSibling : all[to]);
    renumber(list);
    commit(row.__root || scope());
  }
  function selectRow(row) {
    if (!row || !row.parentNode) return;
    var root = row.__root || scope();
    if (!root) return;
    var id = row.getAttribute("data-outline-id");
    root[SEL_KEY] = id;
    markSelected(row.parentNode, id);
    var host = thumbsHost(root);
    var thumbs = host ? host.querySelectorAll(".sl-thumb") : [];
    for (var i = 0; i < thumbs.length; i += 1) {
      if (stableId(thumbs[i], i) === id) {
        if (typeof thumbs[i].click === "function") thumbs[i].click();
        if (typeof thumbs[i].scrollIntoView === "function") {
          thumbs[i].scrollIntoView({ block: "nearest" });
        }
        break;
      }
    }
  }
  function scheduleRender() {
    if (renderPending) return;
    renderPending = true;
    window.setTimeout(function () {
      renderPending = false;
      var root = scope();
      if (!root || !root.querySelector("#" + PANEL_ID)) return;
      build(root);
    }, 30);
  }

  var Observer = window.MutationObserver || window.WebKitMutationObserver;
  function watchThumbs(root) {
    var host = thumbsHost(root);
    if (!host || !Observer) return;
    if (root.__slOutlineHost === host) return;
    root.__slOutlineHost = host;
    if (root.__slOutlineObserver) root.__slOutlineObserver.disconnect();
    var observer = new Observer(scheduleRender);
    observer.observe(host, { childList: true });
    root.__slOutlineObserver = observer;
  }
  function addToggle(root) {
    if (root.querySelector("#" + TOGGLE_ID)) return;
    var bar = root.querySelector(".slides-toolbar-row-main") || root.querySelector(".slides-toolbar");
    if (!bar) return;
    var toggle = button(TOGGLE_ID, "☰", t("slides-sorter", "Slide sorter"));
    toggle.id = TOGGLE_ID;
    toggle.removeAttribute("aria-label");
    toggle.setAttribute("data-i18n-title", "slides-sorter");
    toggle.setAttribute("data-i18n-aria-label", "slides-sorter");
    toggle.setAttribute("aria-label", t("slides-sorter", "Slide sorter"));
    bar.appendChild(toggle);
  }
  function ensure(root) {
    if (!root) return;
    knownRoots = knownRoots.filter(function (old) {
      if (!document.body.contains(old)) {
        if (old.__slOutlineObserver) old.__slOutlineObserver.disconnect();
        return false;
      }
      return true;
    });
    if (knownRoots.indexOf(root) >= 0) return;
    knownRoots.push(root);
    addToggle(root);
    watchThumbs(root);
  }
  function detect() {
    var root = scope();
    if (!root) return;
    ensure(root);
    watchThumbs(root);
  }

  document.addEventListener("click", function (event) {
    var target = event.target;
    if (!target || !target.closest) return;
    if (target.closest("#" + TOGGLE_ID)) {
      event.preventDefault();
      event.stopPropagation();
      togglePanel();
      return;
    }
    var root = scope();
    if (!root) return;
    var railThumb = target.closest(".sl-thumb");
    if (railThumb && root.contains(railThumb)) {
      var host = thumbsHost(root);
      var thumbList = host ? host.querySelectorAll(".sl-thumb") : [];
      for (var i = 0; i < thumbList.length; i += 1) {
        if (thumbList[i] === railThumb) { root[SEL_KEY] = stableId(railThumb, i); break; }
      }
    }
    if (target.closest(".sl-outline-close")) {
      event.preventDefault();
      closePanel();
      return;
    }
    var up = target.closest(".sl-outline-up");
    var down = target.closest(".sl-outline-down");
    if (up || down) {
      var buttonRow = (up || down).closest("." + ROW);
      if (buttonRow && root.contains(buttonRow)) {
        event.preventDefault();
        event.stopPropagation();
        moveRow(buttonRow, up ? -1 : 1);
      }
      return;
    }
    var row = target.closest("." + ROW);
    if (row && root.contains(row) && row.closest("#" + PANEL_ID)) {
      event.preventDefault();
      event.stopPropagation();
      selectRow(row);
    }
  });

  document.addEventListener("pointerdown", function (event) {
    var target = event.target;
    if (!target || !target.closest) return;
    var grip = target.closest(".slides-outline-grip");
    if (!grip) return;
    var row = grip.closest("." + ROW);
    var root = scope();
    if (!row || !root || !root.contains(row) || !row.closest("#" + PANEL_ID)) return;
    event.preventDefault();
    drag = { row: row, root: root, grip: grip, pointerId: event.pointerId };
    row.classList.add("is-dragging");
    if (typeof grip.setPointerCapture === "function" && event.pointerId !== undefined) {
      grip.setPointerCapture(event.pointerId);
    }
  });

  document.addEventListener("pointermove", function (event) {
    if (!drag) return;
    var list = drag.row.parentNode;
    if (!list) return;
    event.preventDefault();
    var under = document.elementFromPoint(event.clientX, event.clientY);
    var target = under && under.closest ? under.closest("." + ROW) : null;
    if (!target || target === drag.row || target.parentNode !== list) return;
    var rect = typeof target.getBoundingClientRect === "function" ? target.getBoundingClientRect() : null;
    var after = rect ? event.clientY > rect.top + rect.height / 2 : true;
    list.insertBefore(drag.row, after ? target.nextSibling : target);
    renumber(list);
  });
  function endDrag() {
    if (!drag) return;
    var root = drag.root;
    drag.row.classList.remove("is-dragging");
    drag = null;
    commit(root);
  }
  document.addEventListener("pointerup", endDrag);
  document.addEventListener("pointercancel", endDrag);

  if (Observer) {
    var docObserver = new Observer(detect);
    docObserver.observe(document.body || document.documentElement, { childList: true, subtree: true });
  }
  detect();

  window.GBSlidesOutlineSorter = {
    refresh: function () { var root = scope(); if (root && root.querySelector("#" + PANEL_ID)) build(root); },
    open: openPanel,
    close: closePanel,
    toggle: togglePanel,
    move: function (index, delta) {
      var root = scope();
      var panel = root && root.querySelector("#" + PANEL_ID);
      var all = panel ? rows(panel) : [];
      if (all[index]) moveRow(all[index], delta);
    },
    scope: scope,
    slides: function () { var root = scope(); return root ? slides(root) : []; },
    selected: function () { var root = scope(); return root ? (root[SEL_KEY] || currentKey(root)) : null; }
  };
})();
