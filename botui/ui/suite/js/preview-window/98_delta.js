"use strict";
/* Preview app — gb-doc-delta/v1 finalize (#2216).
   Generation used to end in chat prose because nothing forwarded the finished
   artifact to a preview surface. `finalize` now opens the file in this window
   (VM conversation output or a direct url/html). */
(function (window, document) {
  function body() {
    return document.getElementById("gbPreviewBody");
  }

  function showUrl(url, name) {
    var host = body();
    if (!host || !url) return;
    var status = document.getElementById("gbPreviewStatus");
    var title = document.getElementById("gbPreviewName");
    if (title && name) title.textContent = String(name).split("/").pop();
    if (status) status.textContent = "Preview";
    host.innerHTML = "";
    var ext = String(name || url).split(".").pop().toLowerCase();
    if (["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"].indexOf(ext) !== -1) {
      var img = document.createElement("img");
      img.src = url;
      img.style.cssText = "max-width:100%;max-height:100%;object-fit:contain;display:block;margin:auto;";
      host.appendChild(img);
      return;
    }
    if (["txt", "csv", "md", "json", "log"].indexOf(ext) !== -1) {
      fetch(url).then(function (r) { return r.text(); }).then(function (text) {
        var pre = document.createElement("pre");
        pre.style.cssText = "white-space:pre-wrap;padding:12px;margin:0;font-size:12.5px;";
        pre.textContent = text;
        host.innerHTML = "";
        host.appendChild(pre);
      }).catch(function () { /* keep the window usable */ });
      return;
    }
    var frame = document.createElement("iframe");
    frame.src = url;
    frame.style.cssText = "width:100%;height:100%;border:0;background:#fff;";
    host.appendChild(frame);
  }

  function showHtml(html) {
    var host = body();
    if (!host) return;
    var frame = document.createElement("iframe");
    frame.style.cssText = "width:100%;height:100%;border:0;background:#fff;";
    frame.srcdoc = String(html || "");
    host.innerHTML = "";
    host.appendChild(frame);
  }

  function finalize(op) {
    var path = op.path || op.file || "";
    var conversation = op.conversation || op.conversation_id || "";
    // Direct payloads render here; only a VM output path needs the poller,
    // and that requires the conversation id.
    if (op.html) {
      showHtml(op.html);
      return;
    }
    if (op.url) {
      showUrl(op.url, path || op.title);
      return;
    }
    if (window.GBPreview && path && conversation) {
      window.GBPreview.open(conversation, path);
    }
  }

  function apply(payload) {
    if (!payload || !Array.isArray(payload.ops)) return;
    payload.ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "finalize") finalize(op);
      if (op.op === "clear") {
        var host = body();
        if (host) host.innerHTML = "";
        var status = document.getElementById("gbPreviewStatus");
        if (status) status.textContent = "";
      }
      if (op.op === "error") {
        var s = document.getElementById("gbPreviewStatus");
        if (s) s.textContent = String(op.message || "Preview failed");
      }
    });
  }

  if (window.GBDocDelta && typeof window.GBDocDelta.register === "function") {
    window.GBDocDelta.register("preview", apply);
  } else document.addEventListener("gb-doc-delta", function (e) {
    var detail = (e && e.detail) || {};
    if (detail.app === "preview" || !detail.app) apply(detail);
  });
})(window, document);
