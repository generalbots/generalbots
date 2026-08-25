/**
 * Attendant Module JavaScript
 * Human agent interface for live chat support
 */

if (window.GBAppLifecycle) GBAppLifecycle.begin("attendant");
(function() {
    'use strict';

    // State
    const state = {
        activeConversation: null,
        quickReplies: [],
        typing: false
    };

    // DOM Elements
    const elements = {
        queueList: document.querySelector('.queue-list'),
        conversationArea: document.querySelector('.conversation-area'),
        conversationMessages: document.querySelector('.conversation-messages'),
        messageInput: document.querySelector('.message-input'),
        userPanel: document.querySelector('.user-panel')
    };

    /**
     * Initialize attendant module
     */
    function init() {
        setupQueueHandlers();
        setupMessageHandlers();
        setupKeyboardShortcuts();
        setupQuickReplies();
        setupWebSocket();
    }

    /**
     * Setup queue item click handlers
     */
    function setupQueueHandlers() {
        if (!elements.queueList) return;

        elements.queueList.addEventListener('click', function(e) {
            const queueItem = e.target.closest('.queue-item');
            if (!queueItem) return;

            // Update active state
            document.querySelectorAll('.queue-item').forEach(item => {
                item.classList.remove('active');
            });
            queueItem.classList.add('active');

            // Load conversation
            const conversationId = queueItem.dataset.conversationId;
            if (conversationId) {
                loadConversation(conversationId);
            }
        });
    }

    /**
     * Setup message input handlers
     */
    function setupMessageHandlers() {
        const input = elements.messageInput;
        if (!input) return;

        // Handle Enter to send
        input.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
            }
        });

        // Auto-resize textarea
        input.addEventListener('input', function() {
            this.style.height = 'auto';
            this.style.height = Math.min(this.scrollHeight, 120) + 'px';
        });

        // Send button
        const sendBtn = document.querySelector('.send-btn');
        if (sendBtn) {
            sendBtn.addEventListener('click', sendMessage);
        }
    }

    /**
     * Setup keyboard shortcuts
     */
    function setupKeyboardShortcuts() {
        document.addEventListener('keydown', function(e) {
            // Ctrl+Enter to send
            if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                sendMessage();
                return;
            }

            // Escape to close panels
            if (e.key === 'Escape') {
                closeModals();
            }

            // Ctrl+K to focus search
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                const searchInput = document.querySelector('.queue-search input');
                if (searchInput) searchInput.focus();
            }

            // Number keys (1-9) for quick replies
            if (e.altKey && e.key >= '1' && e.key <= '9') {
                const index = parseInt(e.key) - 1;
                const quickReply = document.querySelectorAll('.quick-reply')[index];
                if (quickReply) {
                    e.preventDefault();
                    insertQuickReply(quickReply.textContent);
                }
            }
        });
    }

    /**
     * Setup quick reply buttons
     */
    function setupQuickReplies() {
        document.querySelectorAll('.quick-reply').forEach(btn => {
            btn.addEventListener('click', function() {
                insertQuickReply(this.textContent);
            });
        });
    }

    /**
     * Insert quick reply into message input
     */
    function insertQuickReply(text) {
        const input = elements.messageInput;
        if (!input) return;

        input.value = text;
        input.focus();
        input.style.height = 'auto';
        input.style.height = Math.min(input.scrollHeight, 120) + 'px';
    }

    /**
     * Load conversation by ID
     */
    function loadConversation(conversationId) {
        state.activeConversation = conversationId;

        // HTMX will handle the actual loading
        // This is for any additional state management
        updateUserPanel(conversationId);
    }

    /**
     * Update user info panel
     */
    function updateUserPanel(conversationId) {
        // User panel is updated via HTMX
        // Add any additional logic here
    }

    /**
     * Send message
     */
    function sendMessage() {
        const input = elements.messageInput;
        if (!input || !input.value.trim()) return;

        const message = input.value.trim();

        // Add message to UI immediately (optimistic update)
        appendMessage({
            type: 'agent',
            content: message,
            time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
        });

        // Clear input
        input.value = '';
        input.style.height = 'auto';

        // Send via HTMX or WebSocket
        if (window.attendantSocket && window.attendantSocket.readyState === WebSocket.OPEN) {
            window.attendantSocket.send(JSON.stringify({
                type: 'message',
                conversationId: state.activeConversation,
                content: message
            }));
        }
    }

    /**
     * Append message to conversation
     */
    function appendMessage(msg) {
        const container = elements.conversationMessages;
        if (!container) return;

        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${msg.type}`;
        messageDiv.innerHTML = `
            <div class="message-bubble">${escapeHtml(msg.content)}</div>
            <div class="message-time">${msg.time}</div>
        `;

        container.appendChild(messageDiv);
        scrollToBottom();
    }

    /**
     * Scroll messages to bottom
     */
    function scrollToBottom() {
        const container = elements.conversationMessages;
        if (container) {
            container.scrollTop = container.scrollHeight;
        }
    }

    /**
     * Setup WebSocket connection
     */
    function setupWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        // Backend route lives under /api/attendance/ws (botattendance crate);
        // /ws/attendant does not exist and fails the upgrade with HTTP 400.
        const token = localStorage.getItem('gb-access-token') || '';
        const wsUrl = `${protocol}//${window.location.host}/api/attendance/ws?token=${encodeURIComponent(token)}`;

        try {
            window.attendantSocket = new WebSocket(wsUrl);

            window.attendantSocket.onopen = function() {
                console.log('Attendant WebSocket connected');
                updateConnectionStatus('online');
            };

            window.attendantSocket.onmessage = function(event) {
                handleWebSocketMessage(JSON.parse(event.data));
            };

            window.attendantSocket.onclose = function() {
                console.log('Attendant WebSocket disconnected');
                updateConnectionStatus('offline');
                // Attempt reconnection
                setTimeout(setupWebSocket, 5000);
            };

            window.attendantSocket.onerror = function(error) {
                console.error('WebSocket error:', error);
                updateConnectionStatus('error');
            };
        } catch (e) {
            console.warn('WebSocket not available:', e);
        }
    }

    /**
     * Handle incoming WebSocket messages
     */
    function handleWebSocketMessage(data) {
        switch (data.type) {
            case 'new_message':
                if (data.conversationId === state.activeConversation) {
                    appendMessage({
                        type: 'user',
                        content: data.content,
                        time: data.time || new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
                    });
                }
                // Update queue item preview
                updateQueuePreview(data.conversationId, data.content);
                break;

            case 'new_conversation':
                // Refresh queue list
                htmx.trigger('.queue-list', 'refresh');
                showNotification('New conversation', data.userName || 'New user');
                break;

            case 'typing':
                if (data.conversationId === state.activeConversation) {
                    showTypingIndicator(data.isTyping);
                }
                break;

            case 'conversation_closed':
                if (data.conversationId === state.activeConversation) {
                    showConversationClosed();
                }
                break;
        }
    }

    /**
     * Update queue item preview text
     */
    function updateQueuePreview(conversationId, text) {
        const item = document.querySelector(`.queue-item[data-conversation-id="${conversationId}"]`);
        if (item) {
            const preview = item.querySelector('.queue-preview');
            if (preview) {
                preview.textContent = text.substring(0, 50) + (text.length > 50 ? '...' : '');
            }
            // Update time
            const time = item.querySelector('.queue-time');
            if (time) {
                time.textContent = 'Just now';
            }
            // Add unread badge if not active
            if (conversationId !== state.activeConversation) {
                let badge = item.querySelector('.queue-badge');
                if (!badge) {
                    badge = document.createElement('span');
                    badge.className = 'queue-badge';
                    badge.textContent = '1';
                    item.appendChild(badge);
                } else {
                    badge.textContent = parseInt(badge.textContent || 0) + 1;
                }
            }
        }
    }

    /**
     * Show typing indicator
     */
    function showTypingIndicator(isTyping) {
        let indicator = document.querySelector('.typing-indicator');

        if (isTyping) {
            if (!indicator) {
                indicator = document.createElement('div');
                indicator.className = 'typing-indicator';
                indicator.innerHTML = `
                    <span class="typing-dot"></span>
                    <span class="typing-dot"></span>
                    <span class="typing-dot"></span>
                `;
                elements.conversationMessages?.appendChild(indicator);
            }
        } else if (indicator) {
            indicator.remove();
        }
    }

    /**
     * Show conversation closed message
     */
    function showConversationClosed() {
        const container = elements.conversationMessages;
        if (!container) return;

        const closedDiv = document.createElement('div');
        closedDiv.className = 'conversation-closed';
        closedDiv.textContent = 'This conversation has been closed';
        container.appendChild(closedDiv);

        // Disable input
        if (elements.messageInput) {
            elements.messageInput.disabled = true;
            elements.messageInput.placeholder = 'Conversation closed';
        }
    }

    /**
     * Update connection status indicator
     */
    function updateConnectionStatus(status) {
        const indicator = document.querySelector('.connection-status');
        if (indicator) {
            indicator.className = `connection-status ${status}`;
            indicator.title = status.charAt(0).toUpperCase() + status.slice(1);
        }
    }

    /**
     * Show browser notification
     */
    function showNotification(title, body) {
        if (!('Notification' in window)) return;

        if (Notification.permission === 'granted') {
            new Notification(title, { body, icon: '/icons/notification.png' });
        } else if (Notification.permission !== 'denied') {
            Notification.requestPermission().then(permission => {
                if (permission === 'granted') {
                    new Notification(title, { body, icon: '/icons/notification.png' });
                }
            });
        }
    }

    /**
     * Close all modals
     */
    function closeModals() {
        document.querySelectorAll('.modal').forEach(modal => {
            modal.classList.add('hidden');
        });
    }

    /**
     * Escape HTML to prevent XSS
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose for external use
    window.Attendant = {
        sendMessage,
        loadConversation,
        insertQuickReply
    };
})();

        
            // =====================================================================
            // Configuration
            // =====================================================================
            const API_BASE = window.location.origin;
            let currentSessionId = null;
            let currentAttendantId = null;
            let currentAttendantStatus = "online";
            let conversations = [];
            let attendants = [];
            let ws = null;
            let reconnectAttempts = 0;
            const MAX_RECONNECT_ATTEMPTS = 5;

            // LLM Assist configuration
            let llmAssistConfig = {
                tips_enabled: false,
                polish_enabled: false,
                smart_replies_enabled: false,
                auto_summary_enabled: false,
                sentiment_enabled: false
            };
            let conversationHistory = [];
