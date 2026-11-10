/* chat/modules/77_camera.js — take a photo in the browser and send it.
 *
 * The camera button opens a live preview, captures one frame to JPEG and
 * stages it as a normal chat attachment. The server already describes image
 * attachments through BotModels (`/api/vision/describe`), so the assistant
 * analyzes the photo with the same open-source stack as uploaded images.
 * Browsers without getUserMedia fall back to the native capture input.
 *
 * #1898 — WebKitGTK can leave `getUserMedia()` pending forever when no
 * permission handler answers (Tauri 2 desktop). A pending promise used to keep
 * the modal preview on screen with a black video and no way back, which the
 * user perceived as a frozen app. Every request now runs under a watchdog and
 * degrades to the native capture input, and the dialog is always dismissable
 * (buttons, overlay click, Escape).
 *
 * #2106 — a resolved stream is not a picture: WebKitGTK can also leave
 * `video.play()` pending forever, so readiness is now taken from the frames
 * (`77_camera_preview.js`) under its own deadline. Either the shutter becomes
 * usable or the dialog hands the user to the system picker — never "Waiting…".
 */
"use strict";

(function () {
  var stream = null;
  var overlay = null;
  var requestSeq = 0;
  var WATCHDOG_MS = 10000;
  // The play phase needs its own deadline: the getUserMedia watchdog above is
  // already cleared by the time a stream arrives.
  var PLAY_WATCHDOG_MS = 6000;
  var DIAGNOSTIC_MS = 2000;
  var FRAME_POLL_MS = 250;
  var cancelFrameWatch = null;

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
    // Invalidate any in-flight request so a late stream is released instead of
    // leaking a camera that the user already dismissed.
    requestSeq += 1;
    if (cancelFrameWatch) cancelFrameWatch();
    cancelFrameWatch = null;
    stopStream();
    if (overlay && overlay.parentNode) overlay.parentNode.removeChild(overlay);
    overlay = null;
    document.removeEventListener("keydown", onKeydown);
  }

  function onKeydown(event) {
    if (event.key === "Escape" && overlay) {
      event.preventDefault();
      close();
    }
  }

  function fallbackInput() {
    return document.getElementById("cameraFileInput");
  }

  function useFallbackInput(message) {
    var fallback = fallbackInput();
    if (fallback) {
      notify(message, "error");
      fallback.click();
      return true;
    }
    return false;
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
    if (!video || !video.videoWidth) {
      notify(tr("chat-camera-waiting", "Waiting for the camera to start…"), "info");
      return;
    }
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

  function setStatus(text) {
    if (!overlay) return;
    var status = overlay.querySelector(".gb-camera-status");
    if (status) status.textContent = text;
  }

  function setShutterEnabled(enabled) {
    if (!overlay) return;
    var shutter = overlay.querySelector(".gb-camera-shutter");
    if (shutter) shutter.disabled = !enabled;
  }

  function videoReady(video) {
    if (window.GBCameraPreview && typeof GBCameraPreview.videoReady === "function") {
      return GBCameraPreview.videoReady(video);
    }
    return !!video && video.videoWidth > 0 && video.videoHeight > 0;
  }

  /// Camera state for the "still waiting" diagnostic (#2106): a granted
  /// permission with no frames looks very different from having no device.
  function describeDevices() {
    if (!navigator.mediaDevices || typeof navigator.mediaDevices.enumerateDevices !== "function") {
      return Promise.resolve("");
    }
    return navigator.mediaDevices
      .enumerateDevices()
      .then(function (devices) {
        var cameras = devices.filter(function (device) {
          return device.kind === "videoinput";
        });
        if (!cameras.length) return tr("chat-camera-no-device", "no video device");
        return cameras[0].label || tr("chat-camera-device", "video device");
      })
      .catch(function () {
        return "";
      });
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
      '<div class="gb-camera-status">' +
      tr("chat-camera-waiting", "Waiting for the camera to start…") + "</div>" +
      '<div class="gb-camera-actions">' +
      '<button type="button" class="gb-camera-cancel">' + tr("chat-camera-cancel", "Cancel") + "</button>" +
      '<button type="button" class="gb-camera-shutter" disabled>' +
      tr("chat-camera-capture", "Take photo") + "</button>" +
      "</div></div>";
    document.body.appendChild(overlay);
    document.addEventListener("keydown", onKeydown);

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

  /// Asks for a video stream, but never waits forever: a watchdog rejects the
  /// request when the backend (WebKitGTK permission handler) simply never
  /// answers, and a stream that arrives after we gave up is stopped at once.
  function requestCamera() {
    return new Promise(function (resolve, reject) {
      var settled = false;
      var timer = setTimeout(function () {
        if (settled) return;
        settled = true;
        var error = new Error("camera request timed out");
        error.name = "TimeoutError";
        reject(error);
      }, WATCHDOG_MS);
      navigator.mediaDevices
        .getUserMedia({
          video: { facingMode: { ideal: "environment" }, width: { ideal: 1280 }, height: { ideal: 720 } },
          audio: false,
        })
        .then(function (mediaStream) {
          if (settled) {
            mediaStream.getTracks().forEach(function (track) { track.stop(); });
            return;
          }
          settled = true;
          clearTimeout(timer);
          resolve(mediaStream);
        })
        .catch(function (error) {
          if (settled) return;
          settled = true;
          clearTimeout(timer);
          reject(error);
        });
    });
  }

  /// Starts the preview and decides readiness from the frames, never from
  /// `video.play()`: WebKitGTK can leave that promise pending forever even
  /// though the element already holds a capturable frame (#2106).
  ///
  /// Resolves `true` when the shutter may be used, `false` when no frame
  /// arrived before the play watchdog — the caller then hands the user to the
  /// system picker instead of leaving a black modal on "Waiting…".
  function startPreview(video, token) {
    var diagnostic = setTimeout(function () {
      if (token !== requestSeq || !overlay) return;
      describeDevices().then(function (detail) {
        if (token !== requestSeq || !overlay) return;
        var message = tr(
          "chat-camera-waiting-frames",
          "Permission granted; waiting for video frames",
        );
        setStatus(detail ? message + " — " + detail + "…" : message + "…");
      });
    }, DIAGNOSTIC_MS);

    var played = typeof video.play === "function" ? video.play() : Promise.resolve();
    // A blocked autoplay rejects; a stalled pipeline stays pending. Neither
    // says anything about whether frames are decodable, so both are ignored.
    Promise.resolve(played).catch(function () { /* autoplay blocked */ });

    var watcher =
      window.GBCameraPreview && typeof GBCameraPreview.watch === "function"
        ? GBCameraPreview.watch(video, { timeoutMs: PLAY_WATCHDOG_MS, pollMs: FRAME_POLL_MS })
        : null;
    if (!watcher) {
      clearTimeout(diagnostic);
      return Promise.resolve(videoReady(video));
    }
    cancelFrameWatch = watcher.cancel;
    return watcher.promise.then(function (ready) {
      clearTimeout(diagnostic);
      cancelFrameWatch = null;
      if (token !== requestSeq || !overlay) return false;
      if (!ready) return false;
      setShutterEnabled(true);
      setStatus(
        video.paused
          ? tr("chat-camera-started", "Camera started — tap Take photo.")
          : tr("chat-camera-ready", "Camera ready. Take the photo or cancel."),
      );
      return true;
    });
  }

  function open() {
    if (overlay) return;
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
      if (fallbackInput()) fallbackInput().click();
      else notify(tr("chat-camera-unsupported", "This desktop app does not provide camera access."), "error");
      return;
    }
    var token = ++requestSeq;
    var video = buildPreview();
    requestCamera()
      .then(function (mediaStream) {
        if (token !== requestSeq || !overlay) {
          mediaStream.getTracks().forEach(function (track) { track.stop(); });
          return;
        }
        if (typeof mediaStream.getVideoTracks === "function" && mediaStream.getVideoTracks().length === 0) {
          // A grant without a video track can never produce a photo.
          mediaStream.getTracks().forEach(function (track) { track.stop(); });
          close();
          useFallbackInput(tr("chat-camera-missing", "No camera was found on this device."));
          return;
        }
        stream = mediaStream;
        video.srcObject = mediaStream;
        return startPreview(video, token);
      })
      .then(function (ready) {
        if (ready !== false) return;
        if (token !== requestSeq || !overlay) return;
        // No frame arrived: the WebView pipeline is dead for this device. Drop
        // the black modal and offer the native picker with the reason.
        close();
        useFallbackInput(
          tr(
            "chat-camera-no-frames",
            "The camera preview did not start. Choose a photo from the system picker instead.",
          ),
        );
      })
      .catch(function (error) {
        if (token !== requestSeq) return;
        close();
        var name = error && error.name;
        var denied = name === "NotAllowedError" || name === "SecurityError";
        var missing = name === "NotFoundError" || name === "DevicesNotFoundError";
        var unsupported = name === "NotReadableError" || name === "OverconstrainedError";
        if (denied) {
          notify(
            tr(
              "chat-camera-denied",
              "Camera permission was denied. Allow camera access for the desktop app in your system settings to take photos.",
            ),
            "error",
          );
          return;
        }
        if (missing) {
          notify(tr("chat-camera-missing", "No camera was found on this device."), "error");
          if (fallbackInput()) fallbackInput().click();
          return;
        }
        // Timeout: the WebView never answered. The native capture input is the
        // only path that still works, so hand the user over to it.
        if (name === "TimeoutError") {
          useFallbackInput(
            tr(
              "chat-camera-timeout",
              "The camera did not answer. Choose a photo from the system picker instead.",
            ),
          );
          return;
        }
        if (unsupported) {
          notify(tr("chat-camera-error", "Could not start the camera."), "error");
          if (fallbackInput()) fallbackInput().click();
          return;
        }
        notify(tr("chat-camera-error", "Could not start the camera."), "error");
      });
  }

  function bind() {
    var fallback = fallbackInput();
    if (fallback && !fallback.dataset.bound) {
      fallback.dataset.bound = "1";
      fallback.addEventListener("change", function () {
        var file = fallback.files && fallback.files[0];
        fallback.value = "";
        if (file) stageAndSend(file);
      });
    }
  }

  // #1898 — delegation keeps the button alive when the composer is re-rendered
  // after module boot (same late-composer failure as the mic and web search).
  document.addEventListener("click", function (event) {
    var button = event.target && event.target.closest ? event.target.closest("#cameraBtn") : null;
    if (!button) return;
    event.preventDefault();
    open();
  });

  bind();
  document.addEventListener("gb-chat-ready", bind);
  window.GBWebcam = { open: open, close: close };
})();
