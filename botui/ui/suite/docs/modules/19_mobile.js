"use strict";
/* #2515 — Docs touch editing helpers. The editor is contenteditable, so the
   browser already owns selection handles; this module only supplies the
   drawer, keeps the caret visible while the keyboard is open and makes a tap
   anywhere on the page place the caret even when the article was not focused. */

(function () {
  var app = document.getElementById("docs-app");
  if (!app) return;

  var main = app.querySelector(".docs-main");
  var toggle = document.getElementById("docsSidebarToggle");
  var close = document.getElementById("docsSidebarClose");
  var backdrop = document.getElementById("docsSidebarBackdrop");

  function setDrawer(open) {
    if (!main) return;
    main.classList.toggle("sidebar-open", open);
    if (toggle) toggle.setAttribute("aria-expanded", open ? "true" : "false");
  }
  if (toggle) toggle.addEventListener("click", function () {
    setDrawer(!main.classList.contains("sidebar-open"));
  });
  if (close) close.addEventListener("click", function () { setDrawer(false); });
  if (backdrop) backdrop.addEventListener("click", function () { setDrawer(false); });
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && main && main.classList.contains("sidebar-open")) setDrawer(false);
  });

  var isTouch = (function () {
    try {
      return !!(window.matchMedia && (window.matchMedia("(pointer: coarse)").matches ||
        window.matchMedia("(max-width: 760px)").matches));
    } catch (e) { return false; }
  })();
  if (!isTouch) return;

  var page = app.querySelector("article.docs-doc-view");
  var scroller = document.getElementById("docs-content");

  // A tap on the paper (not just on a text node) focuses the article so the
  // caret appears on the tapped line instead of leaving the keyboard closed.
  app.addEventListener("pointerup", function (e) {
    if (e.pointerType !== "touch" || !page || !page.contains(e.target)) return;
    if (window.getSelection && String(window.getSelection())) return;
    if (document.activeElement === page) return;
    page.focus({ preventScroll: true });
  });

  function revealCaret() {
    if (!page || document.activeElement !== page) return;
    var sel = window.getSelection && window.getSelection();
    var node = sel && sel.rangeCount ? sel.getRangeAt(0).getClientRects()[0] : null;
    if (!node || !scroller) return;
    var box = scroller.getBoundingClientRect();
    var keyboard = parseFloat(getComputedStyle(document.documentElement)
      .getPropertyValue("--gb-vv-bottom")) || 0;
    var safeBottom = box.bottom - keyboard - 24;
    if (node.bottom > safeBottom) {
      scroller.scrollTop += node.bottom - safeBottom;
    } else if (node.top < box.top + 8) {
      scroller.scrollTop -= box.top + 8 - node.top;
    }
  }

  var vv = window.visualViewport;
  if (vv) {
    vv.addEventListener("resize", function () { setTimeout(revealCaret, 60); });
    vv.addEventListener("scroll", function () { setTimeout(revealCaret, 60); });
  }
  document.addEventListener("selectionchange", function () {
    if (document.activeElement === page) setTimeout(revealCaret, 30);
  });
  window.addEventListener("orientationchange", function () {
    setTimeout(revealCaret, 300);
  });
})();
