"use strict";
/* GB Snapshot (#1154): capture the screen via getDisplayMedia and provide
 * copy-to-clipboard / PNG download. Frames are held in memory only. */
(function () {
  function root() { return document.getElementById("gb-snap-root"); }

  function boot() {
    var box = root();
    if (!box || box.dataset.snapInit === "1") return;
    box.dataset.snapInit = "1";
    var mediaStream = null;
    var dataUrl = null;

    var capture = box.querySelector("#gb-snap-capture");
    var copyBtn = box.querySelector("#gb-snap-copy");
    var dlBtn = box.querySelector("#gb-snap-download");
    var img = box.querySelector("#gb-snap-preview");

    function enable(has) {
      copyBtn.disabled = !has;
      dlBtn.disabled = !has;
      img.hidden = !has;
    }
    function toast(msg, type) {
      var kind = type || "info";
      if (window.GBToasts && window.GBToasts.show) {
        window.GBToasts.show("Snapshot", msg, kind);
      } else {
        console[kind === "error" ? "error" : "log"]("Snapshot:", msg);
      }
    }

    capture.addEventListener("click", async function () {
      mediaStream = null;
      try {
        mediaStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
        var video = document.createElement("video");
        video.srcObject = mediaStream;
        video.muted = true;
        await video.play();
        await new Promise(function (r, rej) {
          if (video.readyState >= 1) { r(); return; }
          video.onloadedmetadata = function () { r(); };
          video.onerror = function () { rej(new Error("Video load failed")); };
        });
        var canvas = document.createElement("canvas");
        canvas.width = video.videoWidth || 1280;
        canvas.height = video.videoHeight || 720;
        var ctx = canvas.getContext("2d");
        if (!ctx) throw new Error("Canvas context unavailable");
        ctx.drawImage(video, 0, 0);
        dataUrl = canvas.toDataURL("image/png");
        img.src = dataUrl;
        enable(true);
      } catch (e) {
        if (e && (e.name === "NotAllowedError" || e.name === "AbortError")) {
          // user cancelled - silent
        } else {
          toast((e && e.message) ? e.message : "Capture failed", "error");
          console.error("Snapshot capture failed:", e);
        }
      } finally {
        if (mediaStream) {
          try { mediaStream.getTracks().forEach(function (t) { t.stop(); }); } catch (_) { window.gbLogError ? gbLogError(_, "onerror") : console.error("onerror", _); }
          mediaStream = null;
        }
      }
    });

    copyBtn.addEventListener("click", async function () {
      if (!dataUrl) return;
      try {
        var blob = await (await fetch(dataUrl)).blob();
        await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
        toast((typeof window.gbT === "function" ? window.gbT("snapshot-copied", "Copied to clipboard") : "Copied to clipboard"), "success");
      } catch (e) {
        toast((typeof window.gbT === "function" ? window.gbT("snapshot-copy-failed", "Copy failed: clipboard or permissions not available") : "Copy failed: clipboard or permissions not available"), "error");
        console.error("Snapshot copy failed:", e);
      }
    });

    dlBtn.addEventListener("click", function () {
      if (!dataUrl) return;
      try {
        var a = document.createElement("a");
        a.href = dataUrl;
        a.download = "snapshot-" + Date.now() + ".png";
        a.click();
      } catch (e) {
        toast((typeof window.gbT === "function" ? window.gbT("app-kit-download-failed", "Download failed") : "Download failed"), "error");
        console.error("Snapshot download failed:", e);
      }
    });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot, { once: true });
  else boot();
})();
