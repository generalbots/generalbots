"use strict";
/**
 * Chat — preview the files staged in the composer before sending.
 *
 * Clicking the attachment chip opens a small panel under the composer with one
 * row per staged file: images get a thumbnail, everything else shows type and
 * size. The bytes come straight from the File objects, so nothing is uploaded
 * to preview.
 */

(function () {
  var OPEN_CLASS = "gb-staged-preview";

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function fmtBytes(bytes) {
    var value = Number(bytes) || 0;
    if (value < 1024) return value + " B";
    if (value < 1048576) return Math.round(value / 1024) + " KB";
    return (value / 1048576).toFixed(1) + " MB";
  }

  function extension(name) {
    return String(name || "").split(".").pop().toUpperCase();
  }

  function isImage(file) {
    if (file.type && file.type.indexOf("image/") === 0) return true;
    return /\.(png|jpe?g|gif|webp|bmp|svg)$/i.test(file.name || "");
  }

  function close(panel) {
    if (!panel) return;
    var thumbs = panel.querySelectorAll("img[data-blob-url]");
    for (var index = 0; index < thumbs.length; index++) {
      try { URL.revokeObjectURL(thumbs[index].dataset.blobUrl); } catch (error) { /* gone */ }
    }
    panel.remove();
  }

  function buildRow(file) {
    var row = document.createElement("div");
    row.className = "gb-staged-row";

    if (isImage(file)) {
      var image = document.createElement("img");
      image.className = "gb-staged-thumb";
      var url = URL.createObjectURL(file);
      image.dataset.blobUrl = url;
      image.src = url;
      image.alt = "";
      row.appendChild(image);
    } else {
      var badge = document.createElement("span");
      badge.className = "gb-staged-badge";
      badge.textContent = extension(file.name);
      row.appendChild(badge);
    }

    var meta = document.createElement("span");
    meta.className = "gb-staged-meta";
    var name = document.createElement("span");
    name.className = "gb-staged-name";
    name.textContent = file.name;
    var size = document.createElement("span");
    size.className = "gb-staged-size";
    size.textContent = fmtBytes(file.size);
    meta.appendChild(name);
    meta.appendChild(size);
    row.appendChild(meta);
    return row;
  }

  function toggle() {
    var chip = document.getElementById("fileChip");
    var input = document.getElementById("messageFile");
    if (!chip || !input) return;
    var existing = document.getElementById(OPEN_CLASS);
    if (existing) {
      close(existing);
      return;
    }
    var files = input.files;
    if (!files || !files.length) return;

    var panel = document.createElement("div");
    panel.id = OPEN_CLASS;
    panel.className = OPEN_CLASS;
    panel.setAttribute("role", "region");
    panel.setAttribute("aria-label", t("chat-staged-files", "Attached files"));

    for (var index = 0; index < files.length; index++) {
      panel.appendChild(buildRow(files[index]));
    }
    chip.insertAdjacentElement("afterend", panel);

    var closeBtn = document.createElement("button");
    closeBtn.type = "button";
    closeBtn.className = "gb-staged-close";
    closeBtn.setAttribute("aria-label", t("chat-staged-close", "Close preview"));
    closeBtn.textContent = "×";
    closeBtn.addEventListener("click", function (event) {
      event.stopPropagation();
      close(panel);
    });
    panel.appendChild(closeBtn);
  }

  function bind() {
    var chip = document.getElementById("fileChip");
    if (!chip || chip.dataset.gbPreviewBound === "1") return;
    chip.dataset.gbPreviewBound = "1";
    chip.classList.add("gb-file-chip-clickable");
    chip.addEventListener("click", toggle);
    chip.addEventListener("keydown", function (event) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        toggle();
      }
    });
  }

  window.GBChatStagedFiles = { toggle: toggle };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
  window.addEventListener("gb-chat-ready", bind);
})();
