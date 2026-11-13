"use strict";
/* docs save — auto-save, status, AI driver */

function setSaveStatus(text, isError) {
  var el = document.getElementById("saveStatus");
  if (el) {
    el.textContent = text;
    el.style.color = isError ? SAVE_ERR_COLOR : SAVE_OK_COLOR;
  }
}

var saveTimer = null;
function docDraftKey(id) { return "docs:" + (id || "unknown"); }
function resolveDocsSaveId(id) {
  var docId = id;
  if (!docId || docId === "current") {
    var article = typeof getActiveArticle === "function" ? getActiveArticle() : null;
    if (article && typeof docsEnsureArticleDocId === "function") {
      docId = docsEnsureArticleDocId(article);
    } else if (article && article.dataset) {
      docId = article.dataset.docId || "";
    }
  }
  if (!docId || docId === "current") return "";
  return String(docId);
}
function scheduleSave(id, content) {
  // #2253 — a shared-link view has no write path: the server answers 401 for
  // every mutating Docs route without a session, so queueing a save here would
  // only show a spurious "saving" status.
  if (typeof docsIsReadOnly === "function" && docsIsReadOnly()) return;
  var docId = resolveDocsSaveId(id);
  if (!docId) {
    setSaveStatus("Save failed: missing document id", true);
    return;
  }
  clearTimeout(saveTimer);
  setSaveStatus("Saving...", false);
  saveTimer = setTimeout(function () {
    fetch("/api/docs/autosave", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: docId, content: content })
    })
      .then(function (r) { if (!r.ok) throw new Error("http " + r.status); return r.json(); })
      .then(function () {
        setSaveStatus("All changes saved", false);
        if (window.GBOfflineDraft) window.GBOfflineDraft.clear(docDraftKey(docId));
        if (window.GBCollabActivity) {
          window.GBCollabActivity.record({
            resourceType: "docs",
            resourceId: String(docId),
            action: "edit",
            payload: {}
          }).catch(function (err) {
            /* Best-effort activity side effect after a successful save. */
            console.debug("[gb-docs] Activity record failed:", err);
          });
        }
        if (window.GBCollabVersions) {
          window.GBCollabVersions.snapshot({
            resourceType: "docs",
            resourceId: String(docId),
            content: content
          }).catch(function (err) {
            /* Best-effort collaboration snapshot after a successful save. */
            console.debug("[gb-docs] Collab snapshot failed:", err);
          });
        }
        if (typeof window.docsRefreshSidebar === "function") {
          window.docsRefreshSidebar();
        }
      })
      .catch(function () {
        // Offline (or server error): keep the content locally so it survives
        // a reload and is re-synced when the browser reconnects.
        if (window.GBOfflineDraft) window.GBOfflineDraft.save(docDraftKey(docId), content);
        setSaveStatus("Offline — saved locally", true);
        if (window.GBOfflineDraft) {
          window.GBOfflineDraft.showBanner("You're offline — changes are saved locally");
        }
      });
  }, SAVE_DEBOUNCE_MS);
}

/* Restore a local draft (or flush it back to the server after reconnect). */
function restoreDocDraft() {
  var article = getActiveArticle();
  if (!article) return;
  var docId = (typeof docsEnsureArticleDocId === "function")
    ? docsEnsureArticleDocId(article)
    : (article.dataset ? article.dataset.docId : "");
  if (!docId || docId === "current" || !window.GBOfflineDraft) return;
  var draft = window.GBOfflineDraft.load(docDraftKey(docId));
  if (!draft) return;
  if (window.GBOfflineDraft.isOnline()) {
    // Reconnected: push the draft to the server, then clear it.
    fetch("/api/docs/autosave", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: docId, content: draft.c })
    })
      .then(function (r) { if (!r.ok) throw new Error("http " + r.status); return r.json(); })
      .then(function () {
        window.GBOfflineDraft.clear(docDraftKey(docId));
        window.GBOfflineDraft.hideBanner();
        setSaveStatus("All changes saved", false);
      })
      .catch(function () { setSaveStatus("Still offline — saved locally", true); });
  } else {
    // Still offline: restore the draft into the editor so nothing is lost.
    article.innerHTML = draft.c;
    scheduleSave(docId, draft.c);
    if (window.updatePageCount) window.updatePageCount();
  }
}

function checkDocDraftOnLoad() {
  if (!window.GBOfflineDraft) return;
  var article = getActiveArticle();
  if (!article) return;
  var docId = (typeof docsEnsureArticleDocId === "function")
    ? docsEnsureArticleDocId(article)
    : (article.dataset ? article.dataset.docId : "");
  if (!docId || docId === "current") return;
  if (!window.GBOfflineDraft.has(docDraftKey(docId))) return;
  window.GBOfflineDraft.showBanner(
    "You have unsaved changes from a previous session",
    { sticky: true, actionLabel: "Restore", onAction: restoreDocDraft }
  );
  window.GBOfflineDraft.onReconnect(restoreDocDraft);
}

var DocumentAIDriver = {
  summarize: function (id) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "summarize" })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.content || j.response || (j.result && j.result !== "success" ? j.result : ""); }).catch(function () { return ""; });
  },
  expand: function (id, prompt) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "expand", prompt: prompt })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.content || j.response || (j.result && j.result !== "success" ? j.result : ""); }).catch(function () { return ""; });
  },
  improve: function (id) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "improve" })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.content || j.response || (j.result && j.result !== "success" ? j.result : ""); }).catch(function () { return ""; });
  },
  simplify: function (id) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "simplify" })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.content || j.response || (j.result && j.result !== "success" ? j.result : ""); }).catch(function () { return ""; });
  }
};

// Toolbar buttons in docs.html address the active document as `current`; the
// real id lives on the rendered view. Rewrite the parameter once per HTMX
// request so undo/redo, TOC, track-changes and footnote actions target the
// document the user is editing instead of a literal "current".
document.addEventListener("htmx:configRequest", function (event) {
  var parameters = (event.detail && event.detail.parameters) || {};
  // Sidebar panel fragments compute outline/TOC from the live editor HTML;
  // the stored copy lags behind the autosave debounce and a Drive-opened
  // document may not be stored yet.
  var elt = event.detail && event.detail.elt;
  var path = (elt && elt.getAttribute && elt.getAttribute("hx-post")) || "";
  if (path.indexOf("/suite/docs/fragments/") === 0) {
    var scope = elt && elt.closest ? (elt.closest("[id^=window-body-docs]") || document) : document;
    var article = scope.querySelector("#doc-content");
    if (article) parameters.content = article.innerHTML;
  }
  var keys = ["id", "doc_id", "document_id"];
  var needsRewrite = keys.some(function (key) { return parameters[key] === "current"; });
  if (!needsRewrite) return;
  var view = document.querySelector("#doc-view[data-id]");
  var article = document.querySelector("article[data-doc-id]");
  var docId = (view && view.getAttribute("data-id")) ||
    (article && article.dataset ? article.dataset.docId : "") || "";
  if (!docId || docId === "current") {
    if (article && typeof docsEnsureArticleDocId === "function") {
      docId = docsEnsureArticleDocId(article);
    }
  }
  if (!docId || docId === "current") return;
  keys.forEach(function (key) {
    if (parameters[key] === "current") parameters[key] = docId;
  });
});
