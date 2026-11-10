/* chat/modules/77_camera_preview.js — decide when a <video> is capturable.
 *
 * #2106 — the chat camera used to wait for `video.play()` to resolve before it
 * enabled the shutter. WebKitGTK (Tauri desktop) can leave that promise pending
 * forever when the media pipeline never starts, so the dialog sat on "Waiting
 * for the camera to start…" with a disabled shutter and the user could never
 * take a photo. Readiness is decided here from the frames themselves, under an
 * independent deadline: a blocked autoplay still decodes frames, and
 * `videoWidth > 0` is exactly what `canvas.drawImage(video, …)` needs.
 */
"use strict";

(function () {
  var READY_EVENTS = ["loadedmetadata", "loadeddata", "canplay", "playing"];

  function videoReady(video) {
    return !!video && video.videoWidth > 0 && video.videoHeight > 0;
  }

  /// Watches `video` until it presents a frame or the deadline elapses.
  ///
  /// Returns `{ promise, cancel }`. The promise resolves `true` (frames are
  /// there) or `false` (nothing arrived) and is never rejected, so a stalled
  /// pipeline cannot keep the dialog waiting.
  function watch(video, options) {
    var settings = options || {};
    var timeoutMs = settings.timeoutMs > 0 ? settings.timeoutMs : 6000;
    var pollMs = settings.pollMs > 0 ? settings.pollMs : 250;
    var poll = null;
    var deadline = null;
    var frameCallback = null;
    var settled = false;
    var finish = null;

    function cleanup() {
      if (poll) clearTimeout(poll);
      if (deadline) clearTimeout(deadline);
      poll = null;
      deadline = null;
      READY_EVENTS.forEach(function (name) {
        video.removeEventListener(name, probe);
      });
      if (frameCallback !== null && typeof video.cancelVideoFrameCallback === "function") {
        video.cancelVideoFrameCallback(frameCallback);
      }
      frameCallback = null;
    }

    function stop(ready) {
      if (settled) return;
      settled = true;
      cleanup();
      finish(ready);
    }

    function probe() {
      if (settled) return;
      if (videoReady(video)) stop(true);
    }

    function tick() {
      poll = null;
      if (settled) return;
      if (videoReady(video)) {
        stop(true);
        return;
      }
      poll = setTimeout(tick, pollMs);
    }

    var promise = new Promise(function (resolve) {
      finish = resolve;
    });

    READY_EVENTS.forEach(function (name) {
      video.addEventListener(name, probe);
    });

    // The first presented frame is the strongest signal, but older WebKitGTK
    // builds expose the API and then never call back — the poll covers that.
    if (typeof video.requestVideoFrameCallback === "function") {
      try {
        frameCallback = video.requestVideoFrameCallback(function () {
          frameCallback = null;
          probe();
        });
      } catch (e) {
        frameCallback = null;
      }
    }

    poll = setTimeout(tick, pollMs);
    deadline = setTimeout(function () {
      stop(videoReady(video));
    }, timeoutMs);

    return {
      promise: promise,
      cancel: function () {
        stop(false);
      },
    };
  }

  window.GBCameraPreview = { watch: watch, videoReady: videoReady };
})();
