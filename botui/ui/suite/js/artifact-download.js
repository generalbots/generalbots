/* #2144 — download a drawing as SVG or PNG, without any external library.
 *
 * Two panels offer a drawing: the live preview in chat
 * (`chat/chat-live-artifact.js`) and the final artifact panel
 * (`chat/chat-artifacts.js`). Both hand their markup here, so the rules for
 * "what a downloaded drawing looks like" live in one place:
 *
 *   prepare()     markup → a self-contained SVG document: adds the missing
 *                 `xmlns`, derives width/height from the viewBox (an <svg> with
 *                 only a viewBox is rasterised by the browser at 300x150),
 *                 removes the draw-on animation leftovers, and paints the
 *                 background the preview shows.
 *   rasterise()   markup → a <canvas> at a sane scale, via an <img> and a blob
 *                 URL. No library, no server round trip.
 *   saveBlob()    hands the blob to the desktop Save dialog when the suite runs
 *                 inside the Tauri shell (`GBShared.saveBlob`), otherwise uses
 *                 a temporary object URL + <a download>.
 *
 * Streaming matters: the live preview owns a real <svg> element that grows as
 * the model emits tokens. Serialising it always yields a well-formed document
 * (the DOM closes what the model left open), so "download the current state"
 * needs no special case — a half-drawn picture downloads as a half-drawn
 * picture instead of failing.
 */
"use strict";

window.GBArtifactExport = window.GBArtifactExport || (function () {
  var DEFAULT_SCALE = 2;
  var MAX_SIDE = 4096; // a 4k-per-side cap keeps the PNG canvas sane
  var DEFAULT_BACKGROUND = "#ffffff";

  // The draw-on animation (`GBLiveSvg.animateIn`) writes these inline on every
  // stroked path and clears them when the animation finishes. Serialising in
  // the middle of a 520 ms draw would otherwise export a dashed or invisible
  // stroke. The animation also leaves transform-box/origin behind, which is
  // pure noise in a file.
  var ANIMATION_STYLES = [
    "stroke-dasharray",
    "stroke-dashoffset",
    "transform-box",
    "transform-origin",
  ];

  function escapeAttr(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/"/g, "&quot;")
      .replace(/</g, "&lt;");
  }

  function numeric(value) {
    var match = /^\s*(\d+(?:\.\d+)?)(?:px)?\s*$/.exec(String(value == null ? "" : value));
    return match ? parseFloat(match[1]) : 0;
  }

  function openTag(markup) {
    var match = /<svg\b[^>]*>/i.exec(String(markup || ""));
    return match ? match[0] : "";
  }

  function attr(tag, name) {
    var match = new RegExp("\\s" + name + "\\s*=\\s*(\"([^\"]*)\"|'([^']*)')", "i").exec(tag);
    if (!match) return "";
    return match[2] !== undefined ? match[2] : match[3];
  }

  // The intrinsic box of a drawing: width/height when they are plain pixels,
  // otherwise the viewBox extent. Percentage sizes are ignored on purpose —
  // they cannot size an <img>.
  function dimensions(markup) {
    var tag = openTag(markup);
    var width = numeric(attr(tag, "width"));
    var height = numeric(attr(tag, "height"));
    var box = attr(tag, "viewBox").trim();
    var parts = box ? box.split(/[\s,]+/).map(Number) : [];
    if (parts.length === 4 && isFinite(parts[2]) && isFinite(parts[3]) &&
      parts[2] > 0 && parts[3] > 0) {
      return { width: width || parts[2], height: height || parts[3] };
    }
    return { width: width, height: height };
  }

  function stripAnimation(style) {
    var kept = [];
    var pieces = String(style || "").split(";");
    for (var i = 0; i < pieces.length; i += 1) {
      var piece = pieces[i];
      var name = piece.split(":")[0].trim().toLowerCase();
      if (!name) continue;
      if (ANIMATION_STYLES.indexOf(name) >= 0) continue;
      kept.push(piece.trim());
    }
    return kept.join("; ");
  }

  function scrubStyles(markup) {
    return String(markup).replace(
      /\sstyle\s*=\s*("([^"]*)"|'([^']*)')/gi,
      function (all, quoted, doubleQuoted, singleQuoted) {
        var value = doubleQuoted !== undefined ? doubleQuoted : singleQuoted;
        var cleaned = stripAnimation(value);
        return cleaned ? ' style="' + cleaned + '"' : "";
      }
    );
  }

  function setAttr(tag, name, value) {
    var pattern = new RegExp("\\s" + name + "\\s*=\\s*(\"([^\"]*)\"|'([^']*)')", "i");
    if (pattern.test(tag)) {
      return tag.replace(pattern, ' ' + name + '="' + escapeAttr(value) + '"');
    }
    return tag.replace(/^<svg/i, '<svg ' + name + '="' + escapeAttr(value) + '"');
  }

  function sizeOpenTag(tag, options) {
    var out = tag;
    if (!/\sxmlns\s*=/i.test(out)) {
      out = out.replace(/^<svg/i, '<svg xmlns="http://www.w3.org/2000/svg"');
    }
    var box = dimensions(out);
    // Explicit options win (a canvas layer is stretched to the canvas box);
    // otherwise a missing *or non-pixel* size is filled in from the viewBox —
    // a root with width="100%" cannot size an <img>.
    var width = options.width ||
      (numeric(attr(out, "width")) ? "" : (box.width ? String(box.width) : ""));
    var height = options.height ||
      (numeric(attr(out, "height")) ? "" : (box.height ? String(box.height) : ""));
    if (width) out = setAttr(out, "width", width);
    if (height) out = setAttr(out, "height", height);
    return out;
  }

  function backgroundRect(color) {
    // 100% of the viewport equals the drawing box: the root carries explicit
    // width/height after sizeOpenTag(), so this matches the preview surface.
    return '<rect x="0" y="0" width="100%" height="100%" fill="' +
      escapeAttr(color) + '"/>';
  }

  // Markup → a self-contained SVG document. Anything that is not a drawing is
  // handed back untouched so callers can test `prepare(x)` for emptiness.
  function prepare(markup, options) {
    options = options || {};
    var text = String(markup == null ? "" : markup).trim();
    var tag = openTag(text);
    if (!tag) return text;
    var sized = sizeOpenTag(tag, options);
    var start = text.indexOf(tag);
    var out = scrubStyles(text.slice(0, start) + sized + text.slice(start + tag.length));
    var background = options.background === undefined
      ? DEFAULT_BACKGROUND
      : options.background;
    if (background) {
      // Locate the opening tag *after* scrubbing: a style attribute on the
      // root is removed there, so the pre-scrub text no longer matches.
      var cleaned = openTag(out);
      var openEnd = out.indexOf(cleaned) + cleaned.length;
      out = out.slice(0, openEnd) + backgroundRect(background) + out.slice(openEnd);
    }
    return out;
  }

  // Accepts a live element (the streaming preview root) or markup.
  function serialize(source, options) {
    if (!source) return "";
    var markup = typeof source === "string" ? source : source.outerHTML || "";
    return prepare(markup, options);
  }

  function slug(text) {
    var value = String(text || "")
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .replace(/\.(svg|png)$/i, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 60);
    return value || "drawing";
  }

  function fileName(baseName, ext) {
    return slug(baseName) + "." + ext;
  }

  function scaleFor(box, requested) {
    var scale = Number(requested);
    if (!isFinite(scale) || scale <= 0) scale = DEFAULT_SCALE;
    var width = box.width || 300;
    var height = box.height || 150;
    var longest = Math.max(width, height);
    if (longest * scale > MAX_SIDE) {
      // Respect the cap even for a drawing larger than the cap itself.
      scale = MAX_SIDE / longest;
    }
    return scale;
  }

  function rasterise(markup, options) {
    options = options || {};
    return new Promise(function (resolve, reject) {
      var svg = String(markup || "");
      if (!openTag(svg)) {
        reject(new Error("there is no drawing to export"));
        return;
      }
      var box = dimensions(svg);
      var scale = scaleFor(box, options.scale);
      var width = Math.max(1, Math.round((box.width || 300) * scale));
      var height = Math.max(1, Math.round((box.height || 150) * scale));
      var url = null;
      var image = new Image();
      function cleanup() {
        if (url) {
          try { URL.revokeObjectURL(url); } catch (e) { /* already gone */ }
          url = null;
        }
      }
      image.onload = function () {
        try {
          var canvas = document.createElement("canvas");
          canvas.width = width;
          canvas.height = height;
          var ctx = canvas.getContext("2d");
          if (!ctx) throw new Error("this browser cannot rasterise a drawing");
          if (options.background) {
            ctx.fillStyle = options.background;
            ctx.fillRect(0, 0, width, height);
          }
          ctx.drawImage(image, 0, 0, width, height);
          cleanup();
          resolve(canvas);
        } catch (error) {
          cleanup();
          reject(error);
        }
      };
      image.onerror = function () {
        cleanup();
        reject(new Error("the drawing could not be rasterised"));
      };
      url = URL.createObjectURL(new Blob([svg], { type: "image/svg+xml;charset=utf-8" }));
      image.src = url;
    });
  }

  function dataUrlBlob(dataUrl) {
    var parts = String(dataUrl || "").split(",");
    var meta = /data:([^;]+)/.exec(parts[0]) || [];
    var binary = atob(parts[1] || "");
    var bytes = new Uint8Array(binary.length);
    for (var i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return new Blob([bytes], { type: meta[1] || "application/octet-stream" });
  }

  function toBlob(canvas, mime) {
    return new Promise(function (resolve, reject) {
      if (canvas && typeof canvas.toBlob === "function") {
        canvas.toBlob(function (blob) {
          if (blob) resolve(blob);
          else reject(new Error("the image could not be encoded"));
        }, mime);
        return;
      }
      try {
        resolve(dataUrlBlob(canvas.toDataURL(mime)));
      } catch (error) {
        reject(error);
      }
    });
  }

  function notify(message, kind) {
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show("Download", message, kind || "error");
    }
  }

  function saveBlob(blob, name) {
    // The desktop build must use the native Save dialog; the browser path is a
    // plain <a download>. Both are already implemented by gb-shared.js.
    if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
      return Promise.resolve(window.GBShared.saveBlob(blob, name));
    }
    var url = URL.createObjectURL(blob);
    var link = document.createElement("a");
    link.href = url;
    link.download = name;
    link.rel = "noopener";
    if (document.body) document.body.appendChild(link);
    link.click();
    if (link.remove) link.remove();
    setTimeout(function () {
      try { URL.revokeObjectURL(url); } catch (e) { /* already gone */ }
    }, 1000);
    return Promise.resolve(null);
  }

  function downloadSvg(baseName, markup, options) {
    var svg = prepare(markup, options);
    if (!openTag(svg)) return Promise.reject(new Error("there is no drawing to export"));
    return saveBlob(
      new Blob([svg], { type: "image/svg+xml;charset=utf-8" }),
      fileName(baseName, "svg")
    ).then(function (saved) {
      notify(fileName(baseName, "svg") + " downloaded", "success");
      return saved;
    });
  }

  function downloadPng(baseName, markup, options) {
    var settings = options || {};
    // prepare() paints the background into the SVG itself, so the PNG and the
    // SVG download show exactly the same picture.
    var svg = prepare(markup, settings);
    if (!openTag(svg)) return Promise.reject(new Error("there is no drawing to export"));
    return rasterise(svg, { scale: settings.scale })
      .then(function (canvas) { return toBlob(canvas, "image/png"); })
      .then(function (blob) { return saveBlob(blob, fileName(baseName, "png")); })
      .then(function (saved) {
        notify(fileName(baseName, "png") + " downloaded", "success");
        return saved;
      });
  }

  function downloadText(baseName, text, mime, ext) {
    var body = String(text == null ? "" : text);
    if (!body) return Promise.reject(new Error("there is nothing to download"));
    var name = fileName(baseName, ext || "txt");
    return saveBlob(new Blob([body], { type: (mime || "text/plain") + ";charset=utf-8" }), name)
      .then(function (saved) {
        notify(name + " downloaded", "success");
        return saved;
      });
  }

  /* ------------------------------------------------------------- chat panels */

  // A drawing taken from the live preview is named after the moment it was
  // saved: two downloads of a streaming picture must not overwrite each other.
  function drawingName() {
    var now = new Date();
    function pad(value) { return (value < 10 ? "0" : "") + value; }
    return "drawing-" + now.getFullYear() + pad(now.getMonth() + 1) + pad(now.getDate()) +
      "-" + pad(now.getHours()) + pad(now.getMinutes());
  }

  // One entry point for the panels: read the markup lazily (so a drawing that
  // is still streaming downloads its current state), pick the format, and
  // report a failure instead of throwing at the caller.
  function download(format, markupOrGetter, baseName) {
    var markup = typeof markupOrGetter === "function"
      ? markupOrGetter()
      : markupOrGetter;
    if (!markup) {
      notify(window.gbT ? window.gbT("artifact-drawing-not-started", "The drawing has not started yet") : "The drawing has not started yet");
      return Promise.resolve(null);
    }
    var name = baseName || drawingName();
    var task = format === "png" ? downloadPng(name, markup) : downloadSvg(name, markup);
    return task.then(null, function (error) {
      notify((error && error.message) || "The drawing could not be saved");
      return null;
    });
  }

  // The chip both chat panels use. `className` keeps each panel's own styling.
  function drawingButton(format, markupOrGetter, options) {
    options = options || {};
    var label = options.label || (format === "png" ? "Download PNG" : "Download SVG");
    var button = document.createElement("button");
    button.type = "button";
    button.className = options.className || "gb-artifact-download";
    button.textContent = label;
    button.title = options.title || ("Download the drawing as " + String(format).toUpperCase());
    button.setAttribute("aria-label", button.title);
    button.addEventListener("click", function () {
      download(format, markupOrGetter, options.name);
    });
    return button;
  }

  return {
    prepare: prepare,
    serialize: serialize,
    dimensions: dimensions,
    rasterise: rasterise,
    toBlob: toBlob,
    saveBlob: saveBlob,
    downloadSvg: downloadSvg,
    downloadPng: downloadPng,
    downloadText: downloadText,
    download: download,
    drawingButton: drawingButton,
    drawingName: drawingName,
    fileName: fileName,
    slug: slug,
    defaultScale: DEFAULT_SCALE,
  };
})();
