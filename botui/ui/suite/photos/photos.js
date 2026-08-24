"use strict";
/* Photos (#1154): image gallery sourced from the user's Drive. Falls back to
   a local demo gallery when the drive API is unreachable. */

(function () {
  if (window.GBPhotos) return;

  const IMG_EXT = /\.(png|jpe?g|gif|webp|bmp|svg|avif)$/i;

  function load() {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    grid.innerHTML = '<div class="photos-empty">Loading your images from Drive…</div>';
    const count = document.getElementById("photosCount");
    if (count) count.textContent = "";

    fetch("/api/files/list?scope=user&_=" + Date.now())
      .then(function (r) {
        if (!r.ok) throw new Error("list failed");
        return r.json();
      })
      .then(function (items) {
        const images = (items || []).filter(function (f) {
          return !f.is_dir && IMG_EXT.test(f.name || f.path);
        });
        if (!images.length) {
          grid.innerHTML = '<div class="photos-empty">No images found in your Drive yet. Upload some and refresh.</div>';
          return;
        }
        if (count) count.textContent = images.length + " photo" + (images.length === 1 ? "" : "s");
        renderPlaceholders(images);
        loadThumbnails(images, 0);
      })
      .catch(function () {
        renderDemo();
      });
  }

  function renderPlaceholders(images) {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    grid.innerHTML = images
      .map(function (f, i) {
        return (
          '<div class="photo-tile" data-i="' + i + '" data-path="' + escapeAttr(f.path) + '" title="' + escapeHtml(f.name) + '">' +
          '<div style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;color:#9ca3af;font-size:24px">🖼</div>' +
          "</div>"
        );
      })
      .join("");
    bindTiles();
  }

  function loadThumbnails(images, i) {
    if (i >= images.length) return;
    const f = images[i];
    const tile = document.querySelector('.photo-tile[data-path="' + CSS.escape(f.path) + '"]');
    fetch("/api/files/download", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path: f.path, scope: "user" }),
    })
      .then(function (r) { return r.json(); })
      .then(function (data) {
        if (data && data.content && tile) {
          const url = "data:image/png;base64," + data.content;
          tile.innerHTML = '<img src="' + url + '" alt="' + escapeHtml(f.name) + '" />';
          tile.dataset.url = url;
        }
      })
      .catch(function () {})
      .then(function () {
        loadThumbnails(images, i + 1);
      });
  }

  function bindTiles() {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    Array.from(grid.querySelectorAll(".photo-tile")).forEach(function (tile) {
      tile.addEventListener("click", function () {
        const url = tile.dataset.url;
        const lightbox = document.getElementById("photosLightbox");
        const img = document.getElementById("photosLbImg");
        if (lightbox && img) {
          if (url) {
            img.src = url;
            lightbox.style.display = "flex";
          } else {
            window.open("/api/files/download", "_blank");
          }
        }
      });
    });
  }

  function renderDemo() {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    const colors = ["#3b82f6", "#84d669", "#f59e0b", "#ec4899", "#8b5cf6", "#06b6d4"];
    grid.innerHTML = colors
      .map(function (c, i) {
        return (
          '<div class="photo-tile" title="Sample ' + (i + 1) + '">' +
          '<div style="width:100%;height:100%;background:linear-gradient(135deg,' + c + '22,' + c + '66);display:flex;align-items:center;justify-content:center;font-size:34px;color:' + c + '">' + ["🌄", "🌊", "🌿", "🌇", "🪐", "🏔"][i] + "</div>" +
          "</div>"
        );
      })
      .join("");
    const count = document.getElementById("photosCount");
    if (count) count.textContent = "demo mode";
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function escapeAttr(s) {
    return String(s).replace(/"/g, "&quot;");
  }

  document.addEventListener("DOMContentLoaded", function () {
    const refresh = document.getElementById("photosRefresh");
    if (refresh) refresh.addEventListener("click", load);
    const close = document.getElementById("photosLbClose");
    if (close) {
      close.addEventListener("click", function () {
        document.getElementById("photosLightbox").style.display = "none";
      });
    }
    const lb = document.getElementById("photosLightbox");
    if (lb) {
      lb.addEventListener("click", function (e) {
        if (e.target === lb) lb.style.display = "none";
      });
    }
    load();
  });

  window.GBPhotos = { load: load };
})();