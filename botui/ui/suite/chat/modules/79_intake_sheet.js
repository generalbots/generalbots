"use strict";
/* chat/modules/79_intake_sheet.js — one phone intake sheet (#2516).
 *
 * The composer keeps its desktop buttons. On a phone a single "+" opens a
 * bottom sheet with Camera / Photos / File / Voice, each row >= 44px tall.
 * Camera uses the native capture input, Photos an image-only picker, File the
 * existing multi-file input and Voice either GBVoice (when the deferred voice
 * module is present) or a small local MediaRecorder fallback (compact mode
 * skips that module entirely).
 *
 * First camera/mic use goes through a consent card, mirroring the bot-frame
 * consent-card pattern, and the decision is remembered so we ask only once.
 * Every user-facing string comes from gbT(); no auto-* keys. */
(function () {
  if (window.GBIntakeSheet) return;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function isPhone() {
    return !!(window.matchMedia && window.matchMedia("(max-width: 768px)").matches);
  }

  var els = {};
  function nodes() {
    els.sheet = els.sheet || document.getElementById("gbIntakeSheet");
    els.plus = els.plus || document.getElementById("gbIntakeBtn");
    els.status = els.status || document.getElementById("gbIntakeStatus");
    els.consent = els.consent || document.getElementById("gbIntakeConsent");
    els.consentTitle = els.consentTitle || document.getElementById("gbIntakeConsentTitle");
    els.consentDetail = els.consentDetail || document.getElementById("gbIntakeConsentDetail");
    els.consentKind = els.consentKind || document.getElementById("gbIntakeConsentKind");
    els.camera = els.camera || document.getElementById("cameraFileInput");
    els.messageFile = els.messageFile || document.getElementById("messageFile");
    if (!els.photos) {
      els.photos = document.createElement("input");
      els.photos.type = "file";
      els.photos.accept = "image/*";
      els.photos.id = "gbIntakePhotosInput";
      els.photos.style.display = "none";
      (document.body || document.documentElement).appendChild(els.photos);
    }
    return els;
  }

  function setStatus(text) {
    nodes();
    if (els.status) els.status.textContent = text;
  }

  function consentKey(kind) {
    return "gb-intake-consent-" + kind;
  }

  function consentStored(kind) {
    try {
      return window.localStorage.getItem(consentKey(kind)) === "allowed";
    } catch (e) {
      return false;
    }
  }

  function rememberConsent(kind) {
    try {
      window.localStorage.setItem(consentKey(kind), "allowed");
    } catch (e) {
      /* private mode: the card simply asks again next time */
    }
  }

  function hideConsent() {
    nodes();
    if (els.consent) els.consent.hidden = true;
  }

  /* Ask once, then run the action. The card reuses the consent-card shape:
   * heading, detail and an explicit allow/deny action pair. */
  function withConsent(kind, action) {
    nodes();
    if (consentStored(kind)) {
      action();
      return;
    }
    if (!els.consent || !els.consentKind) {
      action();
      return;
    }
    els.consentKind.value = kind;
    els.consentTitle.textContent = kind === "camera"
      ? t("gb-intake-camera-title", "Allow camera access?")
      : t("gb-intake-mic-title", "Allow microphone access?");
    els.consentDetail.textContent = kind === "camera"
      ? t("gb-intake-camera-detail", "Chat needs your camera to take a photo.")
      : t("gb-intake-mic-detail", "Chat needs your microphone to record a voice message.");
    els.consent.hidden = false;
  }

  function stageFile(file) {
    nodes();
    if (!file || !els.messageFile) return;
    try {
      var transfer = new DataTransfer();
      Array.prototype.slice.call(els.messageFile.files || []).forEach(function (existing) {
        transfer.items.add(existing);
      });
      transfer.items.add(file);
      els.messageFile.files = transfer.files;
      els.messageFile.dispatchEvent(new Event("change", { bubbles: true }));
      setStatus(t("gb-intake-attached", "Attachment added."));
    } catch (e) {
      setStatus(t("gb-intake-error", "Could not attach the file."));
    }
  }

  function bindPicker(input) {
    if (!input || input.dataset.gbIntakeBound === "1") return;
    input.dataset.gbIntakeBound = "1";
    input.addEventListener("change", function () {
      var file = input.files && input.files[0];
      input.value = "";
      close();
      if (file) stageFile(file);
    });
  }

  function openCamera() {
    nodes();
    bindPicker(els.camera);
    if (els.camera) els.camera.click();
    else setStatus(t("gb-intake-unsupported", "This option is not available on this device."));
  }

  function openPhotos() {
    nodes();
    bindPicker(els.photos);
    els.photos.click();
  }

  function openFile() {
    nodes();
    if (els.messageFile) els.messageFile.click();
  }

  var recorder = null;
  var recording = false;

  function stopVoiceFallback() {
    if (recorder && recorder.state !== "inactive") recorder.stop();
    else recording = false;
  }

  function startVoiceFallback() {
    if (!navigator.mediaDevices || typeof navigator.mediaDevices.getUserMedia !== "function" ||
        typeof window.MediaRecorder === "undefined") {
      setStatus(t("gb-intake-unsupported", "This option is not available on this device."));
      return;
    }
    navigator.mediaDevices.getUserMedia({ audio: true }).then(function (stream) {
      try {
        recorder = new MediaRecorder(stream);
      } catch (e) {
        stream.getTracks().forEach(function (track) { track.stop(); });
        setStatus(t("gb-intake-unsupported", "This option is not available on this device."));
        return;
      }
      var chunks = [];
      recorder.ondataavailable = function (event) {
        if (event.data && event.data.size) chunks.push(event.data);
      };
      recorder.onstop = function () {
        recording = false;
        stream.getTracks().forEach(function (track) { track.stop(); });
        if (chunks.length) {
          var type = recorder.mimeType || "audio/webm";
          var extension = type.indexOf("ogg") >= 0 ? "ogg" : "webm";
          stageFile(new File(chunks, "voice-" + Date.now() + "." + extension, { type: type }));
          setStatus(t("gb-intake-recorded", "Voice message attached."));
        }
      };
      recorder.onerror = function () {
        recording = false;
        stream.getTracks().forEach(function (track) { track.stop(); });
        setStatus(t("gb-intake-error", "Could not attach the file."));
      };
      recorder.start();
      recording = true;
      setStatus(t("gb-intake-recording", "Recording… tap Voice again to stop."));
      var label = document.querySelector('[data-gb-intake="voice"] .gb-intake-label');
      if (label) label.textContent = t("gb-intake-voice-stop", "Stop recording");
    }).catch(function () {
      setStatus(t("gb-intake-denied", "Permission denied. Enable it in your browser settings to use this option."));
    });
  }

  function openVoice() {
    if (window.GBVoice && typeof window.GBVoice.start === "function") {
      if (window.GBVoice.isRecording && window.GBVoice.isRecording()) window.GBVoice.stop();
      else {
        window.GBVoice.start();
        close();
      }
      return;
    }
    if (recording) {
      stopVoiceFallback();
      return;
    }
    startVoiceFallback();
  }

  function resetVoiceLabel() {
    var label = document.querySelector('[data-gb-intake="voice"] .gb-intake-label');
    if (label) label.textContent = t("gb-intake-voice", "Voice");
  }

  function open() {
    if (!isPhone()) return;
    nodes();
    if (!els.sheet) return;
    hideConsent();
    setStatus("");
    resetVoiceLabel();
    els.sheet.hidden = false;
    var first = els.sheet.querySelector('.gb-intake-row[data-gb-intake="camera"]');
    if (first) first.focus({ preventScroll: true });
  }

  function close() {
    nodes();
    if (els.sheet) els.sheet.hidden = true;
    hideConsent();
    resetVoiceLabel();
    if (els.plus) els.plus.focus({ preventScroll: true });
  }

  function onDocumentClick(event) {
    nodes();
    var target = event.target;
    if (!target || typeof target.closest !== "function") return;
    if (target.closest("#gbIntakeBtn")) {
      event.preventDefault();
      open();
      return;
    }
    if (!els.sheet || els.sheet.hidden) return;
    if (target.closest("[data-gb-intake-close]")) {
      event.preventDefault();
      close();
      return;
    }
    var allow = target.closest("[data-gb-intake-allow]");
    if (allow) {
      event.preventDefault();
      var kindNode = els.consentKind;
      var kind = kindNode ? kindNode.value : "";
      rememberConsent(kind);
      hideConsent();
      if (kind === "camera") openCamera();
      else openVoice();
      return;
    }
    if (target.closest("[data-gb-intake-deny]")) {
      event.preventDefault();
      hideConsent();
      setStatus(t("gb-intake-denied", "Permission denied. Enable it in your browser settings to use this option."));
      return;
    }
    var row = target.closest(".gb-intake-row[data-gb-intake]");
    if (!row) return;
    event.preventDefault();
    var action = row.getAttribute("data-gb-intake");
    if (action === "camera") withConsent("camera", openCamera);
    else if (action === "photos") openPhotos();
    else if (action === "file") openFile();
    else if (action === "voice") withConsent("mic", openVoice);
  }

  function onKeydown(event) {
    nodes();
    if (event.key === "Escape" && els.sheet && !els.sheet.hidden) {
      close();
    }
  }

  document.addEventListener("click", onDocumentClick, false);
  document.addEventListener("keydown", onKeydown, false);
  window.addEventListener("gb-chat-ready", function () {
    // WindowManager re-injects the chat fragment on every open; drop cached
    // nodes so the sheet binds the fresh DOM instead of a detached window.
    els = {};
    nodes();
    bindPicker(els.camera);
  });

  window.GBIntakeSheet = { open: open, close: close };
})();
