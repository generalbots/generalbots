"use strict";
/* Slides outline view (#2433): editable local hierarchy of slide titles and
 * bullets, promote/demote, keyboard + drag reorder, and Word paste-as-bullets.
 * The hierarchy is a local-only document derived from the rendered slides. */
(function () {
  var O = window.GBSlidesOutline;
  var STORE = "outline";

  function blank() { return { rows: [], collapsed: {} }; }
  function data() { return O.load(STORE, blank()); }

  function derivedBullets(slide) {
    var bullets = [];
    slide.elements.concat(slide.canvasElements).forEach(function (element) {
      var type = element.getAttribute("data-type") || "";
      if (type !== "text") return;
      var node = element.querySelector("p,h1,h2,h3") || element;
      String(node.textContent || "").split(/\n+/).forEach(function (line) {
        var clean = line.trim();
        if (clean) bullets.push(clean);
      });
    });
    return bullets;
  }
  function slideTitle(slide, position) {
    var title = "";
    slide.elements.concat(slide.canvasElements).forEach(function (element) {
      if ((element.getAttribute("data-type") || "") === "title" && !title) {
        var node = element.querySelector("h1,h2,h3,p") || element;
        title = (node.textContent || "").trim();
      }
    });
    return title || O.t("slides-outline-untitled", "Slide {n}", { n: position + 1 });
  }
  function rebuild() {
    var model = blank();
    O.slides().forEach(function (slide, position) {
      model.rows.push({
        id: "s" + slide.index, slideIndex: slide.index, kind: "slide",
        text: slideTitle(slide, position), level: 0
      });
      derivedBullets(slide).forEach(function (text, index) {
        model.rows.push({
          id: "b" + slide.index + "-" + index, slideIndex: slide.index,
          kind: "bullet", text: text, level: 1
        });
      });
    });
    O.save(STORE, model);
    return model;
  }
  function ensure() {
    var model = data();
    if (!model.rows || !model.rows.length) return rebuild();
    return model;
  }
  function persist(model) { O.save(STORE, model); }

  function wordLines(raw) {
    return String(raw || "").split(/\r?\n/).map(function (line) {
      var depth = (line.match(/^\s*/) || [""])[0].replace(/[^\t]/g, "").length;
      var clean = line.replace(/^[\s\u00a0]*/, "")
        .replace(/^(?:[•●▪◦‣*\-–—]|\d+[.)]|[a-zA-Z][.)])\s+/, "")
        .replace(/[\s\u00a0]+$/, "");
      return clean ? { text: clean, level: Math.min(5, 1 + depth) } : null;
    }).filter(Boolean);
  }

  function findRow(model, id) {
    for (var i = 0; i < model.rows.length; i++) {
      if (model.rows[i].id === id) return i;
    }
    return -1;
  }
  function moveRow(model, id, delta) {
    var from = findRow(model, id);
    var to = from + delta;
    if (from < 0 || to < 0 || to >= model.rows.length) return false;
    var row = model.rows.splice(from, 1)[0];
    model.rows.splice(to, 0, row);
    return true;
  }
  function updateRowText(model, id, text) {
    var index = findRow(model, id);
    if (index >= 0) model.rows[index].text = text;
  }
  function applyOrder(model, list, order) {
    var byId = {};
    model.rows.forEach(function (row) { byId[row.id] = row; });
    model.rows = order.map(function (id) { return byId[id]; }).filter(Boolean);
    persist(model);
    render(list, model);
  }
  var draggedId = null;

  function iconButton(cls, key, fallback, handler) {
    var button = O.el("button", "slides-tools-icon " + cls, O.t(key, fallback), key);
    button.type = "button";
    button.setAttribute("data-i18n-title", key);
    button.setAttribute("data-i18n-aria-label", key);
    button.title = O.t(key, fallback);
    button.setAttribute("aria-label", O.t(key, fallback));
    button.addEventListener("click", handler);
    return button;
  }

  function renderRow(model, row, list) {
    var line = O.el("div", "slides-outline-row" + (row.kind === "slide" ? " is-slide" : ""));
    line.setAttribute("role", "treeitem");
    line.setAttribute("aria-level", String(row.level + 1));
    line.draggable = true;
    line.dataset.id = row.id;
    line.dataset.slide = String(row.slideIndex);
    line.style.setProperty("--so-indent", String(row.level * 18) + "px");

    var handle = O.el("span", "slides-outline-grip", O.t("slides-outline-drag-hint", "Drag to reorder"), "slides-outline-drag-hint");
    handle.setAttribute("aria-hidden", "true");
    line.appendChild(handle);

    var input = document.createElement(row.kind === "slide" ? "input" : "textarea");
    input.className = "slides-outline-text";
    input.value = row.text || "";
    input.setAttribute("data-i18n-placeholder", "slides-outline-text-placeholder");
    input.placeholder = O.t("slides-outline-text-placeholder", "Outline text");
    input.setAttribute("aria-label", (row.kind === "slide" ? O.t("slides-outline-title-label", "Slide title") : O.t("slides-outline-bullet-label", "Bullet")));
    input.addEventListener("change", function () {
      var before = row.text;
      updateRowText(model, row.id, input.value);
      persist(model);
      O.pushUndo({
        label: O.t("slides-outline-action-edit", "edit outline text"),
        undo: function () { updateRowText(model, row.id, before); persist(model); render(list, model); },
        redo: function () { updateRowText(model, row.id, input.value); persist(model); render(list, model); }
      });
    });
    line.appendChild(input);

    var actions = O.el("div", "slides-outline-row-actions");
    actions.appendChild(iconButton("slides-outline-promote", "slides-outline-promote", "Promote", function () {
      if (row.level <= 0) return;
      var before = row.level; row.level -= 1; persist(model); render(list, model);
      O.pushUndo({
        label: O.t("slides-outline-action-promote", "promote item"),
        undo: function () { row.level = before; persist(model); render(list, model); },
        redo: function () { row.level = before - 1; persist(model); render(list, model); }
      });
      O.status(O.t("slides-outline-promoted", "Item promoted"));
    }));
    actions.appendChild(iconButton("slides-outline-demote", "slides-outline-demote", "Demote", function () {
      if (row.level >= 5) return;
      var before = row.level; row.level += 1; persist(model); render(list, model);
      O.pushUndo({
        label: O.t("slides-outline-action-demote", "demote item"),
        undo: function () { row.level = before; persist(model); render(list, model); },
        redo: function () { row.level = before + 1; persist(model); render(list, model); }
      });
      O.status(O.t("slides-outline-demoted", "Item demoted"));
    }));
    actions.appendChild(iconButton("slides-outline-up", "slides-outline-move-up", "Move up", function () {
      if (moveRow(model, row.id, -1)) { persist(model); render(list, model); O.status(O.t("slides-outline-moved", "Item moved")); }
    }));
    actions.appendChild(iconButton("slides-outline-down", "slides-outline-move-down", "Move down", function () {
      if (moveRow(model, row.id, 1)) { persist(model); render(list, model); O.status(O.t("slides-outline-moved", "Item moved")); }
    }));
    actions.appendChild(iconButton("slides-outline-goto", "slides-outline-jump", "Go to slide", function () {
      O.jump(row.slideIndex);
    }));
    line.appendChild(actions);

    input.addEventListener("keydown", function (event) {
      var key = event.key;
      if (key === "Tab") {
        event.preventDefault();
        actions.children[key === "Tab" && event.shiftKey ? 0 : 1].click();
      } else if (event.altKey && (key === "ArrowUp" || key === "ArrowDown")) {
        event.preventDefault();
        actions.children[key === "ArrowUp" ? 2 : 3].click();
      }
    });
    line.addEventListener("dragstart", function (event) {
      draggedId = row.id;
      line.setAttribute("aria-grabbed", "true");
      if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData("text/plain", row.id);
      }
    });
    line.addEventListener("dragover", function (event) {
      event.preventDefault();
      line.classList.add("is-drop-target");
      if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    });
    line.addEventListener("dragleave", function () { line.classList.remove("is-drop-target"); });
    line.addEventListener("drop", function (event) {
      event.preventDefault();
      line.classList.remove("is-drop-target");
      var sourceId = draggedId || (event.dataTransfer ? event.dataTransfer.getData("text/plain") : "");
      draggedId = null;
      if (!sourceId || sourceId === row.id) return;
      var from = findRow(model, sourceId);
      var to = findRow(model, row.id);
      if (from < 0 || to < 0) return;
      var before = model.rows.map(function (item) { return item.id; });
      var moved = model.rows.splice(from, 1)[0];
      model.rows.splice(to, 0, moved);
      var after = model.rows.map(function (item) { return item.id; });
      persist(model);
      render(list, model);
      O.pushUndo({
        label: O.t("slides-outline-action-reorder", "reorder outline item"),
        undo: function () { applyOrder(model, list, before); },
        redo: function () { applyOrder(model, list, after); }
      });
      O.status(O.t("slides-outline-moved", "Item moved"));
    });
    line.addEventListener("dragend", function () {
      line.removeAttribute("aria-grabbed");
      line.classList.remove("is-drop-target");
      draggedId = null;
    });
    list.appendChild(line);
  }

  function render(list, model) {
    list.textContent = "";
    list.setAttribute("role", "tree");
    model.rows.forEach(function (row) { renderRow(model, row, list); });
  }

  function open() {
    var model = ensure();
    var panel = O.openPanel("slides-outline-panel", function () {
      var root = O.el("section", "slides-tools-panel slides-outline-panel");
      var head = O.el("div", "slides-tools-panel-head");
      head.appendChild(O.el("h2", "", O.t("slides-outline-title", "Outline"), "slides-outline-title"));
      head.appendChild(O.localBadge());
      var closeBtn = O.el("button", "slides-tools-close", O.t("slides-outline-close", "Close"), "slides-outline-close");
      closeBtn.type = "button";
      closeBtn.addEventListener("click", O.closePanels);
      head.appendChild(closeBtn);
      root.appendChild(head);

      var toolbar = O.el("div", "slides-tools-toolbar");
      var pasteBtn = O.el("button", "slides-tools-btn", O.t("slides-outline-paste-bullets", "Paste bullets"), "slides-outline-paste-bullets");
      pasteBtn.type = "button";
      var pasteArea = document.createElement("textarea");
      pasteArea.className = "slides-outline-paste";
      pasteArea.rows = 3;
      pasteArea.setAttribute("data-i18n-placeholder", "slides-outline-paste-placeholder");
      pasteArea.placeholder = O.t("slides-outline-paste-placeholder", "Paste bullet lines from Word here");
      pasteBtn.addEventListener("click", function () {
        if (pasteArea.style.display === "none") { pasteArea.style.display = "block"; pasteArea.focus(); }
        else pasteArea.style.display = "none";
      });
      pasteArea.addEventListener("paste", function (event) {
        var raw = (event.clipboardData || window.clipboardData) ? (event.clipboardData || window.clipboardData).getData("text") : "";
        if (!raw) return;
        event.preventDefault();
        var parsed = wordLines(raw);
        var current = O.slides().length ? O.slides()[0].index : 0;
        var before = model.rows.slice();
        parsed.forEach(function (item, index) {
          model.rows.push({ id: "p" + Date.now() + "-" + index, slideIndex: current, kind: "bullet", text: item.text, level: item.level });
        });
        persist(model); render(list, model);
        O.pushUndo({
          label: O.t("slides-outline-action-paste", "paste bullets"),
          undo: function () { model.rows = before.slice(); persist(model); render(list, model); },
          redo: function () { parsed.forEach(function (item, index) { model.rows.push({ id: "p" + Date.now() + "-" + index, slideIndex: current, kind: "bullet", text: item.text, level: item.level }); }); persist(model); render(list, model); }
        });
        O.status(O.t("slides-outline-pasted", "{n} bullets added locally", { n: parsed.length }));
      });
      var rebuildBtn = O.el("button", "slides-tools-btn", O.t("slides-outline-rebuild", "Rebuild from slides"), "slides-outline-rebuild");
      rebuildBtn.type = "button";
      rebuildBtn.addEventListener("click", function () {
        model = rebuild(); render(list, model);
        O.status(O.t("slides-outline-rebuilt", "Outline rebuilt from slides"));
      });
      var undoBtn = O.el("button", "slides-tools-btn slides-tools-undo", O.t("action-undo", "Undo"), "action-undo");
      undoBtn.type = "button"; undoBtn.addEventListener("click", O.undo);
      var redoBtn = O.el("button", "slides-tools-btn slides-tools-redo", O.t("action-redo", "Redo"), "action-redo");
      redoBtn.type = "button"; redoBtn.addEventListener("click", O.redo);
      toolbar.appendChild(pasteBtn); toolbar.appendChild(rebuildBtn);
      toolbar.appendChild(undoBtn); toolbar.appendChild(redoBtn);
      root.appendChild(toolbar);
      pasteArea.style.display = "none";
      root.appendChild(pasteArea);
      var list = O.el("div", "slides-outline-list");
      root.appendChild(list);
      return root;
    });
    render(panel.querySelector(".slides-outline-list"), model);
    O.updateUndoUI();
    return panel;
  }

  window.GBSlidesOutlineView = { open: open };
})();
