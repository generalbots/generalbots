// Theme management
function initTheme() {
    const savedTheme = localStorage.getItem("theme") || "light";
    document.documentElement.setAttribute("data-theme", savedTheme);
    updateThemeIcon(savedTheme);
}

function toggleTheme() {
    const currentTheme =
        document.documentElement.getAttribute("data-theme");
    const newTheme = currentTheme === "light" ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", newTheme);
    localStorage.setItem("theme", newTheme);
    updateThemeIcon(newTheme);
}

function updateThemeIcon(theme) {
    const icon = document.getElementById("theme-icon");
    icon.textContent = theme === "light" ? "🌙" : "☀️";
}

// Check if in development mode
async function checkDevMode() {
    try {
        const response = await fetch("/api/auth/mode");
        const data = await response.json();
        if (data.mode === "development") {
            document.getElementById(
                "dev-mode-banner",
            ).style.display = "block";
            document.body.style.paddingTop = "2.5rem";
        }
    } catch (err) {
        // Ignore - assume production mode
    }
}

// Load enabled OAuth providers
async function loadOAuthProviders() {
    const loadingEl = document.getElementById("oauth-loading");
    const buttonsEl = document.getElementById("oauth-buttons");
    const noOAuthEl = document.getElementById("no-oauth");

    try {
        const response = await fetch("/auth/oauth/providers");
        const data = await response.json();

        loadingEl.style.display = "none";

        if (data.providers && data.providers.length > 0) {
            buttonsEl.style.display = "grid";

            // Show buttons for enabled providers
            data.providers.forEach((provider) => {
                const btn = document.getElementById(
                    "btn-" + provider.id,
                );
                if (btn) {
                    btn.classList.remove("hidden");
                }
            });

            // Check if any buttons are visible
            const visibleButtons =
                buttonsEl.querySelectorAll("a:not(.hidden)");
            if (visibleButtons.length === 0) {
                buttonsEl.style.display = "none";
                noOAuthEl.style.display = "block";
            }
        } else {
            noOAuthEl.style.display = "block";
        }
    } catch (err) {
        console.error("Failed to load OAuth providers:", err);
        loadingEl.style.display = "none";
        // Show all buttons as fallback (they'll redirect to error page if not configured)
        buttonsEl.style.display = "grid";
        buttonsEl.querySelectorAll("a").forEach((btn) => {
            btn.classList.remove("hidden");
        });
    }
}

// Initialize
document.addEventListener("DOMContentLoaded", () => {
    initTheme();
    checkDevMode();
    loadOAuthProviders();

    // Handle form validation
    const form = document.getElementById("login-form");
    form.addEventListener("submit", (e) => {
        const email = document.getElementById("email").value;
        const password = document.getElementById("password").value;

        if (!email || !password) {
            e.preventDefault();
            showError("Please fill in all fields");
            return false;
        }
    });
});

// Show error message
function showError(message) {
    const container = document.getElementById("message-container");
    container.innerHTML = `<div class="error-message">${message}</div>`;
}

// Handle HTMX events
document.body.addEventListener("htmx:afterRequest", (event) => {
    if (event.detail.xhr.status === 200) {
        // Check if we got a redirect header
        const redirect =
            event.detail.xhr.getResponseHeader("HX-Redirect");
        if (redirect) {
            window.location.href = redirect;
        }
    }
});

document.body.addEventListener("htmx:responseError", (event) => {
    showError(
        "Authentication failed. Please check your credentials.",
    );
});
