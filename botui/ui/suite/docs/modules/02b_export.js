"use strict";
/* Docs export menu. GBDocsExport (docs.html) existed with no caller, so a
   stored document could not be downloaded from the UI at all. The menu is
   built here to keep docs.html from growing another inline block.

   #2457 — export no longer degrades silently: when the document being exported
   actually contains structured elements the format cannot preserve (tables
   flattened, chart placeholders dropped), a "degraded" status is announced in
   a polite live region. The warning is tied to real detected elements only. */

(function () {
  var FORMATS = [
    ["DOCX", "docx"],
    ["ODT", "odt"],
    ["PDF", "pdf"],
    ["HTML", "html"],
    ["Markdown", "md"],
    ["Text", "txt"]
  ];
  var menu = null;
  var STATUS_ID = "docsExportStatus";
  var DEGRADED_FALLBACK = "Exported without tables/charts: source data was unavailable";

  function activeArticle() {
    return document.querySelector("#docs-content article[contenteditable]") ||
      document.querySelector("[id^=window-body-docs] #doc-content") ||
      document.querySelector("#doc-content");
  }

  function activeDocId() {
    var article = activeArticle();
    return (article && article.dataset && article.dataset.docId) || "current";
  }

  /* Real detection, not a guess: mirrors the exporter's drop condition — a
     table/chart element that is present but has no source data is skipped,
     never faked (see modules-export/04_docx_pptx_md.js noteLoss()). */
  function droppedStructured(article) {
    if (!article || typeof article.querySelectorAll !== "function") return 0;
    var dropped = 0;
    var tables = article.querySelectorAll("table");
    for (var i = 0; i < tables.length; i++) {
      var cells = tables[i].querySelectorAll ? tables[i].querySelectorAll("td, th") : [];
      if (!cells.length) dropped++;
    }
    var charts = article.querySelectorAll("[data-element-type='chart'], canvas[data-chart]");
    for (var j = 0; j < charts.length; j++) {
      var data = charts[j].getAttribute && charts[j].getAttribute("data-chart-data");
      if (!data) dropped++;
    }
    return dropped;
  }

  /* Idempotent live region. Reuses the docs status bar rather than adding a
     floating overlay, and keeps the ARIA contract role=status/aria-live=polite. */
  function statusRegion() {
    var bar = document.querySelector(".docs-status-bar");
    if (!bar) return null;
    var node = document.getElementById(STATUS_ID);
    if (!node) {
      node = document.createElement("span");
      node.id = STATUS_ID;
      node.setAttribute("role", "status");
      node.setAttribute("aria-live", "polite");
      node.setAttribute("aria-atomic", "true");
      node.style.color = "var(--palette-fbbf24, #fbbf24)";
      node.style.marginLeft = "12px";
      bar.insertBefore(node, bar.firstChild);
    }
    return node;
  }

  function clearStatus() {
    var node = document.getElementById(STATUS_ID);
    if (node) node.textContent = "";
  }

  function announceDegraded(dropped) {
    if (!dropped) return;
    var msg = typeof docsT === "function"
      ? docsT("docs-export-degraded", DEGRADED_FALLBACK, { count: dropped })
      : DEGRADED_FALLBACK;
    var region = statusRegion();
    if (region) region.textContent = msg;
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show(
        typeof docsT === "function" ? docsT("docs-export-title", "Export") : "Export",
        msg,
        "warn"
      );
    }
  }

  function close() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function exportAs(format) {
    if (typeof window.GBDocsExport !== "function") return;
    var dropped = droppedStructured(activeArticle());
    clearStatus();
    window.GBDocsExport(format, activeDocId());
    announceDegraded(dropped);
  }

  function open(anchor) {
    close();
    menu = document.createElement("div");
    menu.className = "dc-export-menu";
    menu.style.cssText =
      "position:absolute;z-index:80;background:#1e293b;border:1px solid #334155;" +
      "border-radius:6px;box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:150px;overflow:hidden;";
    FORMATS.forEach(function (entry) {
      var button = document.createElement("button");
      button.type = "button";
      button.textContent = entry[0];
      button.style.cssText =
        "display:block;width:100%;padding:8px 14px;background:none;border:none;" +
        "color:#f8fafc;text-align:left;font-size:13px;cursor:pointer;";
      button.addEventListener("mouseover", function () { button.style.background = "#334155"; });
      button.addEventListener("mouseout", function () { button.style.background = "none"; });
      button.addEventListener("click", function () { close(); exportAs(entry[1]); });
      menu.appendChild(button);
    });
    document.body.appendChild(menu);
    var rect = anchor.getBoundingClientRect();
    var left = rect.left;
    var top = rect.bottom + 4;
    if (left + 160 > window.innerWidth) left = window.innerWidth - 170;
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }

  function wire() {
    var right = document.querySelector(".toolbar-right");
    if (!right) {
      setTimeout(wire, 100);
      return;
    }
    if (document.getElementById("docsExportBtn")) return;
    var button = document.createElement("button");
    button.className = "btn-icon";
    button.id = "docsExportBtn";
    button.title = "Export";
    button.setAttribute("aria-label", "Export document");
    button.style.cssText = "font-size:14px;padding:4px 6px;";
    button.textContent = "⭳";
    button.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      open(button);
    });
    var share = document.getElementById("shareBtn");
    right.insertBefore(button, share && share.parentNode === right ? share : null);
    document.addEventListener("pointerdown", function (event) {
      if (menu && !menu.contains(event.target) && event.target !== button) close();
    }, true);
  }

  window.DocsExportMenu = { open: open, close: close };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", wire);
  } else {
    setTimeout(wire, 0);
  }
})();
