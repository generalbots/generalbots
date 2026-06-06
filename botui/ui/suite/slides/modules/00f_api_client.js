// botui/ui/suite/slides/modules/00f_api_client.js
// REST client for /api/slides/* endpoints on botserver.
// Mirrors 00f_api_client.js (sheet, docs) but for the slides suite.
"use strict";

(function () {
  const DEFAULT_TIMEOUT_MS = 8000;
  const MAX_RETRIES = 3;
  const RETRY_BASE_MS = 250;
  const CACHE_TTL_MS = 5000;
  const cache = new Map();

  function getCsrfToken() {
    const meta = document.querySelector('meta[name="csrf-token"]');
    if (meta) return meta.getAttribute("content") || "";
    try {
      const m = document.cookie.match(/(?:^|;\s*)gb\.csrf=([^;]+)/);
      if (m) return decodeURIComponent(m[1]);
    } catch (e) { /* cookie access blocked */ }
    return "";
  }

  function getAuthHeader() {
    try {
      const t = localStorage.getItem("gb.auth.token");
      if (t) return "Bearer " + t;
    } catch (e) { /* localStorage disabled */ }
    return "";
  }

  function delay(ms) { return new Promise(function (resolve) { setTimeout(resolve, ms); }); }

  function cacheGet(key) {
    const entry = cache.get(key);
    if (!entry) return null;
    if (Date.now() - entry.ts > CACHE_TTL_MS) { cache.delete(key); return null; }
    return entry.value;
  }

  function cacheSet(key, value) { cache.set(key, { ts: Date.now(), value: value }); }

  function cacheClear(prefix) {
    if (!prefix) { cache.clear(); return; }
    for (const k of Array.from(cache.keys())) {
      if (k.indexOf(prefix) === 0) cache.delete(k);
    }
  }

  async function request(method, path, body, options) {
    const opts = options || {};
    const timeoutMs = opts.timeout || DEFAULT_TIMEOUT_MS;
    const useCache = opts.cache !== false;
    const cacheKey = method === "GET" ? (method + " " + path) : null;
    if (useCache && cacheKey) {
      const c = cacheGet(cacheKey);
      if (c) return { ok: true, status: 200, data: c, cached: true };
    }

    const headers = { "Content-Type": "application/json", "Accept": "application/json" };
    const csrf = getCsrfToken();
    if (csrf) headers["X-CSRF-Token"] = csrf;
    const auth = getAuthHeader();
    if (auth) headers["Authorization"] = auth;

    const init = { method: method, headers: headers, credentials: "same-origin" };
    if (body !== undefined && body !== null && method !== "GET") init.body = JSON.stringify(body);

    let lastError = null;
    for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
      const controller = (typeof AbortController !== "undefined") ? new AbortController() : null;
      if (controller) init.signal = controller.signal;
      const timer = (controller && typeof setTimeout === "function")
        ? setTimeout(function () { controller.abort(); }, timeoutMs)
        : null;
      try {
        if (!window.fetch) {
          return { ok: false, status: 0, error: { code: "NO_FETCH", message: "fetch API unavailable" } };
        }
        const res = await fetch(path, init);
        if (timer) clearTimeout(timer);
        if (res.status >= 500 && attempt < MAX_RETRIES - 1) {
          lastError = { code: "HTTP_" + res.status, message: res.statusText };
          await delay(RETRY_BASE_MS * Math.pow(2, attempt));
          continue;
        }
        let data = null;
        const ct = res.headers.get("Content-Type") || "";
        if (ct.indexOf("application/json") >= 0) {
          try { data = await res.json(); } catch (e) { data = null; }
        } else {
          try { data = await res.blob(); } catch (e) { data = null; }
        }
        if (!res.ok) {
          return {
            ok: false, status: res.status,
            error: (data && data.error) ? data.error : { code: "HTTP_" + res.status, message: res.statusText || "failed" },
          };
        }
        if (useCache && cacheKey && data !== null) cacheSet(cacheKey, data);
        return { ok: true, status: res.status, data: data, cached: false };
      } catch (err) {
        if (timer) clearTimeout(timer);
        lastError = { code: "NETWORK", message: (err && err.message) || String(err) };
        if (attempt < MAX_RETRIES - 1) {
          await delay(RETRY_BASE_MS * Math.pow(2, attempt));
          continue;
        }
      }
    }
    return { ok: false, status: 0, error: lastError || { code: "UNKNOWN", message: "unknown" } };
  }

  function post(path, body, options) { return request("POST", path, body, options); }
  function get(path, options) { return request("GET", path, null, options); }

  function buildQS(params) {
    if (!params) return "";
    const parts = [];
    for (const k of Object.keys(params)) {
      if (params[k] !== undefined && params[k] !== null) {
        parts.push(encodeURIComponent(k) + "=" + encodeURIComponent(params[k]));
      }
    }
    return parts.length ? "?" + parts.join("&") : "";
  }

  const API = {
    list: function () { return get("/api/slides/list"); },
    search: function (q) { return get("/api/slides/search?q=" + encodeURIComponent(q)); },
    load: function (presId) { return get("/api/slides/load?pres_id=" + encodeURIComponent(presId)); },
    save: function (presId, state) { return post("/api/slides/save", { pres_id: presId, state: state }); },
    delete: function (presId) { return post("/api/slides/delete", { pres_id: presId }); },
    new: function () { return get("/api/slides/new"); },
    addSlide: function (presId, slide) { return post("/api/slides/slide/add", { pres_id: presId, slide: slide }); },
    deleteSlide: function (presId, slideId) { return post("/api/slides/slide/delete", { pres_id: presId, slide_id: slideId }); },
    duplicateSlide: function (presId, slideId) { return post("/api/slides/slide/duplicate", { pres_id: presId, slide_id: slideId }); },
    reorderSlide: function (presId, slideId, newIndex) {
      return post("/api/slides/slide/reorder", { pres_id: presId, slide_id: slideId, new_index: newIndex });
    },
    updateSlideNotes: function (presId, slideId, notes) {
      return post("/api/slides/slide/notes", { pres_id: presId, slide_id: slideId, notes: notes });
    },
    addElement: function (presId, slideId, element) {
      return post("/api/slides/element/add", { pres_id: presId, slide_id: slideId, element: element });
    },
    updateElement: function (presId, slideId, element) {
      return post("/api/slides/element/update", { pres_id: presId, slide_id: slideId, element: element });
    },
    deleteElement: function (presId, slideId, elementId) {
      return post("/api/slides/element/delete", { pres_id: presId, slide_id: slideId, element_id: elementId });
    },
    applyTheme: function (presId, theme) { return post("/api/slides/theme", { pres_id: presId, theme: theme }); },
    exportPresentation: function (presId, format) {
      return post("/api/slides/export", { pres_id: presId, format: format || "pdf" });
    },
    importPresentation: function (file) {
      const init = {
        method: "POST",
        headers: { "Accept": "application/json" },
        body: file,
        credentials: "same-origin",
      };
      const csrf = getCsrfToken();
      if (csrf) init.headers["X-CSRF-Token"] = csrf;
      return fetch("/api/slides/import", init).then(function (r) {
        return r.json().then(function (d) {
          return { ok: r.ok, status: r.status, data: d };
        });
      });
    },
    setTransition: function (presId, slideId, transition) {
      return post("/api/slides/transition", { pres_id: presId, slide_id: slideId, transition: transition });
    },
    clearAllTransitions: function (presId) { return post("/api/slides/transition/all", { pres_id: presId }); },
    removeTransition: function (presId, slideId) {
      return post("/api/slides/transition/remove", { pres_id: presId, slide_id: slideId });
    },
    addMedia: function (presId, media) { return post("/api/slides/media", { pres_id: presId, media: media }); },
    updateMedia: function (presId, mediaId, media) {
      return post("/api/slides/media/update", { pres_id: presId, media_id: mediaId, media: media });
    },
    deleteMedia: function (presId, mediaId) { return post("/api/slides/media/delete", { pres_id: presId, media_id: mediaId }); },
    listMedia: function (presId) { return get("/api/slides/media/list?pres_id=" + encodeURIComponent(presId)); },
    startPresenter: function (presId) { return post("/api/slides/presenter/start", { pres_id: presId }); },
    updatePresenter: function (presId, state) { return post("/api/slides/presenter/update", { pres_id: presId, state: state }); },
    endPresenter: function (presId) { return post("/api/slides/presenter/end", { pres_id: presId }); },
    setPresenterNotes: function (presId, notes) { return post("/api/slides/presenter/notes", { pres_id: presId, notes: notes }); },
    cacheClear: cacheClear,
  };

  window.SlidesAPI = API;
})();
