/* GB Desktop offline shell (#1159).
 * Cache-first ONLY for the core desktop shell assets (re-cached on every
 * version bump); every other asset is network-first with the cache as an
 * offline fallback, so updated code reaches the browser immediately instead
 * of serving a stale cached copy forever (the old cache-first-on-everything
 * policy silently froze window-manager, partials and CSS after their first
 * fetch — "the browser is not loading the updated app"). Version bump
 * invalidates the previous cache. */
var CACHE = "gb-desktop-shell-v46";
var CORE = [
  "/suite/desktop.html",
  // Boot-critical inline layout partials (first paint, auth gate, base layout).
  "/suite/desktop-inline/01_boot_flags.js",
  "/suite/desktop-inline/01_auth_gate.css",
  "/suite/desktop-inline/02_public_flag.js",
  "/suite/desktop-inline/02_base_layout.css",
  "/suite/desktop-inline/03_bbsia_mobile.css",
  "/suite/desktop-inline/03_boot_splash.js",
  "/suite/desktop-inline/04_boot_splash.css",
  "/suite/desktop-inline/04_auth_fullscreen_a.js",
  "/suite/desktop-inline/04_auth_fullscreen_b.js",
  "/suite/desktop-inline/05_offline_shell.js",
  "/suite/js/vendor/htmx.min.js",
  "/suite/js/security-bootstrap/01_tokens.js?v=6",
  "/suite/js/security-bootstrap/02_interceptors.js?v=6",
  "/suite/js/security-bootstrap/99_init.js?v=6",
  // Window manager is a split module set (the old single window-manager.js is gone).
  "/suite/js/window-manager/01_registry.js?v=131",
  "/suite/js/window-manager/02_class_core.js?v=131",
  "/suite/js/window-manager/03_window_chrome.js?v=131",
  "/suite/js/window-manager/04_window_state.js?v=131",
  "/suite/js/window-manager/05_start_menu.js?v=131",
  "/suite/js/window-manager/06_deeplinks.js?v=131",
  "/suite/js/window-manager/07_status_a_core.js?v=133",
  "/suite/js/window-manager/07_status_b_render.js?v=133",
  "/suite/js/window-manager/07_status_c_events.js?v=133",
  "/suite/js/window-manager/08_fullscreen.js?v=1",
  "/suite/js/window-manager/99_init.js?v=131",
  "/suite/js/widget-registry.js?v=2",
  "/suite/js/widget-renderer.js?v=2",
  "/suite/js/sidebar_a_core.js?v=19",
  "/suite/js/sidebar_b_events.js?v=19",
  "/suite/js/vibe-bridge.js?v=4",
  // Theme/identity foundation (needed so the offline shell renders correctly).
  "/suite/identity/fonts.css",
  "/suite/identity/tokens.css",
  "/suite/css/theme-sentient.css",
  "/suite/css/config-colors.css",
  "/suite/css/app-identity.css?v=1",
  // Desktop chrome styles.
  "/suite/css/desktop/widgets.css?v=1",
  "/suite/css/desktop/windows.css?v=15",
  "/suite/css/desktop/taskbar.css?v=4",
  "/suite/css/desktop/sidebar.css?v=5",
  "/suite/css/desktop/start-menu.css?v=1",
  "/suite/css/base.css",
  "/suite/css/app.css",
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

var SHARE_CACHE = "gb-share-inbox-v1";
var SHARE_META_KEY = "/__gb_share__/meta";
var SHARE_FILE_PREFIX = "/__gb_share__/file/";

async function storeSharedPayload(request) {
  try {
    var form = await request.formData();
    var files = form.getAll("files");
    var meta = {
      title: form.get("title") || "",
      text: form.get("text") || "",
      url: form.get("url") || "",
      ts: Date.now(),
      files: []
    };
    var cache = await caches.open(SHARE_CACHE);
    var existing = await cache.keys();
    await Promise.all(existing.map(function (req) { return cache.delete(req); }));
    for (var i = 0; i < files.length; i++) {
      var f = files[i];
      if (!f || typeof f.name === "undefined") continue;
      var key = SHARE_FILE_PREFIX + i;
      meta.files.push({ key: key, name: f.name, type: f.type || "application/octet-stream", size: f.size || 0 });
      await cache.put(new Request(key), new Response(f, { headers: { "Content-Type": f.type || "application/octet-stream" } }));
    }
    await cache.put(new Request(SHARE_META_KEY), new Response(JSON.stringify(meta), { headers: { "Content-Type": "application/json" } }));
    return Response.redirect("/suite/drive/share-intake.html?shared=1", 303);
  } catch (err) {
    return Response.redirect("/suite/drive/share-intake.html?shared=0", 303);
  }
}

self.addEventListener("fetch", function (e) {
  var url = new URL(e.request.url);

  // #2510 — Web Share Target (POST multipart) for files shared into the PWA.
  if (e.request.method === "POST" && url.pathname === "/share-intake") {
    e.respondWith(storeSharedPayload(e.request));
    return;
  }

  if (url.pathname.startsWith("/api/") || e.request.method !== "GET") return;
  // Inside the desktop application the pages come from the application
  // itself and are always available. A fetch from this worker cannot reach
  // them, so its offline fallback answered the splash and the setup pages
  // with the cached desktop shell. Stay out of the way there.
  if (self.location.hostname === "tauri.localhost" || self.location.protocol === "tauri:") return;

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
