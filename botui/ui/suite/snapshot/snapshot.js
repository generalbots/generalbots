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

    capture.addEventListener("click", async function () {
      try {
        mediaStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
        var video = document.createElement("video");
        video.srcObject = mediaStream;
        await video.play();
        await new Promise(function (r) { video.onloadedmetadata = r; });
        var canvas = document.createElement("canvas");
        canvas.width = video.videoWidth || 1280;
        canvas.height = video.videoHeight || 720;
        canvas.getContext("2d").drawImage(video, 0, 0);
        dataUrl = canvas.toDataURL("image/png");
        img.src = dataUrl;
        enable(true);
        mediaStream.getTracks().forEach(function (t) { t.stop(); });
      } catch (e) { /* user cancelled */ }
    });

    copyBtn.addEventListener("click", async function () {
      if (!dataUrl) return;
      try {
        var blob = await (await fetch(dataUrl)).blob();
        await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
      } catch (e) { /* clipboard denied */ }
    });

    dlBtn.addEventListener("click", function () {
      if (!dataUrl) return;
      var a = document.createElement("a");
      a.href = dataUrl;
      a.download = "snapshot-" + Date.now() + ".png";
      a.click();
    });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot, { once: true });
  else boot();
})();
