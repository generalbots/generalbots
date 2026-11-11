"use strict";

// VM file attachments — Drive save, saved report and attachment card event binding.
// Split out of the former 70_vm_attachments module for #1731; globals unchanged.

// #1584 — publish output/ to Drive under the conversation folder.
function vmSaveToDrive(conversationId, files) {
  if (!vmAgentDriveEnabled()) {
    var unavailable = new Error(
      "Saving to Drive needs the agent-vm feature, which this build does not include."
    );
    vmNotifyError(unavailable);
    return Promise.reject(unavailable);
  }
  var headers = { "Content-Type": "application/json" };
  var token = vmAttachmentsToken();
  if (token) headers.Authorization = "Bearer " + token;
  return fetch("/api/agent/drive/save", {
    method: "POST",
    headers: headers,
    body: JSON.stringify({ conversation_id: conversationId, files: files || [] }),
  }).then(function (res) {
    if (!res.ok) {
      var detail = (files || []).join(", ") || "unknown";
      throw new Error(
        res.status === 404
          ? "file not found in conversation output (" + detail + ")"
          : "HTTP " + res.status + " saving " + detail
      );
    }
    return res.json();
  });
}

function vmReportSaved(data) {
  var saved = (data && data.saved) || [];
  var message =
    "Saved " + saved.length + " file" + (saved.length === 1 ? "" : "s") +
    " to Drive (" + ((data && data.folder) || "conversations") + ")";
  var opts = null;
  if (saved.length) {
    var target = vmSplitDrivePath(saved[0].path);
    var app = vmOfficeAppFor(saved[0].path || "");
    if (target && app && vmOfficeAppAvailable(app)) {
      opts = { appId: app, params: { file: target.key, bucket: target.bucket }, label: "Open" };
    }
  }
  if (window.GBToasts && GBToasts.show) GBToasts.show("Saved", message, "success", opts);
  else if (window.showToast) window.showToast(message, "success");
  return message;
}

// #1834 — Open an office output in its built-in editor. Publishing to Drive
// needs the agent VM; when the binary does not ship it (or the product has no
// such app), fall back to the server-rendered Office preview so the button is
// never a dead end.
function vmOpenAttachment(conversationId, path) {
  var fileName = String(path || "").split("/").pop() || "file";
  var app = vmOfficeAppFor(fileName);
  if (!app) {
    throw new Error("this file has no editor app");
  }
  var canPublish = vmAgentDriveEnabled() && vmOfficeAppAvailable(app);
  if (!canPublish) {
    vmShowPreviewUrl(vmOfficePreviewUrl(conversationId, path), path);
    return Promise.resolve(null);
  }
  return vmSaveToDrive(conversationId, ["output/" + fileName]).then(function (data) {
    var saved = (data && data.saved) || [];
    if (!saved.length) throw new Error("file was not saved to Drive");
    vmOpenInOfficeApp(fileName, saved[0].path);
    vmReportSaved(data);
    return saved[0].path;
  });
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
  // Downloads always need the raw file, never the sanitized preview HTML.
  var rawUrlFor = function () {
    return resolve().then(function (convId) {
      if (!convId) throw new Error("No active VM");
      return vmAttachmentUrl(convId, path);
    });
  };
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
              vmDownload(url, fileName);
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
    // #2047 — "Save" is a target menu (internal Drive + external drives).
    saveBtn.title = "Save to General Bots Drive or a connected external drive";
    saveBtn.addEventListener("click", function (e) {
      e.preventDefault();
      resolve()
        .then(function (convId) {
          if (!convId) throw new Error("No active VM");
          return vmSaveWithMenu(saveBtn, convId, ["output/" + fileName]);
        })
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
          return vmSaveToDrive(convId, ["output/" + fileName]);
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
      rawUrlFor()
        .then(function (url) {
          return vmDownload(url, fileName);
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
