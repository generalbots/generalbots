"use strict";
/* Docs read-only state (#2253).
 *
 * A document opened through a share link is a view, not an editing session:
 * the token is the only credential, and the server never accepts a write for
 * it. The app must say so, otherwise the reader types into a contenteditable
 * that silently fails to save.
 *
 * Public API:
 *   docsIsReadOnly()           — is this window in read-only mode?
 *   docsApplyReadOnly(reason)  — switch the window to read-only and show it
 *   refreshDocsShareState()    — mark the Share control when a link is active
 */

var DOCS_READONLY_CLASS = "docs-readonly";

function docsIsReadOnly() {
  var host = document.getElementById("docs-app");
  return !!(host && host.classList.contains(DOCS_READONLY_CLASS));
}

function docsReadOnlyBadge() {
  var badge = document.getElementById("docs-readonly-badge");
  if (badge) return badge;
  var toolbar = document.querySelector(".docs-toolbar-row-main .toolbar-right");
  if (!toolbar) return null;
  badge = document.createElement("span");
  badge.id = "docs-readonly-badge";
  badge.className = "docs-readonly-badge";
  badge.textContent = docsT("docs-readonly", "Read-only");
  badge.title = docsT(
    "docs-readonly-hint",
    "This document was shared as a read-only link."
  );
  var share = document.getElementById("shareBtn");
  if (share && share.parentNode === toolbar) toolbar.insertBefore(badge, share);
  else toolbar.appendChild(badge);
  return badge;
}

/**
 * Switches the window to read-only.
 *
 * `reason` is `shared-link` for a token view; it is kept on the app element so
 * a debugging session can tell a link view from any future read-only grant.
 */
function docsApplyReadOnly(reason) {
  var host = document.getElementById("docs-app");
  if (!host) return;
  host.classList.add(DOCS_READONLY_CLASS);
  host.setAttribute("data-readonly-reason", reason || "shared-link");

  Array.prototype.forEach.call(
    document.querySelectorAll("#docs-content article"),
    function (article) {
      article.setAttribute("contenteditable", "false");
    }
  );

  var title = document.getElementById("docTitle");
  if (title) {
    title.readOnly = true;
    title.setAttribute("aria-readonly", "true");
  }

  // Editing controls do not apply to a reader: disable them instead of
  // leaving buttons that fail silently.
  Array.prototype.forEach.call(
    document.querySelectorAll(
      ".docs-toolbar button, .docs-toolbar select, .docs-toolbar input"
    ),
    function (control) {
      control.disabled = true;
    }
  );

  docsReadOnlyBadge();
  if (typeof setSaveStatus === "function") {
    setSaveStatus(docsT("docs-readonly-status", "Read-only — shared link"), false);
  }
  // The sidebar lists the reader's own documents; a link view has none.
  var files = document.getElementById("sidebar-files");
  if (files) files.innerHTML = "";
}

/**
 * Reflects "a read-only link is active" on the Share control, so the toolbar
 * answers the same question the dialog does (UAT DOC-10, criterion 3).
 */
function refreshDocsShareState() {
  var btn = document.getElementById("shareBtn");
  if (!btn || docsIsReadOnly()) return;
  if (!window.GBCollabShare || typeof window.GBCollabShare.links !== "function") return;
  var article = document.querySelector("#docs-content article");
  var docId = article && article.dataset ? article.dataset.docId : "";
  if (!docId || docId === "current") return;
  window.GBCollabShare.links("docs", String(docId)).then(function (links) {
    var active = (links || []).filter(function (link) {
      return link && link.token;
    });
    if (!active.length) return;
    btn.classList.add("gb-share-active");
    btn.setAttribute("data-shared", "true");
    btn.title = docsT(
      "docs-share-link-active",
      "Shared — a read-only link is active"
    );
    btn.setAttribute("aria-label", btn.title);
  });
}
