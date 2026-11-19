"use strict";
/* docs UI — formatting, toolbar, font/size and header-footer (split from 04_ui.js for #2305). */

function docsFormat(command, value) {
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  try {
    gbFormat("styleWithCSS", true);
    gbFormat(command, value);
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

function docsNormalizeLegacyFontSize(article, ptValue) {
  if (!article) return;
  var markerSizes = {"7":1,"xxx-large":1,"48px":1,"36pt":1,"xx-large":1,"x-large":1,"-webkit-xxx-large":1};
  var fonts = article.querySelectorAll('font[size]');
  for (var i = 0; i < fonts.length; i++) {
    var f = fonts[i];
    var sz = f.getAttribute('size');
    if (sz !== '7' && !markerSizes[sz]) continue;
    var span = document.createElement('span');
    // preserve existing inline styles if font was oddly wrapped
    if (f.getAttribute('style')) span.setAttribute('style', f.getAttribute('style'));
    span.style.fontSize = ptValue + "pt";
    if (f.color && !span.style.color) span.style.color = f.color;
    if (f.face && !span.style.fontFamily) span.style.fontFamily = f.face;
    while (f.firstChild) span.appendChild(f.firstChild);
    if (f.parentNode) f.parentNode.replaceChild(span, f);
  }
  // some browsers with styleWithCSS may create spans with very-large font-size for size 7
  var spans = article.querySelectorAll('span[style*="font-size"]');
  for (var j = 0; j < spans.length; j++) {
    var sp = spans[j];
    var fs = sp.style.fontSize || "";
    if (markerSizes[fs] || fs === "xxx-large" || fs === "48px" || fs === "36pt") {
      sp.style.fontSize = ptValue + "pt";
    }
  }
}

function docsApplyFontSize(value) {
  if (!value) return;
  var raw = String(value).replace(/pt$/i, "").trim();
  var n = parseFloat(raw);
  if (!isFinite(n) || n <= 0) return;
  n = Math.round(n * 100) / 100;
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  var sel = window.getSelection ? window.getSelection() : null;
  if (!sel || sel.rangeCount === 0) return;
  var range = sel.getRangeAt(0);
  if (range.collapsed) {
    var sizeInput0 = document.getElementById("docFontSize") || (document.querySelector("#window-body-docs #docFontSize"));
    if (sizeInput0) sizeInput0.value = String(n);
    return;
  }
  // If selection is fully inside a single element, style it directly to avoid nesting churn.
  try {
    var anc = range.commonAncestorContainer;
    var el = anc && anc.nodeType === 1 ? anc : (anc ? anc.parentNode : null);
    if (el && el !== article && article.contains(el)) {
      var er = document.createRange();
      er.selectNodeContents(el);
      if (range.compareBoundaryPoints(Range.START_TO_START, er) >= 0 &&
          range.compareBoundaryPoints(Range.END_TO_END, er) <= 0) {
        el.style.fontSize = n + "pt";
        var sizeInput1 = document.getElementById("docFontSize") || (document.querySelector("#window-body-docs #docFontSize"));
        if (sizeInput1) sizeInput1.value = String(n);
        article.dispatchEvent(new Event("input", { bubbles: true }));
        docsSyncFormatState();
        return;
      }
    }
  } catch (e0) { window.gbLogError ? gbLogError(e0, "docsApplyFontSize") : console.error("docsApplyFontSize", e0); }
  var span = document.createElement('span');
  span.style.fontSize = n + "pt";
  try {
    range.surroundContents(span);
  } catch (e) {
    try {
      var contents = range.extractContents();
      span.appendChild(contents);
      range.insertNode(span);
    } catch (e2) {
      return;
    }
  }
  // Remove legacy <font size> wrappers that may still exist inside selection.
  try {
    var fonts = span.querySelectorAll('font[size]');
    for (var i = 0; i < fonts.length; i++) {
      var f = fonts[i];
      var inner = document.createElement('span');
      if (f.getAttribute('style')) inner.setAttribute('style', f.getAttribute('style'));
      if (f.face && !inner.style.fontFamily) inner.style.fontFamily = f.face;
      if (f.color && !inner.style.color) inner.style.color = f.color;
      while (f.firstChild) inner.appendChild(f.firstChild);
      if (f.parentNode) f.parentNode.replaceChild(inner, f);
    }
  } catch (e3) { window.gbLogError ? gbLogError(e3, "docsApplyFontSize") : console.error("docsApplyFontSize", e3); }
  try {
    var nr = document.createRange();
    nr.selectNodeContents(span);
    sel.removeAllRanges();
    sel.addRange(nr);
  } catch (e4) { window.gbLogError ? gbLogError(e4, "docsApplyFontSize") : console.error("docsApplyFontSize", e4); }
  var sizeInput = document.getElementById("docFontSize") || (document.querySelector("#window-body-docs #docFontSize"));
  if (sizeInput) sizeInput.value = String(n);
  article.dispatchEvent(new Event("input", { bubbles: true }));
  docsSyncFormatState();
}

function docsApplyFontFamily(value) {
  if (!value) return;
  var article = getActiveArticle();
  if (!article) return;
  article.focus();
  // Apply via CSS span wrapper for consistent, non-legacy behavior.
  try {
    var sel = window.getSelection ? window.getSelection() : null;
    if (!sel || sel.rangeCount === 0) return;
    var range = sel.getRangeAt(0);
    if (range.collapsed) return;
    // If selection is fully inside a single element, style it directly.
    try {
      var anc = range.commonAncestorContainer;
      var el = anc && anc.nodeType === 1 ? anc : (anc ? anc.parentNode : null);
      if (el && el !== article && article.contains(el)) {
        var er = document.createRange();
        er.selectNodeContents(el);
        if (range.compareBoundaryPoints(Range.START_TO_START, er) >= 0 &&
            range.compareBoundaryPoints(Range.END_TO_END, er) <= 0) {
          el.style.fontFamily = value;
          article.dispatchEvent(new Event("input", { bubbles: true }));
          docsSyncFormatState();
          return;
        }
      }
    } catch (e0) { window.gbLogError ? gbLogError(e0, "docsApplyFontFamily") : console.error("docsApplyFontFamily", e0); }
    var span = document.createElement('span');
    span.style.fontFamily = value;
    try {
      range.surroundContents(span);
    } catch (e2) {
      var contents = range.extractContents();
      if (!contents || !contents.childNodes || !contents.childNodes.length) return;
      span.appendChild(contents);
      range.insertNode(span);
    }
    // Clean nested legacy font tags if any
    try {
      var fonts = span.querySelectorAll('font[face]');
      for (var i = 0; i < fonts.length; i++) {
        var f = fonts[i];
        var inner = document.createElement('span');
        if (f.getAttribute('style')) inner.setAttribute('style', f.getAttribute('style'));
        if (f.size && !inner.style.fontSize) inner.style.fontSize = '';
        if (f.color && !inner.style.color) inner.style.color = f.color;
        while (f.firstChild) inner.appendChild(f.firstChild);
        if (f.parentNode) f.parentNode.replaceChild(inner, f);
      }
    } catch (e4) { window.gbLogError ? gbLogError(e4, "docsApplyFontFamily") : console.error("docsApplyFontFamily", e4); }
    var nr = document.createRange();
    nr.selectNodeContents(span);
    sel.removeAllRanges();
    sel.addRange(nr);
  } catch (e3) {
    return;
  }
  article.dispatchEvent(new Event("input", { bubbles: true }));
  docsSyncFormatState();
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
    if (color) {
      var colorTrigger = scope.querySelector('[data-gb-color-for="fmtColor"]');
      if (window.GBColorPicker) window.GBColorPicker.enhance(color, { trigger: colorTrigger });
      color.addEventListener("gb-color-commit", function () { docsFormat("foreColor", color.value); });
      color.addEventListener("change", function () {
        if (!color._gbColorEnhanced) docsFormat("foreColor", color.value);
      });
    }
    var highlight = scope.querySelector("#fmtHighlight");
    if (highlight) {
      var highlightTrigger = scope.querySelector('[data-gb-color-for="fmtHighlight"]');
      if (window.GBColorPicker) window.GBColorPicker.enhance(highlight, { trigger: highlightTrigger });
      highlight.addEventListener("gb-color-commit", function () { docsFormat("hiliteColor", highlight.value); });
      highlight.addEventListener("change", function () {
        if (!highlight._gbColorEnhanced) docsFormat("hiliteColor", highlight.value);
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
window.docsApplyFontFamily = docsApplyFontFamily;

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
    "<div class=\"docs-modal\" id=\"docs-hf-modal\" style=\"position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:var(--z-system);\">",
    "<div style=\"background:#1e293b;border:1px solid #334155;border-radius:12px;width:520px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;\">",
    "<h3 style=\"margin:0;font-size:16px;\">Cabeçalho e Rodapé</h3>",
    "<label style=\"font-size:12px;color:#94a3b8;\">Cabeçalho (aparece no topo de cada página)<input id=\"hf-header\" type=\"text\" placeholder=\"Cabeçalho...\" style=\"width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;\"/></label>",
    "<label style=\"font-size:12px;color:#94a3b8;\">Rodapé (aparece no final de cada página)<input id=\"hf-footer\" type=\"text\" placeholder=\"Rodapé...\" style=\"width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;\"/></label>",
    "<div style=\"display:flex;gap:8px;justify-content:flex-end;\">",
    "<button id=\"hf-cancel\" type=\"button\" style=\"background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;\">" + (window.gbT ? window.gbT("action-cancel", "Cancel") : "Cancel") + "</button>",
    "<button id=\"hf-apply\" type=\"button\" style=\"background:#3b82f6;color:white;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;\">" + (window.gbT ? window.gbT("action-apply", "Apply") : "Apply") + "</button>",
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
    var textResp = "";
    try {
      var data = JSON.parse(detail.xhr.responseText);
      textResp = data.result === "unavailable" && data.error
        ? data.error
        : (data.content || data.response || data.text || data.result || "");
    } catch (error) {
      textResp = detail.xhr.responseText;
    }
    var target = document.getElementById("ai-result-panel") ||
      document.getElementById("ai-result-modal");
    if (target) target.textContent = textResp;
  });
}
