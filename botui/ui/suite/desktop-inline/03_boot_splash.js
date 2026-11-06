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
                window.addEventListener("load", hideBootSplash);
                setTimeout(hideBootSplash, 4000);
            })();
        
