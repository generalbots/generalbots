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
function docDraftKey(id) { return "docs:" + (id || "current"); }
function scheduleSave(id, content) {
  clearTimeout(saveTimer);
  setSaveStatus("Saving...", false);
  saveTimer = setTimeout(function () {
    fetch("/api/docs/autosave", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id || "current", content: content })
    })
      .then(function (r) { if (!r.ok) throw new Error("http " + r.status); return r.json(); })
      .then(function () {
        setSaveStatus("All changes saved", false);
        if (window.GBOfflineDraft) window.GBOfflineDraft.clear(docDraftKey(id));
        if (window.GBCollabActivity) {
          window.GBCollabActivity.record({
            resourceType: "docs",
            resourceId: String(id || "current"),
            action: "edit",
            payload: {}
          }).catch(function () {});
        }
        if (window.GBCollabVersions) {
          window.GBCollabVersions.snapshot({
            resourceType: "docs",
            resourceId: String(id || "current"),
            content: content
          }).catch(function () {});
        }
      })
      .catch(function () {
        // Offline (or server error): keep the content locally so it survives
        // a reload and is re-synced when the browser reconnects.
        if (window.GBOfflineDraft) window.GBOfflineDraft.save(docDraftKey(id), content);
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
  var docId = article.dataset.docId || "current";
  if (!window.GBOfflineDraft) return;
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
  var docId = article.dataset.docId || "current";
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
    }).then(function (r) { return r.json(); }).then(function (j) { return j.result || j.content || ""; }).catch(function () { return ""; });
  },
  expand: function (id, prompt) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "expand", prompt: prompt })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.result || ""; }).catch(function () { return ""; });
  },
  improve: function (id) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "improve" })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.result || ""; }).catch(function () { return ""; });
  },
  simplify: function (id) {
    return fetch("/api/docs/ai", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id, action: "simplify" })
    }).then(function (r) { return r.json(); }).then(function (j) { return j.result || ""; }).catch(function () { return ""; });
  }
};
