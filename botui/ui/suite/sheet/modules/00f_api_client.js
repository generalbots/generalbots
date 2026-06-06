// botui/ui/suite/sheet/modules/00f_api_client.js
// Thin REST client for /api/sheet/* endpoints on botserver (port 8080).
// Replaces local domain logic with server-authoritative calls.
// All other domain modules (27_named_ranges, 28_pivot_table, etc.) use
// this client instead of doing computation in the browser.
//
// Design:
// - One method per endpoint.
// - Returns { ok, status, data, error } shaped responses.
// - Retry with exponential backoff on 5xx and network errors.
// - Configurable timeout (default 8000ms).
// - Reads CSRF token from <meta name="csrf-token"> or gb.csrf cookie.
// - credentials: 'same-origin' always.
//
// All callers should treat this as the single entry point for sheet
// data ops. Do not call fetch() directly from feature modules.
"use strict";

(function () {
  const DEFAULT_TIMEOUT_MS = 8000;
  const MAX_RETRIES = 3;
  const RETRY_BASE_MS = 250;
  const CACHE_TTL_MS = 5000;

  const cache = new Map();

  function getCsrfToken() {
    const meta = document.querySelector('meta[name="csrf-token"]');
    if (meta) {
      const v = meta.getAttribute("content");
      if (v) return v;
    }
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

  function cacheKey(method, url, body) {
    if (method !== "GET") return null;
    return method + " " + url + (body ? " " + JSON.stringify(body) : "");
  }

  function cacheGet(key) {
    if (!key) return null;
    const entry = cache.get(key);
    if (!entry) return null;
    if (Date.now() - entry.ts > CACHE_TTL_MS) {
      cache.delete(key);
      return null;
    }
    return entry.value;
  }

  function cacheSet(key, value) {
    if (!key) return;
    cache.set(key, { ts: Date.now(), value: value });
  }

  function cacheClear(prefix) {
    if (!prefix) {
      cache.clear();
      return;
    }
    for (const k of Array.from(cache.keys())) {
      if (k.indexOf(prefix) === 0) cache.delete(k);
    }
  }

  function delay(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms); });
  }

  async function request(method, path, body, options) {
    const opts = options || {};
    const timeoutMs = opts.timeout || DEFAULT_TIMEOUT_MS;
    const useCache = opts.cache !== false;
    const url = path.startsWith("http") ? path : path;
    const key = cacheKey(method, url, body);
    if (useCache && method === "GET") {
      const cached = cacheGet(key);
      if (cached) return { ok: true, status: 200, data: cached, cached: true };
    }

    const headers = {
      "Content-Type": "application/json",
      "Accept": "application/json",
    };
    const csrf = getCsrfToken();
    if (csrf) headers["X-CSRF-Token"] = csrf;
    const auth = getAuthHeader();
    if (auth) headers["Authorization"] = auth;

    const init = {
      method: method,
      headers: headers,
      credentials: "same-origin",
    };
    if (body !== undefined && body !== null && method !== "GET") {
      init.body = JSON.stringify(body);
    }

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
        const res = await fetch(url, init);
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
            ok: false,
            status: res.status,
            error: (data && typeof data === "object" && data.error)
              ? data.error
              : { code: "HTTP_" + res.status, message: res.statusText || "request failed" },
          };
        }

        if (useCache && method === "GET" && data !== null) cacheSet(key, data);
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
    return { ok: false, status: 0, error: lastError || { code: "UNKNOWN", message: "unknown error" } };
  }

  function post(path, body, options) { return request("POST", path, body, options); }
  function get(path, options) { return request("GET", path, null, options); }
  function put(path, body, options) { return request("PUT", path, body, options); }
  function del(path, body, options) { return request("DELETE", path, body, options); }

  const API = {
    evaluate: function (sheetId, formula) {
      return post("/api/sheet/formula", { sheet_id: sheetId, formula: formula });
    },
    updateCell: function (sheetId, ref, value) {
      return post("/api/sheet/cell", { sheet_id: sheetId, ref: ref, value: value });
    },
    listNamedRanges: function (sheetId) {
      return get("/api/sheet/named-ranges?sheet_id=" + encodeURIComponent(sheetId));
    },
    createNamedRange: function (sheetId, name, range, description) {
      return post("/api/sheet/named-range", {
        sheet_id: sheetId, name: name, range: range, description: description || "",
      });
    },
    updateNamedRange: function (id, range, description) {
      return post("/api/sheet/named-range/update", { id: id, range: range, description: description || "" });
    },
    deleteNamedRange: function (id) {
      return post("/api/sheet/named-range/delete", { id: id });
    },
    filter: function (sheetId, range, criteria) {
      return post("/api/sheet/filter", { sheet_id: sheetId, range: range, criteria: criteria });
    },
    clearFilter: function (sheetId) {
      return post("/api/sheet/filter/clear", { sheet_id: sheetId });
    },
    validate: function (sheetId, ref, rule) {
      return post("/api/sheet/data-validation", { sheet_id: sheetId, ref: ref, rule: rule });
    },
    validateCell: function (sheetId, ref, value) {
      return post("/api/sheet/validate-cell", { sheet_id: sheetId, ref: ref, value: value });
    },
    exportSheet: function (sheetId, format) {
      return post("/api/sheet/export", { sheet_id: sheetId, format: format || "pdf" });
    },
    formatCells: function (sheetId, refs, format) {
      return post("/api/sheet/format", { sheet_id: sheetId, refs: refs, format: format });
    },
    sortRange: function (sheetId, range, key, dir) {
      return post("/api/sheet/sort", { sheet_id: sheetId, range: range, key: key, dir: dir || "asc" });
    },
    mergeCells: function (sheetId, range) {
      return post("/api/sheet/merge", { sheet_id: sheetId, range: range });
    },
    unmergeCells: function (sheetId, range) {
      return post("/api/sheet/unmerge", { sheet_id: sheetId, range: range });
    },
    freezePanes: function (sheetId, row, col) {
      return post("/api/sheet/freeze", { sheet_id: sheetId, row: row, col: col });
    },
    createChart: function (sheetId, config) {
      return post("/api/sheet/chart", { sheet_id: sheetId, config: config });
    },
    deleteChart: function (sheetId, chartId) {
      return post("/api/sheet/chart/delete", { sheet_id: sheetId, chart_id: chartId });
    },
    conditionalFormat: function (sheetId, range, rule) {
      return post("/api/sheet/conditional-format", { sheet_id: sheetId, range: range, rule: rule });
    },
    arrayFormula: function (sheetId, ref, formula) {
      return post("/api/sheet/array-formula", { sheet_id: sheetId, ref: ref, formula: formula });
    },
    createPivot: function (sheetId, config) {
      return post("/api/sheet/pivot", { sheet_id: sheetId, config: config });
    },
    load: function (sheetId) {
      return get("/api/sheet/load?sheet_id=" + encodeURIComponent(sheetId));
    },
    save: function (sheetId, state) {
      return post("/api/sheet/save", { sheet_id: sheetId, state: state });
    },
    list: function () {
      return get("/api/sheet/list");
    },
    cacheClear: cacheClear,
  };

  window.SheetAPI = API;
})();
