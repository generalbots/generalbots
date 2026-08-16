"use strict";
/* GBOfflineDraft — client-side offline draft persistence + reconnect flush.
 *
 * No ServiceWorker needed: the app shell is served by botui, and this helper
 * only preserves *content* so an edit made while offline is never lost.
 *
 * Public API (window.GBOfflineDraft):
 *   save(key, content) -> bool   — persist a draft (localStorage, size-guarded)
 *   load(key) -> {t, c} | null   — read a draft back
 *   clear(key)                   — drop a draft
 *   has(key) -> bool             — is there a draft for this key?
 *   isOnline() -> bool           — navigator.onLine
 *   onReconnect(cb)              — run cb once when the browser comes back online
 *   showBanner(text)             — show the fixed offline banner
 *   hideBanner()                 — hide it
 *
 * Writes are wrapped in try/catch and a 4 MiB guard so a full browser never
 * throws; every call is safe to use unconditionally.
 */
(function (window) {
  var PREFIX = "gb-offline-draft:";
  var MAX_BYTES = 4 * 1024 * 1024;
  var banner = null;
  var bannerTimer = null;

  function store() {
    try { return window.localStorage; } catch (_) { return null; }
  }
  function key(k) { return PREFIX + k; }

  function save(k, content) {
    var s = store();
    if (!s) return false;
    if (typeof content !== "string" || content.length > MAX_BYTES) return false;
    try {
      s.setItem(key(k), JSON.stringify({ t: Date.now(), c: content }));
      return true;
    } catch (_) { return false; }
  }

  function load(k) {
    var s = store();
    if (!s) return null;
    try {
      var raw = s.getItem(key(k));
      if (!raw) return null;
      var obj = JSON.parse(raw);
      return obj && typeof obj.c === "string" ? obj : null;
    } catch (_) { return null; }
  }

  function clear(k) {
    var s = store();
    if (!s) return;
    try { s.removeItem(key(k)); } catch (_) {}
  }

  function has(k) { return load(k) !== null; }

  function isOnline() {
    try { return navigator.onLine !== false; } catch (_) { return true; }
  }

  function onReconnect(cb) {
    if (typeof cb !== "function") return;
    window.addEventListener("online", function handler() {
      window.removeEventListener("online", handler);
      cb();
    });
  }

  function ensureBanner() {
    if (banner && banner.parentNode) return banner;
    banner = document.createElement("div");
    banner.id = "gb-offline-banner";
    banner.setAttribute("role", "status");
    banner.style.cssText =
      "position:fixed;left:50%;bottom:18px;transform:translateX(-50%);z-index:100001;" +
      "background:#7c2d12;color:#fdba74;border:1px solid #9a3412;border-radius:999px;" +
      "padding:8px 18px;font-size:13px;font-weight:600;box-shadow:0 4px 16px rgba(0,0,0,.45);" +
      "font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;display:none;" +
      "align-items:center;gap:10px;white-space:nowrap;";
    document.body.appendChild(banner);
    return banner;
  }

  function showBanner(text, opts) {
    var o = opts || {};
    var b = ensureBanner();
    b.textContent = "";
    var label = document.createElement("span");
    label.textContent = text;
    b.appendChild(label);
    if (o.actionLabel && typeof o.onAction === "function") {
      var btn = document.createElement("button");
      btn.textContent = o.actionLabel;
      btn.style.cssText =
        "background:#fdba74;color:#7c2d12;border:none;border-radius:999px;padding:4px 12px;" +
        "font-size:12px;font-weight:700;cursor:pointer;";
      btn.addEventListener("click", function () { o.onAction(); });
      b.appendChild(btn);
    }
    b.style.display = "flex";
    if (!o.sticky) {
      clearTimeout(bannerTimer);
      bannerTimer = setTimeout(hideBanner, 4000);
    }
  }

  function hideBanner() {
    if (banner) banner.style.display = "none";
  }

  window.GBOfflineDraft = {
    save: save,
    load: load,
    clear: clear,
    has: has,
    isOnline: isOnline,
    onReconnect: onReconnect,
    showBanner: showBanner,
    hideBanner: hideBanner
  };
})(window);
