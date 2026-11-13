"use strict";
/* Spellcheck / review flow — lists corrections, accepts per word, single-step undo. */

var spellcheckCorrections = [];

function activeDocIdSpellcheck() {
  var article = document.querySelector("article[data-doc-id]");
  return (article && article.dataset && article.dataset.docId) || "current";
}

function openSpellcheck() {
  var host = document.getElementById("docs-modal-container");
  if (!host) return;
  host.innerHTML = [
    '<div class="docs-modal" id="spellcheck-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
    '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:480px;max-width:92vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:12px;max-height:80vh;">',
    '<div style="display:flex;align-items:center;justify-content:space-between;">',
    '<h3 style="margin:0;font-size:16px;">Revisão de ortografia</h3>',
    '<button type="button" onclick="closeSpellcheck()" style="background:none;border:none;color:#94a3b8;font-size:20px;cursor:pointer;">×</button>',
    '</div>',
    '<div id="spellcheck-list" style="flex:1;overflow-y:auto;min-height:80px;color:#cbd5e1;font-size:13px;">Verificando…</div>',
    '<div style="display:flex;gap:8px;justify-content:flex-end;">',
    '<button type="button" onclick="closeSpellcheck()" style="background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;">Fechar</button>',
    '</div>',
    '</div></div>'
  ].join("");

  var article = getActiveArticle();
  var text = article ? (article.innerText || article.textContent || "") : "";
  fetch("/api/docs/ai/spellcheck", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      document_id: activeDocIdSpellcheck(),
      action: "spellcheck",
      prompt: text,
      selected_text: text
    })
  })
    .then(function (r) { if (!r.ok) throw new Error("http " + r.status); return r.json(); })
    .then(function (j) {
      spellcheckCorrections = (j && Array.isArray(j.corrections)) ? j.corrections : [];
      renderSpellcheckList();
    })
    .catch(function (e) {
      var el = document.getElementById("spellcheck-list");
      if (el) el.textContent = "Erro ao verificar: " + (e && e.message ? e.message : e);
    });
}

function closeSpellcheck() {
  var host = document.getElementById("docs-modal-container");
  if (host) host.innerHTML = "";
}

function renderSpellcheckList() {
  var el = document.getElementById("spellcheck-list");
  if (!el) return;
  if (!spellcheckCorrections.length) {
    el.innerHTML = '<div style="color:#94a3b8;text-align:center;padding:16px;">Nenhuma correção encontrada.</div>';
    return;
  }
  var html = "";
  spellcheckCorrections.forEach(function (c, idx) {
    if (c.accepted) return;
    html += '<div class="sc-item" data-idx="' + idx + '" style="display:flex;align-items:center;gap:8px;padding:8px;border-bottom:1px solid #334155;">' +
      '<div style="flex:1;font-size:13px;">' +
      '<span style="color:#f87171;text-decoration:line-through;">' + escapeHtmlSc(c.from) + '</span>' +
      ' → ' +
      '<span style="color:#34d399;">' + escapeHtmlSc(c.to) + '</span>' +
      '</div>' +
      '<button type="button" onclick="acceptSpellcheckCorrection(' + idx + ')" style="background:#10b981;color:#fff;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">Aceitar</button>' +
      '</div>';
  });
  if (!html) html = '<div style="color:#94a3b8;text-align:center;padding:16px;">Todas as correções aplicadas.</div>';
  el.innerHTML = html;
}

function escapeHtmlSc(s) {
  return String(s == null ? "" : s)
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function replaceFirstOccurrence(from, to) {
  var article = getActiveArticle();
  if (!article) return false;
  var walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT, null, false);
  var node;
  while ((node = walker.nextNode())) {
    var idx = (node.nodeValue || "").indexOf(from);
    if (idx !== -1) {
      var range = document.createRange();
      range.setStart(node, idx);
      range.setEnd(node, idx + from.length);
      var sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
      article.focus();
      gbFormat("insertText", to);
      return true;
    }
  }
  return false;
}

function acceptSpellcheckCorrection(idx) {
  var c = spellcheckCorrections[idx];
  if (!c || c.accepted) return;
  var ok = replaceFirstOccurrence(c.from, c.to);
  if (!ok) {
    alert("Trecho não encontrado no documento.");
    return;
  }
  c.accepted = true;
  renderSpellcheckList();
  var article = getActiveArticle();
  if (article && typeof scheduleSave === "function") {
    scheduleSave(article.dataset.docId || "current", article.innerHTML);
  }
}

window.openSpellcheck = openSpellcheck;
window.closeSpellcheck = closeSpellcheck;
window.acceptSpellcheckCorrection = acceptSpellcheckCorrection;