"use strict";

// Agent preview window (#1584 follow-up): one surface for everything the
// workspace produces. PDFs and images reload every 3s while the file is still
// being written; HTML is re-rendered as it is built; text/CSV/media use their
// native renderer. Polling stops when the window closes or the file has been
// stable for a few cycles.

(function () {
  var POLL_MS = 3000;
  var STABLE_CYCLES = 3;
  var MAX_POLLS = 200; // ~10 minutes

  var state = {
    conversation: "",
    path: "",
    timer: null,
    polls: 0,
    signature: "",
    stable: 0,
    objectUrl: null,
    bound: false,
  };

  function byId(id) {
    return document.getElementById(id);
  }

  function token() {
    try {
      if (window.getGBAccessToken) {
        var t = window.getGBAccessToken();
        if (t) return t;
      }
      return (
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") ||
        ""
      );
    } catch (e) {
      return "";
    }
  }

  function fileUrl(conversation, path) {
    var url =
      "/api/agent/vm/file?conversation_id=" +
      encodeURIComponent(conversation) +
      "&path=" +
      encodeURIComponent(path) +
      "&raw=1";
    var t = token();
    if (t) url += "&access_token=" + encodeURIComponent(t);
    return url;
  }

  function mimeOf(path) {
    var ext = String(path || "").split(".").pop().toLowerCase();
    return (
      {
        pdf: "application/pdf",
        png: "image/png",
        jpg: "image/jpeg",
        jpeg: "image/jpeg",
        gif: "image/gif",
        webp: "image/webp",
        svg: "image/svg+xml",
        html: "text/html",
        htm: "text/html",
        csv: "text/csv",
        json: "application/json",
        txt: "text/plain",
        md: "text/markdown",
        py: "text/plain",
        js: "text/plain",
        sh: "text/plain",
        mp4: "video/mp4",
        mp3: "audio/mpeg",
        wav: "audio/wav",
        zip: "application/zip",
        xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        xls: "application/vnd.ms-excel",
        docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        doc: "application/msword",
        pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ppt: "application/vnd.ms-powerpoint",
      }[ext] || "application/octet-stream"
    );
  }

  function status(text, kind) {
    var el = byId("gbPreviewStatus");
    if (!el) return;
    el.textContent = text || "";
    el.className = "gb-preview-status" + (kind ? " " + kind : "");
  }

  function revoke() {
    if (state.objectUrl) {
      try {
        URL.revokeObjectURL(state.objectUrl);
      } catch (e) {
        /* already revoked */
      }
      state.objectUrl = null;
    }
  }

  function textPreview(bytes, mime) {
    var limit = 256 * 1024;
    var slice = bytes.length > limit ? bytes.slice(0, limit) : bytes;
    var text = new TextDecoder("utf-8", { fatal: false }).decode(slice).replace(/\u0000/g, "");
    var suffix = bytes.length > limit ? "\n\n… (truncated)" : "";
    if (mime === "text/csv") return csvPreview(text);
    var pre = document.createElement("pre");
    pre.className = "gb-preview-text";
    pre.textContent = text + suffix;
    return pre;
  }

  function csvPreview(text) {
    var rows = text.split(/\r?\n/).filter(function (line, i) {
      return i < 500 && line.length > 0;
    });
    var table = document.createElement("table");
    table.className = "gb-preview-table";
    rows.forEach(function (line, index) {
      var tr = document.createElement("tr");
      // Simple comma split: a preview, not a CSV parser.
      line.split(",").forEach(function (cell) {
        var node = document.createElement(index === 0 ? "th" : "td");
        node.textContent = cell.replace(/^"|"$/g, "");
        tr.appendChild(node);
      });
      table.appendChild(tr);
    });
    return table;
  }

  function render(bytes, mime) {
    var body = byId("gbPreviewBody");
    if (!body) return;
    revoke();
    body.innerHTML = "";

    if (mime === "text/html") {
      var html = new TextDecoder("utf-8", { fatal: false }).decode(bytes);
      var frame = document.createElement("iframe");
      frame.className = "gb-preview-frame";
      // Generated pages are untrusted: scripts may run, but in an isolated
      // origin (no allow-same-origin) so they cannot touch the app.
      frame.setAttribute("sandbox", "allow-scripts allow-forms allow-popups allow-modals");
      frame.srcdoc = html;
      body.appendChild(frame);
      return;
    }

    var blob = new Blob([bytes], { type: mime });
    state.objectUrl = URL.createObjectURL(blob);

    if (mime.indexOf("image/") === 0) {
      var img = document.createElement("img");
      img.className = "gb-preview-image";
      img.alt = state.path;
      img.src = state.objectUrl;
      body.appendChild(img);
      return;
    }
    if (mime === "application/pdf") {
      var pdf = document.createElement("iframe");
      pdf.className = "gb-preview-frame";
      pdf.src = state.objectUrl;
      body.appendChild(pdf);
      return;
    }
    if (mime === "video/mp4") {
      var video = document.createElement("video");
      video.className = "gb-preview-media";
      video.controls = true;
      video.src = state.objectUrl;
      body.appendChild(video);
      return;
    }
    if (mime.indexOf("audio/") === 0) {
      var audio = document.createElement("audio");
      audio.className = "gb-preview-media";
      audio.controls = true;
      audio.src = state.objectUrl;
      body.appendChild(audio);
      return;
    }
    if (
      mime.indexOf("text/") === 0 ||
      mime === "application/json" ||
      mime === "text/markdown"
    ) {
      body.appendChild(textPreview(bytes, mime));
      return;
    }

    // Office documents have no browser renderer. Show the file itself with
    // its size and a working download instead of a dead error.
    var card = document.createElement("div");
    card.className = "gb-preview-filecard";
    var icon = document.createElement("div");
    icon.className = "gb-preview-fileicon";
    icon.textContent = String(state.path || "").split(".").pop().toUpperCase();
    var meta = document.createElement("div");
    meta.className = "gb-preview-filemeta";
    var name = document.createElement("div");
    name.className = "gb-preview-filename";
    name.textContent = String(state.path || "").split("/").pop();
    var size = document.createElement("div");
    size.className = "gb-preview-filesize";
    size.textContent = (bytes.length / 1024).toFixed(1) + " KB · opens after download";
    var button = document.createElement("button");
    button.type = "button";
    button.className = "gb-preview-btn";
    button.textContent = "Download";
    button.onclick = function () {
      var link = document.createElement("a");
      link.href = fileUrl(state.conversation, state.path);
      link.download = String(state.path).split("/").pop() || "file";
      document.body.appendChild(link);
      link.click();
      link.remove();
    };
    meta.appendChild(name);
    meta.appendChild(size);
    card.appendChild(icon);
    card.appendChild(meta);
    card.appendChild(button);
    body.appendChild(card);
  }

  function signatureOf(bytes) {
    var step = Math.max(1, Math.floor(bytes.length / 512));
    var sum = 0;
    for (var i = 0; i < bytes.length; i += step) sum = (sum + bytes[i] * (i + 1)) % 2147483647;
    return bytes.length + ":" + sum;
  }

  function stop() {
    if (state.timer) {
      clearInterval(state.timer);
      state.timer = null;
    }
  }

  function poll() {
    if (!byId("gbPreview")) {
      stop();
      return;
    }
    if (!state.conversation || !state.path) return;
    state.polls += 1;
    if (state.polls > MAX_POLLS) {
      stop();
      status("Stopped watching", "warn");
      return;
    }
    var headers = {};
    var t = token();
    if (t) headers.Authorization = "Bearer " + t;
    fetch(fileUrl(state.conversation, state.path), { headers: headers })
      .then(function (res) {
        if (res.status === 404) {
          status("Waiting for the file…", "busy");
          return null;
        }
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.arrayBuffer().then(function (buffer) {
          return { bytes: new Uint8Array(buffer), mime: mimeOf(state.path) };
        });
      })
      .then(function (data) {
        if (!data) return;
        var signature = signatureOf(data.bytes);
        if (signature === state.signature) {
          state.stable += 1;
          if (state.stable === STABLE_CYCLES) {
            status("Up to date", "ok");
            stop();
          }
          return;
        }
        state.signature = signature;
        state.stable = 0;
        render(data.bytes, data.mime);
        status("Updated · watching for changes", "busy");
      })
      .catch(function (err) {
        status("Preview failed: " + err.message, "error");
      });
  }

  function setName() {
    var name = byId("gbPreviewName");
    if (name) name.textContent = String(state.path || "Preview").split("/").pop();
    var download = byId("gbPreviewDownload");
    var open = byId("gbPreviewOpen");
    var url = state.conversation && state.path ? fileUrl(state.conversation, state.path) : "";
    if (download) {
      download.disabled = !url;
      download.onclick = function () {
        if (!url) return;
        var link = document.createElement("a");
        link.href = url;
        link.download = String(state.path).split("/").pop() || "file";
        document.body.appendChild(link);
        link.click();
        link.remove();
      };
    }
    if (open) {
      open.disabled = !url;
      open.onclick = function () {
        if (url) window.open(url, "_blank", "noopener");
      };
    }
  }

  function open(conversation, path) {
    state.conversation = conversation || "";
    state.path = path || "";
    state.polls = 0;
    state.signature = "";
    state.stable = 0;
    stop();
    setName();
    if (!state.conversation || !state.path) {
      status("", "");
      return;
    }
    status("Loading…", "busy");
    var body = byId("gbPreviewBody");
    if (body) body.innerHTML = "";
    poll();
    state.timer = setInterval(poll, POLL_MS);
  }

  function readParams() {
    var params = (window.__gbAppParams__ || {});
    // Do not clear: the window manager may pass params again on re-target.
    return {
      conversation: params.conversation || params.conversation_id || "",
      path: params.path || params.file || "",
    };
  }

  function bind() {
    if (state.bound) return;
    var root = byId("gbPreview");
    if (!root) return;
    state.bound = true;
    document.addEventListener("gb:deep-link", function (event) {
      var detail = event.detail || {};
      if (detail.appId && detail.appId !== "preview") return;
      var params = detail.params || {};
      open(
        params.conversation || params.conversation_id || "",
        params.path || params.file || ""
      );
    });
    var initial = readParams();
    open(initial.conversation, initial.path);
  }

  window.GBPreview = { open: open };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
})();
