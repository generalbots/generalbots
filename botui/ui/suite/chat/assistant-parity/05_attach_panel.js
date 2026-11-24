/* Assistant parity — generic attachment preview panel inside classic chat.
 *
 * Chat already has two *specific* preview surfaces: the artifact overlay
 * (GBArtifactOverlay + .gb-inline-artifact / .gb-live-artifact) and the VM
 * attachment cards (.gb-attachment, bound by modules/vm_attachments). Those
 * own their clicks. This module only adds the assistant's generic panel for
 * attachment chips that chat does not already handle (a[href], [data-file],
 * [data-name], *.attach* / *.file*).
 */
"use strict";
(function () {
  var P = window.GBChatAssistantParity = window.GBChatAssistantParity || {};
  // Local wrapper keeps gbT() call sites literal while tolerating a late load.
  function gbT(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function el(tag, cls, text) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text !== undefined && text !== null) node.textContent = String(text);
    return node;
  }
  var genCounter = 0;
  var state = null; // { gen, item, nodes, controller, objectUrls }
  var mountListening = false;

  /* ------------------------------------------------------------- helpers */
  function ext(name) {
    var parts = String(name || "").split(".");
    return parts.length > 1 ? parts.pop().toLowerCase() : "";
  }
  function inExt(name, list) {
    return list.indexOf(ext(name)) >= 0;
  }
  function isImage(name) {
    return inExt(name, "jpg jpeg png gif webp bmp svg ico avif".split(" "));
  }
  function isText(name) {
    return inExt(name, "txt md csv json js ts py rs html css xml yaml yml toml sh bas log ini".split(" "));
  }
  function isDoc(name) {
    return ext(name) === "pdf" ||
      inExt(name, "docx xlsx pptx doc xls ppt odt ods odp".split(" "));
  }
  function fmtBytes(bytes) {
    var n = Number(bytes) || 0;
    if (n < 1024) return n + " B";
    if (n < 1048576) return Math.round(n / 1024) + " KB";
    return (n / 1048576).toFixed(1) + " MB";
  }
  function alive(gen) {
    return !!state && state.gen === gen;
  }
  function trackUrl(url) {
    if (state) state.objectUrls.push(url);
    return url;
  }
  function botName() {
    return (window.ChatState && ChatState.currentBotName) ||
      window.__INITIAL_BOT_NAME__ || "default";
  }
  function token() {
    try {
      if (typeof window.getChatAccessToken === "function") {
        return window.getChatAccessToken() || "";
      }
      if (window.GBSecurity && typeof window.GBSecurity.getToken === "function") {
        return window.GBSecurity.getToken() || "";
      }
      return sessionStorage.getItem("gb-access-token") || "";
    } catch (e) {
      return "";
    }
  }
  // Same auth pattern as chat-drive-picker.js: Bearer header, never in the URL.
  function authHeaders(extra) {
    var base = extra || {};
    var value = token();
    if (value) base.Authorization = "Bearer " + value;
    return base;
  }
  function previewUrl(name, item, download) {
    var path = (item && item.drivePath) || "";
    var inboxName = path.indexOf("inbox/") === 0 ? path.substring(6) : name;
    var url = "/api/chat/attachments/preview?bot=" +
      encodeURIComponent(botName()) + "&name=" + encodeURIComponent(inboxName);
    return download ? url + "&download=1" : url;
  }
  // Aborted on close; gen check drops responses from a previous panel.
  function guardedFetch(url, options, gen) {
    if (!alive(gen)) return Promise.reject(new Error("closed"));
    var opts = options || {};
    opts.headers = authHeaders(opts.headers);
    if (state.controller) opts.signal = state.controller.signal;
    return fetch(url, opts).then(function (response) {
      if (!alive(gen)) throw new Error("closed");
      if (!response.ok) throw new Error("HTTP " + response.status);
      return response;
    });
  }

  /* ------------------------------------------------------- panel lifecycle */
  function onKeyDown(event) {
    if (!state) return;
    if (event.key === "Escape" || event.key === "Esc") {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }
  function close() {
    if (!state) return;
    var done = state;
    state = null;
    if (done.controller) {
      try { done.controller.abort(); } catch (e) { /* already aborted */ }
    }
    done.objectUrls.forEach(function (url) {
      try { URL.revokeObjectURL(url); } catch (e) { /* already revoked */ }
    });
    if (done.nodes.overlay.parentNode) {
      done.nodes.overlay.parentNode.removeChild(done.nodes.overlay);
    }
    document.removeEventListener("keydown", onKeyDown, true);
  }
  function setStatus(text) {
    if (!state) return;
    var region = state.nodes.status;
    region.textContent = text || "";
    region.hidden = !text;
  }
  function buildPanel(item) {
    var overlay = el("div", "gb-parity-attach-panel");
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    overlay.setAttribute("aria-label", gbT("chat-parity-attach-title", "Attachment preview"));
    var card = el("div", "gb-parity-attach-panel-card");
    var head = el("div", "gb-parity-attach-panel-head");
    head.appendChild(el("span", "gb-parity-attach-panel-title", item.name));
    var closeBtn = el("button", "gb-parity-attach-panel-close", "\u00d7");
    closeBtn.type = "button";
    closeBtn.setAttribute("aria-label", gbT("action-close", "Close"));
    closeBtn.addEventListener("click", close);
    head.appendChild(closeBtn);
    card.appendChild(head);
    var status = el("div", "gb-parity-attach-panel-status");
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("aria-atomic", "true");
    status.hidden = true;
    card.appendChild(status);
    var body = el("div", "gb-parity-attach-panel-body");
    card.appendChild(body);
    var actions = el("div", "gb-parity-attach-panel-actions");
    var download = el("button", "gb-parity-attach-panel-download", gbT("action-download", "Download"));
    download.type = "button";
    download.addEventListener("click", downloadCurrent);
    actions.appendChild(download);
    card.appendChild(actions);
    overlay.appendChild(card);
    overlay.addEventListener("click", function (event) {
      if (event.target === overlay) close();
    });
    (document.getElementById("chat-app") || document.body).appendChild(overlay);
    return { overlay: overlay, body: body, status: status, download: download };
  }

  /* --------------------------------------------------------- render helpers */
  function metaBar(body, item) {
    var bar = el("div", "gb-parity-attach-panel-meta");
    bar.appendChild(el("span", "gb-parity-attach-panel-meta-name", item.name));
    if (item.type) bar.appendChild(el("span", "gb-parity-attach-panel-meta-info", item.type));
    if (item.size) bar.appendChild(el("span", "gb-parity-attach-panel-meta-info", fmtBytes(item.size)));
    body.appendChild(bar);
  }
  function showImage(body, url, name) {
    var lightbox = el("div", "gb-parity-attach-lightbox");
    var backdrop = el("div", "gb-parity-attach-lightbox-backdrop");
    backdrop.addEventListener("click", close);
    lightbox.appendChild(backdrop);
    var img = el("img", "gb-parity-attach-lightbox-img");
    img.src = url;
    img.alt = name;
    lightbox.appendChild(img);
    body.appendChild(lightbox);
  }
  function showPre(body, item, text) {
    metaBar(body, item);
    var pre = el("pre", "gb-parity-attach-panel-pre");
    pre.textContent = text === null || text === undefined ? "" : String(text);
    body.appendChild(pre);
    if (!pre.textContent) {
      body.appendChild(el("div", "gb-parity-attach-panel-note", gbT("label-empty", "Empty")));
    }
  }
  function showDocHtml(body, item, html) {
    metaBar(body, item);
    var frame = el("iframe", "gb-parity-attach-panel-frame");
    frame.setAttribute("sandbox", "allow-same-origin"); // never allow-scripts
    frame.setAttribute("referrerpolicy", "no-referrer");
    frame.setAttribute("title", item.name);
    frame.srcdoc = html;
    body.appendChild(frame);
  }
  function showPdf(body, item, url) {
    metaBar(body, item);
    var frame = el("iframe", "gb-parity-attach-panel-frame gb-parity-attach-panel-pdf");
    frame.setAttribute("sandbox", "");
    frame.setAttribute("referrerpolicy", "no-referrer");
    frame.setAttribute("title", item.name);
    frame.src = url.split("#")[0] + "#toolbar=0";
    body.appendChild(frame);
  }
  function showMeta(body, item) {
    metaBar(body, item);
    var note = item.blobUrl
      ? gbT("chat-preview-load-failed", "Could not load the preview.")
      : gbT("chat-parity-attach-unavailable", "Content unavailable - use Download to open this file.");
    body.appendChild(el("div", "gb-parity-attach-panel-note", note));
  }

  /* ---------------------------------------------------------- content load */
  function load(item, gen) {
    var body = state.nodes.body;
    var name = item.name;
    if (isImage(name)) {
      if (item.blobUrl && /^(blob:|data:)/.test(item.blobUrl)) {
        showImage(body, item.blobUrl, name);
        setStatus("");
        return;
      }
      guardedFetch(previewUrl(name, item, false), {}, gen)
        .then(function (response) { return response.blob(); })
        .then(function (blob) {
          if (alive(gen)) showImage(body, trackUrl(URL.createObjectURL(blob)), name);
        })
        .catch(function () { if (alive(gen)) showMeta(body, item); })
        .then(function () { if (alive(gen)) setStatus(""); });
      return;
    }
    if (isText(name)) {
      if (typeof item.text === "string") {
        showPre(body, item, item.text);
        setStatus("");
        return;
      }
      if (item.blobUrl && /^(blob:|data:)/.test(item.blobUrl)) {
        fetch(item.blobUrl, { signal: state.controller ? state.controller.signal : undefined })
          .then(function (response) { return response.text(); })
          .then(function (text) { if (alive(gen)) showPre(body, item, text); })
          .catch(function () { if (alive(gen)) showMeta(body, item); })
          .then(function () { if (alive(gen)) setStatus(""); });
        return;
      }
      guardedFetch(previewUrl(name, item, false), {}, gen)
        .then(function (response) { return response.text(); })
        .then(function (text) { if (alive(gen)) showPre(body, item, text); })
        .catch(function () { if (alive(gen)) showMeta(body, item); })
        .then(function () { if (alive(gen)) setStatus(""); });
      return;
    }
    if (isDoc(name)) {
      guardedFetch(previewUrl(name, item, false), {}, gen)
        .then(function (response) {
          var ct = response.headers.get("content-type") || "";
          if (ct.indexOf("text/html") >= 0 || ct.indexOf("xml") >= 0) {
            return response.text().then(function (html) {
              if (alive(gen)) showDocHtml(body, item, html);
            });
          }
          if (ct.indexOf("pdf") >= 0) {
            return response.blob().then(function (blob) {
              if (alive(gen)) showPdf(body, item, trackUrl(URL.createObjectURL(blob)));
            });
          }
          return response.text().then(function (text) {
            if (alive(gen)) showPre(body, item, text);
          });
        })
        .catch(function () { if (alive(gen)) showMeta(body, item); })
        .then(function () { if (alive(gen)) setStatus(""); });
      return;
    }
    showMeta(body, item);
    setStatus("");
  }
  function triggerDownload(url, name) {
    var anchor = el("a");
    anchor.href = url;
    anchor.download = name;
    anchor.style.display = "none";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
  }
  function downloadCurrent() {
    if (!state) return;
    var item = state.item;
    var gen = state.gen;
    if (item.blobUrl && /^(blob:|data:)/.test(item.blobUrl)) {
      triggerDownload(item.blobUrl, item.name);
      return;
    }
    setStatus(gbT("label-downloading", "Downloading..."));
    guardedFetch(previewUrl(item.name, item, true), {}, gen)
      .then(function (response) { return response.blob(); })
      .then(function (blob) {
        if (alive(gen)) triggerDownload(trackUrl(URL.createObjectURL(blob)), item.name);
      })
      .catch(function () {
        if (alive(gen)) {
          setStatus(gbT("chat-parity-attach-download-failed", "Could not download the attachment."));
        }
      })
      .then(function () { if (alive(gen)) setStatus(""); });
  }

  /* ------------------------------------------------------ chip delegation */
  // Anything chat already owns must be left to chat's own handlers.
  var CHAT_OWNED =
    ".gb-artifact-overlay, .gb-inline-artifact, .gb-attachment-inline," +
    ".gb-live-artifact, .gb-user-preview, .gb-attachment," +
    ".gb-inline-artifact-btn, .gb-attachment-inline-btn, .gb-attachment-btn," +
    "[data-gb-artifact], [data-artifact], [data-artifact-id]," +
    "[data-bound], [data-conversation-id]";
  function chatOwns(node) {
    var current = node;
    while (current && current !== document) {
      if (current.nodeType === 1) {
        if (current.matches && current.matches(CHAT_OWNED)) return true;
        var cls = current.className;
        if (typeof cls === "string" &&
            /(gb-artifact|gb-attachment-inline|gb-live-artifact|gb-user-preview)/i.test(cls)) {
          return true;
        }
      }
      current = current.parentNode;
    }
    return false;
  }
  function findChip(node) {
    var current = node;
    while (current && current !== document && current.id !== "messages") {
      if (current.nodeType === 1) {
        if (current.hasAttribute("data-file") ||
            current.hasAttribute("data-name") ||
            current.hasAttribute("data-filename") ||
            current.hasAttribute("data-path") ||
            (current.tagName === "A" && current.getAttribute("href"))) {
          return current;
        }
        var cls = current.className;
        if (typeof cls === "string" && /(attach|file)/i.test(cls) && !chatOwns(current)) {
          return current;
        }
      }
      current = current.parentNode;
    }
    return null;
  }
  function itemFromChip(chip) {
    var name = chip.getAttribute("data-name") ||
      chip.getAttribute("data-file") ||
      chip.getAttribute("data-filename") || "";
    if (!name && chip.tagName === "A") {
      var href = chip.getAttribute("href") || "";
      var clean = href.split(/[?#]/)[0];
      try {
        name = decodeURIComponent(clean.split("/").pop() || "");
      } catch (e) {
        name = clean.split("/").pop() || "";
      }
    }
    if (!name) name = (chip.textContent || "").trim();
    return {
      name: String(name || "file").substring(0, 300),
      drivePath: chip.getAttribute("data-path") || chip.getAttribute("data-drive-path") || "",
      blobUrl: chip.getAttribute("data-blob-url") || chip.getAttribute("src") || "",
      type: chip.getAttribute("data-type") || chip.getAttribute("data-mime") || "",
      size: Number(chip.getAttribute("data-size")) || 0,
      text: null
    };
  }
  function onClick(event) {
    var target = event.target;
    if (!target || !target.closest) return;
    if (chatOwns(target)) return;
    var chip = findChip(target);
    if (!chip) return;
    var item = itemFromChip(chip);
    if (!item.name) return;
    event.preventDefault();
    open(item);
  }
  function bindMessages() {
    var root = document.getElementById("messages");
    if (!root || root.__gbParityAttachBound) return;
    root.__gbParityAttachBound = true;
    root.addEventListener("click", onClick);
  }
  function normalize(item) {
    if (typeof item === "string") {
      return { name: item.substring(0, 300), drivePath: "", blobUrl: "", type: "", size: 0, text: null };
    }
    item = item || {};
    var name = item.name || item.file || item.filename || "file";
    return {
      name: String(name).substring(0, 300),
      drivePath: item.drivePath || item.path || "",
      blobUrl: item.blobUrl || "",
      type: item.type || item.content_type || "",
      size: Number(item.size || item.size_bytes || 0),
      text: typeof item.text === "string" ? item.text : null
    };
  }
  function open(item) {
    var normalized = normalize(item);
    if (!normalized.name) return;
    close();
    var nodes = buildPanel(normalized);
    state = {
      gen: ++genCounter,
      item: normalized,
      nodes: nodes,
      controller: typeof AbortController === "function" ? new AbortController() : null,
      objectUrls: []
    };
    document.addEventListener("keydown", onKeyDown, true);
    setStatus(gbT("label-loading", "Loading..."));
    load(normalized, state.gen);
  }
  function mount() {
    bindMessages();
    if (!mountListening) {
      document.addEventListener("gb-chat-ready", bindMessages);
      mountListening = true;
    }
  }
  P.attachPanel = { mount: mount, open: open };
})();
