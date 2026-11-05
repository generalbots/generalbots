/* #1693 — attach files from Drive to a chat turn.
 *
 * The composer's paperclip handles local files. This picker lists the bot's
 * Drive with the same API the Drive app uses, navigates folders, downloads the
 * chosen file as a Blob and hands it to the existing input, so the send path
 * (multi-file payload, staging, preview) is unchanged.
 */
"use strict";

window.GBDrivePicker = window.GBDrivePicker || (function () {
  var overlay = null;
  var listEl = null;
  var crumbEl = null;
  var statusEl = null;

  function token() {
    try {
      return (
        (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        ""
      );
    } catch (e) {
      return "";
    }
  }

  function headers(extra) {
    var value = token();
    var base = extra || {};
    if (value) base.Authorization = "Bearer " + value;
    return base;
  }

  function formatSize(bytes) {
    var n = Number(bytes) || 0;
    if (n < 1024) return n + " B";
    if (n < 1024 * 1024) return Math.round(n / 1024) + " KB";
    return (n / (1024 * 1024)).toFixed(1) + " MB";
  }

  function close() {
    if (overlay) overlay.remove();
    document.removeEventListener("keydown", onPickerKey);
    overlay = null;
  }

  function onPickerKey(event) {
    if (event.key === "Escape" && overlay) {
      event.preventDefault();
      close();
    }
  }

  function ensureOverlay() {
    if (overlay) return;
    overlay = document.createElement("div");
    overlay.className = "gb-drive-picker-overlay";
    overlay.innerHTML =
      '<div class="gb-drive-picker" role="dialog" aria-modal="true" aria-label="Attach from Drive">' +
      '<div class="gb-drive-picker-head">' +
      '<span class="gb-drive-picker-title">Attach from Drive</span>' +
      '<button type="button" class="gb-drive-picker-close" aria-label="Close">×</button>' +
      "</div>" +
      '<div class="gb-drive-picker-crumb"></div>' +
      '<div class="gb-drive-picker-status" hidden></div>' +
      '<div class="gb-drive-picker-list" role="list"></div>' +
      "</div>";
    overlay.addEventListener("click", function (event) {
      if (event.target === overlay) close();
    });
    overlay.querySelector(".gb-drive-picker-close").addEventListener("click", close);
    document.addEventListener("keydown", onPickerKey);
    document.body.appendChild(overlay);
    listEl = overlay.querySelector(".gb-drive-picker-list");
    crumbEl = overlay.querySelector(".gb-drive-picker-crumb");
    statusEl = overlay.querySelector(".gb-drive-picker-status");
  }

  function setStatus(text) {
    if (!statusEl) return;
    statusEl.hidden = !text;
    statusEl.textContent = text || "";
  }

  function crumb(path) {
    if (!crumbEl) return;
    crumbEl.innerHTML = "";
    var root = document.createElement("button");
    root.type = "button";
    root.className = "gb-drive-picker-crumb-btn";
    root.textContent = "Drive";
    root.addEventListener("click", function () { load(""); });
    crumbEl.appendChild(root);
    var parts = String(path || "").split("/").filter(Boolean);
    var walked = "";
    parts.forEach(function (part) {
      walked += (walked ? "/" : "") + part;
      var next = walked;
      var sep = document.createElement("span");
      sep.className = "gb-drive-picker-sep";
      sep.textContent = "/";
      crumbEl.appendChild(sep);
      var btn = document.createElement("button");
      btn.type = "button";
      btn.className = "gb-drive-picker-crumb-btn";
      btn.textContent = part;
      btn.addEventListener("click", function () { load(next); });
      crumbEl.appendChild(btn);
    });
  }

  function row(item) {
    var el = document.createElement("button");
    el.type = "button";
    el.className = "gb-drive-picker-row" + (item.is_dir ? " is-dir" : "");
    el.setAttribute("role", "listitem");
    var icon = document.createElement("span");
    icon.className = "gb-drive-picker-icon";
    icon.textContent = item.is_dir ? "📁" : "📄";
    el.appendChild(icon);
    var name = document.createElement("span");
    name.className = "gb-drive-picker-name";
    name.textContent = item.name;
    el.appendChild(name);
    var meta = document.createElement("span");
    meta.className = "gb-drive-picker-meta";
    meta.textContent = item.is_dir ? "folder" : formatSize(item.size);
    el.appendChild(meta);
    if (!item.is_dir) {
      var attach = document.createElement("span");
      attach.className = "gb-drive-picker-attach";
      attach.textContent = "Attach";
      el.appendChild(attach);
    }
    el.addEventListener("click", function () {
      if (item.is_dir) load(item.path);
      else attachFile(item);
    });
    return el;
  }

  function load(path) {
    ensureOverlay();
    crumb(path);
    setStatus("Loading…");
    listEl.innerHTML = "";
    fetch("/api/files/list?scope=bot&path=" + encodeURIComponent(path || ""), {
      headers: headers(),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (items) {
        setStatus("");
        var list = Array.isArray(items) ? items : [];
        if (!list.length) setStatus("This folder is empty.");
        list
          .filter(function (item) { return item && item.name; })
          .sort(function (a, b) {
            if (!!a.is_dir !== !!b.is_dir) return a.is_dir ? -1 : 1;
            return String(a.name).localeCompare(String(b.name));
          })
          .forEach(function (item) { listEl.appendChild(row(item)); });
      })
      .catch(function (error) {
        setStatus("Could not list Drive: " + (error && error.message ? error.message : error));
      });
  }

  function attachFile(item) {
    setStatus("Attaching " + item.name + "…");
    fetch("/api/files/download-binary", {
      method: "POST",
      headers: headers({ "Content-Type": "application/json" }),
      body: JSON.stringify({ path: item.path, scope: "bot" }),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.blob();
      })
      .then(function (blob) {
        var input = document.getElementById("messageFile");
        if (!input) throw new Error("composer input missing");
        var file = new File([blob], item.name, {
          type: blob.type || "application/octet-stream",
        });
        var transfer = new DataTransfer();
        Array.prototype.slice.call(input.files || []).forEach(function (existing) {
          transfer.items.add(existing);
        });
        transfer.items.add(file);
        input.files = transfer.files;
        input.dispatchEvent(new Event("change", { bubbles: true }));
        close();
        if (typeof notify === "function") notify("Attached from Drive: " + item.name, "success");
      })
      .catch(function (error) {
        setStatus("Could not attach: " + (error && error.message ? error.message : error));
      });
  }

  function open() {
    ensureOverlay();
    load("");
  }

  function injectButton() {
    if (document.getElementById("driveAttachBtn")) return;
    var fileBtn = document.getElementById("fileBtn");
    if (!fileBtn || !fileBtn.parentNode) return;
    var button = document.createElement("button");
    button.type = "button";
    button.id = "driveAttachBtn";
    button.className = "file-btn";
    button.title = "Attach from Drive";
    button.setAttribute("aria-label", "Attach from Drive");
    button.innerHTML =
      '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">' +
      '<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>' +
      "</svg>";
    button.addEventListener("click", function (event) {
      event.preventDefault();
      open();
    });
    fileBtn.parentNode.insertBefore(button, fileBtn.nextSibling);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", injectButton);
  } else {
    injectButton();
  }
  // The chat partial can re-render after an app switch.
  document.addEventListener("gb-chat-ready", injectButton);

  return { open: open };
})();
