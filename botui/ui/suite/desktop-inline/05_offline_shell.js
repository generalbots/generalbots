            // Offline shell (#1159): register the service worker after load.
            if ("serviceWorker" in navigator) {
                window.addEventListener("load", function () {
                    navigator.serviceWorker.register("/sw.js").catch(function () {});
                });
            }
        
