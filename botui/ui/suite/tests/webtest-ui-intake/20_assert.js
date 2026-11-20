/**
 * Mobile native-intake static audit — characterization locks + criteria (#2510).
 *
 * [BASELINE-OBSERVE] locks the state measured on the current tree. It is a
 * regression detector, not a ratchet: no baseline/guard file is read or written.
 * A deliberate production change must update these constants in review.
 *
 * [STATIC] assertions describe source facts only. The three runtime criteria
 * named in the issue (native camera capture, iOS folder support, OS share
 * sheet) are registered as [STATIC][GAP] and are never counted as PASS.
 * Known bugs are reported with file:line evidence and never patched here.
 */
"use strict";

var h = require("./00_harness");
var m = require("./10_scan").metrics;

/* ------------------------------------------- [BASELINE-OBSERVE] locks: inputs */
h.baseline("drive.html exposes three file inputs at 287/288/289", function () {
  h.equal(m.drive.fileInputs, 3, "file input count");
  h.equal(m.drive.ids.join(","), "file-input,camera-input,folder-input", "input ids");
  h.equal(m.drive.lines.join(","), "file-input:287,camera-input:288,folder-input:289", "input lines");
});
h.baseline("accept on two inputs, capture on one, webkitdirectory on one", function () {
  h.equal(m.drive.withAccept, 2, "inputs with accept");
  h.equal(m.drive.withCapture, 1, "inputs with capture");
  h.equal(m.drive.withWebkitdirectory, 1, "inputs with webkitdirectory");
  h.equal(m.drive.withStandardDirectory, 0, "inputs with standard directory attribute");
});
h.baseline("general file input has 16 accept entries and no capture", function () {
  h.equal(m.drive.fileAcceptEntries, 16, "accept entries on #file-input");
  h.equal(m.drive.cameraOnGeneralInput, false, "capture on #file-input");
});
h.baseline("camera input uses accept=image/* and capture=environment", function () {
  h.equal(m.drive.cameraAccept, "image/*", "camera accept");
  h.equal(m.drive.cameraCapture, "environment", "camera capture");
});
h.baseline("folder input uses the bare non-standard webkitdirectory attribute", function () {
  h.equal(m.drive.folderBareAttribute, true, "bare webkitdirectory");
});
h.baseline("intake is reachable from two affordances per kind plus a note", function () {
  h.equal(m.buttons.upload.join(","), "upload-btn,toolbar-upload-btn", "upload buttons");
  h.equal(m.buttons.camera.join(","), "upload-camera-btn,toolbar-camera-btn", "camera buttons");
  h.equal(m.buttons.folder.join(","), "upload-folder-btn,toolbar-folder-btn", "folder buttons");
  h.equal(m.buttons.folderNote, true, "folder-unsupported-note present");
});

/* ------------------------------------------ [BASELINE-OBSERVE] locks: events */
h.baseline("07_events.js feature-detects webkitdirectory once at bind time", function () {
  h.equal(m.events.supportsFolderProbe, true, "capability probe");
  h.equal(m.events.folderBlockHideCount, 3, "affordances hidden in unsupported branch");
  h.equal(m.events.folderNoteRevealed, true, "unsupported note revealed");
});
h.baseline("07_events.js binds camera and folder handlers", function () {
  h.equal(m.events.cameraClickBindings, 2, "camera button bindings");
  h.equal(m.events.cameraInputBinding, true, "camera input change binding");
  h.equal(m.events.folderClickBindings, 2, "folder button bindings");
});
h.baseline("first camera use is explained once through GBModal", function () {
  h.equal(m.events.explainerFlag, "gb-drive-camera-explained-v1", "explainer flag");
  h.equal(m.events.explainerUsesModal, true, "GBModal.confirm call");
  h.equal(m.events.getUserMedia, false, "no getUserMedia in the Drive event module");
});

/* -------------------------------------- [BASELINE-OBSERVE] locks: share target */
h.baseline("manifest.webmanifest exists and declares share_target at line 7", function () {
  h.equal(m.manifest.exists, true, "manifest present");
  h.equal(m.manifest.hasShareTarget, true, "share_target declared");
  h.equal(m.manifest.shareTargetLine, 7, "share_target line");
  h.equal(m.manifest.name, "General Bots Desktop", "manifest name");
});
h.baseline("share_target is POST multipart to /share-intake with files + metadata", function () {
  h.equal(m.manifest.action, "/share-intake", "share action");
  h.equal(m.manifest.method, "POST", "share method");
  h.equal(m.manifest.enctype, "multipart/form-data", "share enctype");
  h.equal(m.manifest.filesParamName, "files", "files param");
  h.equal(m.manifest.acceptTypes, 6, "declared file accept types");
  h.equal(m.manifest.titleTextUrl, true, "title/text/url params");
});
h.baseline("manifest has no file_handlers and no shortcuts", function () {
  h.equal(m.manifest.hasFileHandlers, false, "file_handlers");
  h.equal(m.manifest.hasShortcuts, false, "shortcuts");
});
h.baseline("service worker intercepts the share POST at line 132", function () {
  h.equal(m.serviceWorker.exists, true, "sw.js present");
  h.equal(m.serviceWorker.interceptSharePost, true, "POST interception");
  h.equal(m.serviceWorker.shareInterceptLine, 132, "intercept line");
  h.equal(m.serviceWorker.sharedRedirect, true, "shared=1 redirect");
  h.equal(m.serviceWorker.storesFilesInCache, true, "shared bytes cached");
});
h.baseline("shell cache is v46 and share inbox is a separate v1 cache", function () {
  h.equal(m.serviceWorker.shellCache, "gb-desktop-shell-v46", "shell cache name");
  h.equal(m.serviceWorker.shareCache, "gb-share-inbox-v1", "share cache name");
  h.equal(m.serviceWorker.deletesOtherCaches, true, "activate prunes other caches");
});
h.baseline("share-intake page has a polite status region and saves to Drive", function () {
  h.equal(m.sharePage.exists, true, "share page present");
  h.equal(m.sharePage.roleStatus, true, "role=status + aria-live=polite");
  h.equal(m.sharePage.writesToDrive, true, "/api/files/write call");
});
h.baseline("desktop shell links the manifest and registers the worker", function () {
  h.equal(m.shell.manifestLinked, true, "manifest link");
  h.equal(m.shell.serviceWorkerRegistered, true, "service worker registration");
});
h.baseline("chat keeps the in-app camera reference used as the pattern", function () {
  h.equal(m.chatReference.cameraCapture, true, "chat camera capture");
  h.equal(m.chatReference.cameraAccept, true, "chat camera accept");
});
h.baseline("no Rust route serves /share-intake (worker-only handler)", function () {
  h.equal(m.serverRoutes.shareIntakeMentions, 0, "Rust mentions of share-intake");
});

/* ---------------------------------------- [BASELINE-OBSERVE] locks: i18n walk */
h.baseline("six intake keys resolve in all eight catalogs", function () {
  h.equal(m.i18n.keys.length, 6, "intake key count");
  h.equal(m.i18n.locales.length, 8, "locale count");
  m.i18n.keys.forEach(function (key) {
    h.equal(m.i18n.byKey[key].join(","), m.i18n.locales.join(","), "locales for " + key);
  });
});

/* ---------------------------------------------- [STATIC] source-only criteria */
h.staticTest("native camera capture is a picker attribute, not an in-app camera", function () {
  h.equal(m.drive.cameraCapture, "environment", "capture attribute declared in Drive markup");
  h.equal(m.events.getUserMedia, false, "no getUserMedia preview in Drive events");
  h.note("a real photo reaching the current folder is a device UAT, not a source fact");
});
h.staticTest("iOS folder support is not shipped: webkitdirectory has no fallback", function () {
  h.equal(m.drive.withWebkitdirectory, 1, "non-standard attribute present");
  h.equal(m.drive.withStandardDirectory, 0, "no standard directory attribute fallback");
  h.equal(m.events.supportsFolderProbe, true, "source hides the control when unsupported");
  h.note("real iOS Safari behaviour cannot be observed from the source tree");
});
h.staticTest("share_target is declared, not a verified OS share sheet", function () {
  h.equal(m.manifest.hasShareTarget, true, "declaration present");
  h.equal(m.manifest.hasFileHandlers, false, "no file_handlers declaration");
  h.equal(m.serverRoutes.shareIntakeMentions, 0, "no server-side fallback route");
  h.note("OS share-sheet listing and landing require an installed PWA on a device");
});
h.staticTest("queue and photo-compression slices exist in source only", function () {
  var queue = h.suiteFile("drive/modules/05c_upload_queue.js");
  var compress = h.suiteFile("drive/modules/05b_photo_compress.js");
  h.assert(queue.indexOf("gbcQueueOfferRecovery") !== -1, "recovery entry point present");
  h.assert(compress.indexOf("gbcCompressImage") !== -1, "compression entry point present");
  h.note("neither module proves a killed-tab Retry or a measured 10 MB -> 2 MB upload");
});
h.staticTest("intake strings are catalog keys, not hardcoded literals", function () {
  h.equal(m.i18n.byKey["drive-take-photo"].length, 8, "camera action key in all catalogs");
  h.equal(m.i18n.byKey["drive-folder-unsupported"].length, 8, "folder note key in all catalogs");
});

/* ------------------------------------------------- KNOWN BUGs (report only) */
h.knownBug("SW activation discards pending shared files on every version bump", function () {
  var source = h.uiFile(h.SERVICE_WORKER);
  var deleteLine = h.lineOf(source, source.indexOf("k !== CACHE"));
  var cacheLine = h.lineOf(source, source.indexOf("var SHARE_CACHE"));
  h.equal(m.serviceWorker.shellCache, "gb-desktop-shell-v46", "retained cache constant");
  h.equal(m.serviceWorker.deletesOtherCaches, true, "all other caches deleted");
  h.assert(m.serviceWorker.shareCache !== m.serviceWorker.shellCache,
    "share inbox is not the retained cache");
  h.note("sw.js:" + deleteLine + " deletes every cache except CACHE; sw.js:" + cacheLine +
    " creates gb-share-inbox-v1, so a bump to sw.js:9 drops files before Save to Drive.");
});
h.knownBug("share POST has no server-side fallback when the worker is not controlling", function () {
  var manifestSource = h.uiFile(h.MANIFEST);
  var actionLine = h.lineOf(manifestSource, manifestSource.indexOf("/share-intake"));
  h.equal(m.manifest.action, "/share-intake", "share target action");
  h.equal(m.serviceWorker.interceptSharePost, true, "worker-only POST handler");
  h.equal(m.serverRoutes.shareIntakeMentions, 0, "no matching Rust route");
  h.note("manifest.webmanifest:" + actionLine + " declares /share-intake and sw.js:" +
    m.serviceWorker.shareInterceptLine + " is its only handler; without an active worker " +
    "the POST has no route (silent share failure).");
});

/* ------------------------------------------------------------- [STATIC][GAP] */
h.gap("CAMERA-RUNTIME",
  "a photo taken from Drive's camera input appears in the current folder at 390x844",
  "markup carries accept/capture but no source can prove a real capture; needs device + CDP screenshot");
h.gap("IOS-FOLDER",
  "iOS Safari hides the folder control instead of opening a dead picker",
  "webkitdirectory is unsupported there; the hide logic is source-only and needs real-device emulation");
h.gap("OS-SHARE",
  "sharing an image from the OS share sheet lists General Bots and lands the file in Drive",
  "requires an installed PWA plus chrome://web-app-internals and a real share-sheet event");
h.gap("UPLOAD-RETRY",
  "killing the tab mid-upload shows a queued/failed item with Retry on reopen",
  "the IndexedDB queue is source-only; needs a deliberately interrupted real upload");
h.gap("PHOTO-COMPRESSION",
  "a 10 MB photo uploads at <= 2 MB with network-panel evidence",
  "the compressor is source-only; byte counts need a real photo and a network panel");
h.gap("SHARE-AUTH",
  "shared files save to Drive automatically without a token in the share URL",
  "save depends on runtime token discovery; needs a device share launch with an authenticated session");

h.note("Offline static audit only: no browser test, no runtime PASS claim, no production file touched.");
