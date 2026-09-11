"use strict";
/* Photo Editor (#1306) — opened by Drive when the user opens an image file.
 * Context (set by drive/modules/02_api.js before opening the window):
 *   window.__PHOTO_FILE_BUCKET, __PHOTO_FILE_PATH, __PHOTO_FILE_SCOPE
 * Loads the file via POST /api/files/read (base64), renders it on a canvas,
 * applies non-destructive CSS-filter adjustments + rotate/flip, and saves
 * back through POST /api/files/write (base64 content, same path) or a copy.
 */
(function () {
  if (window.GBPhotoEditor) return;

  var S = {
    bucket: null,
    path: null,
    scope: null,
    img: null,        // source Image (pristine pixels)
    rotation: 0,      // 0 | 90 | 180 | 270
    flipH: false,
    flipV: false,
    saving: false,
    winId: null,
  };

  function $(id) { return document.getElementById(id); }
  function canvas() { return $("peCanvas"); }
  function setStatus(t) { var el = $("peStatus"); if (el) el.textContent = t; }

  function filterCss() {
    var parts = [];
    var b = $("peBrightness"), c = $("peContrast"), s = $("peSaturation"), bl = $("peBlur");
    if (b) parts.push("brightness(" + b.value + "%)");
    if (c) parts.push("contrast(" + c.value + "%)");
    if (s) parts.push("saturate(" + s.value + "%)");
    if (bl && parseFloat(bl.value) > 0) parts.push("blur(" + bl.value + "px)");
    if ($("peGrayscale") && $("peGrayscale").checked) parts.push("grayscale(1)");
    if ($("peSepia") && $("peSepia").checked) parts.push("sepia(0.8)");
    if ($("peInvert") && $("peInvert").checked) parts.push("invert(1)");
    return parts.join(" ");
  }

  function render() {
    var img = S.img;
    var cv = canvas();
    if (!img || !cv) return;
    var ctx = cv.getContext("2d");
    var rot = S.rotation % 360;
    var swap = rot === 90 || rot === 270;
    cv.width = swap ? img.naturalHeight : img.naturalWidth;
    cv.height = swap ? img.naturalWidth : img.naturalHeight;
    ctx.save();
    ctx.translate(cv.width / 2, cv.height / 2);
    if (rot === 90) ctx.rotate(Math.PI / 2);
    else if (rot === 180) ctx.rotate(Math.PI);
    else if (rot === 270) ctx.rotate(-Math.PI / 2);
    ctx.scale(S.flipH ? -1 : 1, S.flipV ? -1 : 1);
    // Filters must be drawn INTO the pixels (save-back has no CSS), so apply
    // via ctx.filter when supported.
    try { ctx.filter = filterCss() || "none"; } catch (e) { /* older browsers */ }
    ctx.drawImage(img, -img.naturalWidth / 2, -img.naturalHeight / 2);
    ctx.restore();
    var meta = $("peMeta");
    if (meta) meta.textContent = cv.width + "×" + cv.height;
  }

  function bindAdjust() {
    ["peBrightness", "peContrast", "peSaturation", "peBlur"].forEach(function (id) {
      var el = $(id);
      if (!el) return;
      el.addEventListener("input", function () {
        var v = $(id + "Val");
        if (v) v.textContent = el.value;
        render();
      });
    });
    ["peGrayscale", "peSepia", "peInvert"].forEach(function (id) {
      var el = $(id);
      if (el) el.addEventListener("change", render);
    });
    var r = $("peRotateBtn");
    if (r) r.addEventListener("click", function () { S.rotation = (S.rotation + 90) % 360; render(); });
    var fh = $("peFlipHBtn");
    if (fh) fh.addEventListener("click", function () { S.flipH = !S.flipH; render(); });
    var fv = $("peFlipVBtn");
    if (fv) fv.addEventListener("click", function () { S.flipV = !S.flipV; render(); });
    var reset = $("peResetBtn");
    if (reset) reset.addEventListener("click", function () {
      S.rotation = 0; S.flipH = false; S.flipV = false;
      var b = $("peBrightness"), c = $("peContrast"), sa = $("peSaturation"), bl = $("peBlur");
      if (b) { b.value = 100; var bv = $("peBrightnessVal"); if (bv) bv.textContent = "100"; }
      if (c) { c.value = 100; var cv2 = $("peContrastVal"); if (cv2) cv2.textContent = "100"; }
      if (sa) { sa.value = 100; var sv = $("peSaturationVal"); if (sv) sv.textContent = "100"; }
      if (bl) { bl.value = 0; var blv = $("peBlurVal"); if (blv) blv.textContent = "0"; }
      ["peGrayscale", "peSepia", "peInvert"].forEach(function (id) { var el = $(id); if (el) el.checked = false; });
      render();
      setStatus("Reverted.");
    });
  }

  function toBlob(cb) {
    canvas().toBlob(function (blob) { cb(blob); }, "image/png");
  }

  function saveAs(savePath) {
    if (S.saving) return;
    S.saving = true;
    setStatus("Saving…");
    toBlob(function (blob) {
      var fr = new FileReader();
      fr.onload = function () {
        var base64 = String(fr.result).split(",")[1] || "";
        fetch("/api/files/write", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            bucket: S.bucket,
            path: savePath,
            content: base64,
            scope: S.scope || "user",
          }),
        })
          .then(function (r) { return r.json().then(function (d) { return { ok: r.ok, d: d }; }); })
          .then(function (res) {
            S.saving = false;
            if (res.ok) {
              setStatus("Saved: " + savePath);
              if (savePath !== S.path) S.path = savePath;
            } else {
              setStatus("Save failed: " + ((res.d && res.d.error) || "unknown"));
            }
          })
          .catch(function (e) { S.saving = false; setStatus("Save failed: " + e.message); });
      };
      fr.readAsDataURL(blob);
    });
  }

  function save() { saveAs(S.path); }

  function saveAsDialog() {
    if (!S.path) return;
    var base = S.path.replace(/\.[^.]+$/, "");
    var suggested = base + "-edited.png";
    var input = window.prompt("Save copy as (Drive path):", suggested);
    if (input && input.trim()) saveAs(input.trim());
  }

  function fileName() { return (S.path || "").split("/").pop() || "image"; }

  function load() {
    var nameEl = $("peFileName");
    if (nameEl) nameEl.textContent = fileName();
    if (!S.path) {
      var l = $("peLoading");
      if (l) l.textContent = "No file specified — open an image from Drive.";
      return;
    }
    // Binary inline fetch (raw bytes → object URL). Falls back to the base64
    // JSON reader for scope/bucket combos where the inline route may differ.
    fetch("/api/files/download-inline", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ bucket: S.bucket, path: S.path, scope: S.scope || "user" }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.blob();
      })
      .then(function (blob) {
        var url = URL.createObjectURL(blob);
        var img = new Image();
        img.onload = function () {
          URL.revokeObjectURL(url);
          S.img = img;
          var l = $("peLoading");
          if (l) l.remove();
          render();
          setStatus("Ready");
        };
        img.onerror = function () {
          URL.revokeObjectURL(url);
          var l = $("peLoading");
          if (l) l.textContent = "Could not decode this image.";
        };
        img.src = url;
      })
      .catch(function (e) {
        var l = $("peLoading");
        if (l) l.textContent = "Load failed: " + e.message;
        setStatus("Load failed");
      });
  }

  function init(winId) {
    S.winId = winId || S.winId;
    // Two context sources: window-manager globals (set by Drive open) and
    // the URL query (?bucket=&path=&scope=) when launched via deep link.
    S.bucket = window.__PHOTO_FILE_BUCKET || S.bucket;
    S.path = window.__PHOTO_FILE_PATH || S.path;
    S.scope = window.__PHOTO_FILE_SCOPE || S.scope;
    try {
      var qp = new URLSearchParams(window.location.search);
      if (!S.bucket && qp.get("bucket")) S.bucket = qp.get("bucket");
      if (!S.path && qp.get("path")) S.path = qp.get("path");
      if (!S.scope && qp.get("scope")) S.scope = qp.get("scope");
    } catch (e) { /* URLSearchParams unavailable */ }
    var sb = $("peSaveBtn");
    if (sb) sb.addEventListener("click", save);
    var sas = $("peSaveAsBtn");
    if (sas) sas.addEventListener("click", saveAsDialog);
    bindAdjust();
    load();
  }

  window.GBPhotoEditor = { init: init, render: render, save: save };
})();
