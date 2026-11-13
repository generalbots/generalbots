"use strict";

// VM file attachments — inline and window preview.
// Split out of the former 70_vm_attachments module for #1731; globals unchanged.

function vmAuthFetch(url, init) {
  if (typeof vmFetchAuthenticated === "function") return vmFetchAuthenticated(url, init);
  return fetch(url, init);
}

function vmTrackBlobUrl(box, blobUrl) {
  if (!box) return;
  if (!box._gbBlobUrls) box._gbBlobUrls = [];
  box._gbBlobUrls.push(blobUrl);
  if (!box._gbBlobObserver) {
    box._gbBlobObserver = new MutationObserver(function () {
      if (box.isConnected) return;
      var urls = box._gbBlobUrls || [];
      for (var i = 0; i < urls.length; i++) {
        try { URL.revokeObjectURL(urls[i]); } catch (e) {}
      }
      box._gbBlobUrls = [];
      box._gbBlobObserver.disconnect();
      box._gbBlobObserver = null;
    });
    box._gbBlobObserver.observe(document.body, { childList: true, subtree: true });
  }
}

function vmFetchBlobUrl(url, box) {
  return vmAuthFetch(url).then(function (res) {
    if (!res.ok) throw new Error("HTTP " + res.status);
    return res.blob();
  }).then(function (blob) {
    var blobUrl = URL.createObjectURL(blob);
    if (box) vmTrackBlobUrl(box, blobUrl);
    return blobUrl;
  });
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
  expand.setAttribute("aria-expanded", "false");
  expand.addEventListener("click", function () {
    if (window.GBArtifactOverlay) {
      window.GBArtifactOverlay.toggle(box, expand, { label: name });
      return;
    }
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
    if (window.GBArtifactOverlay) window.GBArtifactOverlay.close(box);
    box.remove();
  });
  head.appendChild(close);
  box.appendChild(head);

  var body = document.createElement("div");
  body.className = "gb-attachment-inline-body";
  if (kind === "image") {
    var img = document.createElement("img");
    img.loading = "lazy";
    img.alt = name;
    body.appendChild(img);
    vmFetchBlobUrl(url, box).then(function (blobUrl) {
      if (box.isConnected) img.src = blobUrl;
    }).catch(function () {
      if (box.isConnected) img.alt = "Could not load preview";
    });
  } else if (kind === "audio") {
    var audio = document.createElement("audio");
    audio.controls = true;
    audio.preload = "metadata";
    audio.style.width = "100%";
    body.appendChild(audio);
    var hint = document.createElement("div");
    hint.className = "gb-attachment-inline-note";
    hint.textContent = "Click play to listen inline. Download if playback fails.";
    body.appendChild(hint);
    vmFetchBlobUrl(url, box).then(function (blobUrl) {
      if (box.isConnected) audio.src = blobUrl;
    }).catch(function () {
      if (box.isConnected) hint.textContent = "Could not load audio preview.";
    });
  } else if (kind === "pdf" || kind === "html") {
    var frame = document.createElement("iframe");
    if (kind === "html") {
      frame.setAttribute("sandbox", "allow-scripts allow-forms allow-modals");
    }
    frame.setAttribute("referrerpolicy", "no-referrer");
    frame.setAttribute("title", name);
    body.appendChild(frame);
    vmFetchBlobUrl(url, box).then(function (blobUrl) {
      if (box.isConnected) frame.src = blobUrl;
    }).catch(function () {
      if (!box.isConnected) return;
      var note = document.createElement("div");
      note.className = "gb-attachment-inline-note";
      note.textContent = "Could not load the preview.";
      body.appendChild(note);
    });
  } else if (kind === "office") {
    var officeFrame = document.createElement("iframe");
    officeFrame.className = "gb-attachment-inline-frame";
    officeFrame.setAttribute("sandbox", "allow-same-origin");
    officeFrame.setAttribute("title", name);
    officeFrame.setAttribute("loading", "eager");
    var previewUrl = vmOfficePreviewUrl(convId || "", path);
    var fallbackNote = null;
    function showOfficeFallback() {
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
      if (officeFrame.parentNode) officeFrame.replaceWith(fallbackNote);
    }
    body.appendChild(officeFrame);
    vmAuthFetch(previewUrl, { credentials: "same-origin" })
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.blob();
      })
      .then(function (blob) {
        if (!box.isConnected) return;
        var blobUrl = URL.createObjectURL(blob);
        vmTrackBlobUrl(box, blobUrl);
        officeFrame.src = blobUrl;
      })
      .catch(showOfficeFallback);
  } else if (kind === "text") {
    var pre = document.createElement("pre");
    pre.className = "gb-attachment-inline-text";
    pre.textContent = "Loading…";
    body.appendChild(pre);
    vmAuthFetch(url)
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.text();
      })
      .then(function (text) {
        if (box.isConnected) pre.textContent = text.slice(0, 200000);
      })
      .catch(function () {
        if (box.isConnected) pre.textContent = "Could not load the file preview.";
      });
  } else {
    var other = document.createElement("div");
    other.className = "gb-attachment-inline-note";
    other.textContent =
      "This file type has no inline preview. Open the window preview to view it.";
    var openOther = document.createElement("button");
    openOther.type = "button";
    openOther.className = "gb-attachment-btn";
    openOther.textContent = "Open window preview";
    openOther.addEventListener("click", function () {
      vmShowPreviewUrl(url, path);
    });
    other.appendChild(openOther);
    body.appendChild(other);
  }
  box.appendChild(body);
  card.appendChild(box);
}

function vmShowPreviewUrl(url, path) {
  var mime = vmFileMimeType(path);
  var overlay = document.getElementById("gb-vm-preview");
  if (overlay) {
    if (overlay._gbBlobUrl) {
      try { URL.revokeObjectURL(overlay._gbBlobUrl); } catch (e) {}
    }
  }
  if (!overlay) {
    overlay = document.createElement("div");
    overlay.id = "gb-vm-preview";
    overlay.className = "gb-vm-preview";
    overlay.addEventListener("click", function (e) {
      if (e.target === overlay || e.target.classList.contains("gb-vm-preview-close")) {
        if (overlay._gbBlobUrl) {
          try { URL.revokeObjectURL(overlay._gbBlobUrl); } catch (e) {}
          overlay._gbBlobUrl = null;
        }
        overlay.remove();
      }
    });
    document.body.appendChild(overlay);
  }
  var name = vmEscapeHtml(path.split("/").pop() || "file");
  var downloadUrl = url.indexOf("preview=1") !== -1 ? url.replace("preview=1", "raw=1") : url;
  overlay.innerHTML =
    '<div class="gb-vm-preview-head"><span>' + name + "</span>" +
    '<a class="gb-vm-preview-download" download="' + name + '" href="#">Download</a>' +
    '<button type="button" class="gb-vm-preview-close" aria-label="Close preview">&times;</button></div>' +
    '<div class="gb-vm-preview-body">Loading…</div>';
  var downloadLink = overlay.querySelector(".gb-vm-preview-download");
  if (downloadLink) {
    downloadLink.addEventListener("click", function (e) {
      e.preventDefault();
      vmDownload(downloadUrl, path.split("/").pop() || "file");
    });
  }
  vmFetchBlobUrl(url)
    .then(function (blobUrl) {
      if (!overlay.isConnected) {
        try { URL.revokeObjectURL(blobUrl); } catch (e) {}
        return;
      }
      overlay._gbBlobUrl = blobUrl;
      var body;
      if (mime.indexOf("image/") === 0) {
        body = '<img class="gb-vm-preview-body" src="' + blobUrl + '" alt="' + name + '">';
      } else if (mime.indexOf("audio/") === 0) {
        body = '<audio class="gb-vm-preview-body" controls src="' + blobUrl + '"></audio>';
      } else {
        body = '<iframe class="gb-vm-preview-body" src="' + blobUrl + '" title="' + name + '"></iframe>';
      }
      var existingBody = overlay.querySelector(".gb-vm-preview-body");
      if (existingBody) {
        existingBody.outerHTML = body;
      }
    })
    .catch(function () {
      var existingBody = overlay.querySelector(".gb-vm-preview-body");
      if (existingBody) existingBody.textContent = "Could not load preview.";
    });
}
