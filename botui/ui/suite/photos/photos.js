"use strict";
/* Photos (#1154, #2413): image gallery sourced from the user's Drive. Uses lazy
   loading, bounded-concurrency downloads, r.ok checks, MIME inference, blob URLs
   (revoked on replace/close), and visible error states instead of serial
   full-size base64 tiles. */

(function () {
  if (window.GBPhotos) return;

  const IMG_EXT = /\.(png|jpe?g|gif|webp|bmp|svg|avif)$/i;
  const CONCURRENCY = 4;
  const PAGE_SIZE = 100;

  let allImages = [];
  let renderedCount = 0;
  let observer = null;
  let activeLoads = 0;
  let loadQueue = [];
  const inFlight = new Set();
  const objectUrls = new Map(); // path -> blob URL for grid/thumb

  // gbT with an authored fallback. Interpolate params ourselves when the
  // catalogue has not loaded yet so counts never flash raw "{count}" text.
  function t(key, fallback, params) {
    const raw = window.gbT ? window.gbT(key, fallback, params) : fallback;
    if (!params) return raw;
    return String(raw).replace(/\{\s*\$?(\w+)\s*\}/g, function (_, name) {
      return Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : "";
    });
  }

  function authHeaders() {
    const headers = { "Content-Type": "application/json" };
    try {
      const token =
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        (window.GBSecurity && typeof window.GBSecurity.getToken === "function" ? window.GBSecurity.getToken() : "") ||
        (window.GBAuthGuard && typeof window.GBAuthGuard.getToken === "function" ? window.GBAuthGuard.getToken() : "");
      if (token) headers.Authorization = "Bearer " + token;
    } catch (_) { window.gbLogError ? gbLogError(_, "authHeaders") : console.error("authHeaders", _); }
    return headers;
  }

  function mimeFromName(name) {
    const n = String(name || "").toLowerCase();
    if (n.endsWith(".png")) return "image/png";
    if (n.endsWith(".jpg") || n.endsWith(".jpeg")) return "image/jpeg";
    if (n.endsWith(".gif")) return "image/gif";
    if (n.endsWith(".webp")) return "image/webp";
    if (n.endsWith(".bmp")) return "image/bmp";
    if (n.endsWith(".svg")) return "image/svg+xml";
    if (n.endsWith(".avif")) return "image/avif";
    return "application/octet-stream";
  }

  function revokeUrlFor(path) {
    if (objectUrls.has(path)) {
      try { URL.revokeObjectURL(objectUrls.get(path)); } catch (_) { window.gbLogError ? gbLogError(_, "revokeUrlFor") : console.error("revokeUrlFor", _); }
      objectUrls.delete(path);
    }
  }

  function clearObjectUrls() {
    objectUrls.forEach(function (u) {
      try { URL.revokeObjectURL(u); } catch (_) { window.gbLogError ? gbLogError(_, "clearObjectUrls") : console.error("clearObjectUrls", _); }
    });
    objectUrls.clear();
  }

  function base64ToBlob(base64, mime) {
    const clean = String(base64 || "").replace(/\s/g, "");
    const binary = atob(clean);
    const len = binary.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) bytes[i] = binary.charCodeAt(i);
    return new Blob([bytes], { type: mime || "application/octet-stream" });
  }

  function load() {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    teardownObserver();
    clearObjectUrls();
    allImages = [];
    renderedCount = 0;
    loadQueue = [];
    activeLoads = 0;
    inFlight.clear();
    grid.innerHTML = '<div class="photos-empty" role="status" aria-live="polite">' +
      escapeHtml(t("photos-loading", "Loading your images from Drive…")) + "</div>";
    const count = document.getElementById("photosCount");
    if (count) count.textContent = "";

    fetch("/api/files/list?scope=user&_=" + Date.now(), {
      headers: authHeaders(),
      credentials: "same-origin",
    })
      .then(function (r) {
        if (!r.ok) throw new Error("Drive listing failed (HTTP " + r.status + ")");
        return r.json();
      })
      .then(function (payload) {
        const items = Array.isArray(payload) ? payload : (payload && (payload.files || payload.items)) || [];
        const images = items.filter(function (f) {
          return f && !f.is_dir && IMG_EXT.test(f.name || f.path || "");
        });
        if (!images.length) {
          grid.innerHTML = '<div class="photos-empty" role="status" aria-live="polite">' +
            escapeHtml(t("photos-empty", "No images found in your Drive yet. Upload some and refresh.")) + "</div>";
          return;
        }
        allImages = images;
        if (count) {
          count.textContent = images.length === 1
            ? t("photos-count-one", "1 photo", { count: 1 })
            : t("photos-count-many", "{count} photos", { count: images.length });
        }
        renderNextPage();
      })
      .catch(function (err) {
        renderError(err);
      });
  }

  function renderNextPage() {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    const next = allImages.slice(renderedCount, renderedCount + PAGE_SIZE);
    if (!next.length) return;
    if (renderedCount === 0) grid.innerHTML = "";
    const frag = document.createDocumentFragment();
    next.forEach(function (f) {
      const tile = document.createElement("div");
      tile.className = "photo-tile";
      tile.dataset.path = f.path || "";
      tile.title = f.name || f.path || "";
      tile.innerHTML = '<div class="photo-tile-placeholder" aria-hidden="true">' + (window.GBIcons ? window.GBIcons.html("image", 26) : "🖼") + '</div>';
      tile.addEventListener("click", function () { openLightbox(tile, f); });
      frag.appendChild(tile);
    });
    grid.appendChild(frag);
    renderedCount += next.length;
    bindLazyLoading();
    maybeAddLoadMore();
  }

  function maybeAddLoadMore() {
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    const existing = document.getElementById("photosLoadMore");
    if (existing) existing.remove();
    if (renderedCount >= allImages.length) return;
    const wrap = document.createElement("div");
    wrap.id = "photosLoadMore";
    wrap.style.cssText = "grid-column:1/-1;display:flex;justify-content:center;padding:12px;";
    const btn = document.createElement("button");
    btn.className = "photos-retry";
    btn.type = "button";
    btn.textContent = t("photos-load-more", "Load more ({remaining} remaining)", {
      remaining: allImages.length - renderedCount,
    });
    btn.addEventListener("click", function () { renderNextPage(); });
    wrap.appendChild(btn);
    grid.appendChild(wrap);
  }

  function teardownObserver() {
    if (observer) {
      try { observer.disconnect(); } catch (_) { window.gbLogError ? gbLogError(_, "teardownObserver") : console.error("teardownObserver", _); }
      observer = null;
    }
  }

  function bindLazyLoading() {
    const tiles = Array.from(document.querySelectorAll(".photo-tile:not([data-observed])"));
    if (!tiles.length) return;
    if (!("IntersectionObserver" in window)) {
      tiles.forEach(function (tile) { tile.dataset.observed = "1"; enqueueTile(tile); });
      pumpQueue();
      return;
    }
    if (!observer) {
      observer = new IntersectionObserver(function (entries) {
        entries.forEach(function (entry) {
          if (!entry.isIntersecting) return;
          const tile = entry.target;
          observer.unobserve(tile);
          if (tile.dataset.loaded || tile.dataset.loading) return;
          enqueueTile(tile);
        });
        pumpQueue();
      }, { rootMargin: "200px 0px" });
    }
    tiles.forEach(function (tile) {
      tile.dataset.observed = "1";
      observer.observe(tile);
    });
  }

  function enqueueTile(tile) {
    if (!tile || tile.dataset.loaded || tile.dataset.loading || tile.dataset.queued) return;
    tile.dataset.queued = "1";
    loadQueue.push(tile);
  }

  function pumpQueue() {
    while (activeLoads < CONCURRENCY && loadQueue.length) {
      const tile = loadQueue.shift();
      if (!tile || tile.dataset.loaded || tile.dataset.loading) continue;
      tile.dataset.queued = "";
      loadTile(tile);
    }
  }

  async function fetchImageBlob(path, name) {
    const resp = await fetch("/api/files/download", {
      method: "POST",
      headers: authHeaders(),
      credentials: "same-origin",
      body: JSON.stringify({ path: path, scope: "user" }),
    });
    if (!resp.ok) throw new Error("HTTP " + resp.status);
    const contentType = (resp.headers.get("content-type") || "").toLowerCase();
    if (contentType.indexOf("application/json") !== -1) {
      const data = await resp.json();
      if (!data || !data.content) throw new Error("No image content");
      const mime = data.mime || data.mime_type || data.content_type || mimeFromName(name || path);
      return base64ToBlob(data.content, mime);
    }
    // Fallback: raw image bytes
    const blob = await resp.blob();
    if (blob && blob.type) return blob;
    return new Blob([blob], { type: mimeFromName(name || path) });
  }

  async function loadTile(tile) {
    const path = tile.getAttribute("data-path") || "";
    if (!path || tile.dataset.loaded || tile.dataset.loading || inFlight.has(path)) {
      pumpQueue();
      return;
    }
    inFlight.add(path);
    tile.dataset.loading = "1";
    activeLoads++;
    try {
      const name = tile.getAttribute("title") || path;
      const blob = await fetchImageBlob(path, name);
      revokeUrlFor(path);
      const url = URL.createObjectURL(blob);
      objectUrls.set(path, url);
      if (tile.isConnected) {
        tile.innerHTML = "";
        const img = document.createElement("img");
        img.loading = "lazy";
        img.decoding = "async";
        img.alt = name;
        img.src = url;
        tile.appendChild(img);
        tile.dataset.loaded = "1";
      } else {
        revokeUrlFor(path);
      }
    } catch (err) {
      if (tile.isConnected && !tile.dataset.warned) {
        tile.dataset.warned = "1";
        const label = tile.getAttribute("title") || path;
        tile.title = t("photos-preview-unavailable", "{name} — preview unavailable", { name: label });
        tile.innerHTML = '<div class="photo-tile-placeholder" aria-hidden="true">' + (window.GBIcons ? window.GBIcons.html("image", 26) : "🖼") + '</div>';
      }
    } finally {
      delete tile.dataset.loading;
      inFlight.delete(path);
      activeLoads = Math.max(0, activeLoads - 1);
      pumpQueue();
    }
  }

  async function openLightbox(tile, file) {
    const lightbox = document.getElementById("photosLightbox");
    const img = document.getElementById("photosLbImg");
    if (!lightbox || !img) return;
    const path = (tile && tile.getAttribute("data-path")) || (file && file.path) || "";
    const name = (file && file.name) || path;
    const existing = path ? objectUrls.get(path) : null;
    try {
      if (existing) {
        img.src = existing;
      } else {
        const blob = await fetchImageBlob(path, name);
        const url = URL.createObjectURL(blob);
        img.dataset.tempUrl = url;
        img.src = url;
      }
      lightbox.style.display = "flex";
    } catch (_) {
      renderError(new Error(t("photos-load-failed", "This image could not be loaded from Drive.")));
    }
  }

  function renderError(err) {
    teardownObserver();
    const grid = document.getElementById("photosGrid");
    if (!grid) return;
    const msg = (err && err.message) ? err.message : t("photos-drive-error", "Could not reach your Drive.");
    grid.innerHTML =
      '<div class="photos-empty photos-error" role="alert">' +
      "<p>" + escapeHtml(msg) + "</p>" +
      '<p class="photos-hint">' + escapeHtml(t("photos-error-hint", "Connect a Drive to see your images here. If your Drive is connected, try refreshing.")) + "</p>" +
      '<button class="photos-retry" id="photosRetry" type="button">' + escapeHtml(t("photos-retry", "Retry")) + "</button>" +
      "</div>";
    const retry = document.getElementById("photosRetry");
    if (retry) retry.addEventListener("click", load);
    const count = document.getElementById("photosCount");
    if (count) count.textContent = "";
  }

  function escapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function bootPhotos() {
    const lbImg = document.getElementById("photosLbImg");
    if (lbImg) lbImg.alt = t("photos-preview-alt", "Preview");
    const refresh = document.getElementById("photosRefresh");
    if (refresh && !refresh.dataset.gbPhotosInit) {
      refresh.dataset.gbPhotosInit = "1";
      refresh.addEventListener("click", load);
    }
    const close = document.getElementById("photosLbClose");
    if (close && !close.dataset.gbPhotosInit) {
      close.dataset.gbPhotosInit = "1";
      close.addEventListener("click", function () {
        const lbEl = document.getElementById("photosLightbox");
        const img = document.getElementById("photosLbImg");
        if (img && img.dataset.tempUrl) {
          try { URL.revokeObjectURL(img.dataset.tempUrl); } catch (_) { window.gbLogError ? gbLogError(_, "bootPhotos") : console.error("bootPhotos", _); }
          delete img.dataset.tempUrl;
          img.removeAttribute("src");
        }
        if (lbEl) lbEl.style.display = "none";
      });
    }
    const lb = document.getElementById("photosLightbox");
    if (lb && !lb.dataset.gbPhotosInit) {
      lb.dataset.gbPhotosInit = "1";
      lb.addEventListener("click", function (e) {
        if (e.target === lb) {
          const img = document.getElementById("photosLbImg");
          if (img && img.dataset.tempUrl) {
            try { URL.revokeObjectURL(img.dataset.tempUrl); } catch (_) { window.gbLogError ? gbLogError(_, "bootPhotos") : console.error("bootPhotos", _); }
            delete img.dataset.tempUrl;
            img.removeAttribute("src");
          }
          lb.style.display = "none";
        }
      });
    }
    load();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootPhotos, { once: true });
  } else {
    bootPhotos();
  }

  window.GBPhotos = { load: load };
})();