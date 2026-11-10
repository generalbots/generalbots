/* Drive preview — one shell for every previewable type (#2105).
 *
 * showPreviewModal() keeps its historical signature (fileName, ext, blob) and
 * now takes an optional path so the shell can offer "Copy path". The shell is
 * shared with the Office preview and every renderer lives in
 * 08_preview_render.js; parsing/tokenizing lives in 08_preview_parse.js.
 */
"use strict";

var PREVIEW_IMAGE_EXT = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "avif", "svg"];
var PREVIEW_VIDEO_EXT = ["mp4", "webm", "mov", "m4v", "mkv", "avi", "ogv"];
var PREVIEW_AUDIO_EXT = ["mp3", "wav", "ogg", "oga", "m4a", "flac"];
var PREVIEW_TEXT_EXT = [
  "txt", "md", "markdown", "csv", "tsv", "json", "xml", "log", "out", "err",
  "yml", "yaml", "toml", "js", "mjs", "cjs", "jsx", "ts", "tsx", "py", "rs",
  "go", "java", "c", "cpp", "h", "hpp", "html", "htm", "css", "scss", "bas",
  "sql", "sh", "bash", "bat", "ini", "cfg", "conf"
];

var previewShellObjectUrl = null;
var previewShellKeyHandler = null;

function closeDrivePreview() {
  var modal = document.getElementById("preview-modal");
  if (modal) modal.remove();
  if (previewShellObjectUrl) {
    URL.revokeObjectURL(previewShellObjectUrl);
    previewShellObjectUrl = null;
  }
  if (previewShellKeyHandler) {
    document.removeEventListener("keydown", previewShellKeyHandler);
    previewShellKeyHandler = null;
  }
}

/* Builds the shared overlay: title, type badge, size, actions and body. */
function openDrivePreviewShell(options) {
  var opts = options || {};
  var title = opts.title || "Preview";
  closeDrivePreview();

  var overlay = document.createElement("div");
  overlay.id = "preview-modal";
  overlay.className = "gb-preview-overlay";
  overlay.setAttribute("role", "dialog");
  overlay.setAttribute("aria-modal", "true");
  overlay.setAttribute("aria-label", "Preview of " + title);
  overlay.addEventListener("click", function (event) {
    if (event.target === overlay) closeDrivePreview();
  });

  var shell = document.createElement("div");
  shell.className = "gb-preview-shell";

  var header = document.createElement("div");
  header.className = "gb-preview-header";
  var heading = document.createElement("div");
  heading.className = "gb-preview-heading";
  heading.appendChild(drivePreviewElement("h3", "gb-preview-title", title));
  heading.appendChild(drivePreviewElement("span", "gb-preview-badge", String(opts.badge || "file").toUpperCase()));
  if (opts.size) {
    var size = typeof formatFileSize === "function" ? formatFileSize(opts.size) : opts.size + " B";
    heading.appendChild(drivePreviewElement("span", "gb-preview-size", size));
  }
  header.appendChild(heading);

  var actions = document.createElement("div");
  actions.className = "gb-preview-actions";
  if (opts.blob) {
    actions.appendChild(drivePreviewButton("Download", function () {
      drivePreviewSaveBlob(opts.blob, title);
    }, "is-primary"));
  }
  if (opts.path) {
    var pathButton = drivePreviewButton("Copy path", function () {});
    pathButton.addEventListener("click", function () {
      drivePreviewCopy(opts.path, pathButton);
    });
    actions.appendChild(pathButton);
  }
  var closeButton = drivePreviewElement("button", "gb-preview-close", "\u00D7");
  closeButton.type = "button";
  closeButton.setAttribute("aria-label", "Close preview");
  closeButton.addEventListener("click", closeDrivePreview);
  actions.appendChild(closeButton);
  header.appendChild(actions);

  var body = document.createElement("div");
  body.className = "gb-preview-body";

  shell.appendChild(header);
  shell.appendChild(body);
  overlay.appendChild(shell);
  document.body.appendChild(overlay);

  previewShellKeyHandler = function (event) {
    if (event.key === "Escape") closeDrivePreview();
  };
  document.addEventListener("keydown", previewShellKeyHandler);
  closeButton.focus();

  return { modal: overlay, shell: shell, header: header, body: body };
}

function readDrivePreviewText(blob) {
  return new Promise(function (resolve, reject) {
    if (blob && typeof blob.text === "function") {
      blob.text().then(resolve, function () {
        reject(new Error("The file could not be decoded as text."));
      });
      return;
    }
    var reader = new FileReader();
    reader.onload = function () { resolve(String(reader.result || "")); };
    reader.onerror = function () { reject(new Error("The file could not be read.")); };
    reader.readAsText(blob);
  });
}

function drivePreviewMediaUrl(blob) {
  previewShellObjectUrl = URL.createObjectURL(blob);
  return previewShellObjectUrl;
}

function showDriveImagePreview(body, blob) {
  var image = drivePreviewElement("img", "gb-preview-image");
  image.alt = "";
  image.addEventListener("error", function () {
    image.remove();
    renderDrivePreviewError(body, "Image could not be displayed",
      "The file may be corrupted or use a format this browser cannot decode. Download it to open it locally.");
  });
  body.appendChild(image);
  image.src = drivePreviewMediaUrl(blob);
}

function showDrivePdfPreview(body, blob) {
  var url = drivePreviewMediaUrl(blob);
  var frame = drivePreviewElement("iframe", "gb-preview-frame");
  frame.title = "PDF preview";
  frame.src = url;
  body.appendChild(frame);
  var bar = drivePreviewNoticeBar(["Embedded PDF viewing depends on the browser."], [
    drivePreviewButton("Open in new tab", function () { window.open(url, "_blank", "noopener"); })
  ]);
  body.insertBefore(bar, frame);
}

function showDriveVideoPreview(body, blob) {
  var video = drivePreviewElement("video", "gb-preview-video");
  video.controls = true;
  video.preload = "metadata";
  video.addEventListener("error", function () {
    video.remove();
    renderDrivePreviewError(body, "Video could not be played",
      "This browser cannot decode the file. Download it to watch it locally.");
  });
  body.appendChild(video);
  video.src = drivePreviewMediaUrl(blob);
}

function showDriveAudioPreview(body, blob) {
  var audio = drivePreviewElement("audio", "gb-preview-audio");
  audio.controls = true;
  audio.preload = "metadata";
  audio.addEventListener("error", function () {
    audio.remove();
    renderDrivePreviewError(body, "Audio could not be played",
      "This browser cannot decode the file. Download it to listen to it locally.");
  });
  body.appendChild(audio);
  audio.src = drivePreviewMediaUrl(blob);
}

function renderDrivePreviewText(body, text, context) {
  if (!text || !text.length) {
    renderDrivePreviewError(body, "Empty file", "This file has no text content to preview.");
    return;
  }
  if (context.ext === "csv" || context.ext === "tsv") {
    renderDriveCsvPreview(body, text, context);
    return;
  }
  if (context.ext === "json") {
    renderDriveJsonPreview(body, text, context);
    return;
  }
  if (context.ext === "md" || context.ext === "markdown") {
    renderDriveMarkdownPreview(body, text, context);
    return;
  }
  renderDriveTextPreview(body, text, context);
}

function showPreviewModal(fileName, ext, blob, path) {
  var normalizedExt = String(ext || "").toLowerCase();
  var shell = openDrivePreviewShell({
    title: fileName || "Preview",
    badge: normalizedExt || "file",
    size: blob && blob.size ? blob.size : 0,
    blob: blob,
    path: path
  });
  var body = shell.body;
  var context = { fileName: fileName, ext: normalizedExt, blob: blob, path: path };

  if (!blob || typeof blob.size !== "number") {
    renderDrivePreviewError(body, "Preview unavailable", "The file content could not be loaded. Use Download to fetch it.");
    return;
  }
  if (blob.size === 0) {
    renderDrivePreviewError(body, "Empty file", "This file has no content to preview.");
    return;
  }
  if (PREVIEW_IMAGE_EXT.indexOf(normalizedExt) !== -1) {
    showDriveImagePreview(body, blob);
    return;
  }
  if (normalizedExt === "pdf") {
    showDrivePdfPreview(body, blob);
    return;
  }
  if (PREVIEW_VIDEO_EXT.indexOf(normalizedExt) !== -1) {
    showDriveVideoPreview(body, blob);
    return;
  }
  if (PREVIEW_AUDIO_EXT.indexOf(normalizedExt) !== -1) {
    showDriveAudioPreview(body, blob);
    return;
  }
  if (PREVIEW_TEXT_EXT.indexOf(normalizedExt) !== -1) {
    var loading = renderDrivePreviewMessage(body, "Reading file...");
    // Big logs/CSVs must not be pulled into memory whole: read at most the
    // preview budget and tell the user the view is partial.
    var byteLimited = blob.size > PREVIEW_BYTE_LIMIT;
    var source = byteLimited ? blob.slice(0, PREVIEW_BYTE_LIMIT) : blob;
    context.byteLimited = byteLimited;
    readDrivePreviewText(source).then(function (text) {
      loading.remove();
      renderDrivePreviewText(body, text, context);
    }, function (err) {
      loading.remove();
      renderDrivePreviewError(body, "Preview failed",
        err && err.message ? err.message : "The file could not be read.");
    });
    return;
  }
  renderDrivePreviewError(body, "No preview for ." + (normalizedExt || "file") + " files",
    "This type cannot be rendered in the browser. Download the file to open it with its own application.");
}

/* Office documents (docx/xlsx/pptx) arrive as a server-rendered, escaped
 * HTML fragment: it keeps the shared shell but uses a light document page. */
function showOfficePreviewModal(fileName, html) {
  var shell = openDrivePreviewShell({ title: fileName, badge: "office" });
  var page = drivePreviewElement("div", "gb-preview-office");
  page.innerHTML = html;
  shell.body.appendChild(page);
}
