"use strict";
/* Slides sections (#2433): local-only create/rename/reorder/collapse, section
 * navigation and thumbnail grouping. Sections are metadata in localStorage;
 * round-tripping them into PPTX/p:timing needs a server compile gate and is not
 * attempted here. */
(function () {
  var O = window.GBSlidesOutline;
  var STORE = "sections";
  var COLLAPSE = "sections-collapsed";

  function sections() {
    var list = O.load(STORE, []);
    if (!Array.isArray(list) || !list.length) {
      list = [{ id: "default", name: O.t("slides-outline-section-default", "Default section"), start: 0 }];
    }
    return list;
  }
  function persist(list) { O.save(STORE, list); }
  function collapsed() { return O.load(COLLAPSE, {}); }
  function persistCollapsed(map) { O.save(COLLAPSE, map); }
  function slideCount() { return O.slides().length; }

  function owner(start, list) {
    var owner = list[0];
    list.forEach(function (section) { if (section.start <= start) owner = section; });
    return owner;
  }
  function bounds(section, list) {
    var ordered = list.slice().sort(function (a, b) { return a.start - b.start; });
    var index = ordered.map(function (s) { return s.id; }).indexOf(section.id);
    var from = section.start;
    var next = ordered[index + 1];
    var to = next ? next.start - 1 : Math.max(from, slideCount() - 1);
    return { from: from, to: to };
  }

  function annotate(list) {
    O.slides().forEach(function (slide) {
      if (!slide.thumb) return;
      var section = owner(slide.index, list);
      slide.thumb.setAttribute("data-section-id", section.id);
      slide.thumb.setAttribute("data-section-name", section.name);
      slide.thumb.setAttribute("data-i18n-aria-label", "slides-outline-section-chip");
      slide.thumb.setAttribute("aria-label", O.t("slides-outline-section-chip", "Section: {name}", { name: section.name }));
    });
  }
  function clearAnnotate() {
    O.slides().forEach(function (slide) {
      if (!slide.thumb) return;
      slide.thumb.removeAttribute("data-section-name");
      slide.thumb.removeAttribute("data-section-id");
    });
  }

  function askName(titleKey, title, value, onSave) {
    var dialog = O.dialog({ titleKey: titleKey, title: title });
    var label = O.el("label", "slides-tools-field-label",
      O.t("slides-outline-section-name", "Section name"), "slides-outline-section-name");
    var input = document.createElement("input");
    input.type = "text";
    input.value = value || "";
    input.setAttribute("data-i18n-placeholder", "slides-outline-section-name");
    input.placeholder = O.t("slides-outline-section-name", "Section name");
    label.appendChild(input);
    dialog.body.appendChild(label);
    dialog.body.appendChild(O.localBadge());
    var cancel = O.el("button", "slides-tools-btn", O.t("slides-outline-cancel", "Cancel"), "slides-outline-cancel");
    var save = O.el("button", "slides-tools-btn primary", O.t("slides-outline-save", "Save"), "slides-outline-save");
    cancel.type = "button"; save.type = "button";
    cancel.addEventListener("click", function () { dialog.dismiss(false); });
    save.addEventListener("click", function () {
      var name = (input.value || "").trim();
      if (!name) { O.status(O.t("slides-outline-section-required", "Enter a section name")); return; }
      dialog.dismiss(true);
      onSave(name);
    });
    dialog.foot.appendChild(cancel);
    dialog.foot.appendChild(save);
    window.setTimeout(function () { input.focus(); input.select(); }, 0);
  }

  function createSection(list, render) {
    var count = slideCount();
    askName("slides-outline-section-create", "New section", "", function (name) {
      var start = Math.max(0, Math.min(count - 1, count ? count - 1 : 0));
      var before = list.slice();
      var section = { id: "sec-" + Date.now().toString(36), name: name, start: start };
      list.push(section); persist(list); render();
      O.pushUndo({
        label: O.t("slides-outline-action-section-add", "add section"),
        undo: function () { persist(before); render(); },
        redo: function () { persist(before.concat([section])); render(); }
      });
      O.status(O.t("slides-outline-section-created", "Section added locally"));
    });
  }

  function sectionRow(list, section, render, listHost) {
    var map = collapsed();
    var range = bounds(section, list);
    var row = O.el("div", "slides-section-row");
    row.dataset.id = section.id;
    var toggle = O.el("button", "slides-section-toggle", map[section.id] ? "+" : "−", "slides-outline-section-toggle");
    toggle.type = "button";
    toggle.setAttribute("aria-expanded", map[section.id] ? "false" : "true");
    toggle.addEventListener("click", function () {
      map[section.id] = !map[section.id];
      persistCollapsed(map); render();
    });
    row.appendChild(toggle);
    var name = O.el("button", "slides-section-name", section.name);
    name.type = "button";
    name.addEventListener("click", function () { O.jump(range.from); });
    row.appendChild(name);
    var rangeLabel = O.el("span", "slides-section-range",
      O.t("slides-outline-section-range", "{from}–{to}", { from: range.from + 1, to: range.to + 1 }));
    row.appendChild(rangeLabel);
    var actions = O.el("div", "slides-section-actions");
    [
      { cls: "slides-section-up", key: "slides-outline-move-up", fallback: "Move up", run: function () {
        var i = list.indexOf(section); if (i <= 0) return;
        var before = list.slice();
        list.splice(i, 1); list.splice(i - 1, 0, section); persist(list); render();
        O.pushUndo({ label: O.t("slides-outline-action-section-move", "move section"),
          undo: function () { persist(before); render(); },
          redo: function () { list.splice(list.indexOf(section), 1); list.splice(i - 1, 0, section); persist(list); render(); } });
      } },
      { cls: "slides-section-down", key: "slides-outline-move-down", fallback: "Move down", run: function () {
        var i = list.indexOf(section); if (i < 0 || i >= list.length - 1) return;
        var before = list.slice();
        list.splice(i, 1); list.splice(i + 1, 0, section); persist(list); render();
        O.pushUndo({ label: O.t("slides-outline-action-section-move", "move section"),
          undo: function () { persist(before); render(); },
          redo: function () { list.splice(list.indexOf(section), 1); list.splice(i + 1, 0, section); persist(list); render(); } });
      } },
      { cls: "slides-section-rename", key: "slides-outline-section-rename", fallback: "Rename section", run: function () {
        var before = section.name;
        askName("slides-outline-section-rename", "Rename section", section.name, function (name) {
          section.name = name; persist(list); render();
          O.pushUndo({ label: O.t("slides-outline-action-section-rename", "rename section"),
            undo: function () { section.name = before; persist(list); render(); },
            redo: function () { section.name = name; persist(list); render(); } });
        });
      } },
      { cls: "slides-section-delete", key: "slides-outline-section-delete", fallback: "Delete section", run: function () {
        if (section.id === "default") {
          O.status(O.t("slides-outline-section-default-locked", "The default section cannot be deleted")); return;
        }
        var before = list.slice();
        list.splice(list.indexOf(section), 1); persist(list); render();
        O.pushUndo({ label: O.t("slides-outline-action-section-delete", "delete section"),
          undo: function () { persist(before); render(); },
          redo: function () { list.splice(list.indexOf(section), 1); persist(list); render(); } });
      } }
    ].forEach(function (spec) {
      var button = O.el("button", "slides-tools-icon " + spec.cls, O.t(spec.key, spec.fallback), spec.key);
      button.type = "button";
      button.title = O.t(spec.key, spec.fallback);
      button.setAttribute("aria-label", O.t(spec.key, spec.fallback));
      button.addEventListener("click", spec.run);
      actions.appendChild(button);
    });
    row.appendChild(actions);
    listHost.appendChild(row);
    if (!map[section.id]) {
      var group = O.el("div", "slides-section-group");
      O.slides().forEach(function (slide) {
        if (owner(slide.index, list).id !== section.id) return;
        var item = O.el("button", "slides-section-slide",
          O.t("slides-outline-untitled", "Slide {n}", { n: slide.index + 1 }), "slides-outline-untitled");
        item.type = "button";
        item.setAttribute("data-i18n-params", JSON.stringify({ n: slide.index + 1 }));
        item.addEventListener("click", function () { O.jump(slide.index); });
        group.appendChild(item);
      });
      listHost.appendChild(group);
    }
  }

  function open() {
    var list = sections();
    var panel = O.openPanel("slides-sections-panel", function () {
      var root = O.el("section", "slides-tools-panel slides-sections-panel");
      var head = O.el("div", "slides-tools-panel-head");
      head.appendChild(O.el("h2", "", O.t("slides-outline-sections-title", "Sections"), "slides-outline-sections-title"));
      head.appendChild(O.localBadge());
      var close = O.el("button", "slides-tools-close", O.t("slides-outline-close", "Close"), "slides-outline-close");
      close.type = "button"; close.addEventListener("click", function () { O.closePanels(); clearAnnotate(); });
      head.appendChild(close);
      root.appendChild(head);

      var nav = O.el("div", "slides-tools-toolbar");
      var select = document.createElement("select");
      select.setAttribute("aria-label", O.t("slides-outline-section-nav", "Go to section"));
      select.setAttribute("data-i18n-aria-label", "slides-outline-section-nav");
      list.slice().sort(function (a, b) { return a.start - b.start; }).forEach(function (section) {
        var option = document.createElement("option");
        option.value = String(section.start);
        option.textContent = section.name;
        select.appendChild(option);
      });
      select.addEventListener("change", function () { O.jump(parseInt(select.value, 10) || 0); });
      var create = O.el("button", "slides-tools-btn", O.t("slides-outline-section-create", "New section"), "slides-outline-section-create");
      create.type = "button";
      var undo = O.el("button", "slides-tools-btn slides-tools-undo", O.t("action-undo", "Undo"), "action-undo");
      undo.type = "button"; undo.addEventListener("click", O.undo);
      create.addEventListener("click", function () { createSection(list, render); });
      nav.appendChild(select); nav.appendChild(create); nav.appendChild(undo);
      root.appendChild(nav);
      var host = O.el("div", "slides-sections-list");
      root.appendChild(host);
      return root;
    });
    var listHost = panel.querySelector(".slides-sections-list");
    function render() {
      list = sections();
      list.sort(function (a, b) { return a.start - b.start; });
      listHost.textContent = "";
      list.forEach(function (section) { sectionRow(list, section, render, listHost); });
      annotate(list);
    }
    render();
    O.updateUndoUI();
    return panel;
  }

  window.GBSlidesSections = { open: open, annotate: function () { annotate(sections()); } };
})();
