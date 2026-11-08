            (function () {
                function hideBootSplash() {
                    var splash = document.getElementById("gbBootSplash");
                    if (!splash || splash.dataset.gbHidden === "1") return;
                    splash.dataset.gbHidden = "1";
                    splash.classList.add("gb-boot-hidden");
                    setTimeout(function () {
                        if (splash.parentNode) splash.parentNode.removeChild(splash);
                    }, 400);
                }
                // #1830 — the shell hides the splash through this hook when
                // boot truly completes (default app opened), not on
                // window.load, which fires while apps are still fetching.
                window.gbHideBootSplash = hideBootSplash;
                window.addEventListener("load", hideBootSplash);
                // Safety net: an authentication redirect or a dead API must
                // never trap the user behind the in-page splash.
                setTimeout(hideBootSplash, 30000);
            })();
        
