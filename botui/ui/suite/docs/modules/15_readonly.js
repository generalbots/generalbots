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

/* #2452 criterion 1 — optional expiry on a Docs share link.
 *
 * The share dialog itself is the shared GBCollabShare component (owned by the
 * shell, not by Docs), so this Docs adapter extends that dialog rather than
 * duplicating it. The chosen instant is sent as `expires_at` (RFC3339) and,
 * for the current API which predates that field, as `expires_in_hours`.
 * Leaving the input empty keeps the original no-expiry behaviour.
 *
 * There is no docs-share-specific expiry key in the catalogs. The two keys
 * below are existing generic "optional expiration" strings present in all
 * eight locales; no new user-facing literal is introduced.
 */
var DOCS_EXPIRY_LABEL_KEY = "admin-roles-expiration-optional";
var DOCS_EXPIRY_LABEL_FALLBACK = "Expiration (Optional)";
var DOCS_EXPIRY_HINT_KEY = "admin-roles-expiration-help";
var DOCS_EXPIRY_HINT_FALLBACK = "Leave empty for permanent assignment";
var docsShareExpiryInstalled = false;
var docsShareExpiryPending = "";

function docsShareExpiryInput() {
  return document.getElementById("gbs-docs-expiry");
}

function docsShareExpiryError(message) {
  var el = document.getElementById("gbs-error");
  if (!el) return;
  el.setAttribute("role", "status");
  el.setAttribute("aria-live", "polite");
  el.textContent = message;
  el.style.display = message ? "block" : "none";
}

/** RFC3339 instant from the input, or "" when empty/unparseable. */
function docsShareExpiryIso() {
  var input = docsShareExpiryInput();
  var raw = input && input.value ? String(input.value).trim() : "";
  if (!raw) return "";
  var when = new Date(raw);
  return isFinite(when.getTime()) ? when.toISOString() : "";
}

function docsInstallShareExpiry() {
  if (docsShareExpiryInstalled) return;
  var shared = window.GBCollabShare;
  if (!shared || typeof shared.open !== "function") return;
  docsShareExpiryInstalled = true;

  var originalOpen = shared.open;
  var originalFetch = window.fetch;
  var current = null;

  // Capture phase runs before the dialog's own create-link listener, so an
  // invalid date can stop the request entirely instead of minting a link
  // without the expiry the user asked for.
  function onShareClick(event) {
    var target = event.target;
    if (!current || current.resourceType !== "docs") return;
    if (!target || target.id !== "gbs-create-link") return;
    var input = docsShareExpiryInput();
    var raw = input && input.value ? String(input.value).trim() : "";
    docsShareExpiryPending = "";
    if (!raw) return;
    var iso = docsShareExpiryIso();
    if (!iso) {
      event.preventDefault();
      event.stopImmediatePropagation();
      docsShareExpiryError(
        docsT("error-invalid-value", "The supplied value could not be parsed.")
      );
      return;
    }
    if (Date.parse(iso) <= Date.now()) {
      event.preventDefault();
      event.stopImmediatePropagation();
      docsShareExpiryError(
        docsT("error-validation-date-future", "{ $field } must be in the future.", {
          field: docsT(DOCS_EXPIRY_LABEL_KEY, DOCS_EXPIRY_LABEL_FALLBACK),
        })
      );
      return;
    }
    docsShareExpiryPending = iso;
  }

  // The shared dialog owns `api()` and the POST body; this wrapper only adds
  // the expiry to that one request and consumes the one-shot value at once.
  function linkFetch(input, init) {
    var url = typeof input === "string" ? input : ((input && input.url) || "");
    var isLinkPost = !!(init && init.method === "POST" && url.indexOf("/api/collab/links") !== -1);
    if (!isLinkPost) return originalFetch(input, init);
    var iso = docsShareExpiryPending;
    docsShareExpiryPending = "";
    if (iso && typeof init.body === "string") {
      var body = null;
      try {
        body = JSON.parse(init.body);
      } catch (error) {
        if (window.gbLogError) window.gbLogError(error, "docsShareExpiry");
        else console.error("docsShareExpiry", error);
      }
      if (body && body.resource_type === "docs") {
        if (!body.expires_at) body.expires_at = iso;
        if (!body.expires_in_hours) {
          var hours = Math.ceil((Date.parse(iso) - Date.now()) / 3600000);
          if (hours > 0) body.expires_in_hours = hours;
        }
        init = Object.assign({}, init, { body: JSON.stringify(body) });
      }
    }
    return originalFetch(input, init);
  }

  shared.open = function (opts) {
    current = opts || {};
    originalOpen.call(shared, opts);
    if (current.resourceType !== "docs") return;
    var dialog = document.getElementById("gb-share-dialog");
    if (!dialog || docsShareExpiryInput()) return;
    var create = dialog.querySelector("#gbs-create-link");
    if (!create || !create.parentNode) return;

    var row = document.createElement("div");
    row.className = "gbs-row";
    var label = document.createElement("label");
    label.setAttribute("for", "gbs-docs-expiry");
    label.setAttribute("data-i18n", DOCS_EXPIRY_LABEL_KEY);
    label.textContent = docsT(DOCS_EXPIRY_LABEL_KEY, DOCS_EXPIRY_LABEL_FALLBACK);
    var input = document.createElement("input");
    input.id = "gbs-docs-expiry";
    input.type = "datetime-local";
    input.setAttribute("aria-describedby", "gbs-docs-expiry-hint");
    var hint = document.createElement("span");
    hint.id = "gbs-docs-expiry-hint";
    hint.className = "gbs-hint";
    hint.setAttribute("data-i18n", DOCS_EXPIRY_HINT_KEY);
    hint.textContent = docsT(DOCS_EXPIRY_HINT_KEY, DOCS_EXPIRY_HINT_FALLBACK);
    row.appendChild(label);
    row.appendChild(input);
    row.appendChild(hint);
    create.parentNode.insertBefore(row, create);
    dialog.addEventListener("click", onShareClick, true);
  };

  window.fetch = linkFetch;
}

docsInstallShareExpiry();
