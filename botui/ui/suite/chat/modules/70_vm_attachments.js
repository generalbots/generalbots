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
  return mime === "application/pdf" || mime.indexOf("image/") === 0;
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
      '<a class="gb-attachment-name" href="#" download>' +
      vmEscapeHtml(name) +
      "</a>" +
      '<button type="button" class="gb-attachment-btn gb-attachment-preview">Preview</button>' +
      '<button type="button" class="gb-attachment-btn gb-attachment-download">Download</button>' +
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

function vmNotifyError(err) {
  var message =
    "Attachment failed: " + (err && err.message ? err.message : "unknown error");
  if (window.showToast) window.showToast(message, "error");
  else console.error(message);
}

function vmShowPreviewUrl(url, path) {
  var mime = vmFileMimeType(path);
  var overlay = document.getElementById("gb-vm-preview");
  if (!overlay) {
    overlay = document.createElement("div");
    overlay.id = "gb-vm-preview";
    overlay.className = "gb-vm-preview";
    overlay.addEventListener("click", function (e) {
      if (e.target === overlay || e.target.classList.contains("gb-vm-preview-close")) {
        overlay.remove();
        if (overlay._gbUrl) URL.revokeObjectURL(overlay._gbUrl);
      }
    });
    document.body.appendChild(overlay);
  }
  var name = vmEscapeHtml(path.split("/").pop() || "file");
  var body =
    mime.indexOf("image/") === 0
      ? '<img class="gb-vm-preview-body" src="' + url + '" alt="' + name + '">'
      : '<iframe class="gb-vm-preview-body" src="' + url + '" title="' + name + '"></iframe>';
  overlay.innerHTML =
    '<div class="gb-vm-preview-head"><span>' + name + "</span>" +
    '<a class="gb-vm-preview-download" download="' + name + '" href="' + url + '">Download</a>' +
    '<button type="button" class="gb-vm-preview-close" aria-label="Close preview">&times;</button></div>' +
    body;
  overlay._gbUrl = url;
}

function vmBindAttachment(card) {
  if (!card || card.dataset.bound === "1") return;
  card.dataset.bound = "1";
  var path = card.dataset.path;
  var explicitConv = card.dataset.conversationId || "";
  var resolve = function () {
    return explicitConv ? Promise.resolve(explicitConv) : vmActiveConversationId();
  };
  var nameLink = card.querySelector(".gb-attachment-name");
  var fileName = String(path || "").split("/").pop() || "file";
  var urlFor = function () {
    return resolve().then(function (convId) {
      if (!convId) throw new Error("No active VM");
      var url = vmAttachmentUrl(convId, path);
      if (nameLink && nameLink.tagName === "A") {
        nameLink.href = url;
        nameLink.setAttribute("download", fileName);
      }
      return url;
    });
  };
  // Point the link as soon as the conversation is known, so the file name is
  // a working hyperlink before the user clicks anything.
  urlFor().catch(function () {
    /* resolved again on click */
  });

  var download = card.querySelector(".gb-attachment-download");
  var preview = card.querySelector(".gb-attachment-preview");
  if (download) {
    download.addEventListener("click", function (e) {
      e.preventDefault();
      urlFor()
        .then(function (url) {
          var link = document.createElement("a");
          link.href = url;
          link.download = fileName;
          document.body.appendChild(link);
          link.click();
          link.remove();
        })
        .catch(vmNotifyError);
    });
  }
  if (preview) {
    preview.addEventListener("click", function (e) {
      e.preventDefault();
      resolve()
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          // A dedicated window keeps polling while the file is still being
          // written (PDF/HTML), and survives chat navigation.
          if (window.openDeepLink && window.WindowManager) {
            window.openDeepLink("preview", { conversation: convId, path: path });
            return;
          }
          window.open(vmAttachmentUrl(convId, path), "_blank", "noopener");
        })
        .catch(vmNotifyError);
    });
  }
}

function vmAppendCard(wrapper, conversationId, path, name) {
  var card = document.createElement("span");
  card.className = "gb-attachment";
  card.dataset.path = path;
  card.dataset.conversationId = conversationId || "";
  card.innerHTML =
    '<span class="gb-attachment-icon"></span>' +
    '<a class="gb-attachment-name" href="#" download></a>' +
    '<button type="button" class="gb-attachment-btn gb-attachment-preview">Preview</button>' +
    '<button type="button" class="gb-attachment-btn gb-attachment-download">Download</button>';
  card.querySelector(".gb-attachment-icon").textContent = vmFileIcon(path);
  card.querySelector(".gb-attachment-name").textContent = name;
  wrapper.appendChild(card);
  vmBindAttachment(card);
  return card;
}

// The conversation the user is looking at, not the VM's last active folder.
function vmCurrentConversationId() {
  try {
    if (window.GBTabs && GBTabs.activeTab) {
      var tab = GBTabs.activeTab();
      if (tab && tab.sessionId) return Promise.resolve(tab.sessionId);
    }
    if (window.ChatState && ChatState.currentSessionId) {
      return Promise.resolve(ChatState.currentSessionId);
    }
  } catch (e) { /* fall through */ }
  return vmActiveConversationId();
}

function vmVisiblePane() {
  var panes = document.querySelectorAll(".chat-pane-messages, #messages");
  for (var i = 0; i < panes.length; i++) {
    var rect = panes[i].getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) return panes[i];
  }
  return null;
}

function vmAttachmentAnchor(container) {
  if (container && container.isConnected) {
    var rect = container.getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) return container;
  }
  // The reply may render into a hidden chat tab; attach the manifest to the
  // last visible bot message so the active pane always shows the files.
  var pane = vmVisiblePane();
  if (!pane) return null;
  var bots = pane.querySelectorAll(".message.bot");
  return bots.length ? bots[bots.length - 1] : null;
}

// Bind inline vmfile cards and append output/ files the model did not link.
function hydrateVmAttachments(container) {
  if (!container) return;
  var cards = container.querySelectorAll(".gb-attachment");
  for (var i = 0; i < cards.length; i++) vmBindAttachment(cards[i]);

  var anchor = vmAttachmentAnchor(container);
  if (!anchor) return;

  vmCurrentConversationId()
    .then(function (conversationId) {
      if (!conversationId) return;
      var seen = VmAttachments.renderedPaths[conversationId] || {};
      var present = {};
      var existing = anchor.querySelectorAll(".gb-attachment");
      for (var j = 0; j < existing.length; j++) {
        present[existing[j].dataset.path] = true;
        seen[existing[j].dataset.path] = true;
      }
      return vmAttachmentsRequest(
        "/api/agent/vm/files?conversation_id=" + encodeURIComponent(conversationId)
      ).then(function (data) {
        var files = (data && data.files) || [];
        var missing = files.filter(function (name) {
          var path = "output/" + name;
          return !seen[path] && !present[path];
        });
        if (!missing.length) {
          VmAttachments.renderedPaths[conversationId] = seen;
          return;
        }
        if (!anchor.isConnected) return;
        var wrapper = document.createElement("div");
        wrapper.className = "gb-attachments";
        missing.forEach(function (name) {
          var path = "output/" + name;
          vmAppendCard(wrapper, conversationId, path, name);
          seen[path] = true;
        });
        VmAttachments.renderedPaths[conversationId] = seen;
        var target = anchor.querySelector(".message-content") || anchor;
        target.appendChild(wrapper);
        if (
          typeof ChatState !== "undefined" &&
          !ChatState.isUserScrolling &&
          typeof scrollToBottom === "function"
        ) {
          scrollToBottom(true);
        }
      });
    })
    .catch(function () {});
}
