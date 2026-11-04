/* chat/modules/77_camera.js — take a photo in the browser and send it.
 *
 * The camera button opens a live preview, captures one frame to JPEG and
 * stages it as a normal chat attachment. The server already describes image
 * attachments through BotModels (`/api/vision/describe`), so the assistant
 * analyzes the photo with the same open-source stack as uploaded images.
 * Browsers without getUserMedia fall back to the native capture input.
 */
"use strict";

(function () {
  var stream = null;
  var overlay = null;

  function tr(key, fallback) {
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      return value === key ? fallback : value;
    }
    return fallback;
  }

  function notify(message, level) {
    if (window.GBToasts && typeof GBToasts.show === "function") {
      GBToasts.show(tr("chat-camera", "Camera"), message, level || "info");
    } else if (typeof window.showToast === "function") {
      window.showToast(message, level || "info");
    } else if (level === "error") {
      console.warn("camera:", message);
    }
  }

  function stopStream() {
    if (!stream) return;
    stream.getTracks().forEach(function (track) { track.stop(); });
    stream = null;
  }

  function close() {
    stopStream();
    if (overlay && overlay.parentNode) overlay.parentNode.removeChild(overlay);
    overlay = null;
  }

  function stageAndSend(file) {
    var input = document.getElementById("messageFile");
    if (!input || !file) return false;
    try {
      var transfer = new DataTransfer();
      transfer.items.add(file);
      input.files = transfer.files;
    } catch (e) {
      notify(tr("chat-camera-error", "Could not attach the photo."), "error");
      return false;
    }
    if (typeof window.sendMessage === "function") {
      window.sendMessage("");
      return true;
    }
    return false;
  }

  function capture(video) {
    var width = video.videoWidth || 1280;
    var height = video.videoHeight || 720;
    var canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    var context = canvas.getContext("2d");
    if (!context) {
      notify(tr("chat-camera-error", "Could not attach the photo."), "error");
      return;
    }
    context.drawImage(video, 0, 0, width, height);
    canvas.toBlob(function (blob) {
      if (!blob) {
        notify(tr("chat-camera-error", "Could not attach the photo."), "error");
        close();
        return;
      }
      var file = new File([blob], "camera-" + Date.now() + ".jpg", { type: "image/jpeg" });
      var sent = stageAndSend(file);
      close();
      if (sent) notify(tr("chat-camera-sent", "Photo sent."), "success");
    }, "image/jpeg", 0.9);
  }

  function buildPreview() {
    overlay = document.createElement("div");
    overlay.id = "gbCamera";
    overlay.className = "gb-camera";
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    overlay.innerHTML =
      '<div class="gb-camera-card">' +
      '<div class="gb-camera-head"><span>' + tr("chat-camera-title", "Take a photo") + "</span>" +
      '<button type="button" class="gb-camera-close" aria-label="' +
      tr("chat-camera-cancel", "Cancel") + '">✕</button></div>' +
      '<video autoplay playsinline muted></video>' +
      '<div class="gb-camera-actions">' +
      '<button type="button" class="gb-camera-cancel">' + tr("chat-camera-cancel", "Cancel") + "</button>" +
      '<button type="button" class="gb-camera-shutter">' + tr("chat-camera-capture", "Take photo") + "</button>" +
      "</div></div>";
    document.body.appendChild(overlay);

    var video = overlay.querySelector("video");
    overlay.querySelector(".gb-camera-close").addEventListener("click", close);
    overlay.querySelector(".gb-camera-cancel").addEventListener("click", close);
    overlay.querySelector(".gb-camera-shutter").addEventListener("click", function () {
      capture(video);
    });
    overlay.addEventListener("click", function (event) {
      if (event.target === overlay) close();
    });
    return video;
  }

  function open() {
    if (overlay) return;
    var fallback = document.getElementById("cameraFileInput");
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
      if (fallback) fallback.click();
      else notify(tr("chat-camera-unsupported", "This browser does not provide camera access."), "error");
      return;
    }
    var video = buildPreview();
    navigator.mediaDevices
      .getUserMedia({
        video: { facingMode: { ideal: "environment" }, width: { ideal: 1280 }, height: { ideal: 720 } },
        audio: false,
      })
      .then(function (mediaStream) {
        stream = mediaStream;
        video.srcObject = mediaStream;
        return video.play();
      })
      .catch(function (error) {
        close();
        var denied = error && (error.name === "NotAllowedError" || error.name === "SecurityError");
        var missing = error && error.name === "NotFoundError";
        notify(
          denied
            ? tr("chat-camera-denied", "Camera permission was denied. Allow it in the browser to take photos.")
            : missing
              ? tr("chat-camera-missing", "No camera was found on this device.")
              : tr("chat-camera-error", "Could not start the camera."),
          "error",
        );
        if (missing && fallback) fallback.click();
      });
  }

  function bind() {
    var button = document.getElementById("cameraBtn");
    if (button && !button.dataset.bound) {
      button.dataset.bound = "1";
      button.addEventListener("click", open);
    }
    var fallback = document.getElementById("cameraFileInput");
    if (fallback && !fallback.dataset.bound) {
      fallback.dataset.bound = "1";
      fallback.addEventListener("change", function () {
        var file = fallback.files && fallback.files[0];
        fallback.value = "";
        if (file) stageAndSend(file);
      });
    }
  }

  window.GBWebcam = { open: open, close: close };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
})();
