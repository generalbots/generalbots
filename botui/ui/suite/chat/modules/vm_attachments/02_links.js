"use strict";

// VM file attachments — vmfile anchor rendering, signed URLs and error toast.
// Split out of the former 70_vm_attachments module for #1731 (450-line limit).
// Plain global script: every function stays a page-level global, exactly
// as before the split. Load order follows this directory's name order.

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
      ">" +
      '<span class="gb-attachment-icon">' +
      vmFileIcon(parts.path) +
      "</span>" +
      '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener">' +
      vmEscapeHtml(name) +
      "</a>" +
      '<button type="button" class="gb-attachment-btn gb-attachment-preview">Preview</button>' +
      (vmOfficeAppFor(parts.path)
        ? '<button type="button" class="gb-attachment-btn gb-attachment-open">Open</button>'
        : "") +
      '<button type="button" class="gb-attachment-btn gb-attachment-download">Download</button>' +
      '<button type="button" class="gb-attachment-btn gb-attachment-save">Save</button>' +
      "</span>"
    );
  });
}

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
