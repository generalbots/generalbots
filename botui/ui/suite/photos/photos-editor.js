"use strict";
/* Photo Editor (#1306) — opened by Drive when the user opens an image file.
 * #2282: per-window context/state so two editor windows never share a file.
 * Loads via POST /api/files/download-inline, renders on canvas, saves via
 * POST /api/files/write (base64) to the window's own path.
 */
(function () {
  if (window.GBPhotoEditor) return;

  var states = {};

  function rootFor(winId) {
    return winId ? document.getElementById("window-body-" + winId) : null;
  }
  function q(root, id) {
    if (root && root.querySelector) {
      var el = root.querySelector("#" + id);
      if (el) return el;
    }
    return document.getElementById(id);
  }
  function resolveState(winId) {
    if (winId && states[winId]) return states[winId];
    var keys = Object.keys(states);
    if (keys.length === 1) return states[keys[0]];
    var ae = document.activeElement;
    if (ae && ae.closest) {
      var w = ae.closest('[id^="window-body-photo-editor"]');
      if (w && states[w.dataset.windowId || w.id.replace("window-body-", "")]) {
        return states[w.dataset.windowId || w.id.replace("window-body-", "")];
      }
    }
    return states[keys[keys.length - 1]] || null;
  }

  function filterCss(st) {
    var root = rootFor(st.winId);
    var parts = [];
    var b = q(root, "peBrightness"), c = q(root, "peContrast"), s = q(root, "peSaturation"), bl = q(root, "peBlur");
    if (b) parts.push("brightness(" + b.value + "%)");
    if (c) parts.push("contrast(" + c.value + "%)");
    if (s) parts.push("saturate(" + s.value + "%)");
    if (bl && parseFloat(bl.value) > 0) parts.push("blur(" + bl.value + "px)");
    if (q(root, "peGrayscale") && q(root, "peGrayscale").checked) parts.push("grayscale(1)");
    if (q(root, "peSepia") && q(root, "peSepia").checked) parts.push("sepia(0.8)");
    if (q(root, "peInvert") && q(root, "peInvert").checked) parts.push("invert(1)");
    return parts.join(" ");
  }

  function renderState(st) {
    if (!st || !st.img) return;
    var root = rootFor(st.winId);
    var cv = q(root, "peCanvas");
    if (!cv) return;
    var ctx = cv.getContext("2d");
    var rot = st.rotation % 360;
    var swap = rot === 90 || rot === 270;
    cv.width = swap ? st.img.naturalHeight : st.img.naturalWidth;
    cv.height = swap ? st.img.naturalWidth : st.img.naturalHeight;
    ctx.save();
    ctx.translate(cv.width / 2, cv.height / 2);
    if (rot === 90) ctx.rotate(Math.PI / 2);
    else if (rot === 180) ctx.rotate(Math.PI);
    else if (rot === 270) ctx.rotate(-Math.PI / 2);
    ctx.scale(st.flipH ? -1 : 1, st.flipV ? -1 : 1);
    try { ctx.filter = filterCss(st) || "none"; } catch (e) { /* older browsers */ }
    ctx.drawImage(st.img, -st.img.naturalWidth / 2, -st.img.naturalHeight / 2);
    ctx.restore();
    var meta = q(root, "peMeta");
    if (meta) meta.textContent = cv.width + "×" + cv.height;
  }

  function setStatus(st, t) {
    var root = rootFor(st && st.winId);
    var el = q(root, "peStatus");
    if (el) el.textContent = t;
  }

  function toBlobState(st, cb) {
    var cv = q(rootFor(st.winId), "peCanvas");
    if (cv) cv.toBlob(function (blob) { cb(blob); }, "image/png");
  }

  function saveAsState(st, savePath) {
    if (!st || st.saving || !savePath) return;
    st.saving = true;
    setStatus(st, "Saving…");
    toBlobState(st, function (blob) {
      var fr = new FileReader();
      fr.onload = function () {
        var base64 = String(fr.result).split(",")[1] || "";
        fetch("/api/files/write", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ bucket: st.bucket, path: savePath, content: base64, scope: st.scope || "user" }),
        })
          .then(function (r) { return r.json().then(function (d) { return { ok: r.ok, d: d }; }); })
          .then(function (res) {
            st.saving = false;
            if (res.ok) {
              setStatus(st, "Saved: " + savePath);
              st.path = savePath;
            } else {
              setStatus(st, "Save failed: " + ((res.d && res.d.error) || "unknown"));
            }
          })
          .catch(function (e) { st.saving = false; setStatus(st, "Save failed: " + e.message); });
      };
      fr.readAsDataURL(blob);
    });
  }

  function saveAsDialogState(st) {
    if (!st || !st.path) return;
    var base = st.path.replace(/\.[^.]+$/, "");
    var suggested = base + "-edited.png";
    var input = (window.GBModal || { prompt: prompt }).prompt("Save copy as (Drive path):", suggested);
    if (input && input.trim()) saveAsState(st, input.trim());
  }

  function bindAdjust(st) {
    var root = rootFor(st.winId);
    ["peBrightness", "peContrast", "peSaturation", "peBlur"].forEach(function (id) {
      var el = q(root, id);
      if (!el) return;
      el.addEventListener("input", function () {
        var v = q(root, id + "Val");
        if (v) v.textContent = el.value;
        renderState(st);
      });
    });
    ["peGrayscale", "peSepia", "peInvert"].forEach(function (id) {
      var el = q(root, id);
      if (el) el.addEventListener("change", function () { renderState(st); });
    });
    var r = q(root, "peRotateBtn");
    if (r) r.addEventListener("click", function () { st.rotation = (st.rotation + 90) % 360; renderState(st); });
    var fh = q(root, "peFlipHBtn");
    if (fh) fh.addEventListener("click", function () { st.flipH = !st.flipH; renderState(st); });
    var fv = q(root, "peFlipVBtn");
    if (fv) fv.addEventListener("click", function () { st.flipV = !st.flipV; renderState(st); });
    var reset = q(root, "peResetBtn");
    if (reset) reset.addEventListener("click", function () {
      st.rotation = 0; st.flipH = false; st.flipV = false;
      var b = q(root, "peBrightness"), c = q(root, "peContrast"), sa = q(root, "peSaturation"), bl = q(root, "peBlur");
      if (b) { b.value = 100; var bv = q(root, "peBrightnessVal"); if (bv) bv.textContent = "100"; }
      if (c) { c.value = 100; var cv2 = q(root, "peContrastVal"); if (cv2) cv2.textContent = "100"; }
      if (sa) { sa.value = 100; var sv = q(root, "peSaturationVal"); if (sv) sv.textContent = "100"; }
      if (bl) { bl.value = 0; var blv = q(root, "peBlurVal"); if (blv) blv.textContent = "0"; }
      ["peGrayscale", "peSepia", "peInvert"].forEach(function (id) { var el = q(root, id); if (el) el.checked = false; });
      renderState(st);
      setStatus(st, "Reverted.");
    });
  }

  function loadState(st) {
    var root = rootFor(st.winId);
    var nameEl = q(root, "peFileName");
    if (nameEl) nameEl.textContent = (st.path || "").split("/").pop() || "image";
    if (!st.path) {
      var l = q(root, "peLoading");
      if (l) l.textContent = "No file specified — open an image from Drive.";
      return;
    }
    fetch("/api/files/download-inline", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ bucket: st.bucket, path: st.path, scope: st.scope || "user" }),
    })
      .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.blob(); })
      .then(function (blob) {
        var url = URL.createObjectURL(blob);
        var img = new Image();
        img.onload = function () {
          URL.revokeObjectURL(url);
          st.img = img;
          var l = q(root, "peLoading");
          if (l) l.remove();
          renderState(st);
          setStatus(st, "Ready");
        };
        img.onerror = function () {
          URL.revokeObjectURL(url);
          var l = q(root, "peLoading");
          if (l) l.textContent = "Could not decode this image.";
        };
        img.src = url;
      })
      .catch(function (e) {
        var l = q(root, "peLoading");
        if (l) l.textContent = "Load failed: " + e.message;
        setStatus(st, "Load failed");
      });
  }

  function init(winId) {
    var root = rootFor(winId);
    var ds = root && root.dataset || {};
    var map = (window.__PHOTO_FILE_CTX && window.__PHOTO_FILE_CTX[winId]) ||
      (window.__GB_WINDOW_FILE_CTX && window.__GB_WINDOW_FILE_CTX[winId]) || {};
    var st = states[winId] || { bucket: null, path: null, scope: null, img: null, rotation: 0, flipH: false, flipV: false, saving: false, winId: winId };
    st.winId = winId;
    st.bucket = ds.gbFileBucket || map.bucket || window.__PHOTO_FILE_BUCKET || st.bucket;
    st.path = ds.gbFilePath || map.path || window.__PHOTO_FILE_PATH || st.path;
    st.scope = ds.gbFileScope || map.scope || window.__PHOTO_FILE_SCOPE || st.scope;
    try {
      var qp = new URLSearchParams(window.location.search);
      if (!st.bucket && qp.get("bucket")) st.bucket = qp.get("bucket");
      if (!st.path && qp.get("path")) st.path = qp.get("path");
      if (!st.scope && qp.get("scope")) st.scope = qp.get("scope");
    } catch (e) { /* URLSearchParams unavailable */ }
    states[winId] = st;
    var sb = q(root, "peSaveBtn");
    if (sb && !sb.dataset.peBound) { sb.dataset.peBound = "1"; sb.addEventListener("click", function () { saveAsState(st, st.path); }); }
    var sas = q(root, "peSaveAsBtn");
    if (sas && !sas.dataset.peBound) { sas.dataset.peBound = "1"; sas.addEventListener("click", function () { saveAsDialogState(st); }); }
    bindAdjust(st);
    loadState(st);
  }

  window.GBPhotoEditor = {
    init: init,
    render: function () { var st = resolveState(); if (st) renderState(st); },
    save: function () { var st = resolveState(); if (st) saveAsState(st, st.path); },
  };
})();
