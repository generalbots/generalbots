/**
 * Attendant Module JavaScript
 * Human agent interface for live chat support
 */
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
        const wsUrl = `${protocol}//${window.location.host}/ws/attendant`;

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

            // =====================================================================
            // Initialization
            // =====================================================================
            document.addEventListener("DOMContentLoaded", async () => {
                await checkCRMEnabled();
                setupEventListeners();
            });

            async function checkCRMEnabled() {
                // CRM is now enabled by default
                try {
                    const response = await fetch(
                        `${API_BASE}/api/attendance/attendants`,
                    );
                    const data = await response.json();

                    if (response.ok && Array.isArray(data)) {
                        attendants = data;
                        if (attendants.length > 0) {
                            // Set current attendant (first one for now, should come from auth)
                            currentAttendantId = attendants[0].attendant_id;
                            document.getElementById(
                                "attendantName",
                            ).textContent = attendants[0].attendant_name;
                        } else {
                            // No attendants configured, use default
                            document.getElementById(
                                "attendantName",
                            ).textContent = "Agent";
                        }
                    } else {
                        // API error, use default
                        document.getElementById(
                            "attendantName",
                        ).textContent = "Agent";
                    }

                    // Always load queue and connect WebSocket - CRM enabled by default
                    await loadQueue();
                    connectWebSocket();
                } catch (error) {
                    console.error("Failed to load attendants:", error);
                    // Still enable the console with default settings
                    document.getElementById("attendantName").textContent = "Agent";
                    await loadQueue();
                    connectWebSocket();
                }
            }

            function showCRMDisabled() {
                // Kept for backwards compatibility but no longer used by default
                document.getElementById("crmDisabled").classList.add("active");
                document.getElementById("crmDisabled").style.display = "flex";
                document.getElementById("mainLayout").style.display = "none";
            }

            function setupEventListeners() {
                // Chat input auto-resize
                const chatInput = document.getElementById("chatInput");
                chatInput.addEventListener("input", function () {
                    this.style.height = "auto";
                    this.style.height = Math.min(this.scrollHeight, 120) + "px";
                });

                // Send on Enter (without Shift)
                chatInput.addEventListener("keydown", (e) => {
                    if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        sendMessage();
                    }
                });

                // Close dropdown on outside click
                document.addEventListener("click", (e) => {
                    if (!e.target.closest("#attendantStatus")) {
                        document
                            .getElementById("statusDropdown")
                            .classList.remove("show");
                    }
                });
            }

            // =====================================================================
            // Queue Management
            // =====================================================================
            async function loadQueue() {
                try {
                    const response = await fetch(
                        `${API_BASE}/api/attendance/queue`,
                    );
                    if (response.ok) {
                        conversations = await response.json();
                        renderConversations();
                        updateStats();
                    }
                } catch (error) {
                    console.error("Failed to load queue:", error);
                    showToast("Failed to load queue", "error");
                }
            }

            function renderConversations() {
                const list = document.getElementById("conversationList");
                const emptyState = document.getElementById("emptyQueue");

                if (conversations.length === 0) {
                    emptyState.style.display = "flex";
                    return;
                }

                emptyState.style.display = "none";

                // Sort by priority and waiting time
                conversations.sort((a, b) => {
                    if (b.priority !== a.priority)
                        return b.priority - a.priority;
                    return b.waiting_time_seconds - a.waiting_time_seconds;
                });

                list.innerHTML =
                    conversations
                        .map(
                            (conv) => `
                <div class="conversation-item ${conv.session_id === currentSessionId ? "active" : ""} ${conv.status === "waiting" ? "unread" : ""}"
                     onclick="selectConversation('${conv.session_id}')"
                     data-session-id="${conv.session_id}">
                    <div class="conversation-header">
                        <span class="customer-name">${escapeHtml(conv.user_name || "Anonymous")}</span>
                        <span class="conversation-time">${formatTime(conv.last_message_time)}</span>
                    </div>
                    <div class="conversation-preview">${escapeHtml(conv.last_message || "No messages")}</div>
                    <div class="conversation-meta">
                        <span class="channel-tag channel-${conv.channel.toLowerCase()}">${conv.channel}</span>
                        ${conv.priority >= 2 ? `<span class="priority-tag priority-${conv.priority >= 3 ? "urgent" : "high"}">🔥 ${conv.priority >= 3 ? "Urgent" : "High"}</span>` : ""}
                        <span class="waiting-time ${conv.waiting_time_seconds > 300 ? "long" : ""}">${formatWaitTime(conv.waiting_time_seconds)}</span>
                    </div>
                </div>
            `,
                        )
                        .join("") +
                    `<div class="empty-queue" id="emptyQueue" style="display: none;">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M20 6L9 17l-5-5"/>
                    </svg>
                    <p>No conversations in queue</p>
                    <small>New conversations will appear here</small>
                </div>`;
            }

            function updateStats() {
                const waiting = conversations.filter(
                    (c) => c.status === "waiting",
                ).length;
                const active = conversations.filter(
                    (c) => c.status === "active",
                ).length;
                const resolved = conversations.filter(
                    (c) => c.status === "resolved",
                ).length;
                const mine = conversations.filter(
                    (c) => c.assigned_to === currentAttendantId,
                ).length;

                document.getElementById("waitingCount").textContent = waiting;
                document.getElementById("activeCount").textContent = active;
                document.getElementById("resolvedCount").textContent = resolved;

                document.getElementById("allBadge").textContent =
                    conversations.length;
                document.getElementById("waitingBadge").textContent = waiting;
                document.getElementById("mineBadge").textContent = mine;
            }

            function filterQueue(filter) {
                document.querySelectorAll(".filter-btn").forEach((btn) => {
                    btn.classList.toggle(
                        "active",
                        btn.dataset.filter === filter,
                    );
                });

                const items = document.querySelectorAll(".conversation-item");
                items.forEach((item) => {
                    const sessionId = item.dataset.sessionId;
                    const conv = conversations.find(
                        (c) => c.session_id === sessionId,
                    );
                    if (!conv) return;

                    let show = true;
                    switch (filter) {
                        case "waiting":
                            show = conv.status === "waiting";
                            break;
                        case "mine":
                            show = conv.assigned_to === currentAttendantId;
                            break;
                        case "high":
                            show = conv.priority >= 2;
                            break;
                    }
                    item.style.display = show ? "block" : "none";
                });
            }

            // =====================================================================
            // Conversation Selection & Chat
            // =====================================================================
            async function selectConversation(sessionId) {
                currentSessionId = sessionId;
                conversationHistory = []; // Reset history for new conversation
                const conv = conversations.find(
                    (c) => c.session_id === sessionId,
                );
                if (!conv) return;

                // Update UI
                document
                    .querySelectorAll(".conversation-item")
                    .forEach((item) => {
                        item.classList.toggle(
                            "active",
                            item.dataset.sessionId === sessionId,
                        );
                        if (item.dataset.sessionId === sessionId) {
                            item.classList.remove("unread");
                        }
                    });

                document.getElementById("noConversation").style.display =
                    "none";
                document.getElementById("activeChat").style.display = "flex";

                // Update header
                document.getElementById("customerAvatar").textContent =
                    (conv.user_name || "A")[0].toUpperCase();
                document.getElementById("customerName").textContent =
                    conv.user_name || "Anonymous";
                document.getElementById("customerChannel").textContent =
                    conv.channel;
                document.getElementById("customerChannel").className =
                    `channel-tag channel-${conv.channel.toLowerCase()}`;

                // Show customer details
                document.getElementById("customerDetails").style.display =
                    "block";
                document.getElementById("detailEmail").textContent =
                    conv.user_email || "-";

                // Load messages
                await loadMessages(sessionId);

                // Load AI insights
                await loadInsights(sessionId);

                // Assign to self if unassigned
                if (!conv.assigned_to && currentAttendantId) {
                    await assignConversation(sessionId, currentAttendantId);
                }
            }

            async function loadMessages(sessionId) {
                const container = document.getElementById("chatMessages");
                container.innerHTML = '<div class="loading-spinner"></div>';

                try {
                    // For now, show the last message from queue data
                    const conv = conversations.find(
                        (c) => c.session_id === sessionId,
                    );

                    // In real implementation, fetch from /api/sessions/{id}/messages
                    container.innerHTML = "";

                    if (conv && conv.last_message) {
                        addMessage(
                            "customer",
                            conv.last_message,
                            conv.last_message_time,
                        );
                    }

                    // Add system message for transfer
                    if (conv && conv.assigned_to_name) {
                        addSystemMessage(
                            `Assigned to ${conv.assigned_to_name}`,
                        );
                    }
                } catch (error) {
                    console.error("Failed to load messages:", error);
                    container.innerHTML =
                        '<p style="text-align: center; color: var(--text-muted);">Failed to load messages</p>';
                }
            }

            function addMessage(type, content, time = null) {
                const container = document.getElementById("chatMessages");
                const timeStr = time
                    ? formatTime(time)
                    : new Date().toLocaleTimeString([], {
                          hour: "2-digit",
                          minute: "2-digit",
                      });

                const avatarContent =
                    type === "customer" ? "C" : type === "bot" ? "🤖" : "You";
                const avatarClass = type === "bot" ? "bot" : "";

                const messageHtml = `
                <div class="message ${type}">
                    <div class="message-avatar ${avatarClass}">${avatarContent}</div>
                    <div class="message-content">
                        <div class="message-bubble">${escapeHtml(content)}</div>
                        <div class="message-meta">
                            <span>${timeStr}</span>
                            ${type === "bot" ? '<span class="bot-badge">Bot</span>' : ""}
                        </div>
                    </div>
                </div>
            `;

                container.insertAdjacentHTML("beforeend", messageHtml);
                container.scrollTop = container.scrollHeight;
            }

            function addSystemMessage(content) {
                const container = document.getElementById("chatMessages");
                const messageHtml = `
                <div class="message system">
                    <div class="message-content">
                        <div class="message-bubble">${escapeHtml(content)}</div>
                    </div>
                </div>
            `;
                container.insertAdjacentHTML("beforeend", messageHtml);
            }

            async function sendMessage() {
                const input = document.getElementById("chatInput");
                const message = input.value.trim();

                if (!message || !currentSessionId) return;

                input.value = "";
                input.style.height = "auto";

                // Add to UI immediately
                addMessage("attendant", message);

                // Add to conversation history
                conversationHistory.push({
                    role: "attendant",
                    content: message,
                    timestamp: new Date().toISOString()
                });

                try {
                    // Send to attendance respond API
                    const response = await fetch(
                        `${API_BASE}/api/attendance/respond`,
                        {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({
                                session_id: currentSessionId,
                                message: message,
                                attendant_id: currentAttendantId,
                            }),
                        },
                    );

                    const result = await response.json();
                    if (!result.success) {
                        throw new Error(
                            result.error || "Failed to send message",
                        );
                    }

                    showToast(result.message, "success");

                    // Refresh smart replies after sending
                    if (llmAssistConfig.smart_replies_enabled) {
                        loadSmartReplies(currentSessionId);
                    }
                } catch (error) {
                    console.error("Failed to send message:", error);
                    showToast(
                        "Failed to send message: " + error.message,
                        "error",
                    );
                }
            }

            function useQuickResponse(text) {
                document.getElementById("chatInput").value = text;
                document.getElementById("chatInput").focus();
            }

            function useSuggestion(element) {
                const text = element
                    .querySelector(".suggested-reply-text")
                    .textContent.trim();
                document.getElementById("chatInput").value = text;
                document.getElementById("chatInput").focus();
            }

            // =====================================================================
            // Transfer & Assignment
            // =====================================================================
            async function assignConversation(sessionId, attendantId) {
                try {
                    const response = await fetch(
                        `${API_BASE}/api/attendance/assign`,
                        {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({
                                session_id: sessionId,
                                attendant_id: attendantId,
                            }),
                        },
                    );

                    if (response.ok) {
                        showToast("Conversation assigned", "success");
                        await loadQueue();
                    }
                } catch (error) {
                    console.error("Failed to assign conversation:", error);
                }
            }

            function showTransferModal() {
                if (!currentSessionId) return;

                const list = document.getElementById("attendantList");
                list.innerHTML = attendants
                    .filter((a) => a.attendant_id !== currentAttendantId)
                    .map(
                        (a) => `
                    <div class="attendant-option" onclick="selectTransferTarget(this, '${a.attendant_id}')">
                        <div class="status-indicator ${a.status.toLowerCase()}"></div>
                        <div>
                            <div style="font-weight: 500;">${escapeHtml(a.attendant_name)}</div>
                            <div style="font-size: 12px; color: var(--text-secondary);">${a.preferences} • ${a.channel}</div>
                        </div>
                    </div>
                `,
                    )
                    .join("");

                document.getElementById("transferModal").classList.add("show");
            }

            function closeTransferModal() {
                document
                    .getElementById("transferModal")
                    .classList.remove("show");
                document.getElementById("transferReason").value = "";
            }

            let selectedTransferTarget = null;

            function selectTransferTarget(element, attendantId) {
                document
                    .querySelectorAll(".attendant-option")
                    .forEach((el) => el.classList.remove("selected"));
                element.classList.add("selected");
                selectedTransferTarget = attendantId;
            }

            async function confirmTransfer() {
                if (!selectedTransferTarget || !currentSessionId) {
                    showToast("Please select an attendant", "warning");
                    return;
                }

                const reason = document.getElementById("transferReason").value;

                try {
                    const response = await fetch(
                        `${API_BASE}/api/attendance/transfer`,
                        {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({
                                session_id: currentSessionId,
                                from_attendant_id: currentAttendantId,
                                to_attendant_id: selectedTransferTarget,
                                reason: reason,
                            }),
                        },
                    );

                    if (response.ok) {
                        showToast("Conversation transferred", "success");
                        closeTransferModal();
                        currentSessionId = null;
                        document.getElementById(
                            "noConversation",
                        ).style.display = "flex";
                        document.getElementById("activeChat").style.display =
                            "none";
                        await loadQueue();
                    } else {
                        throw new Error("Transfer failed");
                    }
                } catch (error) {
                    console.error("Failed to transfer:", error);
                    showToast("Failed to transfer conversation", "error");
                }
            }

            async function resolveConversation() {
                if (!currentSessionId) return;

                try {
                    const response = await fetch(
                        `${API_BASE}/api/attendance/resolve/${currentSessionId}`,
                        {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                        },
                    );

                    if (response.ok) {
                        showToast("Conversation resolved", "success");
                        currentSessionId = null;
                        document.getElementById(
                            "noConversation",
                        ).style.display = "flex";
                        document.getElementById("activeChat").style.display =
                            "none";
                        await loadQueue();
                    } else {
                        throw new Error("Failed to resolve");
                    }
                } catch (error) {
                    console.error("Failed to resolve:", error);
                    showToast("Failed to resolve conversation", "error");
                }
            }

            // =====================================================================
            // Status Management
            // =====================================================================
            function toggleStatusDropdown() {
                document
                    .getElementById("statusDropdown")
                    .classList.toggle("show");
            }

            async function setStatus(status) {
                currentAttendantStatus = status;
                document.getElementById("statusIndicator").className =
                    `status-indicator ${status}`;
                document
                    .getElementById("statusDropdown")
                    .classList.remove("show");

                const statusTexts = {
                    online: "Online - Ready for conversations",
                    busy: "Busy - Handling conversations",
                    away: "Away - Temporarily unavailable",
                    offline: "Offline - Not accepting conversations",
                };
                document.getElementById("statusText").textContent =
                    statusTexts[status];

                try {
                    await fetch(
                        `${API_BASE}/api/attendance/attendants/${currentAttendantId}/status`,
                        {
                            method: "PUT",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({ status: status }),
                        },
                    );
                } catch (error) {
                    console.error("Failed to update status:", error);
                }
            }

            // =====================================================================
            // AI Insights
            // =====================================================================
            async function loadInsights(sessionId) {
                // Update sentiment (loading state)
                document.getElementById("sentimentValue").innerHTML =
                    "😐 Analyzing...";
                document.getElementById("intentValue").textContent =
                    "Analyzing conversation...";
                document.getElementById("summaryValue").textContent =
                    "Loading summary...";

                const conv = conversations.find(c => c.session_id === sessionId);

                // Load LLM Assist config for this bot
                try {
                    const configResponse = await fetch(`${API_BASE}/api/attendance/llm/config/${conv?.bot_id || 'default'}`);
                    if (configResponse.ok) {
                        llmAssistConfig = await configResponse.json();
                    }
                } catch (e) {
                    console.log("LLM config not available, using defaults");
                }

                // Load real insights using LLM Assist APIs
                try {
                    // Generate summary if enabled
                    if (llmAssistConfig.auto_summary_enabled) {
                        const summaryResponse = await fetch(`${API_BASE}/api/attendance/llm/summary/${sessionId}`);
                        if (summaryResponse.ok) {
                            const summaryData = await summaryResponse.json();
                            if (summaryData.success) {
                                document.getElementById("summaryValue").textContent = summaryData.summary.brief || "No summary available";
                                document.getElementById("intentValue").textContent =
                                    summaryData.summary.customer_needs?.join(", ") || "General inquiry";
                            }
                        }
                    } else {
                        document.getElementById("summaryValue").textContent =
                            `Customer ${conv?.user_name || "Anonymous"} via ${conv?.channel || "web"}`;
                        document.getElementById("intentValue").textContent = "General inquiry";
                    }

                    // Analyze sentiment if we have the last message
                    if (llmAssistConfig.sentiment_enabled && conv?.last_message) {
                        const sentimentResponse = await fetch(`${API_BASE}/api/attendance/llm/sentiment`, {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({
                                session_id: sessionId,
                                message: conv.last_message,
                                history: conversationHistory
                            })
                        });
                        if (sentimentResponse.ok) {
                            const sentimentData = await sentimentResponse.json();
                            if (sentimentData.success) {
                                const s = sentimentData.sentiment;
                                const sentimentClass = s.overall === 'positive' ? 'sentiment-positive' :
                                                       s.overall === 'negative' ? 'sentiment-negative' : 'sentiment-neutral';
                                document.getElementById("sentimentValue").innerHTML =
                                    `<span class="sentiment-indicator ${sentimentClass}">${s.emoji} ${s.overall.charAt(0).toUpperCase() + s.overall.slice(1)}</span>`;

                                // Show warning for high escalation risk
                                if (s.escalation_risk === 'high') {
                                    showToast("⚠️ High escalation risk detected", "warning");
                                }
                            }
                        }
                    } else {
                        document.getElementById("sentimentValue").innerHTML =
                            `<span class="sentiment-indicator sentiment-neutral">😐 Neutral</span>`;
                    }

                    // Generate smart replies if enabled
                    if (llmAssistConfig.smart_replies_enabled) {
                        await loadSmartReplies(sessionId);
                    } else {
                        loadDefaultReplies();
                    }

                } catch (error) {
                    console.error("Failed to load insights:", error);
                    // Show fallback data
                    document.getElementById("sentimentValue").innerHTML =
                        `<span class="sentiment-indicator sentiment-neutral">😐 Neutral</span>`;
                    document.getElementById("summaryValue").textContent =
                        `Customer ${conv?.user_name || "Anonymous"} via ${conv?.channel || "web"}`;
                    loadDefaultReplies();
                }
            }

            // Load smart replies from LLM
            async function loadSmartReplies(sessionId) {
                try {
                    const response = await fetch(`${API_BASE}/api/attendance/llm/smart-replies`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            session_id: sessionId,
                            history: conversationHistory
                        })
                    });

                    if (response.ok) {
                        const data = await response.json();
                        if (data.success && data.replies.length > 0) {
                            const repliesHtml = data.replies.map(reply => `
                                <div class="suggested-reply" onclick="useSuggestion(this)">
                                    <div class="suggested-reply-text">${escapeHtml(reply.text)}</div>
                                    <div class="suggestion-meta">
                                        <span class="suggestion-confidence">${Math.round(reply.confidence * 100)}% match</span>
                                        <span class="suggestion-source">${reply.tone} • AI</span>
                                    </div>
                                </div>
                            `).join('');
                            document.getElementById("suggestedReplies").innerHTML = repliesHtml;
                            return;
                        }
                    }
                } catch (e) {
                    console.error("Failed to load smart replies:", e);
                }
                loadDefaultReplies();
            }

            // Load default replies when LLM is unavailable
            function loadDefaultReplies() {
                document.getElementById("suggestedReplies").innerHTML = `
                    <div class="suggested-reply" onclick="useSuggestion(this)">
                        <div class="suggested-reply-text">Hello! Thank you for reaching out. How can I assist you today?</div>
                        <div class="suggestion-meta">
                            <span class="suggestion-confidence">Template</span>
                            <span class="suggestion-source">Quick Reply</span>
                        </div>
                    </div>
                    <div class="suggested-reply" onclick="useSuggestion(this)">
                        <div class="suggested-reply-text">I'd be happy to help you with that. Let me look into it.</div>
                        <div class="suggestion-meta">
                            <span class="suggestion-confidence">Template</span>
                            <span class="suggestion-source">Quick Reply</span>
                        </div>
                    </div>
                    <div class="suggested-reply" onclick="useSuggestion(this)">
                        <div class="suggested-reply-text">Is there anything else I can help you with?</div>
                        <div class="suggestion-meta">
                            <span class="suggestion-confidence">Template</span>
                            <span class="suggestion-source">Quick Reply</span>
                        </div>
                    </div>
                `;
            }

            // Generate tips when customer message arrives
            async function generateTips(sessionId, customerMessage) {
                if (!llmAssistConfig.tips_enabled) return;

                try {
                    const response = await fetch(`${API_BASE}/api/attendance/llm/tips`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            session_id: sessionId,
                            customer_message: customerMessage,
                            history: conversationHistory
                        })
                    });

                    if (response.ok) {
                        const data = await response.json();
                        if (data.success && data.tips.length > 0) {
                            displayTips(data.tips);
                        }
                    }
                } catch (e) {
                    console.error("Failed to generate tips:", e);
                }
            }

            // Display tips in the UI
            function displayTips(tips) {
                const tipsContainer = document.getElementById("tipsContainer");
                if (!tipsContainer) {
                    // Create tips container if it doesn't exist
                    const insightsSection = document.querySelector(".insights-sidebar .sidebar-section");
                    if (insightsSection) {
                        const tipsDiv = document.createElement("div");
                        tipsDiv.id = "tipsContainer";
                        tipsDiv.className = "ai-insight";
                        tipsDiv.innerHTML = `
                            <div class="insight-header">
                                <span class="insight-icon">💡</span>
                                <span class="insight-label">Tips</span>
                            </div>
                            <div class="insight-value" id="tipsValue"></div>
                        `;
                        insightsSection.insertBefore(tipsDiv, insightsSection.firstChild);
                    }
                }

                const tipsValue = document.getElementById("tipsValue");
                if (tipsValue) {
                    const tipsHtml = tips.map(tip => {
                        const emoji = tip.tip_type === 'warning' ? '⚠️' :
                                     tip.tip_type === 'intent' ? '🎯' :
                                     tip.tip_type === 'action' ? '✅' : '💡';
                        return `<div style="margin-bottom: 8px;">${emoji} ${escapeHtml(tip.content)}</div>`;
                    }).join('');
                    tipsValue.innerHTML = tipsHtml;

                    // Show toast for high priority tips
                    const highPriorityTip = tips.find(t => t.priority === 1);
                    if (highPriorityTip) {
                        showToast(`💡 ${highPriorityTip.content}`, "info");
                    }
                }
            }

            // Polish message before sending
            async function polishMessage() {
                if (!llmAssistConfig.polish_enabled) {
                    showToast("Message polish feature is disabled", "info");
                    return;
                }

                const input = document.getElementById("chatInput");
                const message = input.value.trim();

                if (!message || !currentSessionId) {
                    showToast("Enter a message first", "info");
                    return;
                }

                showToast("✨ Polishing message...", "info");

                try {
                    const response = await fetch(`${API_BASE}/api/attendance/llm/polish`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            session_id: currentSessionId,
                            message: message,
                            tone: "professional"
                        })
                    });

                    if (response.ok) {
                        const data = await response.json();
                        if (data.success && data.polished !== message) {
                            input.value = data.polished;
                            input.style.height = "auto";
                            input.style.height = Math.min(input.scrollHeight, 120) + "px";

                            if (data.changes.length > 0) {
                                showToast(`✨ Message polished: ${data.changes.join(", ")}`, "success");
                            } else {
                                showToast("✨ Message polished!", "success");
                            }
                        } else {
                            showToast("Message looks good already!", "success");
                        }
                    }
                } catch (e) {
                    console.error("Failed to polish message:", e);
                    showToast("Failed to polish message", "error");
                }
            }

            // =====================================================================
            // WebSocket
            // =====================================================================
            function connectWebSocket() {
                if (!currentAttendantId) {
                    console.warn(
                        "No attendant ID, skipping WebSocket connection",
                    );
                    return;
                }

                try {
                    const protocol =
                        window.location.protocol === "https:" ? "wss:" : "ws:";
                    ws = new WebSocket(
                        `${protocol}//${window.location.host}/ws/attendant?attendant_id=${encodeURIComponent(currentAttendantId)}`,
                    );

                    ws.onopen = () => {
                        console.log(
                            "WebSocket connected for attendant:",
                            currentAttendantId,
                        );
                        showToast(
                            "Connected to notification service",
                            "success",
                        );
                    };

                    ws.onmessage = (event) => {
                        const data = JSON.parse(event.data);
                        console.log("WebSocket message received:", data);
                        handleWebSocketMessage(data);
                    };

                    ws.onclose = () => {
                        console.log("WebSocket disconnected");
                        attemptReconnect();
                    };

                    ws.onerror = (error) => {
                        console.error("WebSocket error:", error);
                    };
                } catch (error) {
                    console.error("Failed to connect WebSocket:", error);
                    attemptReconnect();
                }
            }

            function attemptReconnect() {
                if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
                    reconnectAttempts++;
                    setTimeout(() => {
                        console.log(
                            `Reconnecting... attempt ${reconnectAttempts}`,
                        );
                        connectWebSocket();
                    }, 2000 * reconnectAttempts);
                }
            }

            function handleWebSocketMessage(data) {
                const msgType = data.type || data.notification_type;

                switch (msgType) {
                    case "connected":
                        console.log("WebSocket connected:", data.message);
                        reconnectAttempts = 0;
                        break;
                    case "new_conversation":
                        showToast("New conversation in queue", "info");
                        loadQueue();
                        // Play notification sound
                        playNotificationSound();
                        break;
                    case "new_message":
                        // Message from customer
                        showToast(
                            `New message from ${data.user_name || "Customer"}`,
                            "info",
                        );
                        if (data.session_id === currentSessionId) {
                            addMessage(
                                "customer",
                                data.content,
                                data.timestamp,
                            );

                            // Add to conversation history for context
                            conversationHistory.push({
                                role: "customer",
                                content: data.content,
                                timestamp: data.timestamp || new Date().toISOString()
                            });

                            // Generate tips for this new message
                            generateTips(data.session_id, data.content);

                            // Refresh sentiment analysis
                            if (llmAssistConfig.sentiment_enabled) {
                                loadInsights(data.session_id);
                            }
                        }
                        loadQueue();
                        playNotificationSound();
                        break;
                    case "attendant_response":
                        // Response from another attendant
                        if (
                            data.session_id === currentSessionId &&
                            data.assigned_to !== currentAttendantId
                        ) {
                            addMessage(
                                "attendant",
                                data.content,
                                data.timestamp,
                            );
                        }
                        break;
                    case "queue_update":
                        loadQueue();
                        break;
                    case "transfer":
                        if (data.assigned_to === currentAttendantId) {
                            showToast(
                                `Conversation transferred to you`,
                                "info",
                            );
                            loadQueue();
                            playNotificationSound();
                        }
                        break;
                    default:
                        console.log(
                            "Unknown WebSocket message type:",
                            msgType,
                            data,
                        );
                }
            }

            function playNotificationSound() {
                // Create a simple beep sound
                try {
                    const audioContext = new (window.AudioContext ||
                        window.webkitAudioContext)();
                    const oscillator = audioContext.createOscillator();
                    const gainNode = audioContext.createGain();

                    oscillator.connect(gainNode);
                    gainNode.connect(audioContext.destination);

                    oscillator.frequency.value = 800;
                    oscillator.type = "sine";
                    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
                    gainNode.gain.exponentialRampToValueAtTime(
                        0.01,
                        audioContext.currentTime + 0.3,
                    );

                    oscillator.start(audioContext.currentTime);
                    oscillator.stop(audioContext.currentTime + 0.3);
                } catch (e) {
                    // Audio not available
                    console.log("Could not play notification sound");
                }
            }

            // =====================================================================
            // Utility Functions
            // =====================================================================
            function escapeHtml(text) {
                const div = document.createElement("div");
                div.textContent = text || "";
                return div.innerHTML;
            }

            function formatTime(timestamp) {
                if (!timestamp) return "";
                const date = new Date(timestamp);
                const now = new Date();
                const diff = (now - date) / 1000;

                if (diff < 60) return "Just now";
                if (diff < 3600) return `${Math.floor(diff / 60)} min`;
                if (diff < 86400)
                    return date.toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                    });
                return date.toLocaleDateString();
            }

            function formatWaitTime(seconds) {
                if (!seconds || seconds < 0) return "";
                if (seconds < 60) return `${seconds}s`;
                if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
                return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
            }

            function showToast(message, type = "info") {
                const container = document.getElementById("toastContainer");
                const toast = document.createElement("div");
                toast.className = `toast ${type}`;
                toast.innerHTML = `
                <span>${escapeHtml(message)}</span>
            `;
                container.appendChild(toast);

                setTimeout(() => {
                    toast.style.opacity = "0";
                    setTimeout(() => toast.remove(), 300);
                }, 3000);
            }

            function attachFile() {
                showToast("File attachment coming soon", "info");
            }

            function insertEmoji() {
                showToast("Emoji picker coming soon", "info");
            }

            function loadHistoricalConversation(id) {
                showToast("Loading conversation history...", "info");
            }

            // Periodic refresh (every 30 seconds if WebSocket not connected)
            setInterval(() => {
                if (currentAttendantStatus === "online") {
                    // Only refresh if WebSocket is not connected
                    if (!ws || ws.readyState !== WebSocket.OPEN) {
                        loadQueue();
                    }
                }
            }, 30000);

            // Send status updates via WebSocket
            function sendWebSocketMessage(data) {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(JSON.stringify(data));
                }
            }

            // Send typing indicator
            function sendTypingIndicator() {
                if (currentSessionId) {
                    sendWebSocketMessage({
                        type: "typing",
                        session_id: currentSessionId,
                    });
                }
            }

            // Mark messages as read
            function markAsRead(sessionId) {
                sendWebSocketMessage({
                    type: "read",
                    session_id: sessionId,
                });
            }
