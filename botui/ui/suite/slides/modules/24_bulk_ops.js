"use strict";
/* Slides bulk operations (#2433): accessible multi-slide selection with move,
 * duplicate, delete and hide. All changes are local DOM/localStorage only, one
 * step undoable, and confirmed through the in-app accessible dialog. */
(function () {
  var O = window.GBSlidesOutline;
  var STORE = { hidden: "bulk-hidden", removed: "bulk-removed", order: "bulk-order" };

  function selected(host) {
    return O.$$("input[data-bulk-index]:checked", host).map(function (box) {
      return parseInt(box.getAttribute("data-bulk-index"), 10);
    });
  }
  function thumbFor(index) {
    var found = null;
    O.slides().forEach(function (slide) { if (slide.index === index) found = slide.thumb; });
    return found;
  }
  function label(indices) {
    return indices.map(function (index) { return O.t("slides-outline-untitled", "Slide {n}", { n: index + 1 }); });
  }
  function snapshot() {
    return O.slides().map(function (slide) { return slide.thumb; }).filter(Boolean);
  }
  function restoreOrder(thumbs) {
    var rail = O.$("#sidebar-thumbs");
    if (!rail || !thumbs.length) return;
    thumbs.forEach(function (thumb) { if (thumb.parentNode) rail.appendChild(thumb); });
    O.save(STORE.order, thumbs.map(function (thumb) { return thumb.getAttribute("data-index"); }));
  }

  function requireSelection(host) {
    var indices = selected(host);
    if (!indices.length) {
      O.status(O.t("slides-outline-bulk-none", "Select at least one slide first"));
      return null;
    }
    return indices;
  }
  function confirmAction(host, titleKey, title, confirmKey, confirm, verb, action) {
    var indices = requireSelection(host);
    if (!indices) return;
    O.confirm({
      titleKey: titleKey, title: title, confirmKey: confirmKey, confirm: confirm,
      messageKey: "slides-outline-bulk-message",
      message: O.t("slides-outline-bulk-message", "Apply {verb} to the selected slides?", { verb: verb }),
      detail: label(indices)
    }).then(function (ok) { if (ok) action(indices); });
  }

  function move(host, direction) {
    var indices = requireSelection(host);
    if (!indices) return;
    var thumbs = indices.map(thumbFor).filter(Boolean);
    var before = snapshot();
    var rail = O.$("#sidebar-thumbs");
    if (!rail || !thumbs.length) return;
    thumbs.forEach(function (thumb) {
      var siblings = Array.prototype.slice.call(rail.children);
      var position = siblings.indexOf(thumb);
      var target = position + direction;
      if (target < 0 || target >= siblings.length) return;
      rail.insertBefore(thumb, direction > 0 ? siblings[target].nextSibling : siblings[target]);
    });
    restoreOrder(snapshot());
    O.pushUndo({
      label: O.t("slides-outline-action-bulk-move", "move slides"),
      undo: function () { restoreOrder(before); },
      redo: function () { restoreOrder(snapshot()); }
    });
    O.status(O.t("slides-outline-bulk-moved", "Slides moved locally"));
  }

  function duplicate(host) {
    confirmAction(host, "slides-outline-bulk-dup-title", "Duplicate slides",
      "slides-outline-bulk-dup", "Duplicate", "duplicate", function (indices) {
        var before = snapshot();
        var created = [];
        indices.forEach(function (index) {
          var thumb = thumbFor(index);
          if (!thumb || !thumb.parentNode) return;
          var clone = thumb.cloneNode(true);
          clone.setAttribute("data-local-duplicate", "true");
          clone.setAttribute("data-index", "local-" + Date.now().toString(36) + "-" + created.length);
          clone.setAttribute("data-slide", String(index));
          thumb.parentNode.insertBefore(clone, thumb.nextSibling);
          created.push(clone);
        });
        restoreOrder(snapshot());
        O.pushUndo({
          label: O.t("slides-outline-action-bulk-dup", "duplicate slides"),
          undo: function () { created.forEach(function (node) { if (node.parentNode) node.parentNode.removeChild(node); }); restoreOrder(before); },
          redo: function () { created.forEach(function (node, i) { var source = thumbFor(indices[i]); if (source && source.parentNode) source.parentNode.insertBefore(node, source.nextSibling); }); restoreOrder(snapshot()); }
        });
        O.status(O.t("slides-outline-bulk-duplicated", "Slides duplicated locally"));
      });
  }

  function remove(host) {
    confirmAction(host, "slides-outline-bulk-del-title", "Delete slides",
      "slides-outline-bulk-delete", "Delete", "delete", function (indices) {
        var stored = O.load(STORE.removed, []);
        var removed = [];
        indices.forEach(function (index) {
          var thumb = thumbFor(index);
          if (!thumb || !thumb.parentNode) return;
          removed.push({ node: thumb, parent: thumb.parentNode, next: thumb.nextSibling, index: index });
          thumb.parentNode.removeChild(thumb);
          if (stored.indexOf(index) === -1) stored.push(index);
        });
        O.save(STORE.removed, stored);
        O.pushUndo({
          label: O.t("slides-outline-action-bulk-delete", "delete slides"),
          undo: function () {
            removed.forEach(function (item) { item.parent.insertBefore(item.node, item.next); });
            O.save(STORE.removed, O.load(STORE.removed, []).filter(function (i) { return removed.map(function (r) { return r.index; }).indexOf(i) === -1; }));
          },
          redo: function () {
            removed.forEach(function (item) { if (item.node.parentNode) item.node.parentNode.removeChild(item.node); });
            O.save(STORE.removed, stored);
          }
        });
        O.status(O.t("slides-outline-bulk-deleted", "Slides removed locally"));
        renderRows(host);
      });
  }

  function hide(host) {
    confirmAction(host, "slides-outline-bulk-hide-title", "Hide slides",
      "slides-outline-bulk-hide", "Hide", "hide", function (indices) {
        var stored = O.load(STORE.hidden, []);
        indices.forEach(function (index) {
          var thumb = thumbFor(index);
          if (thumb) thumb.setAttribute("data-hidden", "true");
          if (stored.indexOf(index) === -1) stored.push(index);
        });
        O.save(STORE.hidden, stored);
        O.pushUndo({
          label: O.t("slides-outline-action-bulk-hide", "hide slides"),
          undo: function () {
            indices.forEach(function (index) { var thumb = thumbFor(index); if (thumb) thumb.removeAttribute("data-hidden"); });
            O.save(STORE.hidden, stored.filter(function (i) { return indices.indexOf(i) === -1; }));
          },
          redo: function () {
            indices.forEach(function (index) { var thumb = thumbFor(index); if (thumb) thumb.setAttribute("data-hidden", "true"); });
            O.save(STORE.hidden, stored);
          }
        });
        O.status(O.t("slides-outline-bulk-hidden", "Slides hidden locally"));
      });
  }

  function renderRows(host) {
    host.textContent = "";
    O.slides().forEach(function (slide) {
      if (String(slide.index).indexOf("local-") === 0) return;
      var row = O.el("label", "slides-bulk-row");
      var box = document.createElement("input");
      box.type = "checkbox";
      box.setAttribute("data-bulk-index", String(slide.index));
      row.appendChild(box);
      row.appendChild(O.el("span", "", O.t("slides-outline-untitled", "Slide {n}", { n: slide.index + 1 }), "slides-outline-untitled"));
      if (slide.thumb && slide.thumb.getAttribute("data-hidden") === "true") {
        row.appendChild(O.el("span", "slides-bulk-hidden-flag",
          O.t("slides-outline-bulk-is-hidden", "Hidden"), "slides-outline-bulk-is-hidden"));
      }
      host.appendChild(row);
    });
    if (!host.children.length) {
      host.appendChild(O.el("p", "slides-tools-empty", O.t("slides-outline-bulk-empty", "No slides loaded"), "slides-outline-bulk-empty"));
    }
  }

  function open() {
    var panel = O.openPanel("slides-bulk-panel", function () {
      var root = O.el("section", "slides-tools-panel slides-bulk-panel");
      var head = O.el("div", "slides-tools-panel-head");
      head.appendChild(O.el("h2", "", O.t("slides-outline-bulk-title", "Bulk slide actions"), "slides-outline-bulk-title"));
      head.appendChild(O.localBadge());
      var close = O.el("button", "slides-tools-close", O.t("slides-outline-close", "Close"), "slides-outline-close");
      close.type = "button"; close.addEventListener("click", O.closePanels);
      head.appendChild(close);
      root.appendChild(head);

      var toolbar = O.el("div", "slides-tools-toolbar");
      var actions = [
        { key: "slides-outline-bulk-move-up", fallback: "Move up", cls: "", run: function () { move(host, -1); } },
        { key: "slides-outline-bulk-move-down", fallback: "Move down", cls: "", run: function () { move(host, 1); } },
        { key: "slides-outline-bulk-duplicate", fallback: "Duplicate", cls: "", run: function () { duplicate(host); } },
        { key: "slides-outline-bulk-delete", fallback: "Delete", cls: "danger", run: function () { remove(host); } },
        { key: "slides-outline-bulk-hide", fallback: "Hide", cls: "", run: function () { hide(host); } }
      ];
      actions.forEach(function (spec) {
        var button = O.el("button", "slides-tools-btn " + spec.cls, O.t(spec.key, spec.fallback), spec.key);
        button.type = "button"; button.addEventListener("click", spec.run);
        toolbar.appendChild(button);
      });
      var undo = O.el("button", "slides-tools-btn slides-tools-undo", O.t("action-undo", "Undo"), "action-undo");
      undo.type = "button"; undo.addEventListener("click", O.undo);
      toolbar.appendChild(undo);
      root.appendChild(toolbar);
      var host = O.el("div", "slides-bulk-list");
      host.setAttribute("role", "group");
      host.setAttribute("data-i18n-aria-label", "slides-outline-bulk-title");
      root.appendChild(host);
      return root;
    });
    renderRows(panel.querySelector(".slides-bulk-list"));
    O.updateUndoUI();
    return panel;
  }

  function restore() {
    var hidden = O.load(STORE.hidden, []);
    O.slides().forEach(function (slide) {
      if (slide.thumb && hidden.indexOf(slide.index) !== -1) slide.thumb.setAttribute("data-hidden", "true");
    });
  }

  window.GBSlidesBulk = { open: open, restore: restore };
})();
