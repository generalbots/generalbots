// HTMX-based application initialization
(function() {
    'use strict';

    // Configuration
    const config = {
        wsUrl: '/ws',
        apiBase: '/api',
        reconnectDelay: 3000,
        maxReconnectAttempts: 5
    };

    // State
    let reconnectAttempts = 0;
    let wsConnection = null;

    // Initialize HTMX extensions
    function initHTMX() {
        // Configure HTMX
        htmx.config.defaultSwapStyle = 'innerHTML';
        htmx.config.defaultSettleDelay = 100;
        htmx.config.timeout = 10000;

        // Add CSRF token to all requests if available
        document.body.addEventListener('htmx:configRequest', (event) => {
            const token = localStorage.getItem('csrf_token');
            if (token) {
                event.detail.headers['X-CSRF-Token'] = token;
            }
        });

        // Handle errors globally
        document.body.addEventListener('htmx:responseError', (event) => {
            console.error('HTMX Error:', event.detail);
            showNotification('Connection error. Please try again.', 'error');
        });

        // Handle successful swaps
        document.body.addEventListener('htmx:afterSwap', (event) => {
            // Auto-scroll messages if in chat
            const messages = document.getElementById('messages');
            if (messages && event.detail.target === messages) {
                messages.scrollTop = messages.scrollHeight;
            }
        });

        // Handle WebSocket messages
        document.body.addEventListener('htmx:wsMessage', (event) => {
            handleWebSocketMessage(JSON.parse(event.detail.message));
        });

        // Handle WebSocket connection events
        document.body.addEventListener('htmx:wsConnecting', () => {
            updateConnectionStatus('connecting');
        });

        document.body.addEventListener('htmx:wsOpen', () => {
            updateConnectionStatus('connected');
            reconnectAttempts = 0;
        });

        document.body.addEventListener('htmx:wsClose', () => {
            updateConnectionStatus('disconnected');
            attemptReconnect();
        });
    }

    // Handle WebSocket messages
    function handleWebSocketMessage(message) {
        switch(message.type) {
            case 'message':
                appendMessage(message);
                break;
            case 'notification':
                showNotification(message.text, message.severity);
                break;
            case 'status':
                updateStatus(message);
                break;
            case 'suggestion':
                addSuggestion(message.text);
                break;
            default:
                console.log('Unknown message type:', message.type);
        }
    }

    // Append message to chat
    function appendMessage(message) {
        const messagesEl = document.getElementById('messages');
        if (!messagesEl) return;

        const messageEl = document.createElement('div');
        messageEl.className = `message ${message.sender === 'user' ? 'user' : 'bot'}`;
        messageEl.innerHTML = `
            <div class="message-content">
                <span class="sender">${message.sender}</span>
                <span class="text">${escapeHtml(message.text)}</span>
                <span class="time">${formatTime(message.timestamp)}</span>
            </div>
        `;

        messagesEl.appendChild(messageEl);
        messagesEl.scrollTop = messagesEl.scrollHeight;
    }

    // Add suggestion chip
    function addSuggestion(text) {
        const suggestionsEl = document.getElementById('suggestions');
        if (!suggestionsEl) return;

        const chip = document.createElement('button');
        chip.className = 'suggestion-chip';
        chip.textContent = text;
        chip.setAttribute('hx-post', '/api/sessions/current/message');
        chip.setAttribute('hx-vals', JSON.stringify({content: text}));
        chip.setAttribute('hx-target', '#messages');
        chip.setAttribute('hx-swap', 'beforeend');

        suggestionsEl.appendChild(chip);
        htmx.process(chip);
    }

    // Update connection status
    function updateConnectionStatus(status) {
        const statusEl = document.getElementById('connectionStatus');
        if (!statusEl) return;

        statusEl.className = `connection-status ${status}`;
        statusEl.textContent = status.charAt(0).toUpperCase() + status.slice(1);
    }

    // Update general status
    function updateStatus(message) {
        const statusEl = document.getElementById('status-' + message.id);
        if (statusEl) {
            statusEl.textContent = message.text;
            statusEl.className = `status ${message.severity}`;
        }
    }

    // Show notification
    function showNotification(text, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = text;

        const container = document.getElementById('notifications') || document.body;
        container.appendChild(notification);

        setTimeout(() => {
            notification.classList.add('fade-out');
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }

    // Attempt to reconnect WebSocket
    function attemptReconnect() {
        if (reconnectAttempts >= config.maxReconnectAttempts) {
            showNotification('Connection lost. Please refresh the page.', 'error');
            return;
        }

        reconnectAttempts++;
        setTimeout(() => {
            console.log(`Reconnection attempt ${reconnectAttempts}...`);
            htmx.trigger(document.body, 'htmx:wsReconnect');
        }, config.reconnectDelay);
    }

    // Utility: Escape HTML
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Utility: Format timestamp
    function formatTime(timestamp) {
        if (!timestamp) return '';
        const date = new Date(timestamp);
        return date.toLocaleTimeString('en-US', {
            hour: 'numeric',
            minute: '2-digit',
            hour12: true
        });
    }

    // Handle navigation
    function initNavigation() {
        // Update active nav item on page change
        document.addEventListener('htmx:pushedIntoHistory', (event) => {
            const path = event.detail.path;
            updateActiveNav(path);
        });

        // Handle browser back/forward
        window.addEventListener('popstate', (event) => {
            updateActiveNav(window.location.pathname);
        });
    }

    // Update active navigation item
    function updateActiveNav(path) {
        document.querySelectorAll('.nav-item, .app-item').forEach(item => {
            const href = item.getAttribute('href');
            if (href === path || (path === '/' && href === '/chat')) {
                item.classList.add('active');
            } else {
                item.classList.remove('active');
            }
        });
    }

    // Initialize keyboard shortcuts
    function initKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Send message on Enter (when in input)
            if (e.key === 'Enter' && !e.shiftKey) {
                const input = document.getElementById('messageInput');
                if (input && document.activeElement === input) {
                    e.preventDefault();
                    const form = input.closest('form');
                    if (form) {
                        htmx.trigger(form, 'submit');
                    }
                }
            }

            // Focus input on /
            if (e.key === '/' && document.activeElement.tagName !== 'INPUT') {
                e.preventDefault();
                const input = document.getElementById('messageInput');
                if (input) input.focus();
            }

            // Escape to blur input
            if (e.key === 'Escape') {
                const input = document.getElementById('messageInput');
                if (input && document.activeElement === input) {
                    input.blur();
                }
            }
        });
    }

    // Initialize scroll behavior
    function initScrollBehavior() {
        const scrollBtn = document.getElementById('scrollToBottom');
        const messages = document.getElementById('messages');

        if (scrollBtn && messages) {
            // Show/hide scroll button
            messages.addEventListener('scroll', () => {
                const isAtBottom = messages.scrollHeight - messages.scrollTop <= messages.clientHeight + 100;
                scrollBtn.style.display = isAtBottom ? 'none' : 'flex';
            });

            // Scroll to bottom on click
            scrollBtn.addEventListener('click', () => {
                messages.scrollTo({
                    top: messages.scrollHeight,
                    behavior: 'smooth'
                });
            });
        }
    }

    // Initialize theme if ThemeManager exists
    function initTheme() {
        if (window.ThemeManager) {
            ThemeManager.init();
        }
    }

    // Main initialization
    function init() {
        console.log('Initializing HTMX application...');

        // Initialize HTMX
        initHTMX();

        // Initialize navigation
        initNavigation();

        // Initialize keyboard shortcuts
        initKeyboardShortcuts();

        // Initialize scroll behavior
        initScrollBehavior();

        // Initialize theme
        initTheme();

        // Set initial active nav
        updateActiveNav(window.location.pathname);

        console.log('HTMX application initialized');
    }

    // Wait for DOM and HTMX to be ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose public API
    window.BotServerApp = {
        showNotification,
        appendMessage,
        updateConnectionStatus,
        config
    };
})();
