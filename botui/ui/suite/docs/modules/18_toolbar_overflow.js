"use strict";
/* #2290 — toolbar overflow affordance for Docs.
 * Trailing chevron appears when a toolbar row overflows and exposes hidden
 * commands in a popover menu (mouse reachable in <=2 interactions). */
(function () {
  function isFullyVisible(container, el) {
    var c = container.getBoundingClientRect(), r = el.getBoundingClientRect();
    return r.left >= c.left - 1 && r.right <= c.right + 1;
  }
  function labelFor(el) {
    return el.getAttribute("aria-label") || el.getAttribute("title") || (el.textContent || "").replace(/\s+/g, " ").trim() || "Command";
  }
  function actionableElements(container) {
    return Array.prototype.slice.call(container.querySelectorAll("button, select, input, .docs-color, [role='button']"))
      .filter(function (el) { return !el.classList.contains("toolbar-overflow-btn"); });
  }
  function closeAllMenus(except) {
    document.querySelectorAll(".toolbar-overflow-menu").forEach(function (m) {
      if (m !== except) { m.hidden = true; var b = m.__tbBtn; if (b) b.setAttribute("aria-expanded", "false"); }
    });
  }
  function buildMenu(container, menu) {
    menu.innerHTML = "";
    var items = actionableElements(container).filter(function (el) { return !isFullyVisible(container, el); });
    if (!items.length) {
      var empty = document.createElement("div"); empty.className = "tb-overflow-item"; empty.textContent = "No hidden commands"; empty.setAttribute("aria-disabled", "true");
      menu.appendChild(empty); return;
    }
    items.forEach(function (orig) {
      var tag = (orig.tagName || "").toLowerCase();
      var entry = document.createElement("button"); entry.type = "button"; entry.className = "tb-overflow-item"; entry.textContent = labelFor(orig);
      entry.addEventListener("click", function (ev) {
        ev.preventDefault(); ev.stopPropagation(); closeAllMenus();
        try { orig.scrollIntoView({ block: "nearest", inline: "center" }); } catch (e) {}
        if (tag === "button" || orig.getAttribute("role") === "button") orig.click();
        else if (tag === "select" || tag === "input" || tag === "label") { try { orig.focus(); } catch (e) {} if (tag !== "select") { try { orig.click(); } catch (e) {} } }
        else orig.click();
      });
      menu.appendChild(entry);
    });
  }
  function ensureOverflow(container) {
    if (!container || container.__tbOverflowInit) return;
    container.__tbOverflowInit = true;
    var btn = document.createElement("button");
    btn.type = "button"; btn.className = "toolbar-overflow-btn";
    btn.setAttribute("aria-label", "More commands"); btn.setAttribute("title", "More commands");
    btn.setAttribute("aria-haspopup", "menu"); btn.setAttribute("aria-expanded", "false");
    btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><polyline points="9 6 15 12 9 18"></polyline><polyline points="4 6 10 12 4 18"></polyline></svg>';
    var menu = document.createElement("div"); menu.className = "toolbar-overflow-menu"; menu.hidden = true; menu.setAttribute("role", "menu"); menu.__tbBtn = btn;
    document.body.appendChild(menu); container.appendChild(btn);
    function update() {
      var overflow = container.scrollWidth > container.clientWidth + 1;
      btn.classList.toggle("visible", overflow);
      if (!overflow) { menu.hidden = true; btn.setAttribute("aria-expanded", "false"); }
    }
    btn.addEventListener("click", function (ev) {
      ev.preventDefault(); ev.stopPropagation();
      var open = !menu.hidden; closeAllMenus(menu);
      if (open) { menu.hidden = true; btn.setAttribute("aria-expanded", "false"); return; }
      buildMenu(container, menu);
      var r = btn.getBoundingClientRect();
      menu.style.top = Math.min(r.bottom + 4, window.innerHeight - 40) + "px";
      menu.style.left = Math.min(Math.max(8, r.right - 260), window.innerWidth - 268) + "px";
      menu.hidden = false; btn.setAttribute("aria-expanded", "true");
    });
    container.addEventListener("scroll", function () { if (!menu.hidden) buildMenu(container, menu); });
    window.addEventListener("resize", update);
    if (typeof ResizeObserver !== "undefined") new ResizeObserver(update).observe(container);
    document.addEventListener("click", function () { closeAllMenus(); });
    menu.addEventListener("click", function (ev) { ev.stopPropagation(); });
    update(); setTimeout(update, 50); setTimeout(update, 300);
  }
  function init() {
    Array.prototype.forEach.call(document.querySelectorAll(".docs-toolbar-row, .docs-toolbar .toolbar-center"), ensureOverflow);
  }
  window.docsInitToolbarOverflow = init;
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init); else init();
  setTimeout(init, 300);
})();
