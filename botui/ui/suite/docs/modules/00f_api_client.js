// botui/ui/suite/docs/modules/00f_api_client.js
// REST client for /api/docs/* endpoints on botserver.
// Mirrors 00f_api_client.js (sheet) but for the docs suite.
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
          try { data = await res.text(); } catch (e) { data = null; }
        }
        if (!res.ok) {
          return {
            ok: false, status: res.status,
            error: (data && data.error) ? data.error : { code: "HTTP_" + res.status, message: res.statusText || "failed" },
          };
        }
        let payload = data;
        if (data && typeof data === "object" && "ok" in data && "data" in data && Object.keys(data).length <= 4) {
          payload = data.data;
        }
        if (useCache && cacheKey && payload !== null) cacheSet(cacheKey, payload);
        return { ok: true, status: res.status, data: payload, cached: false };
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

  const API = {
    load: function (docId) { return get("/api/docs/load?doc_id=" + encodeURIComponent(docId)); },
    save: function (docId, state) { return post("/api/docs/save", { doc_id: docId, state: state }); },
    autosave: function (docId, state) { return post("/api/docs/autosave", { doc_id: docId, state: state }); },
    list: function () { return get("/api/docs/list"); },
    deleteDoc: function (docId) { return post("/api/docs/delete", { doc_id: docId }); },
    search: function (q) { return get("/api/docs/search?q=" + encodeURIComponent(q)); },
    newDoc: function (template) {
      const t = template ? "?template=" + encodeURIComponent(template) : "";
      return get("/api/docs/new" + t);
    },
    exportPdf: function (docId) { return get("/api/docs/export/pdf?doc_id=" + encodeURIComponent(docId)); },
    exportDocx: function (docId) { return get("/api/docs/export/docx?doc_id=" + encodeURIComponent(docId)); },
    exportMd: function (docId) { return get("/api/docs/export/md?doc_id=" + encodeURIComponent(docId)); },
    exportHtml: function (docId) { return get("/api/docs/export/html?doc_id=" + encodeURIComponent(docId)); },
    exportTxt: function (docId) { return get("/api/docs/export/txt?doc_id=" + encodeURIComponent(docId)); },
    importDoc: function (file) {
      const init = {
        method: "POST",
        headers: { "Accept": "application/json" },
        body: file,
        credentials: "same-origin",
      };
      const csrf = getCsrfToken();
      if (csrf) init.headers["X-CSRF-Token"] = csrf;
      return fetch("/api/docs/import", init).then(function (r) {
        return r.json().then(function (d) {
          return { ok: r.ok, status: r.status, data: d };
        });
      });
    },
    addComment: function (docId, ref, text) { return post("/api/docs/comment", { doc_id: docId, ref: ref, text: text }); },
    replyComment: function (commentId, text) { return post("/api/docs/comment/reply", { comment_id: commentId, text: text }); },
    resolveComment: function (commentId) { return post("/api/docs/comment/resolve", { comment_id: commentId }); },
    deleteComment: function (commentId) { return post("/api/docs/comment/delete", { comment_id: commentId }); },
    listComments: function (docId) { return get("/api/docs/comments?doc_id=" + encodeURIComponent(docId)); },
    generateToc: function (docId) { return post("/api/docs/toc/generate", { doc_id: docId }); },
    updateToc: function (docId) { return post("/api/docs/toc/update", { doc_id: docId }); },
    addFootnote: function (docId, ref, text) { return post("/api/docs/footnote", { doc_id: docId, ref: ref, text: text }); },
    addEndnote: function (docId, ref, text) { return post("/api/docs/endnote", { doc_id: docId, ref: ref, text: text }); },
    applyStyle: function (docId, ref, style) { return post("/api/docs/style/apply", { doc_id: docId, ref: ref, style: style }); },
    listStyles: function () { return get("/api/docs/styles"); },
    getOutline: function (docId) { return post("/api/docs/outline", { doc_id: docId }); },
    enableTrackChanges: function (docId) { return post("/api/docs/track-changes/enable", { doc_id: docId }); },
    aiSummarize: function (docId) { return post("/api/docs/ai/summarize", { doc_id: docId }); },
    aiExpand: function (docId, ref) { return post("/api/docs/ai/expand", { doc_id: docId, ref: ref }); },
    aiImprove: function (docId, ref) { return post("/api/docs/ai/improve", { doc_id: docId, ref: ref }); },
    aiTranslate: function (docId, targetLang) { return post("/api/docs/ai/translate", { doc_id: docId, target_lang: targetLang }); },
    cacheClear: cacheClear,
  };

  window.DocsAPI = API;
})();
