            // Offline shell (#1159): register the service worker after load.
            // Not inside the desktop application: it is always local, and the
            // worker's offline fallback replaced the application's own pages
            // (the splash) with the cached desktop. Remove one left behind by
            // an earlier version instead.
            if ("serviceWorker" in navigator) {
                if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
                    navigator.serviceWorker.getRegistrations().then(function (registrations) {
                        registrations.forEach(function (registration) { registration.unregister(); });
                    }).catch(function (err) {
                        console.debug("[gb-desktop] Could not list service workers:", err);
                    });
                } else {
                    window.addEventListener("load", function () {
                        navigator.serviceWorker.register("/sw.js").catch(function (err) {
                            /* Best-effort offline shell registration; app remains usable online. */
                            console.debug("[gb-desktop] Service worker registration failed:", err);
                        });
                    });
                }
            }
