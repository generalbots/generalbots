/* GB Desktop offline shell (#1159).
 * Cache-first for the core desktop shell assets; network fallback for
 * everything else. Version bump invalidates the previous cache. */
var CACHE = "gb-desktop-shell-v1";
var CORE = [
  "/suite/desktop.html",
  "/suite/js/vendor/htmx.min.js",
  "/suite/js/security-bootstrap.js?v=2",
  "/suite/js/window-manager.js?v=13",
  "/suite/js/widget-registry.js?v=2",
  "/suite/js/widget-renderer.js?v=2",
  "/suite/js/sidebar.js?v=5",
  "/suite/css/desktop/widgets.css?v=1",
];

self.addEventListener("install", function (e) {
  e.waitUntil(caches.open(CACHE).then(function (c) { return c.addAll(CORE); }));
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
          caches.open(CACHE).then(function (c) { c.put(e.request, copy); });
        }
        return resp;
      });
    })
  );
});
