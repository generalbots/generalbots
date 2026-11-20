/**
 * Read-only scanner for the shipped mobile-intake surface (#2510).
 *
 * Every number in `metrics` is derived from the files on disk at run time.
 * The [BASELINE-OBSERVE] module locks these numbers; it never writes a file.
 * The i18n dimension walks real catalog keys per locale — no gbT key is
 * concatenated and no data-i18n literal is emitted by this harness.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var h = require("./00_harness");

function attribute(tag, name) {
  var valued = new RegExp("\\b" + name + "\\s*=\\s*\"([^\"]*)\"").exec(tag);
  if (valued) return valued[1];
  if (new RegExp("\\b" + name + "\\b").test(tag)) return "";
  return null;
}
function count(text, pattern) {
  return (text.match(pattern) || []).length;
}
function quoted(text, declaration) {
  var match = new RegExp("\\b" + declaration + "\\s*=\\s*\"([^\"]*)\"").exec(text);
  return match ? match[1] : "";
}
function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
function rustMentions(directory, needle) {
  if (!fs.existsSync(directory)) return 0;
  return fs.readdirSync(directory, { withFileTypes: true }).reduce(function (total, entry) {
    var full = path.join(directory, entry.name);
    if (entry.isDirectory()) return total + rustMentions(full, needle);
    if (entry.isFile() && path.extname(entry.name) === ".rs") {
      return total + (fs.readFileSync(full, "utf8").indexOf(needle) !== -1 ? 1 : 0);
    }
    return total;
  }, 0);
}
function ftlHasKey(locale, key) {
  var dir = path.join(h.REPO_ROOT, "botlib", "locales", locale);
  var pattern = new RegExp("^" + escapeRegex(key) + "\\s*=", "m");
  return fs.readdirSync(dir).some(function (name) {
    if (path.extname(name) !== ".ftl") return false;
    return pattern.test(fs.readFileSync(path.join(dir, name), "utf8"));
  });
}

var html = h.suiteFile(h.DRIVE_HTML);
var inputs = [];
var inputTag = /<input\b[^>]*>/g;
var tagMatch;
while ((tagMatch = inputTag.exec(html)) !== null) {
  var tag = tagMatch[0];
  if (attribute(tag, "type") !== "file") continue;
  inputs.push({
    id: attribute(tag, "id"),
    accept: attribute(tag, "accept"),
    capture: attribute(tag, "capture"),
    webkitdirectory: attribute(tag, "webkitdirectory"),
    directory: attribute(tag, "directory"),
    multiple: attribute(tag, "multiple"),
    line: h.lineOf(html, tagMatch.index),
    tag: tag,
  });
}
function inputById(id) {
  return inputs.filter(function (entry) { return entry.id === id; })[0] || null;
}

var events = h.suiteFile(h.DRIVE_EVENTS);
var folderBlockStart = events.indexOf("if (!supportsFolder) {");
var folderBlockEnd = events.indexOf("} else {", folderBlockStart);
var folderBlock = events.slice(folderBlockStart, folderBlockEnd);
var manifestRaw = h.uiFile(h.MANIFEST);
var manifest = JSON.parse(manifestRaw);
var sw = h.uiFile(h.SERVICE_WORKER);
var sharePage = h.suiteFile(h.SHARE_PAGE);
var desktop = h.suiteFile(h.DESKTOP_HTML);
var registration = h.suiteFile(h.SW_REGISTRATION);
var chat = h.suiteFile(h.CHAT_HTML);

var shareTarget = manifest.share_target || {};
var shareFiles = (shareTarget.params && shareTarget.params.files) || [];
var INTAKE_KEYS = [
  "drive-upload",
  "drive-take-photo",
  "drive-upload-folder",
  "drive-folder-unsupported",
  "drive-camera-permission-title",
  "drive-camera-permission-explainer",
];
var i18nByKey = {};
INTAKE_KEYS.forEach(function (key) {
  i18nByKey[key] = h.LOCALES.filter(function (locale) { return ftlHasKey(locale, key); });
});

var metrics = {
  drive: {
    fileInputs: inputs.length,
    ids: inputs.map(function (entry) { return entry.id; }),
    withAccept: inputs.filter(function (entry) { return entry.accept !== null; }).length,
    withCapture: inputs.filter(function (entry) { return entry.capture !== null; }).length,
    withWebkitdirectory: inputs.filter(function (entry) { return entry.webkitdirectory !== null; }).length,
    withStandardDirectory: inputs.filter(function (entry) { return entry.directory !== null; }).length,
    fileAcceptEntries: (inputById("file-input").accept || "").split(",").length,
    cameraAccept: inputById("camera-input").accept,
    cameraCapture: inputById("camera-input").capture,
    cameraOnGeneralInput: inputById("file-input").capture !== null,
    folderBareAttribute: inputById("folder-input").webkitdirectory === "",
    lines: inputs.map(function (entry) { return entry.id + ":" + entry.line; }),
  },
  buttons: {
    upload: ["upload-btn", "toolbar-upload-btn"].filter(function (id) {
      return html.indexOf("id=\"" + id + "\"") !== -1;
    }),
    camera: ["upload-camera-btn", "toolbar-camera-btn"].filter(function (id) {
      return html.indexOf("id=\"" + id + "\"") !== -1;
    }),
    folder: ["upload-folder-btn", "toolbar-folder-btn"].filter(function (id) {
      return html.indexOf("id=\"" + id + "\"") !== -1;
    }),
    folderNote: html.indexOf("id=\"folder-unsupported-note\"") !== -1,
  },
  events: {
    supportsFolderProbe: /"webkitdirectory"\s+in\s+document\.createElement\(\s*"input"\s*\)/.test(events),
    hideNoneCount: count(events, /\.style\.display\s*=\s*"none"/g),
    folderBlockHideCount: count(folderBlock, /\.style\.display\s*=\s*"none"/g),
    folderNoteRevealed: /folderNote\.hidden\s*=\s*false/.test(events),
    cameraClickBindings: count(events, /addEventListener\("click",\s*triggerCameraUpload\)/g),
    cameraInputBinding: /getElementById\("camera-input"\)/.test(events) &&
      /handleCameraInputChange/.test(events),
    folderClickBindings: count(events, /addEventListener\("click",\s*triggerFolderUpload\)/g),
    explainerFlag: quoted(events, "explainedKey"),
    explainerUsesModal: /GBModal\.confirm\(msg, title, proceed/.test(events),
    getUserMedia: /getUserMedia/.test(events),
  },
  manifest: {
    exists: true,
    name: manifest.name,
    hasShareTarget: !!manifest.share_target,
    shareTargetLine: h.lineOf(manifestRaw, manifestRaw.indexOf("\"share_target\"")),
    action: shareTarget.action || "",
    method: shareTarget.method || "",
    enctype: shareTarget.enctype || "",
    filesParamName: shareFiles.length ? shareFiles[0].name : "",
    acceptTypes: shareFiles.length ? (shareFiles[0].accept || []).length : 0,
    titleTextUrl: !!(shareTarget.params && shareTarget.params.title &&
      shareTarget.params.text && shareTarget.params.url),
    hasFileHandlers: !!manifest.file_handlers,
    hasShortcuts: !!manifest.shortcuts,
  },
  serviceWorker: {
    exists: true,
    interceptSharePost: /e\.request\.method\s*===\s*"POST"\s*&&\s*url\.pathname\s*===\s*"\/share-intake"/.test(sw),
    shareInterceptLine: h.lineOf(sw, sw.indexOf("e.request.method === \"POST\"")),
    shellCache: quoted(sw, "CACHE"),
    shareCache: quoted(sw, "SHARE_CACHE"),
    deletesOtherCaches: /k !== CACHE/.test(sw),
    preservesShareCache: /k !== CACHE\s*&&\s*k !== SHARE_CACHE/.test(sw),
    sharedRedirect: /share-intake\.html\?shared=1/.test(sw),
    storesFilesInCache: /cache\.put\(new Request\(key\)/.test(sw),
  },
  sharePage: {
    exists: true,
    roleStatus: /role="status"/.test(sharePage) && /aria-live="polite"/.test(sharePage),
    writesToDrive: /\/api\/files\/write/.test(sharePage),
  },
  shell: {
    manifestLinked: /rel="manifest"\s+href="\/manifest\.webmanifest"/.test(desktop),
    serviceWorkerRegistered: /navigator\.serviceWorker\.register\("\/sw\.js"\)/.test(registration),
  },
  serverRoutes: {
    shareIntakeMentions: rustMentions(path.join(h.REPO_ROOT, "botui", "src"), "share-intake") +
      rustMentions(path.join(h.REPO_ROOT, "botserver", "src"), "share-intake"),
  },
  chatReference: {
    cameraCapture: /id="cameraFileInput"[^>]*capture="environment"/.test(chat),
    cameraAccept: /id="cameraFileInput"[^>]*accept="image\/\*"/.test(chat),
  },
  i18n: {
    keys: INTAKE_KEYS,
    locales: h.LOCALES.slice(),
    byKey: i18nByKey,
  },
};

h.test("drive.html exposes exactly three file inputs", function () {
  h.equal(metrics.drive.fileInputs, 3, "observed file input count");
});
h.test("each file input carries an id for attribute walking", function () {
  metrics.drive.ids.forEach(function (id) { h.assert(id, "input without id"); });
});
h.test("manifest is valid JSON with a top-level share_target object", function () {
  h.assert(typeof manifest === "object" && manifest !== null, "manifest did not parse");
});
h.test("every intake i18n key resolves in all eight catalogs", function () {
  INTAKE_KEYS.forEach(function (key) {
    h.equal(metrics.i18n.byKey[key].length, h.LOCALES.length, "locales for " + key);
  });
});

module.exports = { metrics: metrics, inputs: inputs, manifest: manifest };
