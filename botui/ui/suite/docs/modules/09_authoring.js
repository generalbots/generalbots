"use strict";
/* docs authoring module — tables, images, citations (references).
 *
 * The docs editor is a single contenteditable <article>. This module adds the
 * three authoring primitives a professional document needs that the base
 * editor lacks:
 *
 *   insertTable(rows, cols)  — an editable table at the caret
 *   insertImage(url, alt)    — an <img> at the caret
 *   addReference(...)        — append a bibliography entry (persists in the
 *                              document itself, so it round-trips with save)
 *   insertCitation(index)    — a [n] marker that links to the reference
 *
 * Equations are intentionally out of scope here: they need a math renderer
 * (KaTeX/MathJax) which is not vendored, and the project forbids CDN assets.
 *
 * Public API (window.DocsAuthoring):
 *   insertTable, openTableModal, insertImage, openImageModal,
 *   addReference, insertCitation, openReferences, closeReferences
 */
(function (window) {
  var CSS_ID = "gb-docs-authoring-css";
  var panel = null;

  function article() {
    return document.querySelector("article[contenteditable]");
  }
  function save() {
    var a = article();
    if (!a) return;
    if (typeof scheduleSave === "function") scheduleSave(a.dataset.docId, a.innerHTML);
    if (typeof updatePageCount === "function") updatePageCount();
  }
  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "article.docs-doc-view table.docs-table{border-collapse:collapse;margin:0.8em 0;width:100%;}",
      "article.docs-doc-view table.docs-table td,article.docs-doc-view table.docs-table th{",
      "border:1px solid #cbd5e1;padding:6px 10px;min-width:60px;font-size:15px;vertical-align:top;}",
      "article.docs-doc-view table.docs-table th{background:#f1f5f9;font-weight:600;}",
      "article.docs-doc-view img.docs-image{max-width:100%;height:auto;border-radius:4px;margin:0.5em 0;}",
      "article.docs-doc-view sup.docs-citation{cursor:pointer;color:#2563eb;font-weight:600;",
      "padding:0 2px;white-space:nowrap;}",
      "article.docs-doc-view .docs-references{margin-top:2em;border-top:1px solid #cbd5e1;padding-top:1em;",
      "font-size:13px;color:#475569;}",
      "article.docs-doc-view .docs-references h4{margin:0 0 0.5em;color:#0f172a;font-size:14px;}",
      "article.docs-doc-view .docs-references ol{margin:0 0 0 1.5em;padding:0;}",
      "article.docs-doc-view .docs-references li{margin:0.3em 0;}",
      "article.docs-doc-view .docs-equation{display:inline-flex;align-items:center;padding:2px 6px;",
      "margin:0 2px;background:#f8fafc;border-radius:4px;font-family:'Cambria Math',Georgia,serif;",
      "font-size:1.05em;color:#0f172a;}",
      "article.docs-doc-view .eq-frac{display:inline-flex;flex-direction:column;align-items:center;",
      "vertical-align:middle;margin:0 2px;}",
      "article.docs-doc-view .eq-frac-top{border-bottom:1px solid #0f172a;padding:0 4px;line-height:1.1;}",
      "article.docs-doc-view .eq-frac-bot{padding:0 4px;line-height:1.1;}",
      "article.docs-doc-view .eq-sup{font-size:0.7em;vertical-align:super;}",
      "article.docs-doc-view .eq-sub{font-size:0.7em;vertical-align:sub;}",
      "article.docs-doc-view .eq-sqrt{display:inline-flex;align-items:center;margin:0 2px;}",
      "article.docs-doc-view .eq-sqrt-rad{border-top:1px solid #0f172a;padding:0 2px;}",
      "#gb-refs-panel{position:fixed;top:0;right:0;bottom:0;width:380px;max-width:94vw;",
      "background:#0f172a;border-left:1px solid #334155;z-index:100000;display:flex;flex-direction:column;",
      "box-shadow:-8px 0 24px rgba(0,0,0,.4);transform:translateX(100%);transition:transform .2s ease;",
      "font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-refs-panel.gref-open{transform:translateX(0);}",
      "#gb-refs-panel .gref-header{display:flex;align-items:center;gap:8px;padding:12px 14px;",
      "border-bottom:1px solid #334155;background:#1e293b;}",
      "#gb-refs-panel .gref-title{flex:1;color:#f8fafc;font-size:14px;font-weight:600;}",
      "#gb-refs-panel .gref-close{background:none;border:none;color:#94a3b8;font-size:20px;",
      "line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-refs-panel .gref-close:hover{color:#f8fafc;}",
      "#gb-refs-panel .gref-list{flex:1;overflow-y:auto;padding:12px 14px;display:flex;",
      "flex-direction:column;gap:10px;}",
      "#gb-refs-panel .gref-empty{color:#94a3b8;font-size:13px;text-align:center;padding:24px 8px;}",
      "#gb-refs-panel .gref-item{background:#1e293b;border:1px solid #334155;border-radius:8px;padding:10px 12px;}",
      "#gb-refs-panel .gref-item-top{display:flex;align-items:center;gap:8px;margin-bottom:4px;}",
      "#gb-refs-panel .gref-num{background:#3b82f6;color:#fff;font-size:11px;font-weight:600;",
      "min-width:20px;height:20px;border-radius:999px;display:inline-flex;align-items:center;",
      "justify-content:center;padding:0 4px;}",
      "#gb-refs-panel .gref-cite{margin-left:auto;background:#0f172a;border:1px solid #334155;",
      "color:#93c5fd;border-radius:6px;padding:3px 10px;font-size:12px;cursor:pointer;}",
      "#gb-refs-panel .gref-cite:hover{background:#334155;}",
      "#gb-refs-panel .gref-meta{color:#cbd5e1;font-size:12.5px;line-height:1.5;}",
      "#gb-refs-panel .gref-form{display:flex;flex-direction:column;gap:8px;padding:12px 14px;",
      "border-top:1px solid #334155;}",
      "#gb-refs-panel .gref-form input{background:#1e293b;border:1px solid #334155;border-radius:6px;",
      "color:#f8fafc;padding:8px 10px;font-size:13px;}",
      "#gb-refs-panel .gref-form button{background:#3b82f6;border:none;color:#fff;border-radius:6px;",
      "padding:8px;font-size:13px;cursor:pointer;}",
      "#gb-refs-panel .gref-form button:hover{background:#2563eb;}"
    ].join("");
    var style = document.createElement("style");
    style.id = CSS_ID;
    style.textContent = css;
    document.head.appendChild(style);
  }

  /* Insert a DOM node at the caret, then place the caret after it. */
  function insertNodeAtCaret(node) {
    var a = article();
    if (!a) return false;
    a.focus();
    var sel = window.getSelection();
    if (sel && sel.rangeCount > 0) {
      var range = sel.getRangeAt(0);
      range.collapse(false);
      range.insertNode(node);
      range.setStartAfter(node);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    } else {
      a.appendChild(node);
    }
    save();
    return true;
  }

  /* ---- Tables ---- */
  function buildTable(rows, cols) {
    var table = document.createElement("table");
    table.className = "docs-table";
    table.contentEditable = "false";
    table.dataset.docsTable = "1";
    var thead = document.createElement("thead");
    var htr = document.createElement("tr");
    for (var c = 0; c < cols; c++) {
      var th = document.createElement("th");
      th.contentEditable = "true";
      th.textContent = "Header " + (c + 1);
      htr.appendChild(th);
    }
    thead.appendChild(htr);
    table.appendChild(thead);
    var tbody = document.createElement("tbody");
    for (var r = 0; r < rows; r++) {
      var tr = document.createElement("tr");
      for (var c2 = 0; c2 < cols; c2++) {
        var td = document.createElement("td");
        td.contentEditable = "true";
        td.innerHTML = "&nbsp;";
        tr.appendChild(td);
      }
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);
    return table;
  }

  function insertTable(rows, cols) {
    rows = Math.max(1, Math.min(12, rows || 3));
    cols = Math.max(1, Math.min(8, cols || 3));
    insertNodeAtCaret(buildTable(rows, cols));
  }

  function openTableModal() {
    var host = document.getElementById("modal-container");
    if (!host) return;
    host.innerHTML = [
      '<div class="docs-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
      '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:420px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;">',
      '<h3 style="margin:0;font-size:16px;">Insert table</h3>',
      '<div style="display:flex;gap:14px;">',
      '<label style="flex:1;font-size:12px;color:#94a3b8;">Rows<input id="tbl-rows" type="number" min="1" max="12" value="3" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;"/></label>',
      '<label style="flex:1;font-size:12px;color:#94a3b8;">Columns<input id="tbl-cols" type="number" min="1" max="8" value="3" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;"/></label>',
      '</div>',
      '<div style="display:flex;gap:8px;justify-content:flex-end;">',
      '<button type="button" id="tbl-cancel" style="background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;">Cancel</button>',
      '<button type="button" id="tbl-apply" style="background:#3b82f6;color:#fff;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;">Insert</button>',
      '</div></div></div>'
    ].join("");
    document.getElementById("tbl-cancel").onclick = function () { host.innerHTML = ""; };
    document.getElementById("tbl-apply").onclick = function () {
      var r = parseInt(document.getElementById("tbl-rows").value, 10) || 3;
      var c = parseInt(document.getElementById("tbl-cols").value, 10) || 3;
      host.innerHTML = "";
      insertTable(r, c);
    };
  }

  /* ---- Images ---- */
  function insertImage(url, alt) {
    if (!url) return;
    var img = document.createElement("img");
    img.className = "docs-image";
    img.src = url;
    img.alt = alt || "";
    img.contentEditable = "false";
    insertNodeAtCaret(img);
  }

  function openImageModal() {
    var host = document.getElementById("modal-container");
    if (!host) return;
    host.innerHTML = [
      '<div class="docs-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
      '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:460px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;">',
      '<h3 style="margin:0;font-size:16px;">Insert image</h3>',
      '<label style="font-size:12px;color:#94a3b8;">Image URL<input id="img-url" type="text" placeholder="https://…" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;"/></label>',
      '<label style="font-size:12px;color:#94a3b8;">Alt text<input id="img-alt" type="text" placeholder="Describe the image" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;"/></label>',
      '<div style="display:flex;gap:8px;justify-content:flex-end;">',
      '<button type="button" id="img-cancel" style="background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;">Cancel</button>',
      '<button type="button" id="img-apply" style="background:#3b82f6;color:#fff;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;">Insert</button>',
      '</div></div></div>'
    ].join("");
    document.getElementById("img-cancel").onclick = function () { host.innerHTML = ""; };
    document.getElementById("img-apply").onclick = function () {
      var url = document.getElementById("img-url").value.trim();
      var alt = document.getElementById("img-alt").value.trim();
      host.innerHTML = "";
      insertImage(url, alt);
    };
  }

  /* ---- Citations / references ---- */
  function referencesList() {
    var a = article();
    if (!a) return null;
    var list = a.querySelector(".docs-references ol");
    if (list) return list;
    var section = document.createElement("div");
    section.className = "docs-references";
    section.contentEditable = "false";
    var h = document.createElement("h4");
    h.textContent = "References";
    section.appendChild(h);
    list = document.createElement("ol");
    section.appendChild(list);
    a.appendChild(section);
    return list;
  }

  function formatRef(ref) {
    var parts = [];
    if (ref.author) parts.push(ref.author);
    if (ref.year) parts.push("(" + ref.year + ")");
    if (ref.title) parts.push("“" + ref.title + "”");
    if (ref.url) parts.push(ref.url);
    return parts.join(". ") + ".";
  }

  function addReference(ref) {
    ref = ref || {};
    var list = referencesList();
    if (!list) return -1;
    var li = document.createElement("li");
    li.dataset.refId = "ref-" + Date.now();
    li.textContent = formatRef(ref);
    list.appendChild(li);
    save();
    return list.children.length - 1;
  }

  function insertCitation(index) {
    var sup = document.createElement("sup");
    sup.className = "docs-citation";
    sup.contentEditable = "false";
    sup.textContent = "[" + (index + 1) + "]";
    sup.title = "Reference " + (index + 1);
    insertNodeAtCaret(sup);
  }

  function readReferences() {
    var a = article();
    if (!a) return [];
    var lis = a.querySelectorAll(".docs-references ol li");
    return Array.prototype.map.call(lis, function (li) {
      var text = li.textContent || "";
      return { id: li.dataset.refId || "", text: text };
    });
  }

  function renderRefs() {
    if (!panel) return;
    var listEl = panel.querySelector(".gref-list");
    var refs = readReferences();
    if (!refs.length) {
      listEl.innerHTML = '<div class="gref-empty">No references yet — add one below, then insert a citation.</div>';
      return;
    }
    listEl.innerHTML = refs.map(function (ref, i) {
      return '<div class="gref-item">' +
        '<div class="gref-item-top">' +
        '<span class="gref-num">' + (i + 1) + '</span>' +
        '<button class="gref-cite" data-idx="' + i + '">Cite</button>' +
        '</div>' +
        '<div class="gref-meta">' + esc(ref.text) + '</div>' +
        '</div>';
    }).join("");
    listEl.querySelectorAll(".gref-cite").forEach(function (btn) {
      btn.addEventListener("click", function () {
        closeReferences();
        insertCitation(parseInt(btn.dataset.idx, 10));
      });
    });
  }

  function ensurePanel() {
    ensureCss();
    if (panel && panel.parentNode) return panel;
    panel = document.createElement("div");
    panel.id = "gb-refs-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", "References");
    panel.innerHTML =
      '<div class="gref-header">' +
      '<span class="gref-title">References &amp; citations</span>' +
      '<button class="gref-close" title="Close" aria-label="Close">×</button>' +
      '</div>' +
      '<div class="gref-list"></div>' +
      '<div class="gref-form">' +
      '<input id="gref-author" placeholder="Author" aria-label="Author" />' +
      '<input id="gref-title" placeholder="Title" aria-label="Title" />' +
      '<input id="gref-year" placeholder="Year" aria-label="Year" />' +
      '<input id="gref-url" placeholder="URL (optional)" aria-label="URL" />' +
      '<button id="gref-add">Add reference</button>' +
      '</div>';
    document.body.appendChild(panel);
    panel.querySelector(".gref-close").addEventListener("click", closeReferences);
    panel.querySelector("#gref-add").addEventListener("click", function () {
      addReference({
        author: panel.querySelector("#gref-author").value.trim(),
        title: panel.querySelector("#gref-title").value.trim(),
        year: panel.querySelector("#gref-year").value.trim(),
        url: panel.querySelector("#gref-url").value.trim()
      });
      panel.querySelector("#gref-author").value = "";
      panel.querySelector("#gref-title").value = "";
      panel.querySelector("#gref-year").value = "";
      panel.querySelector("#gref-url").value = "";
      renderRefs();
    });
    return panel;
  }

  function openReferences() {
    ensurePanel();
    panel.classList.add("gref-open");
    renderRefs();
  }

  function closeReferences() {
    if (panel) panel.classList.remove("gref-open");
  }

  /* ---- Equations (self-contained LaTeX subset — no KaTeX/MathJax) ---- */
  var SYMBOLS = {
    times: "×", pm: "±", mp: "∓", le: "≤", leq: "≤", ge: "≥", geq: "≥",
    neq: "≠", ne: "≠", approx: "≈", infty: "∞", cdot: "·", sum: "∑",
    prod: "∏", int: "∫", alpha: "α", beta: "β", gamma: "γ", delta: "δ",
    Delta: "Δ", epsilon: "ε", varepsilon: "ε", theta: "θ", Theta: "Θ",
    lambda: "λ", Lambda: "Λ", mu: "μ", pi: "π", Pi: "Π", sigma: "σ",
    Sigma: "Σ", phi: "φ", varphi: "φ", Phi: "Φ", psi: "ψ", omega: "ω",
    Omega: "Ω", partial: "∂", nabla: "∇", propto: "∝", to: "→",
    rightarrow: "→", leftarrow: "←", leftrightarrow: "↔", forall: "∀",
    exists: "∃", in: "∈", notin: "∉", subset: "⊂", subseteq: "⊆",
    cup: "∪", cap: "∩", emptyset: "∅", degree: "°", div: "÷"
  };

  function parseBrace(tex, i) {
    var depth = 0, j = i, content = "";
    for (; j < tex.length; j++) {
      var c = tex[j];
      if (c === "{") { depth++; if (depth > 1) content += c; }
      else if (c === "}") { depth--; if (depth === 0) { j++; break; } content += c; }
      else content += c;
    }
    return { content: content, next: j };
  }

  function renderLatex(tex) {
    var out = "", i = 0;
    while (i < tex.length) {
      var ch = tex[i];
      if (ch === "\\") {
        var m = /^\\[a-zA-Z]+/.exec(tex.slice(i));
        if (m) {
          var name = m[0].slice(1);
          if (name === "frac" || name === "dfrac") {
            var g1 = parseBrace(tex, i + m[0].length);
            var g2 = parseBrace(tex, g1.next);
            out += '<span class="eq-frac"><span class="eq-frac-top">' + renderLatex(g1.content) +
              '</span><span class="eq-frac-bot">' + renderLatex(g2.content) + '</span></span>';
            i = g2.next;
            continue;
          }
          if (name === "sqrt") {
            var g = parseBrace(tex, i + m[0].length);
            out += '<span class="eq-sqrt">√<span class="eq-sqrt-rad">' + renderLatex(g.content) + '</span></span>';
            i = g.next;
            continue;
          }
          out += SYMBOLS[name] !== undefined ? SYMBOLS[name] : esc(name);
          i += m[0].length;
          continue;
        }
        out += esc(ch);
        i++;
        continue;
      }
      if (ch === "^" || ch === "_") {
        var isSup = ch === "^";
        var nxt = tex[i + 1];
        if (nxt === "{") {
          var grp = parseBrace(tex, i + 1);
          out += isSup ? '<sup class="eq-sup">' + renderLatex(grp.content) + '</sup>'
                       : '<sub class="eq-sub">' + renderLatex(grp.content) + '</sub>';
          i = grp.next;
        } else if (nxt) {
          out += isSup ? '<sup class="eq-sup">' + esc(nxt) + '</sup>'
                       : '<sub class="eq-sub">' + esc(nxt) + '</sub>';
          i += 2;
        } else {
          out += esc(ch);
          i++;
        }
        continue;
      }
      out += esc(ch);
      i++;
    }
    return out;
  }

  function insertEquation(tex) {
    if (!tex) return;
    var span = document.createElement("span");
    span.className = "docs-equation";
    span.contentEditable = "false";
    span.innerHTML = renderLatex(tex);
    insertNodeAtCaret(span);
  }

  function openEquationModal() {
    var host = document.getElementById("modal-container");
    if (!host) return;
    host.innerHTML = [
      '<div class="docs-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
      '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:480px;max-width:90vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:14px;">',
      '<h3 style="margin:0;font-size:16px;">Insert equation</h3>',
      '<label style="font-size:12px;color:#94a3b8;">LaTeX (subset: \\frac, \\sqrt, ^ _, Greek &amp; symbols)<input id="eq-tex" type="text" placeholder="\\frac{a}{b} + x^2" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;font-family:monospace;"/></label>',
      '<div style="font-size:12px;color:#94a3b8;">Preview: <span id="eq-preview" style="color:#f8fafc;font-size:16px;margin-left:6px;"></span></div>',
      '<div style="display:flex;gap:8px;justify-content:flex-end;">',
      '<button type="button" id="eq-cancel" style="background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;">Cancel</button>',
      '<button type="button" id="eq-apply" style="background:#3b82f6;color:#fff;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;">Insert</button>',
      '</div></div></div>'
    ].join("");
    var input = document.getElementById("eq-tex");
    var preview = document.getElementById("eq-preview");
    input.addEventListener("input", function () {
      preview.innerHTML = renderLatex(input.value);
    });
    document.getElementById("eq-cancel").onclick = function () { host.innerHTML = ""; };
    document.getElementById("eq-apply").onclick = function () {
      var tex = input.value.trim();
      host.innerHTML = "";
      insertEquation(tex);
    };
  }

  window.DocsAuthoring = {
    insertTable: insertTable,
    openTableModal: openTableModal,
    insertImage: insertImage,
    openImageModal: openImageModal,
    addReference: addReference,
    insertCitation: insertCitation,
    openReferences: openReferences,
    closeReferences: closeReferences,
    insertEquation: insertEquation,
    openEquationModal: openEquationModal
  };
})(window);
