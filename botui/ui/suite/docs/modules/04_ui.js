"use strict";
/* docs UI — styles, page break, header/footer modal */

function injectEditorStyles() {
  if (document.getElementById("docs-editor-styles")) return;
  var style = document.createElement("style");
  style.id = "docs-editor-styles";
  style.textContent = [
    "article.docs-doc-view{background:#ffffff;color:#1e293b;padding:96px;width:816px;margin:32px auto;border:1px solid #e2e8f0;border-radius:4px;font-family:Georgia,serif;font-size:16px;line-height:1.7;min-height:1056px;outline:none;box-shadow:0 10px 30px rgba(0,0,0,0.25);box-sizing:border-box;}",
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
    "@media screen{body{background:#0f172a;}}"
  ].join("");
  document.head.appendChild(style);
}

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
  var host = document.getElementById("modal-container");
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
