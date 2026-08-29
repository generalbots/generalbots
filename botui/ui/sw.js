/* GB Desktop offline shell (#1159).
 * Cache-first ONLY for the core desktop shell assets (re-cached on every
 * version bump); every other asset is network-first with the cache as an
 * offline fallback, so updated code reaches the browser immediately instead
 * of serving a stale cached copy forever (the old cache-first-on-everything
 * policy silently froze window-manager, partials and CSS after their first
 * fetch — "the browser is not loading the updated app"). Version bump
 * invalidates the previous cache. */
var CACHE = "gb-desktop-shell-v20";
var CORE = [
  "/suite/desktop.html",
  "/suite/js/vendor/htmx.min.js",
  "/suite/js/security-bootstrap.js?v=4",
  "/suite/js/window-manager.js?v=28",
  "/suite/js/widget-registry.js?v=2",
  "/suite/js/widget-renderer.js?v=2",
  "/suite/js/sidebar.js?v=7",
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

function isCore(url) {
  return CORE.some(function (entry) {
    var e = new URL(entry, self.location.origin);
    return e.pathname === url.pathname && e.search === url.search;
  });
}

self.addEventListener("fetch", function (e) {
  var url = new URL(e.request.url);
  if (url.pathname.startsWith("/api/") || e.request.method !== "GET") return;

  if (isCore(url)) {
    // Core shell: cache-first (these define the offline shell and are
    // re-cached fresh on every CACHE version bump).
    e.respondWith(
      caches.match(e.request).then(function (hit) {
        return hit || fetch(e.request);
      }).catch(function () {
        return fetch(e.request);
      })
    );
    return;
  }

  // Non-core /suite/ assets (app partials, CSS, JS): network-first so live
  // code always loads; the cache is a pure offline fallback.
  e.respondWith(
    fetch(e.request)
      .then(function (resp) {
        if (resp.ok && url.pathname.startsWith("/suite/")) {
          var copy = resp.clone();
          caches.open(CACHE).then(function (c) { return c.put(e.request, copy); })
            .catch(function () { /* cache write failures must never break the response */ });
        }
        return resp;
      })
      .catch(function () {
        return caches.match(e.request).then(function (hit) {
          if (hit) return hit;
          if (e.request.mode === "navigate") {
            return caches.match("/suite/desktop.html").then(function (fb) {
              return fb || new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
            });
          }
          return new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
        });
      })
  );
});
