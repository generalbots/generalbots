"use strict";

// VM file attachments — vmfile anchor rendering, signed URLs and error toast.
// Split out of the former 70_vm_attachments module for #1731 (450-line limit).
// Plain global script: every function stays a page-level global, exactly
// as before the split. Load order follows this directory's name order.

// #1834 — the Open action is gated by the product's app list, not by the raw
// agent-vm flag. An edition that ships the Sheet app can open an xlsx even
// when the Drive publish step is unavailable (the handler falls back to the
// server-rendered preview in that case).
function vmOfficeAppAvailable(app) {
  if (!app) return false;
  var cfg = window.productConfig;
  if (!cfg || !Array.isArray(cfg.apps)) return true;
  var apps = cfg.apps.map(function (entry) {
    return String(entry).toLowerCase();
  });
  return apps.indexOf(String(app).toLowerCase()) !== -1;
}

// One source of truth for the attachment actions so the inline-link, folder
// and hydration card paths cannot drift apart again (#1834).
function vmAttachmentButtons(path) {
  var app = vmOfficeAppFor(path);
  var open =
    app && vmOfficeAppAvailable(app)
      ? '<button type="button" class="gb-attachment-btn gb-attachment-open" title="Open in app">Open in app</button>'
      : "";
  return (
    '<button type="button" class="gb-attachment-btn gb-attachment-preview">Preview</button>' +
    open +
    '<button type="button" class="gb-attachment-btn gb-attachment-download">Download</button>' +
    (vmAgentDriveEnabled()
      ? '<button type="button" class="gb-attachment-btn gb-attachment-save">Save</button>'
      : "")
  );
}

// Replace <a href="vmfile:..."> anchors produced by marked with attachment
// cards. Runs on the rendered HTML before it is inserted in the message.
function renderVmFileLinks(html) {
  if (!html || html.indexOf("vmfile:") === -1) return html;
  var pattern = /<a\s+[^>]*href="vmfile:([^"]+)"[^>]*>([\s\S]*?)<\/a>/gi;
  return html.replace(pattern, function (match, target, label) {
    var parts = vmSplitTarget(target);
    var text = String(label || "").replace(/<[^>]*>/g, "").trim();
    var name = text || parts.path.split("/").pop();
    var convAttr = parts.conversationId
      ? ' data-conversation-id="' + vmEscapeHtml(parts.conversationId) + '"'
      : "";
    return (
      '<span class="gb-attachment" data-path="' +
      vmEscapeHtml(parts.path) +
      '"' +
      convAttr +
      ' data-content-type="' +
      vmEscapeHtml(vmFileMimeType(parts.path)) +
      '" data-order="0">' +
      '<span class="gb-attachment-icon">' +
      vmFileIcon(parts.path) +
      "</span>" +
      '<span class="gb-attachment-order">0</span>' +
      '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener">' +
      vmEscapeHtml(name) +
      "</a>" +
      '<span class="gb-content-type-badge gb-type-badge">' +
      vmEscapeHtml(vmContentTypeBadge(parts.path)) +
      "</span>" +
      '<span class="gb-attachment-desc gb-attachment-minor">' +
      vmEscapeHtml(vmAttachmentDescription(parts.path)) +
      "</span>" +
      '<span class="gb-latest-chip gb-attachment-latest">Latest</span>' +
      vmAttachmentButtons(parts.path) +
      "</span>"
    );
  });
}

// Hard invariant (#2167): a `vmfile:` anchor must never be visible as a link.
// Every renderer is supposed to convert them before inserting, so anything
// caught here is a renderer that skipped its pass — the console warning names
// it so the gap gets fixed upstream instead of silently patched downstream.
function enforceNoVmFileLinks(root) {
  if (!root || !root.querySelectorAll) return 0;
  var anchors = root.querySelectorAll('a[href^="vmfile:"]');
  var fixed = 0;
  for (var i = 0; i < anchors.length; i++) {
    var anchor = anchors[i];
    var wrapper = document.createElement("div");
    wrapper.innerHTML = renderVmFileLinks(anchor.outerHTML);
    if (wrapper.firstElementChild) {
      anchor.replaceWith(wrapper.firstElementChild);
      fixed++;
    }
  }
  if (fixed && window.console && console.warn) {
    console.warn(
      "[vmfile] converted " + fixed + " raw vmfile anchor(s) at DOM insertion;" +
        " the renderer should have called renderVmFileLinks",
    );
  }
  return fixed;
}
window.enforceNoVmFileLinks = enforceNoVmFileLinks;

// A real, browser-consumable URL. The endpoint also accepts the suite token
// as a query parameter, so a plain anchor/iframe can fetch the file without
// the blob dance (which made the name look like a dead link).
function vmAttachmentUrl(conversationId, path) {
  var token = vmAttachmentsToken();
  var url =
    "/api/agent/vm/file?conversation_id=" +
    encodeURIComponent(conversationId) +
    "&path=" +
    encodeURIComponent(path) +
    "&raw=1";
  if (token) url += "&access_token=" + encodeURIComponent(token);
  return url;
}

// Office conversion endpoint: same signed pipeline, server-rendered and
// sanitized (no scripts, no frames, no external fetch) (#1672).
function vmOfficePreviewUrl(conversationId, path) {
  var token = vmAttachmentsToken();
  var url =
    "/api/agent/vm/file?conversation_id=" +
    encodeURIComponent(conversationId) +
    "&path=" +
    encodeURIComponent(path) +
    "&preview=1";
  if (token) url += "&access_token=" + encodeURIComponent(token);
  return url;
}

function vmNotifyError(err) {
  var message =
    "Attachment failed: " + (err && err.message ? err.message : "unknown error");
  // window.showToast does not exist on the desktop shell; GBToasts does.
  // Without this order every attachment failure was console-only.
  if (window.GBToasts && GBToasts.show) GBToasts.show("Attachment", message, "error");
  else if (window.showToast) window.showToast(message, "error");
  else console.error(message);
}

// #1834 — use the native Save dialog in the desktop WebView; the plain
// anchor is the web-build fallback.
function vmDownload(url, fileName) {
  if (window.GBShared && typeof window.GBShared.download === "function") {
    return window.GBShared.download(url, fileName);
  }
  var link = document.createElement("a");
  link.href = url;
  link.download = fileName || "";
  document.body.appendChild(link);
  link.click();
  link.remove();
  return Promise.resolve(null);
}
