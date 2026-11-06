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
