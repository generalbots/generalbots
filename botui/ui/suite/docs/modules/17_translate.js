"use strict";
/* Document translation — formatting-preserving in-editor apply with undo. */

function activeDocIdTranslate() {
  var article = document.querySelector("article[data-doc-id]");
  return (article && article.dataset && article.dataset.docId) || "current";
}

async function translateDocument(lang) {
  lang = lang || "English";
  var article = getActiveArticle();
  if (!article) return false;
  var docId = activeDocIdTranslate();
  var original = article.innerHTML;

  // Seed server-side history with the pre-translation version so a single undo
  // restores it. Save the original first, then apply + save the translation.
  try {
    await fetch("/api/docs/autosave", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify({ id: docId, content: original })
    });
  } catch (e) {}

  var translated = "";
  try {
    var res = await fetch("/api/docs/ai/translate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify({
        document_id: docId,
        action: "translate",
        prompt: lang,
        selected_text: original,
        translate_lang: lang
      })
    });
    if (!res.ok) throw new Error("http " + res.status);
    var j = await res.json();
    translated = (j && (j.content || j.response || j.text || j.result)) || "";
  } catch (e) {
    translated = "";
  }

  if (!translated) {
    if (typeof setSaveStatus === "function") setSaveStatus("Translation unavailable", true);
    return false;
  }

  // Apply preserving formatting: if the model returned HTML, use it directly;
  // otherwise render markdown-lite blocks.
  if (/<[a-z][\s\S]*>/i.test(translated)) {
    article.innerHTML = translated;
  } else if (typeof docsRenderBlocks === "function") {
    article.innerHTML = docsRenderBlocks(translated);
  } else {
    article.innerHTML = translated;
  }
  article.dispatchEvent(new Event("input", { bubbles: true }));
  if (typeof updatePageCount === "function") updatePageCount();

  try {
    await fetch("/api/docs/autosave", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify({ id: docId, content: article.innerHTML })
    });
  } catch (e) {}

  if (typeof setSaveStatus === "function") setSaveStatus("Translated to " + lang, false);
  if (window.GBCollabVersions && typeof window.GBCollabVersions.snapshot === "function") {
    try { window.GBCollabVersions.snapshot({ resourceType: "docs", resourceId: String(docId), content: article.innerHTML }); } catch (e) {}
  }
  return true;
}

function openTranslateModal() {
  var host = document.getElementById("docs-modal-container");
  if (!host) return;
  host.innerHTML = [
    '<div class="docs-modal" id="translate-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
    '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:360px;max-width:92vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:12px;">',
    '<h3 style="margin:0;font-size:16px;">Translate document</h3>',
    '<label style="font-size:12px;color:#94a3b8;">Target language',
    '<select id="translate-lang" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;">',
    '<option value="English">English</option>',
    '<option value="Portuguese">Portuguese</option>',
    '<option value="Spanish">Spanish</option>',
    '<option value="French">French</option>',
    '<option value="German">German</option>',
    '<option value="Italian">Italian</option>',
    '</select></label>',
    '<div style="display:flex;gap:8px;justify-content:flex-end;">',
    '<button type="button" onclick="closeTranslateModal()" style="background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;">Cancel</button>',
    '<button type="button" id="translate-apply" style="background:#3b82f6;color:#fff;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;">Translate</button>',
    '</div></div></div>'
  ].join("");

  var btn = document.getElementById("translate-apply");
  if (btn) {
    btn.onclick = async function () {
      var sel = document.getElementById("translate-lang");
      var lang = sel ? sel.value : "English";
      btn.disabled = true;
      btn.textContent = "Translating...";
      await translateDocument(lang);
      closeTranslateModal();
    };
  }
}

function closeTranslateModal() {
  var host = document.getElementById("docs-modal-container");
  if (host) host.innerHTML = "";
}

window.openTranslateModal = openTranslateModal;
window.closeTranslateModal = closeTranslateModal;
window.GBDocsTranslate = translateDocument;
window.translateDocument = translateDocument;