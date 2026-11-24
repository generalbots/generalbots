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
  } catch (e) { window.gbLogError ? gbLogError(e, "translateDocument") : console.error("translateDocument", e); }

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
    // #2457 — the backend returns HTTP 200 even when the LLM failed, with
    // result:"unavailable" + error:"..." (and content = the ORIGINAL text).
    // Never treat that as a successful translation or inject it — doing so
    // showed "Translated to X" over an unchanged document. Only accept a
    // genuine success result with non-empty content.
    if (j && (j.result === "unavailable" || j.error)) throw new Error(j.error || "translation unavailable");
    translated = (j && j.result === "success" && (j.content || j.response || j.text)) || "";
    if (!translated) translated = (j && (j.content || j.response || j.text)) || "";
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
  } catch (e) { window.gbLogError ? gbLogError(e, "translateDocument") : console.error("translateDocument", e); }

  if (typeof setSaveStatus === "function") setSaveStatus("Translated to " + lang, false);
  if (window.GBCollabVersions && typeof window.GBCollabVersions.snapshot === "function") {
    try { window.GBCollabVersions.snapshot({ resourceType: "docs", resourceId: String(docId), content: article.innerHTML }); } catch (e) { window.gbLogError ? gbLogError(e, "translateDocument") : console.error("translateDocument", e); }
  }
  return true;
}

function openTranslateModal() {
  var host = document.getElementById("docs-modal-container");
  if (!host) return;
  host.innerHTML = [
    '<div class="docs-modal" id="translate-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:var(--z-system);">',
    '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:360px;max-width:92vw;padding:20px;color:#f8fafc;display:flex;flex-direction:column;gap:12px;">',
    '<h3 style="margin:0;font-size:16px;">' + (window.gbT ? window.gbT("docs-translate-title", "Translate document") : "Translate document") + '</h3>',
    '<label style="font-size:12px;color:#94a3b8;">' + (window.gbT ? window.gbT("docs-translate-target-language", "Target language") : "Target language"),
    '<select id="translate-lang" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;margin-top:4px;">',
    '<option value="English">' + (window.gbT ? window.gbT("docs-lang-lang-en", "English") : "English") + '</option>',
    '<option value="Portuguese">' + (window.gbT ? window.gbT("docs-lang-lang-pt", "Portuguese") : "Portuguese") + '</option>',
    '<option value="Spanish">' + (window.gbT ? window.gbT("docs-lang-lang-es", "Spanish") : "Spanish") + '</option>',
    '<option value="French">' + (window.gbT ? window.gbT("docs-lang-lang-fr", "French") : "French") + '</option>',
    '<option value="German">' + (window.gbT ? window.gbT("docs-lang-lang-de", "German") : "German") + '</option>',
    '<option value="Italian">' + (window.gbT ? window.gbT("docs-lang-lang-it", "Italian") : "Italian") + '</option>',
    '</select></label>',
    '<div style="display:flex;gap:8px;justify-content:flex-end;">',
    '<button type="button" onclick="closeTranslateModal()" style="background:#334155;color:#f8fafc;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;">' + (window.gbT ? window.gbT("action-cancel", "Cancel") : "Cancel") + '</button>',
    '<button type="button" id="translate-apply" style="background:#3b82f6;color:#fff;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;font-weight:600;">' + (window.gbT ? window.gbT("docs-translate-action", "Translate") : "Translate") + '</button>',
    '</div></div></div>'
  ].join("");

  var btn = document.getElementById("translate-apply");
  if (btn) {
    btn.onclick = async function () {
      var sel = document.getElementById("translate-lang");
      var lang = sel ? sel.value : "English";
      btn.disabled = true;
      btn.textContent = (typeof window.gbT === "function" ? window.gbT("docs-translating", "Translating...") : "Translating...");
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