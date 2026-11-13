(function (window, document) {
  "use strict";
  var N = document.__gb_mod_preview_window;
  if (!N) return;
  var state = N.state;
  var byId = N.byId;
  var MAX_POLLS = N.MAX_POLLS;
  var status = N.status;
  var token = N.token;
  var fileUrl = N.fileUrl;
  var mimeOf = N.mimeOf;
  var signatureOf = N.signatureOf;
  var STABLE_CYCLES = N.STABLE_CYCLES;
  var render = N.render;
  var POLL_MS = N.POLL_MS;
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

  function bytesToBase64(bytes) {
    var binary = "";
    var chunk = 0x8000;
    for (var i = 0; i < bytes.length; i += chunk) {
      binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
    }
    return btoa(binary);
  }

  function saveToDrive() {
    if (!state.conversation || !state.path) return;
    status("Saving to Drive…", "busy");
    var headers = {};
    var t = token();
    if (t) headers.Authorization = "Bearer " + t;
    fetch(fileUrl(state.conversation, state.path), { headers: headers })
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.arrayBuffer();
      })
      .then(function (buffer) {
        var name = String(state.path).split("/").pop() || "file";
        var writeHeaders = { "Content-Type": "application/json" };
        if (t) writeHeaders.Authorization = "Bearer " + t;
        // The user-scope default bucket (`user-<id>`) may not exist; ask the
        // drive API for a bucket this caller can actually write to.
        return fetch("/api/files/buckets", { headers: headers })
          .then(function (r) { return r.ok ? r.json() : []; })
          .catch(function () { return []; })
          .then(function (list) {
            var items = Array.isArray(list) ? list : (list && list.buckets) || [];
            var names = items
              .map(function (b) { return b && (b.name || b.bucket || b); })
              .filter(function (n) { return typeof n === "string" && n; });
            var bucket =
              names.filter(function (n) { return /\.gbai$/.test(n); })[0] || names[0];
            var body = {
              path: name,
              content: bytesToBase64(new Uint8Array(buffer)),
              scope: "user",
            };
            if (bucket) body.bucket = bucket;
            return fetch("/api/files/write", {
              method: "POST",
              headers: writeHeaders,
              body: JSON.stringify(body),
            });
          });
      })
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        status("Saved to Drive", "ok");
        if (window.GBToasts) {
          GBToasts.show("Saved", "File saved to Drive", "success", { appId: "preview", params: { conversation: state.conversation, path: state.path }, label: "Open" });
        }
      })
      .catch(function (err) {
        status("Save failed: " + err.message, "error");
      });
  }

  function shareFile() {
    if (!state.conversation || !state.path) return;
    status("Creating share link…", "busy");
    var headers = { "Content-Type": "application/json" };
    var t = token();
    if (t) headers.Authorization = "Bearer " + t;
    var name = String(state.path).split("/").pop() || "file";
    fetch("/api/agent/drive/share", {
      method: "POST",
      headers: headers,
      body: JSON.stringify({ conversation_id: state.conversation, file: "output/" + name }),
    })
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.json();
      })
      .then(function (data) {
        var url = (data && data.url) || "";
        if (!url) throw new Error("no link");
        var absolute = url.indexOf("http") === 0 ? url : window.location.origin + url;
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(absolute).catch(function () {});
        }
        status("Share link copied", "ok");
        if (window.GBToasts) {
          GBToasts.show("Link ready", absolute, "success");
        }
      })
      .catch(function (err) {
        status("Share failed: " + err.message, "error");
      });
  }

  function setName() {
    var name = byId("gbPreviewName");
    if (name) name.textContent = String(state.path || "Preview").split("/").pop();
    var download = byId("gbPreviewDownload");
    var open = byId("gbPreviewOpen");
    var url = state.conversation && state.path ? fileUrl(state.conversation, state.path) : "";
    var save = byId("gbPreviewSave");
    if (save) {
      save.disabled = !url;
      save.onclick = saveToDrive;
    }
    var share = byId("gbPreviewShare");
    if (share) {
      share.disabled = !url;
      share.onclick = shareFile;
    }
    if (download) {
      download.disabled = !url;
      download.onclick = function () {
        if (!url) return;
        var downloadName = String(state.path).split("/").pop() || "file";
        if (window.GBShared && typeof window.GBShared.download === "function") {
          window.GBShared.download(url, downloadName);
          return;
        }
        var link = document.createElement("a");
        link.href = url;
        link.download = downloadName;
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
    // "Preview the last generated file" carries no conversation/path, so fall
    // back to the newest attachment chat rendered and the live session (#2216).
    var last = window.__gbLastArtifact || {};
    var session = (window.ChatState &&
      (window.ChatState.sessionId || window.ChatState.conversationId)) || "";
    return {
      conversation:
        params.conversation || params.conversation_id || last.conversation || session,
      path: params.path || params.file || last.path || "",
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

  N.stop = stop;
  N.poll = poll;
  N.bytesToBase64 = bytesToBase64;
  N.saveToDrive = saveToDrive;
  N.shareFile = shareFile;
  N.setName = setName;
  N.open = open;
  N.readParams = readParams;
  N.bind = bind;
  delete document.__gb_mod_preview_window;
})(window, document);
