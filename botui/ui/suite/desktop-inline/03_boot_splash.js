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
                // #1894 — the desktop is a native app, not a browser: the
                // WebView default context menu (Back / Reload / Inspect /
                // Copy Link) must never appear on the splash or the shell.
                // Intentional custom menus (simple-context-menu.js, desktop
                // wallpaper menu, sidebar sections) handle their own event and
                // stay untouched; text fields keep the native editing menu so
                // copy/paste still works. Debug builds keep it behind
                // gbDebugContextMenu for inspection.
                function suppressNativeContextMenu() {
                    document.addEventListener(
                        "contextmenu",
                        function (e) {
                            if (window.gbDebugContextMenu === true) return;
                            var t = e.target;
                            while (t && t !== document.body) {
                                var tag = (t.tagName || "").toUpperCase();
                                if (tag === "INPUT" || tag === "TEXTAREA" || t.isContentEditable) return;
                                t = t.parentElement;
                            }
                            e.preventDefault();
                        },
                        false
                    );
                }
                suppressNativeContextMenu();
                // #1830 — the shell hides the splash through this hook when
                // boot truly completes (default app opened), not on
                // window.load, which fires while apps are still fetching.
                window.gbHideBootSplash = hideBootSplash;
                window.addEventListener("load", hideBootSplash);
                // Safety net: an authentication redirect or a dead API must
                // never trap the user behind the in-page splash.
                setTimeout(hideBootSplash, 30000);
            })();
        
