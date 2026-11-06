    // Auto-open Chat window maximized on desktop load (only for certain bots)
    document.addEventListener("DOMContentLoaded", async () => {
        // Initialize WindowManager reference
        if (typeof window.WindowManager !== "undefined") {
            window.wm = window.WindowManager;
        }

        // Get bot name from URL path
        const pathParts = window.location.pathname.split('/');
        const botName = pathParts[1] || 'default';
        
        // Auto-open Chat window maximized for all bots. Cache-buster keeps
        // the injected chat scripts fresh (a heuristically-cached partial
        // would ship stale chat-init.js and miss deep-link re-targets).
            try {
                const response = await fetch("/suite/partials/chat.html?_=" + Date.now());
                if (response.ok) {
                    const htmlContent = await response.text();
                    window.wm.open("chat", "Chat", htmlContent);
                }
            } catch (err) {
                console.error("Failed to auto-open chat:", err);
            }
        }
    });

    // Simple Clock implementation matching the screenshot bottom right corner
    setInterval(() => {
        const now = new Date();
        const timeEl = document.getElementById("clock-time");
        const dateEl = document.getElementById("clock-date");
        if (timeEl)
            timeEl.textContent = now.toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
            });
        if (dateEl) dateEl.textContent = now.toLocaleDateString();
    }, 1000);
