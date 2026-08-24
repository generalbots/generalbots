"use strict";
/* docs references & citations & equations (#1145 split).
 * Bibliography persists inside the document so it round-trips with save.
 */

(function (window) {
  var panel = null;
  function article() {
    return document.querySelector("article[contenteditable]");
  }
  function save() {
    var a = article();
    if (!a) return;
    a.dispatchEvent(new Event("input", { bubbles: true }));
  }
  function esc(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
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

  // Merged public API: core authoring primitives (09_authoring.js) plus
  // this module's references/citations/equations surface.
  var core = window.DocsAuthoringCore || {};
  window.DocsAuthoring = Object.assign({}, core, {
    addReference: addReference,
    insertCitation: insertCitation,
    openReferences: openReferences,
    closeReferences: closeReferences,
    insertEquation: insertEquation,
    openEquationModal: openEquationModal
  });
})(window);
