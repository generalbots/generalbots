"use strict";
/* Docs export menu. GBDocsExport (docs.html) existed with no caller, so a
   stored document could not be downloaded from the UI at all. The menu is
   built here to keep docs.html from growing another inline block. */

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

  function activeDocId() {
    var article = document.querySelector("#docs-content article[contenteditable]") ||
      document.querySelector("[id^=window-body-docs] #doc-content");
    return (article && article.dataset && article.dataset.docId) || "current";
  }

  function close() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function exportAs(format) {
    if (typeof window.GBDocsExport === "function") {
      window.GBDocsExport(format, activeDocId());
    }
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
    document.addEventListener("mousedown", function (event) {
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
