"use strict";
/* Slides delivery mode (#2432) — named slide sections with a jump list and
 * section navigation. Sections are local metadata; no PPTX section compile or
 * server write is attempted. */
(function (window) {
  var D = window.GBSlidesDelivery;

  function ordered() {
    return D.load().sections.slice().sort(function (a, b) {
      return (Number(a.start) || 0) - (Number(b.start) || 0);
    });
  }
  function owner(index, list) {
    var all = list || ordered();
    var found = all.length ? all[0] : null;
    all.forEach(function (section) {
      if ((Number(section.start) || 0) <= index) found = section;
    });
    return found;
  }
  function bounds(section, list) {
    var all = list || ordered();
    var position = all.map(function (item) { return item.id; }).indexOf(section.id);
    var next = all[position + 1];
    var from = Number(section.start) || 0;
    var to = next ? (Number(next.start) || 0) - 1 : Math.max(from, D.slideCount() - 1);
    return { from: from, to: to };
  }
  function create(name, start) {
    var section = { id: D.uid("sec"), name: name, start: start };
    D.patch(function (data) { data.sections.push(section); });
    return section;
  }
  function remove(id) {
    if (id === "default") return false;
    D.patch(function (data) {
      data.sections = data.sections.filter(function (section) { return section.id !== id; });
    });
    return true;
  }
  function neighbours(current) {
    var all = ordered();
    var currentSection = owner(current, all);
    var at = currentSection ? all.map(function (item) { return item.id; }).indexOf(currentSection.id) : -1;
    return {
      previous: at > 0 ? all[at - 1] : null,
      next: at >= 0 && at < all.length - 1 ? all[at + 1] : null
    };
  }
  function jumpTo(section) {
    if (!section) return false;
    D.jump(section.start);
    D.status("slides-delivery-sections-jumped", "Jumped to section { $name }", { name: section.name });
    if (D.console) D.console.render();
    return true;
  }
  function jumpNext() {
    var target = neighbours(D.currentSlide()).next;
    if (!target) {
      D.status("slides-delivery-sections-none-next", "Already at the last section.");
      return false;
    }
    return jumpTo(target);
  }
  function jumpPrev() {
    var target = neighbours(D.currentSlide()).previous;
    if (!target) {
      D.status("slides-delivery-sections-none-prev", "Already at the first section.");
      return false;
    }
    return jumpTo(target);
  }

  function open() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-sections-title",
      titleFallback: "Sections",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        var nav = D.el("div", "sd-section-nav");
        nav.appendChild(D.button("slides-delivery-sections-prev", "Previous section", jumpPrev));
        nav.appendChild(D.button("slides-delivery-sections-next", "Next section", jumpNext, "primary"));
        api.body.appendChild(nav);

        api.body.appendChild(D.el("h3", "sd-subhead", "slides-delivery-sections-jump", "Jump list"));
        var list = D.el("div", "sd-section-list");
        list.id = "sd-section-list";
        api.body.appendChild(list);

        var nameField = D.field("slides-delivery-name", "Name", "");
        api.body.appendChild(nameField);
        var startField = D.field("slides-delivery-sections-start", "Starts at slide", "1");
        startField.control.type = "number";
        startField.control.min = "1";
        startField.control.max = String(D.slideCount());
        api.body.appendChild(startField);
        api.addButton("slides-delivery-sections-create", "Add section", function () {
          var name = String(nameField.control.value || "").trim();
          var start = parseInt(startField.control.value, 10);
          if (!name || !isFinite(start) || start < 1 || start > D.slideCount()) {
            api.setStatus("slides-delivery-sections-invalid",
              "Enter a name and a slide number from 1 to { $n }.", { n: D.slideCount() });
            return;
          }
          create(name, start - 1);
          api.setStatus("slides-delivery-saved", "Saved locally");
          renderRows();
        }, "primary");

        function renderRows() {
          list.textContent = "";
          var all = ordered();
          if (!all.length) {
            list.appendChild(D.el("p", "sd-hint", "slides-delivery-sections-empty", "No sections yet."));
            return;
          }
          all.forEach(function (section) {
            var range = bounds(section, all);
            var row = D.el("div", "sd-section-row");
            var jump = D.button(null, null, function () { jumpTo(section); }, "link");
            var label = D.tr("slides-delivery-sections-range", "Slides { $from }–{ $to }",
              { from: range.from + 1, to: range.to + 1 }) + " · " + section.name;
            jump.textContent = label;
            jump.setAttribute("aria-label", label);
            row.appendChild(jump);
            row.appendChild(D.button("slides-delivery-delete", "Delete", function () {
              remove(section.id);
              api.setStatus("slides-delivery-sections-deleted", "Section deleted");
              renderRows();
            }));
            list.appendChild(row);
          });
        }
        renderRows();
        api.addButton("slides-delivery-close", "Close", function () { api.close(); });
      }
    });
    return dialog;
  }

  D.sections = {
    ordered: ordered,
    owner: owner,
    bounds: bounds,
    create: create,
    remove: remove,
    neighbours: neighbours,
    jumpTo: jumpTo,
    jumpNext: jumpNext,
    jumpPrev: jumpPrev,
    open: open
  };
})(window);
