"use strict";

// BBSIA-CHAT-02 (#1582): render VM files as chat attachments with preview and
// download buttons. Bot replies reference generated files as
// [name](vmfile:CONVERSATION_ID/output/name) (agent system prompt). When the
// model omits the links, the conversation output/ folder is listed as a
// fallback so attachments still reach the user.

var VmAttachments = {
  conversationId: null,
  renderedPaths: {},
};

function vmAttachmentsToken() {
  try {
    return (
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  } catch (e) {
    return "";
  }
}

function vmAttachmentsRequest(url) {
  var token = vmAttachmentsToken();
  var headers = token ? { Authorization: "Bearer " + token } : {};
  return fetch(url, { headers: headers }).then(function (res) {
    if (!res.ok) throw new Error("HTTP " + res.status);
    return res.json();
  });
}

function vmActiveConversationId() {
  // Always refresh: the active conversation switches to the current chat
  // session on the first message, so a cached id would list the wrong folder.
  return vmAttachmentsRequest("/api/agent/vm/status")
    .then(function (data) {
      if (data && data.has_vm && data.active_conversation_id) {
        VmAttachments.conversationId = data.active_conversation_id;
        return VmAttachments.conversationId;
      }
      return VmAttachments.conversationId;
    })
    .catch(function () {
      return VmAttachments.conversationId;
    });
}

function vmEscapeHtml(value) {
  return String(value == null ? "" : value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function vmFileMimeType(path) {
  var ext = String(path || "").split(".").pop().toLowerCase();
  var types = {
    pdf: "application/pdf",
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    gif: "image/gif",
    webp: "image/webp",
    svg: "image/svg+xml",
    txt: "text/plain",
    csv: "text/csv",
    json: "application/json",
    md: "text/markdown",
    html: "text/html",
    zip: "application/zip",
    mp4: "video/mp4",
    mp3: "audio/mpeg",
    xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    xls: "application/vnd.ms-excel",
    docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    doc: "application/msword",
    pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    ppt: "application/vnd.ms-powerpoint",
  };
  return types[ext] || "application/octet-stream";
}

function vmFilePreviewable(path) {
  var mime = vmFileMimeType(path);
  return (
    mime === "application/pdf" ||
    mime === "text/html" ||
    mime === "text/plain" ||
    mime === "text/csv" ||
    mime === "text/markdown" ||
    mime === "application/json" ||
    mime.indexOf("image/") === 0 ||
    mime.indexOf("officedocument") !== -1 ||
    mime === "application/msword" ||
    mime === "application/vnd.ms-excel" ||
    mime === "application/vnd.ms-powerpoint"
  );
}

// #1672 — preview kinds: image renders as <img>, pdf/html as a sandboxed
// iframe, office renders inline through the server conversion endpoint
// (`preview=1`, sanitized script-free HTML with sheet/slide sections), with
// the window preview kept as a secondary action.
function vmInlineKind(path) {
  var mime = vmFileMimeType(path);
  if (mime.indexOf("image/") === 0) return "image";
  if (mime === "application/pdf") return "pdf";
  if (mime === "text/html") return "html";
  if (
    mime === "text/plain" ||
    mime === "text/csv" ||
    mime === "text/markdown" ||
    mime === "application/json"
  ) {
    return "text";
  }
  // The server-side OOXML converter only handles docx/xlsx/pptx. Legacy
  // .doc/.xls/.ppt and ODF/RTF returned HTTP 415 inside the iframe, so they
  // take the honest "no inline preview" path (Download/Open still work).
  var extension = String(path || "").split(".").pop().toLowerCase();
  if (extension === "docx" || extension === "xlsx" || extension === "pptx") {
    return "office";
  }
  return "other";
}

function vmFileIcon(path) {
  var mime = vmFileMimeType(path);
  if (mime === "application/pdf") return "PDF";
  if (mime.indexOf("image/") === 0) return "IMG";
  if (mime === "video/mp4") return "VID";
  if (mime === "audio/mpeg") return "AUD";
  return "FILE";
}

function vmSplitTarget(target) {
  var value = String(target || "");
  try {
    value = decodeURIComponent(value);
  } catch (e) {}
  var slash = value.indexOf("/");
  if (slash === -1) return { conversationId: null, path: value };
  var head = value.slice(0, slash);
  var rest = value.slice(slash + 1);
  if (head === "output" || head === "program" || head === "data") {
    return { conversationId: null, path: value };
  }
  return { conversationId: head, path: rest };
}

// #1711 — the app that can edit an Office artifact produced in chat.
function vmOfficeAppFor(path) {
  var ext = String(path || "").split(".").pop().toLowerCase();
  // Only formats the editors can actually open: the docs converter reads
  // docx (not legacy .doc/ODF/RTF) and slides imports pptx (not OLE .ppt).
  if (ext === "docx") return "docs";
  if (["xlsx", "xls", "ods", "csv"].indexOf(ext) !== -1) return "sheet";
  if (ext === "pptx") return "slides";
  return null;
}

// drive_bridge::save_object returns "bucket/key"; split it for the app
// deep link ({file, bucket}).
function vmSplitDrivePath(savedPath) {
  var value = String(savedPath || "");
  var slash = value.indexOf("/");
  if (slash === -1) return null;
  return { bucket: value.slice(0, slash), key: value.slice(slash + 1) };
}

function vmOpenInOfficeApp(path, savedPath) {
  var app = vmOfficeAppFor(path);
  var target = vmSplitDrivePath(savedPath);
  if (!app || !target) throw new Error("this file has no editor app");
  if (!window.WindowManager || !window.WindowManager.openDeepLink) {
    throw new Error("desktop shell unavailable");
  }
  window.WindowManager.openDeepLink(app, { file: target.key, bucket: target.bucket });
}
