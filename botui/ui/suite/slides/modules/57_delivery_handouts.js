"use strict";
/* Slides delivery mode (#2432) — handout layout preview. 1/2/4/6 slides per
 * page with a page-count preview. This is a layout preview only: no PDF export,
 * no note lines on a printed page and no server write are produced here. */
(function (window) {
  var D = window.GBSlidesDelivery;
  var ALLOWED = [1, 2, 4, 6];

  function perPage() {
    var value = Number(D.load().handouts.perPage);
    return ALLOWED.indexOf(value) !== -1 ? value : 4;
  }
  function setPerPage(value) {
    var number = Number(value);
    if (ALLOWED.indexOf(number) === -1) return perPage();
    D.patch(function (data) { data.handouts.perPage = number; });
    return number;
  }
  function pageCount(slideCount, per) {
    var count = Math.max(0, Number(slideCount) || 0);
    var size = Math.max(1, Number(per) || 1);
    return Math.ceil(count / size);
  }

  function open() {
    var dialog = D.openDialog({
      titleKey: "slides-delivery-handouts-title",
      titleFallback: "Handout layout preview",
      onBuild: function (api) {
        api.body.appendChild(D.localNote());
        api.body.appendChild(D.el("p", "sd-hint", "slides-delivery-handouts-export-note",
          "PDF export is not available in this local preview."));

        var controls = D.el("div", "sd-handout-controls");
        var label = D.field("slides-delivery-handouts-per-page", "Slides per page", perPage());
        label.control.type = "select";
        var select = document.createElement("select");
        D.labelled(select, "data-i18n-aria-label", "slides-delivery-handouts-per-page", "Slides per page");
        ALLOWED.forEach(function (value) {
          var option = document.createElement("option");
          option.value = String(value);
          option.textContent = String(value);
          select.appendChild(option);
        });
        select.value = String(perPage());
        label.control.parentNode.replaceChild(select, label.control);
        label.control = select;
        controls.appendChild(label);
        api.body.appendChild(controls);

        var count = D.el("div", "sd-handout-count");
        count.id = "sd-handout-count";
        count.setAttribute("role", "status");
        count.setAttribute("aria-live", "polite");
        api.body.appendChild(count);

        var pages = D.el("div", "sd-handout-pages");
        pages.id = "sd-handout-pages";
        api.body.appendChild(pages);

        function render() {
          var size = perPage();
          var total = D.slideCount();
          var pagesNeeded = pageCount(total, size);
          count.textContent = D.tr("slides-delivery-handouts-pages",
            "{ $n } page(s) for { $s } slides", { n: pagesNeeded, s: total });
          pages.textContent = "";
          for (var page = 0; page < pagesNeeded; page += 1) {
            var sheet = D.el("section", "sd-handout-page sd-handout-" + size);
            var heading = D.el("h4", "sd-handout-page-label");
            heading.textContent = D.tr("slides-delivery-handouts-page", "Page { $n }", { n: page + 1 });
            sheet.appendChild(heading);
            for (var cell = 0; cell < size; cell += 1) {
              var index = page * size + cell;
              var slot = D.el("div", "sd-handout-cell");
              if (index < total) {
                var cellLabel = D.el("span", "sd-handout-cell-label");
                cellLabel.textContent = D.tr("slides-delivery-slide", "Slide { $n }", { n: index + 1 });
                slot.appendChild(cellLabel);
              } else {
                slot.className += " sd-handout-cell-empty";
              }
              sheet.appendChild(slot);
            }
            pages.appendChild(sheet);
          }
        }
        select.addEventListener("change", function () {
          setPerPage(select.value);
          render();
        });
        render();
        api.addButton("slides-delivery-close", "Close", function () { api.close(); });
      }
    });
    return dialog;
  }

  D.handouts = {
    ALLOWED: ALLOWED,
    perPage: perPage,
    setPerPage: setPerPage,
    pageCount: pageCount,
    open: open
  };
})(window);
