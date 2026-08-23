
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
