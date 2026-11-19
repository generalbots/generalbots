"use strict";
/* Slides delivery mode (#2432) — custom shows. A named subset of slides with
 * an explicit play order, stored locally. Playing a show walks that order and
 * returns to the original slide when the subset ends. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var active = null;

  function list() { return D.load().shows; }
  function find(id) {
    var all = list();
    for (var i = 0; i < all.length; i += 1) {
      if (all[i].id === id) return all[i];
    }
    return null;
  }
  function isActive() { return !!active; }

  function parseOrder(text, slideCount) {
    var parts = String(text === undefined || text === null ? "" : text)
      .split(/[\s,;]+/).filter(function (part) { return part.length > 0; });
    var order = [];
    var count = Math.max(0, Number(slideCount) || 0);
    for (var i = 0; i < parts.length; i += 1) {
      var number = parseInt(parts[i], 10);
      if (!isFinite(number) || number < 1 || number > count) return { order: [], invalid: true };
      var index = number - 1;
      if (order.indexOf(index) !== -1) return { order: [], invalid: true };
      order.push(index);
    }
    return { order: order, invalid: order.length === 0 };
  }

  function create(name, order) {
    var show = { id: D.uid("show"), name: name, order: order.slice() };
    D.patch(function (data) { data.shows.push(show); });
    return show;
  }
  function remove(id) {
    D.patch(function (data) {
      data.shows = data.shows.filter(function (show) { return show.id !== id; });
    });
    if (active && active.id === id) active = null;
  }

  function start(id) {
    var show = find(id);
    if (!show || !show.order.length) return null;
    active = { id: show.id, name: show.name, order: show.order.slice(), returnTo: D.currentSlide() };
    D.jump(active.order[0]);
    D.status("slides-delivery-shows-started", "Playing custom show { $name }", { name: show.name });
    if (D.console) D.console.render();
    return active;
  }
  function finish() {
    if (!active) return;
    var returnTo = active.returnTo;
    active = null;
    D.jump(returnTo);
    D.status("slides-delivery-shows-finished", "Custom show finished — back to the deck");
    if (D.console) D.console.render();
  }

  function position(current) {
    if (!active) return -1;
    return active.order.indexOf(Number(current) || 0);
  }
  function nextIndex(current) {
    if (!active) return D.slideCount() - 1;
    var at = position(current);
    if (at < 0) return active.order[0];
    return at < active.order.length - 1 ? active.order[at + 1] : active.returnTo;
  }
  function prevIndex(current) {
    if (!active) return 0;
    var at = position(current);
    if (at <= 0) return active.order[0];
    return active.order[at - 1];
  }
  function advance() {
    if (!active) return;
    var at = position(D.currentSlide());
    if (at < 0) { D.jump(active.order[0]); return; }
    if (at >= active.order.length - 1) { finish(); return; }
    D.jump(active.order[at + 1]);
    D.status("slides-delivery-shows-progress", "Slide { $i } of { $n } in this show",
      { i: at + 2, n: active.order.length });
  }

  function open() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-shows-title",
      titleFallback: "Custom shows",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        api.body.appendChild(D.el("p", "sd-hint", "slides-delivery-shows-hint",
          "Enter slide numbers in the order they should play, separated by commas."));
        var nameField = D.field("slides-delivery-name", "Name", "");
        api.body.appendChild(nameField);
        var orderField = D.field("slides-delivery-shows-order", "Play order", "",
          "slides-delivery-shows-order-placeholder", "e.g. 3, 1, 5");
        api.body.appendChild(orderField);
        var preview = D.el("div", "sd-show-preview");
        preview.id = "sd-show-preview";
        api.body.appendChild(preview);
        orderField.control.addEventListener("input", function () {
          var result = parseOrder(orderField.control.value, D.slideCount());
          preview.textContent = result.invalid
            ? D.tr("slides-delivery-shows-invalid", "Use slide numbers between 1 and { $n }, without repeats.",
              { n: D.slideCount() })
            : result.order.map(function (index) { return index + 1; }).join(" → ");
        });
        api.addButton("slides-delivery-shows-create", "Create show", function () {
          var name = String(nameField.control.value || "").trim();
          var result = parseOrder(orderField.control.value, D.slideCount());
          if (!name || result.invalid) {
            api.setStatus(result.invalid ? "slides-delivery-shows-invalid" : "slides-delivery-empty-name",
              result.invalid ? "Use slide numbers between 1 and { $n }, without repeats."
                : "Enter a name.", { n: D.slideCount() });
            return;
          }
          create(name, result.order);
          api.setStatus("slides-delivery-saved", "Saved locally");
          renderRows();
        }, "primary");

        var rows = D.el("div", "sd-show-list");
        api.body.appendChild(rows);
        function renderRows() {
          rows.textContent = "";
          var shows = list();
          if (!shows.length) {
            rows.appendChild(D.el("p", "sd-hint", "slides-delivery-shows-empty", "No custom shows yet."));
            return;
          }
          shows.forEach(function (show) {
            var row = D.el("div", "sd-show-row");
            var name = D.el("span", "sd-show-name");
            name.textContent = show.name;
            row.appendChild(name);
            var orderText = D.el("span", "sd-show-order");
            orderText.textContent = show.order.map(function (index) { return index + 1; }).join(" → ");
            row.appendChild(orderText);
            row.appendChild(D.button("slides-delivery-play", "Play", function () {
              start(show.id);
              api.setStatus("slides-delivery-shows-started", "Playing custom show { $name }",
                { name: show.name });
            }, "primary"));
            row.appendChild(D.button("slides-delivery-delete", "Delete", function () {
              remove(show.id);
              api.setStatus("slides-delivery-shows-deleted", "Custom show deleted");
              renderRows();
            }));
            rows.appendChild(row);
          });
        }
        renderRows();
        api.addButton("slides-delivery-close", "Close", function () { api.close(); });
      }
    });
    return dialog;
  }

  D.customShows = {
    list: list,
    find: find,
    isActive: isActive,
    active: function () { return active; },
    parseOrder: parseOrder,
    create: create,
    remove: remove,
    start: start,
    advance: advance,
    finish: finish,
    nextIndex: nextIndex,
    prevIndex: prevIndex,
    open: open
  };
})(window);
