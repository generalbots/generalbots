"use strict";
/* Slides review tools bootstrap (#2433): injects one accessible Review menu
 * into the existing toolbar and re-applies local-only markers after a refresh. */
(function () {
  var O = window.GBSlidesOutline;
  var MENU_ID = "slides-review-menu";

  var ITEMS = [
    { key: "slides-outline-menu-outline", fallback: "Outline", run: function () { window.GBSlidesOutlineView.open(); } },
    { key: "slides-outline-fr-title", fallback: "Find and replace", run: function () { window.GBSlidesFindReplace.open(); } },
    { key: "slides-outline-sections-title", fallback: "Sections", run: function () { window.GBSlidesSections.open(); } },
    { key: "slides-outline-bulk-title", fallback: "Bulk slide actions", run: function () { window.GBSlidesBulk.open(); } },
    { key: "slides-outline-notes-title", fallback: "Notes workspace", run: function () { window.GBSlidesNotes.open(); } },
    { key: "slides-outline-a11y-title", fallback: "Accessibility", run: function () { window.GBSlidesAccessibility.open(); } },
    { key: "slides-outline-props-title-dialog", fallback: "Deck properties", run: function () { window.GBSlidesProperties.open(); } }
  ];

  function closeMenu(menu) {
    menu.setAttribute("hidden", "");
    var trigger = document.getElementById("slides-review-btn");
    if (trigger) trigger.setAttribute("aria-expanded", "false");
  }
  function openMenu(menu) {
    menu.removeAttribute("hidden");
    var trigger = document.getElementById("slides-review-btn");
    if (trigger) trigger.setAttribute("aria-expanded", "true");
    var first = menu.querySelector("button");
    if (first) first.focus();
  }

  function build() {
    var scope = O.scope();
    var toolbar = O.$(".slides-toolbar-row-main .toolbar-right", scope);
    if (!toolbar || O.$("#" + MENU_ID, scope)) return;
    var wrap = O.el("div", "slides-review-wrap");
    var trigger = O.el("button", "btn-icon slides-review-btn",
      O.t("slides-outline-menu", "Review"), "slides-outline-menu");
    trigger.type = "button";
    trigger.id = "slides-review-btn";
    trigger.setAttribute("aria-haspopup", "true");
    trigger.setAttribute("aria-expanded", "false");
    trigger.setAttribute("aria-label", O.t("slides-outline-menu-aria", "Open review tools"));
    trigger.setAttribute("data-i18n-aria-label", "slides-outline-menu-aria");
    var menu = O.el("div", "slides-review-menu");
    menu.id = MENU_ID;
    menu.setAttribute("role", "menu");
    menu.setAttribute("hidden", "");
    ITEMS.forEach(function (spec) {
      var item = O.el("button", "slides-review-item", O.t(spec.key, spec.fallback), spec.key);
      item.type = "button";
      item.setAttribute("role", "menuitem");
      item.addEventListener("click", function () {
        closeMenu(menu);
        O.closePanels();
        spec.run();
      });
      menu.appendChild(item);
    });
    trigger.addEventListener("click", function (event) {
      event.stopPropagation();
      if (menu.hasAttribute("hidden")) openMenu(menu); else closeMenu(menu);
    });
    menu.addEventListener("keydown", function (event) {
      var items = O.$$("button", menu);
      var index = items.indexOf(document.activeElement);
      if (event.key === "Escape") { event.preventDefault(); closeMenu(menu); trigger.focus(); }
      else if (event.key === "ArrowDown") { event.preventDefault(); items[(index + 1) % items.length].focus(); }
      else if (event.key === "ArrowUp") { event.preventDefault(); items[(index - 1 + items.length) % items.length].focus(); }
    });
    document.addEventListener("click", function (event) {
      if (!menu.hasAttribute("hidden") && !wrap.contains(event.target)) closeMenu(menu);
    });
    wrap.appendChild(trigger);
    wrap.appendChild(menu);
    toolbar.insertBefore(wrap, toolbar.firstChild);
  }

  function restore() {
    if (window.GBSlidesSections) window.GBSlidesSections.annotate();
    if (window.GBSlidesBulk) window.GBSlidesBulk.restore();
  }

  window.GBSlidesReview = {
    outline: function () { return window.GBSlidesOutlineView.open(); },
    find: function () { return window.GBSlidesFindReplace.open(); },
    sections: function () { return window.GBSlidesSections.open(); },
    bulk: function () { return window.GBSlidesBulk.open(); },
    notes: function () { return window.GBSlidesNotes.open(); },
    accessibility: function () { return window.GBSlidesAccessibility.open(); },
    properties: function () { return window.GBSlidesProperties.open(); }
  };

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", build);
  else build();
  document.addEventListener("htmx:afterSwap", function () { window.setTimeout(restore, 0); });
})();
