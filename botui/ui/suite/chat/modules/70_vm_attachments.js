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
  if (
    mime.indexOf("officedocument") !== -1 ||
    mime === "application/msword" ||
    mime === "application/vnd.ms-excel" ||
    mime === "application/vnd.ms-powerpoint"
  ) {
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
  if (["docx", "doc", "odt", "rtf"].indexOf(ext) !== -1) return "docs";
  if (["xlsx", "xls", "ods", "csv"].indexOf(ext) !== -1) return "sheet";
  if (["pptx", "ppt"].indexOf(ext) !== -1) return "slides";
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
  if (window.showToast) window.showToast(message, "error");
  else console.error(message);
}

// #1672 — inline preview inside the attachment card. The frame is created
// lazily on the first click; nothing is fetched while the card is idle.
function vmToggleInlinePreview(card, url, path, convId) {
  if (!card) return;
  var existing = card.querySelector(".gb-attachment-inline");
  if (existing) {
    existing.remove();
    return;
  }
  var kind = vmInlineKind(path);
  var name = String(path || "").split("/").pop() || "file";
  var box = document.createElement("div");
  box.className = "gb-attachment-inline";

  var head = document.createElement("div");
  head.className = "gb-attachment-inline-head";
  var title = document.createElement("span");
  title.textContent = name;
  head.appendChild(title);

  var expand = document.createElement("button");
  expand.type = "button";
  expand.className = "gb-attachment-inline-btn";
  expand.textContent = "Expand";
  expand.addEventListener("click", function () {
    box.classList.toggle("expanded");
    expand.textContent = box.classList.contains("expanded") ? "Collapse" : "Expand";
  });
  head.appendChild(expand);

  var open = document.createElement("button");
  open.type = "button";
  open.className = "gb-attachment-inline-btn";
  open.textContent = "Open in window";
  open.addEventListener("click", function () {
    vmShowPreviewUrl(url, path);
  });
  head.appendChild(open);

  var close = document.createElement("button");
  close.type = "button";
  close.className = "gb-attachment-inline-btn";
  close.setAttribute("aria-label", "Close preview");
  close.textContent = "×";
  close.addEventListener("click", function () {
    box.remove();
  });
  head.appendChild(close);
  box.appendChild(head);

  var body = document.createElement("div");
  body.className = "gb-attachment-inline-body";
  if (kind === "image") {
    var img = document.createElement("img");
    img.src = url;
    img.loading = "lazy";
    img.alt = name;
    body.appendChild(img);
  } else if (kind === "pdf" || kind === "html") {
    var frame = document.createElement("iframe");
    // Interactive HTML artifacts (infographics, charts, dashboards) need
    // scripts and get an opaque origin: they cannot read the suite DOM,
    // cookies or storage. A PDF must NOT be sandboxed — Chrome refuses to run
    // its built-in viewer inside a sandboxed frame and shows a broken-file
    // icon. The PDF response is application/pdf + nosniff, so it cannot
    // execute as the parent origin.
    if (kind === "html") {
      frame.setAttribute("sandbox", "allow-scripts allow-forms allow-modals");
    }
    frame.setAttribute("referrerpolicy", "no-referrer");
    frame.setAttribute("title", name);
    frame.src = url;
    body.appendChild(frame);
  } else if (kind === "office") {
    // #1672 — Office renders INLINE now: the server converts the package to
    // sanitized, script-free HTML (xlsx sheets, docx paragraphs, pptx
    // slides) through the same signed pipeline. Fully opaque iframe: the
    // document cannot touch the conversation, cookies or any network.
    var officeFrame = document.createElement("iframe");
    officeFrame.className = "gb-attachment-inline-frame";
    officeFrame.setAttribute("sandbox", "");
    officeFrame.setAttribute("title", name);
    officeFrame.setAttribute(
      "loading",
      "lazy"
    );
    officeFrame.src = vmOfficePreviewUrl(convId || "", path);
    var fallbackNote = null;
    officeFrame.addEventListener("error", function () {
      if (fallbackNote) return;
      fallbackNote = document.createElement("div");
      fallbackNote.className = "gb-attachment-inline-note";
      fallbackNote.textContent =
        "This file has no inline conversion. Open the window preview to read it.";
      var openWindow = document.createElement("button");
      openWindow.type = "button";
      openWindow.className = "gb-attachment-btn";
      openWindow.textContent = "Open window preview";
      openWindow.addEventListener("click", function () {
        vmShowPreviewUrl(url, path);
      });
      fallbackNote.appendChild(openWindow);
      officeFrame.replaceWith(fallbackNote);
    });
    body.appendChild(officeFrame);
  } else if (kind === "text") {
    // Plain text, CSV, markdown and JSON render inline: fetch the signed URL
    // and write it into a <pre> as text (never parsed as HTML).
    var pre = document.createElement("pre");
    pre.className = "gb-attachment-inline-text";
    pre.textContent = "Loading…";
    body.appendChild(pre);
    fetch(url)
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.text();
      })
      .then(function (text) {
        pre.textContent = text.slice(0, 200000);
      })
      .catch(function () {
        pre.textContent = "Could not load the file preview.";
      });
  } else {
    var other = document.createElement("div");
    other.className = "gb-attachment-inline-note";
    other.textContent = "This file type has no inline preview.";
    body.appendChild(other);
  }
  box.appendChild(body);
  card.appendChild(box);
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

// #1584 — publish output/ to Drive under the conversation folder.
function vmSaveToDrive(conversationId, files) {
  var headers = { "Content-Type": "application/json" };
  var token = vmAttachmentsToken();
  if (token) headers.Authorization = "Bearer " + token;
  return fetch("/api/agent/drive/save", {
    method: "POST",
    headers: headers,
    body: JSON.stringify({ conversation_id: conversationId, files: files || [] }),
  }).then(function (res) {
    if (!res.ok) throw new Error("HTTP " + res.status);
    return res.json();
  });
}

function vmReportSaved(data) {
  var saved = (data && data.saved) || [];
  var message =
    "Saved " + saved.length + " file" + (saved.length === 1 ? "" : "s") +
    " to Drive (" + ((data && data.folder) || "conversations") + ")";
  if (window.GBToasts && GBToasts.show) GBToasts.show("Saved", message, "success");
  else if (window.showToast) window.showToast(message, "success");
  return message;
}

function vmBindAttachment(card) {
  if (!card || card.dataset.bound === "1") return;
  card.dataset.bound = "1";
  var path = card.dataset.path;
  var explicitConv = card.dataset.conversationId || "";
  var resolvedConvId = explicitConv;
  var resolve = function () {
    return explicitConv
      ? Promise.resolve(explicitConv)
      : vmActiveConversationId().then(function (convId) {
          resolvedConvId = convId || "";
          return convId;
        });
  };
  var nameLink = card.querySelector(".gb-attachment-name");
  var fileName = String(path || "").split("/").pop() || "file";
  // Files a browser renders natively open in a new tab. Office documents go
  // through the server-side converter (`preview=1`) so the tab shows readable
  // HTML instead of downloading a binary. Anything else keeps a download.
  var inlineViewable = /\.(png|jpe?g|gif|webp|svg|bmp|ico|pdf|txt|md|markdown|csv|tsv|json|xml|ya?ml|log|html?|mp3|wav|ogg|m4a|mp4|webm|mov)$/i.test(fileName);
  var officeDoc = /\.(docx?|xlsx?|pptx?|odt|ods|odp|rtf)$/i.test(fileName);
  var urlFor = function () {
    return resolve().then(function (convId) {
      if (!convId) throw new Error("No active VM");
      var url = officeDoc && !inlineViewable
        ? vmOfficePreviewUrl(convId, path)
        : vmAttachmentUrl(convId, path);
      if (nameLink && nameLink.tagName === "A") {
        nameLink.href = url;
        nameLink.target = "_blank";
        nameLink.rel = "noopener";
        if (inlineViewable || officeDoc) {
          nameLink.removeAttribute("download");
        } else {
          nameLink.setAttribute("download", fileName);
        }
      }
      return url;
    });
  };
  // Point the link as soon as the conversation is known, so the file name is
  // a working hyperlink before the user clicks anything.
  urlFor().catch(function () {
    /* resolved again on click */
  });
  // A generated HTML infographic should land on the chat surface, not hide
  // behind a Preview button. Auto-open only when this card belongs to the
  // newest bot message, so history stays quiet.
  if (/\.html?$/i.test(fileName)) {
    var message = card.closest(".message");
    var botMessages = document.querySelectorAll("#messages .message.bot");
    var newest = botMessages.length ? botMessages[botMessages.length - 1] : null;
    if (message && newest === message) {
      setTimeout(function () {
        var preview = card.querySelector(".gb-attachment-preview");
        if (preview && !card.querySelector(".gb-attachment-inline")) {
          preview.click();
        }
      }, 350);
    }
  }
  if (nameLink) {
    nameLink.addEventListener("click", function (e) {
      // Before the first resolve the href is still "#": keep the browser on
      // the page and open the real URL once it is known.
      if (!nameLink.href || nameLink.getAttribute("href") === "#") {
        e.preventDefault();
        urlFor()
          .then(function (url) {
            if (inlineViewable || officeDoc) {
              window.open(url, "_blank", "noopener");
            } else {
              var link = document.createElement("a");
              link.href = url;
              link.download = fileName;
              document.body.appendChild(link);
              link.click();
              link.remove();
            }
          })
          .catch(vmNotifyError);
      }
    });
  }

  var download = card.querySelector(".gb-attachment-download");
  var preview = card.querySelector(".gb-attachment-preview");
  var saveBtn = card.querySelector(".gb-attachment-save");
  if (saveBtn) {
    saveBtn.addEventListener("click", function (e) {
      e.preventDefault();
      resolve()
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          return vmSaveToDrive(convId, [fileName]);
        })
        .then(vmReportSaved)
        .catch(vmNotifyError);
    });
  }
  var openBtn = card.querySelector(".gb-attachment-open");
  if (openBtn) {
    openBtn.addEventListener("click", function (e) {
      e.preventDefault();
      resolve()
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          return vmSaveToDrive(convId, [fileName]);
        })
        .then(function (data) {
          var saved = (data && data.saved) || [];
          if (!saved.length) throw new Error("file was not saved to Drive");
          vmOpenInOfficeApp(fileName, saved[0].path);
          vmReportSaved(data);
        })
        .catch(vmNotifyError);
    });
  }
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
      urlFor()
        .then(function (url) {
          if (!resolvedConvId) throw new Error("No active VM");
          vmToggleInlinePreview(card, url, path, resolvedConvId);
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
    '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener"></a>' +
    '<button type="button" class="gb-attachment-btn gb-attachment-preview">Preview</button>' +
    '<button type="button" class="gb-attachment-btn gb-attachment-download">Download</button>' +
    '<button type="button" class="gb-attachment-btn gb-attachment-save">Save</button>';
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

function vmLatestBotMessage() {
  var pane = vmVisiblePane();
  if (!pane) return null;
  var bots = pane.querySelectorAll(".message.bot");
  return bots.length ? bots[bots.length - 1] : null;
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

// Folder rows follow the server's newest-first listing (#1693). Reply links
// arrive in tool order, which can put an older file first; the VM listing is
// ordered by mtime, so it is the source of truth. Cached per conversation and
// invalidated on every live reply that may add files.
var vmFolderOrder = {};
function vmApplyFolderOrder(grid, rank) {
  var items = Array.prototype.slice.call(grid.querySelectorAll(".gb-attachment"));
  items.sort(function (a, b) {
    var an = String(a.dataset.path || "").split("/").pop();
    var bn = String(b.dataset.path || "").split("/").pop();
    var ar = Object.prototype.hasOwnProperty.call(rank, an) ? rank[an] : 1e9;
    var br = Object.prototype.hasOwnProperty.call(rank, bn) ? rank[bn] : 1e9;
    return ar - br;
  });
  items.forEach(function (el) { grid.appendChild(el); });
}
var vmFolderPending = {};
function vmOrderFolder(grid) {
  if (!grid || grid.querySelectorAll(".gb-attachment").length < 2) return;
  vmCurrentConversationId()
    .then(function (conversationId) {
      if (!conversationId) return;
      if (vmFolderOrder[conversationId]) {
        vmApplyFolderOrder(grid, vmFolderOrder[conversationId]);
        return;
      }
      // History replay groups every reply at once; one listing per
      // conversation is enough, so concurrent folders share the request.
      if (!vmFolderPending[conversationId]) {
        vmFolderPending[conversationId] = vmAttachmentsRequest(
          "/api/agent/vm/files?conversation_id=" + encodeURIComponent(conversationId)
        )
          .then(function (data) {
            var rank = {};
            ((data && data.files) || []).forEach(function (name, i) { rank[name] = i; });
            vmFolderOrder[conversationId] = rank;
            return rank;
          })
          .finally(function () { delete vmFolderPending[conversationId]; });
      }
      return vmFolderPending[conversationId].then(function (rank) {
        vmApplyFolderOrder(grid, rank);
      });
    })
    .catch(function () { /* keep the reply order */ });
}

// One folder card for every attachment of the reply (#1584): the user sees
// "output" as a folder with all generated files, not a flat list of links.
function vmGroupAttachments(anchor) {
  if (!anchor) return;
  var cards = [];
  var all = anchor.querySelectorAll(".gb-attachment");
  for (var i = 0; i < all.length; i++) {
    if (!all[i].closest(".gb-attachment-folder")) cards.push(all[i]);
  }
  var folder = anchor.querySelector(".gb-attachment-folder");
  if (!folder) {
    if (!cards.length) return;
    folder = document.createElement("div");
    folder.className = "gb-attachment-folder";
    folder.innerHTML =
      '<div class="gb-folder-head">' +
      '<span class="gb-folder-icon">\u{1F4C1}</span>' +
      '<span class="gb-folder-title">output</span>' +
      '<span class="gb-folder-count"></span>' +
      '<button type="button" class="gb-folder-save gb-attachment-btn">Save to Drive</button>' +
      "</div>" +
      '<div class="gb-folder-grid"></div>';
    var target = anchor.querySelector(".message-content") || anchor;
    target.appendChild(folder);
  }
  var grid = folder.querySelector(".gb-folder-grid");
  // New cards lead the folder (#1693): the file the user just asked for is
  // the first row, older outputs follow as history. Inserting in reverse
  // keeps the source order (newest first) while placing it on top.
  for (var j = cards.length - 1; j >= 0; j--) {
    grid.insertBefore(cards[j], grid.firstChild);
  }
  var count = grid.querySelectorAll(".gb-attachment").length;
  folder.querySelector(".gb-folder-count").textContent =
    count + " file" + (count === 1 ? "" : "s");
  vmOrderFolder(grid);
  var saveAll = folder.querySelector(".gb-folder-save");
  if (saveAll && !saveAll.dataset.bound) {
    saveAll.dataset.bound = "1";
    saveAll.addEventListener("click", function () {
      vmCurrentConversationId()
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          return vmSaveToDrive(convId, []);
        })
        .then(vmReportSaved)
        .catch(vmNotifyError);
    });
  }
}

// Markdown the model emits for generated files — `[name](vmfile:CONV/output/name)`
// — does not survive the markdown/sanitizer pipeline on every path, so it can
// reach the DOM as plain text ("[a.pdf](vmfile:...)") with no anchor to click.
// Scan the rendered text nodes and replace each reference with a real
// attachment card (Preview / Download / Save).
function vmRenderInlineReferences(root) {
  if (!root || !root.querySelectorAll) return;
  var nodes = root.querySelectorAll(".message-content, .bot-message, .message-text");
  var pattern = /\[([^\]\n]{1,200})\]\(vmfile:([^)\s]+)\)/g;
  for (var i = 0; i < nodes.length; i++) {
    var node = nodes[i];
    if (node.querySelector(".gb-attachment")) continue;
    var walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT, null);
    var targets = [];
    var textNode;
    while ((textNode = walker.nextNode())) {
      if (textNode.nodeValue && textNode.nodeValue.indexOf("vmfile:") !== -1) {
        targets.push(textNode);
      }
    }
    if (!targets.length) continue;
    for (var t = 0; t < targets.length; t++) {
      var current = targets[t];
      var value = current.nodeValue;
      var replaced = document.createDocumentFragment();
      var last = 0;
      var hit;
      pattern.lastIndex = 0;
      while ((hit = pattern.exec(value)) !== null) {
        if (hit.index > last) {
          replaced.appendChild(document.createTextNode(value.slice(last, hit.index)));
        }
        var parts = vmSplitTarget(hit[2]);
        var card = document.createElement("span");
        card.className = "gb-attachment";
        card.dataset.path = parts.path;
        card.dataset.conversationId = parts.conversationId || "";
        card.innerHTML =
          '<span class="gb-attachment-icon">' + vmFileIcon(parts.path) + "</span>" +
          '<a class="gb-attachment-name" href="#" target="_blank" rel="noopener"></a>' +
          '<button type="button" class="gb-attachment-btn gb-attachment-preview">Preview</button>' +
          '<button type="button" class="gb-attachment-btn gb-attachment-download">Download</button>' +
          '<button type="button" class="gb-attachment-btn gb-attachment-save">Save</button>';
        card.querySelector(".gb-attachment-name").textContent =
          (hit[1] || "").trim() || parts.path.split("/").pop();
        replaced.appendChild(card);
        last = hit.index + hit[0].length;
      }
      if (!replaced.childNodes.length) continue;
      if (last < value.length) {
        replaced.appendChild(document.createTextNode(value.slice(last)));
      }
      if (current.parentNode) current.parentNode.replaceChild(replaced, current);
    }
  }
}

// A card that replaced the only text of a markdown list item leaves
// `<ul><li></li></ul>` behind — the empty bullet the user sees above the folder
// card. Lift the card out of the list and drop the now empty list.
function vmDropEmptyListItems(root) {
  if (!root || !root.querySelectorAll) return;
  var items = root.querySelectorAll("li");
  for (var i = 0; i < items.length; i++) {
    var li = items[i];
    if (li.textContent.trim() !== "") continue;
    if (!li.querySelector(".gb-attachment")) continue;
    var list = li.parentElement;
    if (!list || !list.parentElement) continue;
    while (li.firstChild) {
      list.parentElement.insertBefore(li.firstChild, list);
    }
    li.remove();
    if (!list.querySelector("li")) list.remove();
  }
}

// Bind inline vmfile cards and append output/ files the model did not link.
function hydrateVmAttachments(container) {
  if (!container) return;
  // A live reply can add files; the cached newest-first ranking is stale.
  if (typeof ChatState === "undefined" || !ChatState.suppressPersist) {
    vmFolderOrder = {};
  }
  vmRenderInlineReferences(container);
  vmDropEmptyListItems(container);
  var cards = container.querySelectorAll(".gb-attachment");
  for (var i = 0; i < cards.length; i++) vmBindAttachment(cards[i]);

  var anchor = vmAttachmentAnchor(container);
  if (!anchor) return;
  vmGroupAttachments(anchor);

  // History replay already carries the files the model linked. The
  // conversation-wide fallback must not staple today's output/ folder onto
  // old turns, so it only runs for a live reply. Record the conversation's
  // existing files as seen first, otherwise the next live answer would
  // re-attach every PDF produced before the reload.
  if (typeof ChatState !== "undefined" && ChatState.suppressPersist) {
    vmCurrentConversationId().then(function (conversationId) {
      if (!conversationId) return;
      var seen = VmAttachments.renderedPaths[conversationId];
      if (!seen) {
        seen = {};
        VmAttachments.renderedPaths[conversationId] = seen;
      }
      var cards = anchor.querySelectorAll(".gb-attachment");
      for (var index = 0; index < cards.length; index++) {
        if (cards[index].dataset.path) seen[cards[index].dataset.path] = true;
      }
      return vmAttachmentsRequest(
        "/api/agent/vm/files?conversation_id=" + encodeURIComponent(conversationId)
      ).then(function (data) {
        ((data && data.files) || []).forEach(function (name) {
          seen["output/" + name] = true;
        });
      });
    }).catch(function () { /* replay stays best-effort */ });
    return;
  }
  vmCurrentConversationId()
    .then(function (conversationId) {
      if (!conversationId) return;
      // Claim the conversation synchronously: replay hydrates every message
      // at once, and each callback used to create its own empty set, so the
      // same output/ files were appended to all 34 replies.
      var seen = VmAttachments.renderedPaths[conversationId];
      if (!seen) {
        seen = {};
        VmAttachments.renderedPaths[conversationId] = seen;
      }
      // The fallback folder belongs to the newest reply, never to an older
      // turn that happened to hydrate first.
      var fallback = vmLatestBotMessage() || anchor;
      var present = {};
      var existing = fallback.querySelectorAll(".gb-attachment");
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
          vmGroupAttachments(fallback);
          return;
        }
        if (!fallback.isConnected) return;
        var wrapper = document.createElement("div");
        wrapper.className = "gb-attachments";
        missing.forEach(function (name) {
          var path = "output/" + name;
          vmAppendCard(wrapper, conversationId, path, name);
          seen[path] = true;
        });
        var target = fallback.querySelector(".message-content") || fallback;
        target.appendChild(wrapper);
        vmGroupAttachments(fallback);
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
