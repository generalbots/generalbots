
            // =====================================================================
            // Initialization
            // =====================================================================
            (function(){ var __cb = async () => {
                await checkCRMEnabled();
                setupEventListeners();
            }; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

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

                // Populate customer details + tags from the real session record
                try {
                    const sessionResp = await fetch(
                        `${API_BASE}/api/attendant/sessions/${encodeURIComponent(sessionId)}`,
                    );
                    if (sessionResp.ok) {
                        const sessionData = await sessionResp.json();
                        const s = sessionData && sessionData.session ? sessionData.session : null;
                        if (s) {
                            const detailEmail = document.getElementById("detailEmail");
                            const detailPhone = document.getElementById("detailPhone");
                            const detailLocation = document.getElementById("detailLocation");
                            const detailTags = document.getElementById("detailTags");
                            if (detailEmail) {
                                detailEmail.textContent = s.customer_email || conv.user_email || "-";
                            }
                            if (detailPhone) {
                                detailPhone.textContent = s.customer_phone || "-";
                            }
                            if (detailLocation) {
                                const loc =
                                    s.metadata && s.metadata.location ? s.metadata.location : "-";
                                detailLocation.textContent = loc;
                            }
                            if (detailTags) {
                                const tags = Array.isArray(s.tags) ? s.tags : [];
                                detailTags.innerHTML = tags.length
                                    ? tags
                                          .map(
                                              (t) =>
                                                  `<span class="tag">${escapeHtml(t)}</span>`,
                                          )
                                          .join("")
                                    : '<span class="tag tag-empty">no tags</span>';
                            }
                        }
                    }
                } catch (e) {
                    console.warn("Failed to load customer details", e);
                }

                // Load previous conversations for this customer
                loadHistoricalConversation(sessionId);

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
