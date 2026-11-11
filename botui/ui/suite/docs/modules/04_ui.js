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
  var html = "";
  for (var i = 1; i < pages; i++) {
    var y = Math.round(top + i * pageH);
    html += '<div class="docs-page-gap" style="position:absolute;left:' + Math.round(left) + 'px;width:' + Math.round(rect.width) + 'px;top:' + y + 'px;border-top:1px dashed #94a3b8;"></div>';
    html += '<span class="docs-page-num" style="position:absolute;left:' + Math.round(left + rect.width - 54) + 'px;top:' + (y + 4) + 'px;font-size:11px;color:#64748b;background:#f8fafc;padding:0 6px;border-radius:8px;border:1px solid #e2e8f0;">Page ' + (i + 1) + "</span>";
  }
  guide.innerHTML = html;
}
window.renderPageGuides = renderPageGuides;

function watchPageGuides() {
  if (window.__docsPageGuideBound) return;
  window.__docsPageGuideBound = true;
  var timer = null;
  var schedule = function () {
    clearTimeout(timer);
    timer = setTimeout(function () {
      var article = typeof getActiveArticle === "function" ? getActiveArticle() : null;
      if (article) renderPageGuides(article);
    }, 150);
  };
  document.addEventListener("input", schedule, true);
  window.addEventListener("resize", schedule);
}

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
function insertPageBreak() {
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  var br = document.createElement("div");
  br.className = "docs-page-break";
  br.contentEditable = "false";
  var sel = window.getSelection();
  if (sel && sel.rangeCount > 0) {
    var range = sel.getRangeAt(0);
    range.collapse(false);
    range.insertNode(br);
    range.setStartAfter(br);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
  } else {
    article.appendChild(br);
  }
  scheduleSave(article.dataset.docId, article.innerHTML);
  updatePageCount();
}

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
