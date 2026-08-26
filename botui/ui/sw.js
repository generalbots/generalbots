/* GB Desktop offline shell (#1159).
 * Cache-first for the core desktop shell assets; network fallback for
 * everything else. Version bump invalidates the previous cache. */
var CACHE = "gb-desktop-shell-v12";
var CORE = [
  "/suite/desktop.html",
  "/suite/js/vendor/htmx.min.js",
  "/suite/js/security-bootstrap.js?v=4",
  "/suite/js/window-manager.js?v=20",
  "/suite/js/widget-registry.js?v=2",
  "/suite/js/widget-renderer.js?v=2",
  "/suite/js/sidebar.js?v=6",
  "/suite/css/desktop/widgets.css?v=1",
];

// Install must never fail wholesale: `c.addAll` rejects if ANY core URL
// 404s, which aborts the install, leaves a broken/stale worker active and
// spams the console with "FetchEvent ... network error". Cache each file
// individually and ignore per-file failures instead.
self.addEventListener("install", function (e) {
  e.waitUntil(
    caches.open(CACHE).then(function (c) {
      return Promise.all(
        CORE.map(function (url) {
          return fetch(url, { cache: "no-cache" })
            .then(function (resp) { return resp.ok ? c.put(url, resp) : null; })
            .catch(function () { return null; });
        })
      );
    })
  );
  self.skipWaiting();
});

self.addEventListener("activate", function (e) {
  e.waitUntil(
    caches.keys().then(function (keys) {
      return Promise.all(keys.filter(function (k) { return k !== CACHE; })
        .map(function (k) { return caches.delete(k); }));
    })
  );
  self.clients.claim();
});

self.addEventListener("fetch", function (e) {
  var url = new URL(e.request.url);
  if (url.pathname.startsWith("/api/") || e.request.method !== "GET") return;
  e.respondWith(
    caches.match(e.request).then(function (hit) {
      return hit || fetch(e.request).then(function (resp) {
        if (resp.ok && url.pathname.startsWith("/suite/")) {
          var copy = resp.clone();
          caches.open(CACHE).then(function (c) { return c.put(e.request, copy); })
            .catch(function () { /* cache write failures must never break the response */ });
        }
        return resp;
      }).catch(function () {
        // Network failed and nothing cached: serve an offline fallback so
        // respondWith never rejects (a rejected promise kills the page load).
        if (e.request.mode === "navigate") {
          return caches.match("/index.html").then(function (fb) {
            return fb || new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
          });
        }
        return new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
      });
    }).catch(function () {
      return new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
    })
  );
});
