"use strict";

// VM file attachments — inline and window preview.
// Split out of the former 70_vm_attachments module for #1731; globals unchanged.

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
    var expanded = box.classList.contains("expanded");
    expand.textContent = expanded ? "Collapse" : "Expand";
    if (expanded) {
      box.style.position = "sticky";
      box.style.top = "0";
      box.style.zIndex = "4";
      box.style.width = "100%";
      box.style.maxWidth = "100%";
    } else {
      box.style.position = "";
      box.style.top = "";
      box.style.zIndex = "";
      box.style.width = "";
      box.style.maxWidth = "";
    }
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
      officeFrame.replaceWith(fallbackNote);
    }
    officeFrame.addEventListener("error", showOfficeFallback);
    body.appendChild(officeFrame);
    // An iframe fires no `error` event for an HTTP error page (the 415 used
    // for unconvertible packages renders the raw response), so probe the
    // conversion endpoint before loading it.
    fetch(previewUrl, { method: "GET", credentials: "same-origin" })
      .then(function (res) {
        if (!res.ok) {
          showOfficeFallback();
          return;
        }
        officeFrame.src = previewUrl;
      })
      .catch(showOfficeFallback);
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
    other.textContent =
      "This file type has no inline preview. Open the window preview to view it.";
    // #1753 — same fallback affordance as the office branch: unknown types
    // still open through the signed window preview instead of being a dead end.
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
  var downloadUrl = url.indexOf("preview=1") !== -1 ? url.replace("preview=1", "raw=1") : url;
  var body =
    mime.indexOf("image/") === 0
      ? '<img class="gb-vm-preview-body" src="' + url + '" alt="' + name + '">'
      : '<iframe class="gb-vm-preview-body" src="' + url + '" title="' + name + '"></iframe>';
  overlay.innerHTML =
    '<div class="gb-vm-preview-head"><span>' + name + "</span>" +
    '<a class="gb-vm-preview-download" download="' + name + '" href="' + downloadUrl + '">Download</a>' +
    '<button type="button" class="gb-vm-preview-close" aria-label="Close preview">&times;</button></div>' +
    body;
  var downloadLink = overlay.querySelector(".gb-vm-preview-download");
  if (downloadLink) {
    downloadLink.addEventListener("click", function (e) {
      e.preventDefault();
      vmDownload(downloadUrl, path.split("/").pop() || "file");
    });
  }
  overlay._gbUrl = url;
}
