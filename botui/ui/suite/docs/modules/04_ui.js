"use strict";
/* docs UI — styles, page break, header/footer modal */

function injectEditorStyles() {
  if (document.getElementById("docs-editor-styles")) return;
  var style = document.createElement("style");
  style.id = "docs-editor-styles";
  style.textContent = [
    // UAT DOC-01 — the article is a flex item of `.docs-content` (a fixed-height
    // column scroller). With the default `flex-shrink:1` the browser clamped it
    // to `min-height` (1056px = one A4 page) instead of its content height, so
    // everything after page 1 was painted outside the white page: dark text
    // (#1e293b) on the dark app background (#0f172a), i.e. invisible. The page
    // must grow with its content and let the scroller scroll.
    "article.docs-doc-view{flex-shrink:0;background:#ffffff;color:#1e293b;padding:96px;width:816px;margin:32px auto;border:1px solid #e2e8f0;border-radius:4px;font-family:Georgia,serif;font-size:16px;line-height:1.7;min-height:1056px;outline:none;box-shadow:0 10px 30px rgba(0,0,0,0.25);box-sizing:border-box;}",
    "article.docs-doc-view h1{font-size:32px;font-weight:700;margin:0.8em 0 0.4em;color:#0f172a;}",
    "article.docs-doc-view h2{font-size:24px;font-weight:600;margin:0.7em 0 0.35em;color:#0f172a;}",
    "article.docs-doc-view h3{font-size:20px;font-weight:600;margin:0.6em 0 0.3em;color:#334155;}",
    "article.docs-doc-view p{margin:0.5em 0;}",
    "article.docs-doc-view blockquote{border-left:3px solid #3b82f6;padding-left:16px;color:#475569;font-style:italic;margin:0.8em 0;}",
    "article.docs-doc-view ul,article.docs-doc-view ol{margin:0.5em 0 0.5em 1.5em;}",
    "article.docs-doc-view li{margin:0.25em 0;}",
    "article.docs-doc-view a{color:#2563eb;text-decoration:underline;}",
    "article.docs-doc-view code{background:#f1f5f9;padding:2px 6px;border-radius:3px;font-family:'Courier New',monospace;font-size:0.9em;color:#b45309;}",
    "article.docs-doc-view pre{background:#f1f5f9;padding:12px 16px;border-radius:6px;overflow-x:auto;color:#334155;}",
    "article.docs-doc-view:focus{outline:2px solid #3b82f6;outline-offset:-2px;}",
    "article.docs-doc-view .docs-page-break{break-after:page;page-break-after:always;display:block;height:0;border:0;border-top:1px dashed #cbd5e1;margin:24px 0;}",
    "article.docs-doc-view .docs-header-zone{min-height:32px;padding:6px 0;border-bottom:1px dotted #e2e8f0;margin-bottom:18px;color:#64748b;font-size:12px;font-style:italic;}",
    "article.docs-doc-view .docs-footer-zone{min-height:32px;padding:6px 0;border-top:1px dotted #e2e8f0;margin-top:18px;color:#64748b;font-size:12px;font-style:italic;}",
    "@media print{article.docs-doc-view{box-shadow:none;border:none;margin:0;}.docs-page-break{break-after:page;page-break-after:always;}}",
    ".docs-content{background:repeating-linear-gradient(to bottom,#0f172a 0px,#0f172a 32px,#0b1220 32px,#0b1220 48px);}",
    // Appearance pass: the header and status bar used the light default while
    // the app body is dark, so Docs looked like two apps stacked together, and
    // the save status was a flex item that wrapped one word per line.
    // #2184 — classic two-row, document-oriented toolbar (title/menus on the
    // first row, character and paragraph formatting on the second), the layout
    // people know from 90s word processors.
    ".docs-toolbar{background:#111827;border-bottom:1px solid #1f2937;color:#e5e7eb;display:flex;flex-direction:column;padding:0;min-height:0;overflow:hidden;}",
    ".docs-toolbar-row{display:flex;align-items:center;gap:8px;padding:5px 10px;min-height:34px;width:100%;box-sizing:border-box;overflow-x:auto;scrollbar-width:none;}",
    ".docs-toolbar-row::-webkit-scrollbar{height:0;display:none;}",
    ".docs-toolbar-row-main{border-bottom:1px solid #1f2937;}",
    ".docs-toolbar-row-format{background:#0f172a;flex-wrap:wrap;row-gap:2px;overflow-x:visible;}",
    ".docs-toolbar .toolbar-left{display:flex;align-items:center;gap:8px;flex:0 0 auto;min-width:0;}",
    ".docs-toolbar .toolbar-center{display:flex;align-items:center;gap:1px;flex:1 1 auto;min-width:0;flex-wrap:nowrap;overflow-x:auto;scrollbar-width:none;}",
    ".docs-toolbar .toolbar-center::-webkit-scrollbar{height:0;display:none;}",
    ".docs-toolbar .toolbar-format{display:flex;align-items:center;gap:1px;flex-wrap:wrap;row-gap:2px;}",
    ".docs-toolbar .toolbar-right{display:flex;align-items:center;gap:4px;margin-left:auto;flex:0 0 auto;}",
    ".docs-toolbar .btn-icon{padding:4px;}",
    ".docs-toolbar .btn-icon svg{width:14px;height:14px;}",
    ".docs-toolbar .toolbar-left .docs-title-input{max-width:220px;}",
    "#docs-app .docs-content{overflow:auto;}",
    "#docs-app article.docs-doc-view{margin:24px auto;}",
    ".docs-toolbar .toolbar-group{display:flex;align-items:center;gap:2px;}",
    ".docs-toolbar .toolbar-divider{width:1px;height:18px;background:#1f2937;margin:0 4px;flex:0 0 auto;}",
    ".docs-toolbar .btn-icon{color:#cbd5e1;background:transparent;border:1px solid transparent;border-radius:6px;padding:5px;cursor:pointer;display:inline-flex;align-items:center;justify-content:center;}",
    ".docs-toolbar .btn-icon:hover{background:#1f2937;color:#f8fafc;}",
    ".docs-toolbar .btn-icon[aria-pressed=\"true\"]{background:#1d4ed8;color:#fff;border-color:#1d4ed8;}",
    ".docs-toolbar .btn-fmt{min-width:26px;height:24px;font:600 13px/1 system-ui,sans-serif;}",
    ".docs-toolbar .docs-select{background:#0f172a;color:#e5e7eb;border:1px solid #334155;border-radius:6px;padding:3px 6px;font-size:12px;height:26px;}",
    ".docs-toolbar .docs-select-size{width:82px;}",
    // The formatting row packs every character/paragraph control, so its
    // buttons are one step tighter than the command row (#2184).
    ".docs-toolbar-row-format .btn-icon{padding:1px;width:22px;min-width:22px;height:22px;}",
    ".docs-toolbar-row-format .btn-icon svg{width:13px;height:13px;}",
    ".docs-toolbar-row-format .toolbar-group{gap:0;}",
    ".docs-toolbar-row-format .toolbar-divider{height:16px;margin:0 2px;}",
    ".docs-toolbar-row-format .docs-color{padding:0 2px;height:22px;}",
    ".docs-toolbar .docs-color{position:relative;display:inline-flex;align-items:center;gap:3px;height:26px;padding:0 5px;border:1px solid transparent;border-radius:6px;color:#cbd5e1;cursor:pointer;font-size:13px;}",
    ".docs-toolbar .docs-color:hover{background:#1f2937;color:#f8fafc;}",
    ".docs-toolbar .docs-color input[type=color]{position:absolute;inset:0;opacity:0;cursor:pointer;border:0;padding:0;}",
    ".docs-title-input{background:transparent;border:1px solid transparent;color:#f8fafc;font-size:15px;font-weight:600;padding:4px 8px;border-radius:6px;min-width:120px;max-width:280px;}",
    ".docs-title-input:focus{border-color:#3b82f6;outline:none;background:#0f172a;}",
    ".save-status{white-space:nowrap;overflow:hidden;text-overflow:ellipsis;flex:0 0 auto;max-width:320px;}",
    ".docs-status-bar{background:#111827;border-top:1px solid #1f2937;color:#94a3b8;}",
    ".docs-pages{display:flex;flex-direction:column;align-items:center;padding:24px 0;}",
    ".dc-file-row:hover{background:#1f2937;}",
    // The suite theme exposes light surface tokens, and a later rule painted the
    // header from --sentient-bg-secondary (rgb(252,251,248)) on top of the dark
    // app body. Re-pin the tokens inside the app so every part agrees.
    "#docs-app{--sentient-bg-secondary:#111827;--sentient-bg-primary:#0f172a;--sentient-bg:#0f172a;--sentient-bg-elevated:#1e293b;--sentient-text-primary:#e5e7eb;--sentient-text-secondary:#94a3b8;--sentient-border:#1f2937;background:#0f172a;color:#e5e7eb;}",
    "#docs-app .docs-toolbar{background:#111827;border-bottom:1px solid #1f2937;color:#e5e7eb;}",
    "#docs-app .docs-status-bar{background:#111827;border-top:1px solid #1f2937;color:#94a3b8;}",
    "#docs-app .docs-sidebar{background:#0b1220;border-right:1px solid #1f2937;color:#cbd5e1;}",
    "#docs-app .docs-title-input{color:#f8fafc;}",
    "@media screen{body{background:#0f172a;}}"
  ].join("");
  document.head.appendChild(style);
}

function openPageLayoutModal() {
  var host = document.getElementById("docs-modal-container");
  if (!host) return;
  host.innerHTML = [
    "<div class=\"docs-modal\" style=\"position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;\">",
    "<div style=\"background:#1e293b;border:1px solid #334155;border-radius:12px;width:420px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;\">",
    "<h3 style=\"margin:0;font-size:16px;\">Page layout</h3>",
    "<label style=\"font-size:13px;\">Paper size<select id=\"pl-size\" style=\"margin-left:8px;padding:4px;\"><option value=\"a4\">A4</option><option value=\"letter\">Letter</option></select></label>",
    "<label style=\"font-size:13px;\">Orientation<select id=\"pl-orient\" style=\"margin-left:8px;padding:4px;\"><option value=\"portrait\">Portrait</option><option value=\"landscape\">Landscape</option></select></label>",
    "<label style=\"font-size:13px;\">Margins<input id=\"pl-margins\" type=\"number\" value=\"96\" style=\"margin-left:8px;padding:4px;width:70px;\"> px</label>",
    "<div style=\"display:flex;gap:8px;justify-content:flex-end;\"><button onclick=\"document.getElementById('docs-modal-container').innerHTML='';\" style=\"padding:6px 12px;border-radius:6px;border:1px solid #334155;background:#0f172a;color:#f8fafc;cursor:pointer;\">Close</button><button onclick=\"window.DocsEditor.applyPageLayout();\" style=\"padding:6px 12px;border-radius:6px;border:none;background:#3b82f6;color:#fff;cursor:pointer;\">Apply</button></div>",
    "</div></div>"
  ].join("");
}
/* Visible page separation: a continuous white sheet gives no clue where a
 * printed page ends, so dashed guides plus a page number are drawn at every
 * page height while the document grows (#2159). */
function renderPageGuides(article) {
  if (!article) return;
  var host = article.parentElement;
  if (!host) return;
  var pageH = parseInt(article.dataset.pageH || "1123", 10);
  if (!pageH || pageH < 200) pageH = 1123;
  if (getComputedStyle(host).position === "static") host.style.position = "relative";
  var guide = host.querySelector(".docs-page-guide");
  if (!guide) {
    guide = document.createElement("div");
    guide.className = "docs-page-guide";
    guide.contentEditable = "false";
    guide.setAttribute("aria-hidden", "true");
    guide.style.cssText = "position:absolute;inset:0;pointer-events:none;z-index:2;";
    host.appendChild(guide);
  }
  var hostRect = host.getBoundingClientRect();
  var rect = article.getBoundingClientRect();
  var left = rect.left - hostRect.left + (host.scrollLeft || 0);
  var top = rect.top - hostRect.top + (host.scrollTop || 0);
  var pages = Math.max(1, Math.ceil((article.scrollHeight || rect.height) / pageH));
  var width = Math.round(rect.width);
  var html = "";
  for (var i = 1; i < pages; i++) {
    var y = Math.round(top + i * pageH);
    // A dashed rule alone was easy to miss on a dark shell; the band plus the
    // zebra gap makes the boundary read as a real page edge (#2184).
    html += '<div class="docs-page-gap" style="position:absolute;left:' + Math.round(left) + 'px;width:' + width + 'px;top:' + y + 'px;height:0;border-top:2px dashed #94a3b8;box-shadow:0 6px 12px -8px rgba(15,23,42,.45);"></div>';
    html += '<span class="docs-page-num" style="position:absolute;left:' + Math.round(left + width / 2 - 34) + 'px;top:' + (y - 9) + 'px;font:11px/1.5 system-ui,sans-serif;letter-spacing:.06em;text-transform:uppercase;color:#475569;background:#f8fafc;padding:0 8px;border-radius:9px;border:1px solid #cbd5e1;box-shadow:0 1px 2px rgba(15,23,42,.08);">Page ' + (i + 1) + " of " + pages + "</span>";
  }
  guide.innerHTML = html;
}
window.renderPageGuides = renderPageGuides;

/* The page keeps its real A4 width, so a narrow window used to clip it and the
 * reader had to scroll sideways. Scale the whole page down (never up) until it
 * fits the content area, the way a word processor's "fit to width" does. */
function fitPageToWidth(article) {
  if (!article) return;
  var host = article.parentElement;
  if (!host) return;
  article.style.zoom = "1";
  var available = host.clientWidth - 48;
  var natural = article.offsetWidth;
  if (!natural || available <= 0) return;
  var scale = Math.min(1, available / natural);
  if (scale < 0.999) {
    article.style.zoom = String(Math.round(scale * 1000) / 1000);
  }
  renderPageGuides(article);
}
window.fitPageToWidth = fitPageToWidth;

function watchPageGuides() {
  if (window.__docsPageGuideBound) return;
  window.__docsPageGuideBound = true;
  var timer = null;
  var schedule = function () {
    clearTimeout(timer);
    timer = setTimeout(function () {
      var article = typeof getActiveArticle === "function" ? getActiveArticle() : null;
      if (article) {
        fitPageToWidth(article);
        renderPageGuides(article);
      }
    }, 150);
  };
  document.addEventListener("input", schedule, true);
  window.addEventListener("resize", schedule);
}

/* #2184 — content loaded programmatically (VM file, Drive file, ui-hint fill)
   never fires `input`, so the page separators were never drawn: an opened
   .docx looked like a single endless sheet. Re-fit the sheet and repaint the
   guides after every load, and again when its lazy images finish loading
   (their height changes the page count). */
function docsAfterContentLoad(article) {
  if (!article) return;
  if (!article.dataset.pageH) article.dataset.pageH = "1123";
  var refresh = function () {
    if (typeof fitPageToWidth === "function") fitPageToWidth(article);
    if (typeof renderPageGuides === "function") renderPageGuides(article);
    if (typeof updatePageCount === "function") updatePageCount();
  };
  refresh();
  var images = article.querySelectorAll("img");
  for (var i = 0; i < images.length; i++) {
    if (images[i].complete) continue;
    images[i].addEventListener("load", refresh);
    images[i].addEventListener("error", refresh);
  }
  setTimeout(refresh, 400);
}
window.docsAfterContentLoad = docsAfterContentLoad;

function applyPageLayout() {
  var size = document.getElementById("pl-size");
  var orient = document.getElementById("pl-orient");
  var margins = document.getElementById("pl-margins");
  var article = getActiveArticle();
  if (!article) return;
  // A4 is 210x297mm = 794x1123 px at 96dpi; Letter is 8.5x11in = 816x1056 px.
  // Both used to resolve to 816x1056, so "A4" drew a Letter page.
  var w = 794, h = 1123;
  if (size && size.value === "letter") { w = 816; h = 1056; }
  if (orient && orient.value === "landscape") { var t = w; w = h; h = t; }
  var m = margins ? parseInt(margins.value, 10) || 96 : 96;
  article.style.width = w + "px";
  article.style.minHeight = h + "px";
  article.style.padding = m + "px";
  // Inline styles survive the innerHTML save, so size/orientation/margins are
  // persisted with the document; the page height drives the visible guides.
  article.dataset.pageH = String(h);
  renderPageGuides(article);
  var host = document.getElementById("docs-modal-container");
  if (host) host.innerHTML = "";
  scheduleSave(article.dataset.docId, article.innerHTML);
}
window.applyPageLayout = applyPageLayout;
/* Caret position measured from the top of the article. Both rects are
   viewport-relative, so the difference is unaffected by the scroll pane or
   the window position. */
function caretOffsetInArticle(article, range) {
  if (!range) return article.scrollHeight;
  try {
    var caret = range.getBoundingClientRect();
    var box = article.getBoundingClientRect();
    var offset = caret.top - box.top;
    if (caret.height === 0 && caret.top === 0) return article.scrollHeight;
    return offset > 0 ? offset : 0;
  } catch (e) {
    return article.scrollHeight;
  }
}

function insertPageBreak() {
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  var pageH = parseInt(article.dataset.pageH, 10) || 1123;
  var sel = window.getSelection();
  var range = sel && sel.rangeCount > 0 ? sel.getRangeAt(0) : null;
  var br = document.createElement("div");
  br.className = "docs-page-break";
  br.contentEditable = "false";
  // Inline styles survive the innerHTML save, and the spacer height pushes the
  // following content onto the next page guide instead of leaving an invisible
  // zero-height div (the button appeared to do nothing). #2184
  var remaining = pageH - (caretOffsetInArticle(article, range) % pageH);
  br.style.cssText =
    "display:block;height:" + Math.max(32, Math.round(remaining)) + "px;" +
    "border-top:1px dashed #94a3b8;margin:0 0 8px;";
  // Typing after a break needs an editable line: a non-editable spacer alone
  // leaves the caret with nowhere to go.
  var next = document.createElement("p");
  next.innerHTML = "<br>";
  if (range) {
    range.collapse(false);
    range.insertNode(next);
    next.parentNode.insertBefore(br, next);
    var after = document.createRange();
    after.setStart(next, 0);
    after.collapse(true);
    sel.removeAllRanges();
    sel.addRange(after);
  } else {
    article.appendChild(br);
    article.appendChild(next);
  }
  scheduleSave(article.dataset.docId, article.innerHTML);
  updatePageCount();
  if (typeof docsAfterContentLoad === "function") docsAfterContentLoad(article);
}
window.insertPageBreak = insertPageBreak;

/* #2184 — Ctrl+Enter (Cmd+Enter on macOS) starts a new page at the caret.
   Scoped to the Docs window so the chat composer keeps its own Enter keys. */
function bindPageBreakShortcut(host) {
  var scope = host || document;
  if (scope.__gbPageBreakBound) return;
  scope.__gbPageBreakBound = true;
  scope.addEventListener(
    "keydown",
    function (e) {
      if (!(e.ctrlKey || e.metaKey) || e.key !== "Enter") return;
      var target = e.target || {};
      var inDocs = target.isContentEditable === true ||
        (target.closest && target.closest("#docs-content, .docs-app"));
      if (!inDocs) return;
      e.preventDefault();
      e.stopPropagation();
      insertPageBreak();
    },
    true,
  );
}
window.bindPageBreakShortcut = bindPageBreakShortcut;

/* #2184 — classic two-row toolbar. Every button in the formatting row carries
   `data-cmd` and drives the editor through execCommand, so the controls act on
   the document exactly like a 90s word processor's toolbar. */
function docsFormat(command, value) {
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  try {
    document.execCommand("styleWithCSS", false, true);
    document.execCommand(command, false, value === undefined ? null : value);
  } catch (e) {
    return;
  }
  article.dispatchEvent(new Event("input", { bubbles: true }));
  docsSyncFormatState();
}

function docsApplyBlockStyle(tag) {
  if (!tag) return;
  docsFormat("formatBlock", "<" + tag + ">");
}

function docsApplyFontSize(value) {
  if (!value) return;
  docsFormat("fontSize", String(value));
}

/* Highlights the controls that match the current selection (bold button lit
   while the caret is inside bold text, and so on). */
function docsSyncFormatState() {
  var host = document.getElementById("window-body-docs") || document;
  var buttons = host.querySelectorAll("[data-cmd]");
  for (var i = 0; i < buttons.length; i++) {
    var command = buttons[i].getAttribute("data-cmd");
    var active = false;
    try {
      active = document.queryCommandState(command);
    } catch (e) {
      active = false;
    }
    buttons[i].setAttribute("aria-pressed", active ? "true" : "false");
  }
}

function bindFormatToolbar(host) {
  var scope = host || document;
  if (!scope.__gbFormatBound) {
    scope.__gbFormatBound = true;
    scope.addEventListener("click", function (event) {
      var button = event.target.closest ? event.target.closest("[data-cmd]") : null;
      if (!button || !scope.contains(button)) return;
      event.preventDefault();
      docsFormat(button.getAttribute("data-cmd"));
    });
    var color = scope.querySelector("#fmtColor");
    if (color) color.addEventListener("input", function () { docsFormat("foreColor", color.value); });
    var highlight = scope.querySelector("#fmtHighlight");
    if (highlight) {
      highlight.addEventListener("input", function () {
        docsFormat("hiliteColor", highlight.value);
      });
    }
    scope.addEventListener("keyup", docsSyncFormatState);
    scope.addEventListener("mouseup", docsSyncFormatState);
  }
  document.addEventListener("selectionchange", function () {
    var article = getActiveArticle();
    if (!article) return;
    var selection = window.getSelection();
    if (selection && selection.rangeCount > 0 && article.contains(selection.anchorNode)) {
      docsSyncFormatState();
    }
  });
}
window.bindFormatToolbar = bindFormatToolbar;
window.docsFormat = docsFormat;
window.docsSyncFormatState = docsSyncFormatState;
window.docsApplyBlockStyle = docsApplyBlockStyle;
window.docsApplyFontSize = docsApplyFontSize;

function insertHeaderFooterZone(kind) {
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  var zone = document.createElement("div");
  zone.className = kind === "header" ? "docs-header-zone" : "docs-footer-zone";
  zone.contentEditable = "true";
  zone.dataset.zoneKind = kind;
  zone.setAttribute("data-placeholder", kind === "header" ? "Cabeçalho — clique para editar" : "Rodapé — clique para editar");
  if (!zone.textContent) zone.textContent = zone.getAttribute("data-placeholder");
  if (kind === "header") article.insertBefore(zone, article.firstChild);
  else article.appendChild(zone);
  scheduleSave(article.dataset.docId, article.innerHTML);
}

function openHeaderFooterModal() {
  var host = document.getElementById("docs-modal-container");
  if (!host) return;
  host.innerHTML = [
    "<div class=\"docs-modal\" id=\"docs-hf-modal\" style=\"position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;\">",
    "<div style=\"background:#1e293b;border:1px solid #334155;border-radius:12px;width:520px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;\">",
    "<h3 style=\"margin:0;font-size:16px;\">Cabeçalho e Rodapé</h3>",
    "<label style=\"font-size:12px;color:#94a3b8;\">Cabeçalho (aparece no topo de cada página)<input id=\"hf-header\" type=\"text\" placeholder=\"Cabeçalho...\" style=\"width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;\"/></label>",
    "<label style=\"font-size:12px;color:#94a3b8;\">Rodapé (aparece no final de cada página)<input id=\"hf-footer\" type=\"text\" placeholder=\"Rodapé...\" style=\"width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;\"/></label>",
    "<div style=\"display:flex;gap:8px;justify-content:flex-end;\">",
    "<button id=\"hf-cancel\" type=\"button\" style=\"background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;\">Cancelar</button>",
    "<button id=\"hf-apply\" type=\"button\" style=\"background:#3b82f6;color:white;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;\">Aplicar</button>",
    "</div></div></div>"
  ].join("");
  document.getElementById("hf-cancel").onclick = function () { host.innerHTML = ""; };
  document.getElementById("hf-apply").onclick = function () {
    var header = document.getElementById("hf-header").value || "";
    var footer = document.getElementById("hf-footer").value || "";
    var article = getActiveArticle();
    if (!article) { host.innerHTML = ""; return; }
    var h = article.querySelector(".docs-header-zone");
    if (!h) { h = document.createElement("div"); h.className = "docs-header-zone"; h.contentEditable = "true"; article.insertBefore(h, article.firstChild); }
    h.textContent = header;
    var f = article.querySelector(".docs-footer-zone");
    if (!f) { f = document.createElement("div"); f.className = "docs-footer-zone"; f.contentEditable = "true"; article.appendChild(f); }
    f.textContent = footer;
    scheduleSave(article.dataset.docId, article.innerHTML);
    host.innerHTML = "";
  };
}

// AI endpoints answer JSON ({response}); htmx used to swap the object into
// the panel/modal as text. Render the answer field instead.
if (!window.__gbDocsAiHook) {
  window.__gbDocsAiHook = true;
  document.addEventListener("htmx:afterRequest", function (event) {
    var detail = event.detail || {};
    var path = (detail.pathInfo && detail.pathInfo.requestPath) || "";
    if (path.indexOf("/api/docs/ai/") !== 0 || !detail.successful) return;
    var text = "";
    try {
      var data = JSON.parse(detail.xhr.responseText);
      // A batch answer that could not run returns the selection unchanged
      // plus an explicit error. Showing `content` alone echoed the user's own
      // text and hid the reason, so surface the error first.
      text = data.result === "unavailable" && data.error
        ? data.error
        : (data.content || data.response || data.text || data.result || "");
    } catch (error) {
      text = detail.xhr.responseText;
    }
    var target = document.getElementById("ai-result-panel") ||
      document.getElementById("ai-result-modal");
    if (target) target.textContent = text;
  });
}
