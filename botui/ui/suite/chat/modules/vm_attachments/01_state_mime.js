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
  if (mime.indexOf("audio/") === 0) return "audio";
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

// Content-type badge label for an attachment (short, uppercase extension).
function vmContentTypeBadge(path) {
  var ext = String(path || "").split(".").pop().toUpperCase();
  return ext && ext !== String(path || "").toUpperCase() ? ext : "FILE";
}

// Minor descriptive text for an attachment (rendered in italic).
function vmAttachmentDescription(path) {
  var mime = vmFileMimeType(path);
  if (mime.indexOf("image/") === 0) return "Image file";
  if (mime === "application/pdf") return "PDF document";
  if (mime.indexOf("officedocument") !== -1) return "Office document";
  if (mime === "application/msword") return "Word document";
  if (mime === "application/vnd.ms-excel") return "Spreadsheet";
  if (mime === "application/vnd.ms-powerpoint") return "Presentation";
  if (mime.indexOf("text/") === 0) return "Text file";
  if (mime.indexOf("video/") === 0) return "Video file";
  if (mime.indexOf("audio/") === 0) return "Audio file";
  if (mime === "application/zip") return "Archive";
  return "Generated file";
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
  if (!vmOfficeAppAvailable(app)) {
    throw new Error("this edition does not include the " + app + " app");
  }
  if (!window.WindowManager || !window.WindowManager.openDeepLink) {
    throw new Error("desktop shell unavailable");
  }
  window.WindowManager.openDeepLink(app, { file: target.key, bucket: target.bucket });
  // #2169/#2171 — side-by-side: chat on the left, document on the right.
  // Snap after a short tick so the deep-linked window has been created.
  setTimeout(function () {
    try {
      if (window.WindowManager && typeof window.WindowManager.snapWindow === "function") {
        if (window.WindowManager.getWindow && window.WindowManager.getWindow("chat")) {
          window.WindowManager.snapWindow("chat", "left");
        }
        window.WindowManager.snapWindow(app, "right");
      }
      // #2171 — return focus to the chat composer.
      var input = document.querySelector("#chat-app textarea, .chat-input textarea, #messageInput");
      if (input && typeof input.focus === "function") input.focus({ preventScroll: true });
    } catch (e) {}
  }, 120);
}

// #2184 — open a VM (agent workspace) office file directly in its editor
// without requiring a prior publish to Drive. The docs/sheet/slides editors
// gain a VM mode that fetches the converted preview from the agent VM.
function vmOpenInOfficeAppVm(path, convId, vmPath) {
  var app = vmOfficeAppFor(path);
  if (!app) throw new Error("this file has no editor app");
  if (!vmOfficeAppAvailable(app)) {
    throw new Error("this edition does not include the " + app + " app");
  }
  if (!window.WindowManager || !window.WindowManager.openDeepLink) {
    throw new Error("desktop shell unavailable");
  }
  window.WindowManager.openDeepLink(app, { vmConvId: convId || "", vmPath: vmPath || ("output/" + String(path || "").split("/").pop()) });
  setTimeout(function () {
    try {
      if (window.WindowManager && typeof window.WindowManager.snapWindow === "function") {
        if (window.WindowManager.getWindow && window.WindowManager.getWindow("chat")) {
          window.WindowManager.snapWindow("chat", "left");
        }
        window.WindowManager.snapWindow(app, "right");
      }
      var input = document.querySelector("#chat-app textarea, .chat-input textarea, #messageInput");
      if (input && typeof input.focus === "function") input.focus({ preventScroll: true });
    } catch (e) {}
  }, 120);
}
/* The Save-to-Drive actions call /api/agent/drive/save, which botserver only
   registers with the agent-vm feature. Without this guard the button rendered
   in every build and failed with HTTP 404. */
function vmAgentDriveEnabled() {
  var cfg = window.productConfig;
  if (!cfg || !Array.isArray(cfg.compiled_features)) return false;
  return cfg.compiled_features.some(function (feature) {
    return String(feature).toLowerCase() === "agent-vm";
  });
}
