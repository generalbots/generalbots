"use strict";
/* Slides advanced module: 06_comments — element-anchored threaded comments.
 *
 * Reaches sheet parity: users can comment on a specific canvas element
 * (right-click → "Comment…"), and the toolbar Comments button aggregates every
 * element comment into the deck-level panel via include_children.
 *
 * Element comments are addressed as resourceType "slides:element" with
 * resourceId "{presentationId}:{elementId}", so the shared /api/collab/*
 * backend groups them under the deck (resourceType "slides").
 */
(function () {
  var menu = null;

  function presentationId() {
    return (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current";
  }

  function elementFromTarget(t) {
    return t && t.closest ? t.closest(".sl-element") : null;
  }

  function openElementComments(el) {
    if (!window.GBCollabComments || !el || !el.dataset.id) return;
    var slide = (el.closest(".sl-canvas") && el.closest(".sl-canvas").dataset.slideId) || "0";
    window.GBCollabComments.open({
      resourceType: "slides:element",
      resourceId: presentationId() + ":" + el.dataset.id,
      title: "Comments on element (slide " + slide + ")",
    });
  }

  function closeMenu() {
    if (menu) { menu.remove(); menu = null; }
  }

  function openMenu(el, e) {
    closeMenu();
    menu = document.createElement("div");
    menu.className = "sl-comment-menu";
    menu.style.cssText =
      "position:fixed;z-index:70;background:#1e293b;border:1px solid #334155;border-radius:6px;" +
      "box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:180px;overflow:hidden;";
    var item = function (label, fn) {
      var b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.style.cssText =
        "display:block;width:100%;padding:8px 14px;background:none;border:none;color:#f8fafc;" +
        "text-align:left;font-size:13px;cursor:pointer;";
      b.addEventListener("mouseover", function () { b.style.background = "#334155"; });
      b.addEventListener("mouseout", function () { b.style.background = "none"; });
      b.addEventListener("click", function () { closeMenu(); fn(); });
      return b;
    };
    menu.appendChild(item("Comment\u2026", function () { openElementComments(el); }));
    document.body.appendChild(menu);
    var left = e.clientX;
    var top = e.clientY;
    if (left + 190 > window.innerWidth) left = window.innerWidth - 200;
    if (top + 40 > window.innerHeight) top = window.innerHeight - 50;
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }

  function onContextMenu(e) {
    var el = elementFromTarget(e.target);
    if (!el) return;
    e.preventDefault();
    openMenu(el, e);
  }

  document.addEventListener("contextmenu", onContextMenu, true);
  document.addEventListener("mousedown", function (e) {
    if (menu && !menu.contains(e.target)) closeMenu();
  }, true);
})();
