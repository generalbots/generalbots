// Minimal HTMX Application Initialization
// Pure HTMX-based with no external dependencies except HTMX itself

(function() {
    'use strict';

    // Configuration
    const config = {
        sessionRefreshInterval: 15 * 60 * 1000, // 15 minutes
        tokenKey: 'auth_token',
        themeKey: 'app_theme'
    };

    // Initialize HTMX settings
    function initHTMX() {
        // Configure HTMX
        htmx.config.defaultSwapStyle = 'innerHTML';
        htmx.config.defaultSettleDelay = 100;
        htmx.config.timeout = 10000;
        htmx.config.scrollBehavior = 'smooth';

        // Add authentication token to all requests
        document.body.addEventListener('htmx:configRequest', (event) => {
            // Get token from cookie (httpOnly cookies are automatically sent)
            // For additional security, we can also check localStorage
            const token = localStorage.getItem(config.tokenKey);
            if (token) {
                event.detail.headers['Authorization'] = `Bearer ${token}`;
            }
        });

        // Handle authentication errors
        document.body.addEventListener('htmx:responseError', (event) => {
            if (event.detail.xhr.status === 401) {
                // Unauthorized - redirect to login
                window.location.href = '/login?redirect=' + encodeURIComponent(window.location.pathname);
            } else if (event.detail.xhr.status === 403) {
                // Forbidden - show error
                showNotification('Access denied', 'error');
            }
        });

        // Handle successful responses
        document.body.addEventListener('htmx:afterSwap', (event) => {
            // Auto-initialize any new HTMX elements
            htmx.process(event.detail.target);

            // Trigger any custom events
            if (event.detail.target.dataset.afterSwap) {
                htmx.trigger(event.detail.target, event.detail.target.dataset.afterSwap);
            }
        });

        // Handle redirects
        document.body.addEventListener('htmx:beforeSwap', (event) => {
            if (event.detail.xhr.getResponseHeader('HX-Redirect')) {
                event.detail.shouldSwap = false;
                window.location.href = event.detail.xhr.getResponseHeader('HX-Redirect');
            }
        });
    }

    // Theme management
    function initTheme() {
        // Get saved theme or default to system preference
        const savedTheme = localStorage.getItem(config.themeKey) ||
                          (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');

        document.documentElement.setAttribute('data-theme', savedTheme);

        // Listen for theme changes
        document.body.addEventListener('theme-changed', (event) => {
            const newTheme = event.detail.theme ||
                           (document.documentElement.getAttribute('data-theme') === 'light' ? 'dark' : 'light');

            document.documentElement.setAttribute('data-theme', newTheme);
            localStorage.setItem(config.themeKey, newTheme);

            // Update theme icons
            document.querySelectorAll('[data-theme-icon]').forEach(icon => {
                icon.textContent = newTheme === 'light' ? '🌙' : '☀️';
            });
        });
    }

    // Session management
    function initSession() {
        // Check session validity on page load
        checkSession();

        // Periodically refresh token
        setInterval(refreshToken, config.sessionRefreshInterval);

        // Check session before page unload
        window.addEventListener('beforeunload', () => {
            // Save any pending data
            htmx.trigger(document.body, 'save-pending');
        });
    }

    // Check if user session is valid
    async function checkSession() {
        try {
            const response = await fetch('/api/auth/check');
            const data = await response.json();

            if (!data.authenticated && !isPublicPath()) {
                window.location.href = '/login?redirect=' + encodeURIComponent(window.location.pathname);
            }
        } catch (err) {
            console.error('Session check failed:', err);
        }
    }

    // Refresh authentication token
    async function refreshToken() {
        if (!isPublicPath()) {
            try {
                const response = await fetch('/api/auth/refresh', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });

                if (response.ok) {
                    const data = await response.json();
                    if (data.refreshed && data.token) {
                        localStorage.setItem(config.tokenKey, data.token);
                    }
                } else if (response.status === 401) {
                    // Token expired, redirect to login
                    window.location.href = '/login?redirect=' + encodeURIComponent(window.location.pathname);
                }
            } catch (err) {
                console.error('Token refresh failed:', err);
            }
        }
    }

    // Check if current path is public (doesn't require auth)
    function isPublicPath() {
        const publicPaths = ['/login', '/logout', '/auth/callback', '/health', '/register', '/forgot-password'];
        const currentPath = window.location.pathname;
        return publicPaths.some(path => currentPath.startsWith(path));
    }

    // Show notification
    function showNotification(message, type = 'info') {
        const container = document.getElementById('notifications') || createNotificationContainer();

        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <span class="notification-message">${escapeHtml(message)}</span>
            <button class="notification-close" onclick="this.parentElement.remove()">×</button>
        `;

        container.appendChild(notification);

        // Auto-dismiss after 5 seconds
        setTimeout(() => {
            notification.classList.add('fade-out');
            setTimeout(() => notification.remove(), 300);
        }, 5000);
    }

    // Create notification container if it doesn't exist
    function createNotificationContainer() {
        const container = document.createElement('div');
        container.id = 'notifications';
        container.className = 'notifications-container';
        document.body.appendChild(container);
        return container;
    }

    // Escape HTML to prevent XSS
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Handle keyboard shortcuts
    function initKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Ctrl/Cmd + K - Quick search
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                const searchInput = document.querySelector('[data-search-input]');
                if (searchInput) {
                    searchInput.focus();
                }
            }

            // Escape - Close modals
            if (e.key === 'Escape') {
                const modal = document.querySelector('.modal.active');
                if (modal) {
                    htmx.trigger(modal, 'close-modal');
                }
            }
        });
    }

    // Handle form validation
    function initFormValidation() {
        document.addEventListener('htmx:validateUrl', (event) => {
            // Custom URL validation if needed
            return true;
        });

        document.addEventListener('htmx:beforeRequest', (event) => {
            // Add loading state to forms
            const form = event.target.closest('form');
            if (form) {
                form.classList.add('loading');
                // Disable submit buttons
                form.querySelectorAll('[type="submit"]').forEach(btn => {
                    btn.disabled = true;
                });
            }
        });

        document.addEventListener('htmx:afterRequest', (event) => {
            // Remove loading state from forms
            const form = event.target.closest('form');
            if (form) {
                form.classList.remove('loading');
                // Re-enable submit buttons
                form.querySelectorAll('[type="submit"]').forEach(btn => {
                    btn.disabled = false;
                });
            }
        });
    }

    // Initialize offline detection
    function initOfflineDetection() {
        window.addEventListener('online', () => {
            document.body.classList.remove('offline');
            showNotification('Connection restored', 'success');
            // Retry any pending requests
            htmx.trigger(document.body, 'retry-pending');
        });

        window.addEventListener('offline', () => {
            document.body.classList.add('offline');
            showNotification('No internet connection', 'warning');
        });
    }

    // Main initialization
    function init() {
        console.log('Initializing HTMX application...');

        // Initialize core features
        initHTMX();
        initTheme();
        initSession();
        initKeyboardShortcuts();
        initFormValidation();
        initOfflineDetection();

        // Mark app as initialized
        document.body.classList.add('app-initialized');

        console.log('Application initialized successfully');
    }

    // Wait for DOM to be ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        // DOM is already ready
        init();
    }

    // Expose public API for other scripts if needed
    window.BotServerApp = {
        showNotification,
        checkSession,
        refreshToken,
        config
    };
})();
