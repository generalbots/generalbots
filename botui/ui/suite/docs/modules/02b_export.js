"use strict";
/* Docs export menu. GBDocsExport (docs-export-helpers.js) now receives an
   AbortSignal and lifecycle hooks: a fast export settles before the delay so
   no UI is painted; a slow one shows progress + Cancel in a polite region.
   The degraded warning is tied to real detected elements only. */
(function () {
  var FORMATS = [["DOCX", "docx"], ["ODT", "odt"], ["PDF", "pdf"], ["HTML", "html"], ["Markdown", "md"], ["Text", "txt"]];
  var menu = null, STATUS_ID = "docsExportStatus";
  var DEGRADED_FALLBACK = "Exported without tables/charts: source data was unavailable";
  var PROGRESS_DELAY_MS = 200, exportController = null, progressTimer = null;

  function activeArticle() {
    return document.querySelector("#docs-content article[contenteditable]") ||
      document.querySelector("[id^=window-body-docs] #doc-content") || document.querySelector("#doc-content");
  }
  function activeDocId() {
    var article = activeArticle();
    return (article && article.dataset && article.dataset.docId) || "current";
  }
  /* Mirrors the exporter's drop condition: an empty table or a chart with no
     source data is skipped — real detection, never a guess. */
  function droppedStructured(article) {
    if (!article || typeof article.querySelectorAll !== "function") return 0;
    var dropped = 0, tables = article.querySelectorAll("table"), i, cells;
    for (i = 0; i < tables.length; i++) {
      cells = tables[i].querySelectorAll ? tables[i].querySelectorAll("td, th") : [];
      if (!cells.length) dropped++;
    }
    var charts = article.querySelectorAll("[data-element-type='chart'], canvas[data-chart]");
    for (i = 0; i < charts.length; i++) {
      if (!charts[i].getAttribute || !charts[i].getAttribute("data-chart-data")) dropped++;
    }
    return dropped;
  }
  /* One polite live region (role=status / aria-live=polite), no overlay. The
     text lives in a child span so clearing status never removes Cancel. */
  function statusRegion() {
    var bar = document.querySelector(".docs-status-bar") || document.body;
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
      ? docsT("docs-export-degraded", DEGRADED_FALLBACK, { count: dropped }) : DEGRADED_FALLBACK;
    var region = statusRegion();
    if (region) region.textContent = msg;
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show(typeof docsT === "function" ? docsT("docs-export-title", "Export") : "Export", msg, "warn");
    }
  }
  /* #2459 — progress text and Cancel share the live region, so a slow export
     is visible and interruptible without a backend job API. */
  function progressControls() {
    var region = statusRegion();
    if (!region) return null;
    var cancel = region.querySelector(".docs-export-cancel");
    if (!cancel) {
      cancel = document.createElement("button");
      cancel.type = "button";
      cancel.className = "docs-export-cancel";
      cancel.appendChild(document.createTextNode(docsT("action-cancel", "Cancel")));
      cancel.addEventListener("click", cancelExport);
      region.appendChild(cancel);
    }
    return cancel;
  }
  function showProgress() {
    progressTimer = null;
    var region = statusRegion();
    if (!region) return;
    var label = region.querySelector(".docs-export-progress-text");
    if (!label) {
      label = document.createElement("span");
      label.className = "docs-export-progress-text";
      region.insertBefore(label, region.firstChild);
    }
    label.appendChild(document.createTextNode(docsT("status-in-progress", "In Progress")));
    progressControls();
  }
  /* Idempotent idle restore: drops the delayed paint (a fast export shows
     nothing), the controller and the Cancel button. */
  function clearProgress() {
    if (progressTimer !== null) { clearTimeout(progressTimer); progressTimer = null; }
    exportController = null;
    clearStatus();
  }
  function cancelExport() {
    var controller = exportController;
    exportController = null;
    if (controller) controller.abort();
    clearProgress();
  }
  function close() {
    if (menu) { menu.remove(); menu = null; }
  }
  function exportAs(format) {
    if (typeof window.GBDocsExport !== "function") return;
    var dropped = droppedStructured(activeArticle());
    cancelExport();
    clearStatus();
    var AbortCtor = typeof window.AbortController === "function" ? window.AbortController
      : (typeof AbortController === "function" ? AbortController : null);
    var controller = AbortCtor ? new AbortCtor() : null;
    exportController = controller;
    window.GBDocsExport(format, activeDocId(), {
      signal: controller ? controller.signal : null,
      onStart: function () {
        if (progressTimer !== null) clearTimeout(progressTimer);
        progressTimer = setTimeout(showProgress, PROGRESS_DELAY_MS);
      },
      onSettle: function (outcome) { clearProgress(); if (outcome !== "cancelled") announceDegraded(dropped); }
    });
  }
  function open(anchor) {
    close();
    menu = document.createElement("div");
    menu.className = "dc-export-menu";
    menu.style.cssText = "position:absolute;z-index:80;background:#1e293b;border:1px solid #334155;border-radius:6px;box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:150px;overflow:hidden;";
    FORMATS.forEach(function (entry) {
      var button = document.createElement("button");
      button.type = "button";
      button.textContent = entry[0];
      button.style.cssText = "display:block;width:100%;padding:8px 14px;background:none;border:none;color:#f8fafc;text-align:left;font-size:13px;cursor:pointer;";
      button.addEventListener("mouseover", function () { button.style.background = "#334155"; });
      button.addEventListener("mouseout", function () { button.style.background = "none"; });
      button.addEventListener("click", function () { close(); exportAs(entry[1]); });
      menu.appendChild(button);
    });
    document.body.appendChild(menu);
    var rect = anchor.getBoundingClientRect(), left = rect.left, top = rect.bottom + 4;
    if (left + 160 > window.innerWidth) left = window.innerWidth - 170;
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }
  function wire() {
    var right = document.querySelector(".toolbar-right");
    if (!right) { setTimeout(wire, 100); return; }
    if (document.getElementById("docsExportBtn")) return;
    // #2448 criterion 6: the Export action declares the catalog verb it
    // performs (`docs.export`, registered in botserver/src/apps/commands).
    var holder = document.createElement("div");
    holder.innerHTML = '<button type="button" class="btn-icon" id="docsExportBtn" data-verb="docs.export" title="Export" aria-label="Export document" style="font-size:14px;padding:4px 6px;">⭳</button>';
    var button = holder.firstChild;
    button.addEventListener("click", function (event) {
      event.preventDefault(); event.stopPropagation(); open(button);
    });
    var share = document.getElementById("shareBtn");
    right.insertBefore(button, share && share.parentNode === right ? share : null);
    document.addEventListener("pointerdown", function (event) {
      if (menu && !menu.contains(event.target) && event.target !== button) close();
    }, true);
  }
  window.DocsExportMenu = { open: open, close: close };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", wire);
  else setTimeout(wire, 0);
})();
