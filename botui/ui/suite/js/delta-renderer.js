"use strict";
/* gb-doc-delta/v1 renderer helpers (#2172, #2174).
   Apps call GBDocDelta.register(appId, handler) where handler receives
   { file, rev, ops } and applies them to the live app model. Unknown ops
   must be ignored. The shared `applyBasic` helpers cover common cases. */
(function () {
  var handlers = {};
  var revByFile = {};
  // A batch can arrive while the app module is still loading (deep link opens
  // the window, the delta follows immediately). Dropping it lost the first
  // batch forever, so hold it per app and replay on register.
  var pending = {};
  var PENDING_MAX = 20;

  function register(appId, handler) {
    if (!appId || typeof handler !== "function") return;
    handlers[appId] = handler;
    var queued = pending[appId];
    if (!queued || !queued.length) return;
    delete pending[appId];
    queued.forEach(function (payload) { apply(payload); });
  }

  function apply(payload) {
    if (!payload || payload.v !== 1 || !Array.isArray(payload.ops)) return;
    var file = payload.file || "";
    var rev = Number(payload.rev || 0);
    if (file && rev && revByFile[file] && rev <= revByFile[file]) return; // stale
    var h = handlers[payload.app];
    if (!h) {
      // No renderer yet (the app module loads after the deep link): queue it
      // and do NOT record the rev. Keeping the rev unrecorded left the first
      // batch unapplied, so the app opened empty (#2189/#2191/#2186).
      var q = pending[payload.app] || (pending[payload.app] = []);
      q.push(payload);
      if (q.length > PENDING_MAX) q.shift();
      return;
    }
    // Only a delivered batch advances the revision watermark.
    if (file && rev) revByFile[file] = rev;
    try { h(payload); } catch (e) { console.warn("[gb-doc-delta] handler failed", e); }
    // Also dispatch a generic event so apps not pre-registered can react.
    document.dispatchEvent(new CustomEvent("gb-doc-delta:applied", { detail: payload }));
  }

  // Listen via the deep-link path: window-manager forwards gb-doc-delta by
  // reopening the app with params.delta (JSON). Apps may also listen directly.
  document.addEventListener("gb:deep-link", function (e) {
    var d = (e && e.detail) || {};
    var params = d.params || {};
    if (!params.delta) return;
    try { apply(typeof params.delta === "string" ? JSON.parse(params.delta) : params.delta); } catch (_) {}
  });
  document.addEventListener("gb-doc-delta", function (e) {
    apply((e && e.detail) || {});
  });

  window.GBDocDelta = { register: register, apply: apply, revByFile: revByFile };
})();
