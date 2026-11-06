        function processMessageContent(r) {
            if (isContextChange) {
                isContextChange = false;
                return;
            }
            if (r.context_usage !== undefined) {
                updateContextUsage(r.context_usage);
            }
            if (r.suggestions && r.suggestions.length > 0) {
                handleSuggestions(r.suggestions);
            }
            if (r.is_complete) {
                if (isStreaming) {
                    finalizeStreamingMessage();
                    isStreaming = false;
                    streamingMessageId = null;
                    currentStreamingContent = "";
                }
                if (r.content && r.content.trim() !== "") {
                    addMessage("assistant", r.content, false);
                }
            } else {
                if (!isStreaming) {
                    isStreaming = true;
                    streamingMessageId = "streaming-" + Date.now();
                    currentStreamingContent = r.content || "";
                    addMessage(
                        "assistant",
                        currentStreamingContent,
                        true,
                        streamingMessageId,
                    );
                } else {
                    currentStreamingContent += r.content || "";
                    updateStreamingMessage(currentStreamingContent);
                }
            }
        }

        function handleEvent(t, d) {
            console.log("Event received:", t, d);
            switch (t) {
                case "thinking_start":
                    showThinkingIndicator();
                    break;
                case "thinking_end":
                    hideThinkingIndicator();
                    break;
                case "warn":
                    showWarning(d.message);
                    break;
                case "context_usage":
                    updateContextUsage(d.usage);
                    break;
                case "change_theme":
                    if (d.color1) themeColor1 = d.color1;
                    if (d.color2) themeColor2 = d.color2;
                    if (d.logo_url) customLogoUrl = d.logo_url;
                    if (d.title) document.title = d.title;
                    if (d.logo_text) {
                        sidebarTitle.textContent = d.logo_text;
                    }
                    applyTheme();
                    break;
            }
        }

        function showThinkingIndicator() {
            if (isThinking) return;
            const t = document.createElement("div");
            t.id = "thinking-indicator";
            t.className = "message-container";
            t.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="thinking-indicator"><div class="typing-dots"><div class="typing-dot"></div><div class="typing-dot"></div><div class="typing-dot"></div></div></div></div>`;
            messagesDiv.appendChild(t);
            if (!isUserScrolling) {
                scrollToBottom();
            }
            thinkingTimeout = setTimeout(() => {
                if (isThinking) {
                    hideThinkingIndicator();
                    showWarning(
                        "O servidor pode estar ocupado. A resposta está demorando demais.",
                    );
                }
            }, 60000);
            isThinking = true;
        }

        function hideThinkingIndicator() {
            if (!isThinking) return;
            const t = document.getElementById("thinking-indicator");
            if (t) t.remove();
            if (thinkingTimeout) {
                clearTimeout(thinkingTimeout);
                thinkingTimeout = null;
            }
            isThinking = false;
        }

        function showWarning(m) {
            const w = document.createElement("div");
            w.className = "warning-message";
            w.innerHTML = `⚠️ ${m}`;
            messagesDiv.appendChild(w);
            if (!isUserScrolling) {
                scrollToBottom();
            }
            setTimeout(() => {
                if (w.parentNode) {
                    w.classList.add("warning-hiding");
                    setTimeout(() => w.remove(), 300);
                }
            }, 5000);
        }

        function showContinueButton() {
            const c = document.createElement("div");
            c.className = "message-container";
            c.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content"><p>A conexão foi interrompida. Clique em "Continuar" para tentar recuperar a resposta.</p><button class="continue-button" onclick="continueInterruptedResponse()">Continuar</button></div></div>`;
            messagesDiv.appendChild(c);
            if (!isUserScrolling) {
                scrollToBottom();
            }
        }

        function continueInterruptedResponse() {
            if (!ws || ws.readyState !== WebSocket.OPEN) {
                connectWebSocket();
            }
            if (ws && ws.readyState === WebSocket.OPEN) {
                const d = {
                    bot_id: "default_bot",
                    user_id: currentUserId,
                    session_id: currentSessionId,
                    channel: "web",
                    content: "continue",
                    message_type: MessageType.CONTINUE,
                    media_url: null,
                    timestamp: new Date().toISOString(),
                };
                ws.send(JSON.stringify(d));
            }
            document.querySelectorAll(".continue-button").forEach((b) => {
                b.parentElement.parentElement.parentElement.remove();
            });
        }

        function addMessage(
            role,
            content,
            streaming = false,
            msgId = null,
        ) {
            const m = document.createElement("div");
            m.className = "message-container";
            if (role === "user") {
                m.innerHTML = `<div class="user-message"><div class="user-message-content">${escapeHtml(content)}</div></div>`;
                updateContextUsage(contextUsage + 0.05);
            } else if (role === "assistant") {
                const isHtml = content.trim().startsWith('<');
                const rendered = streaming ? "" : (isHtml ? content : marked.parse(content));
                m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content markdown-content" id="${msgId || ""}">${rendered}</div></div>`;
                updateContextUsage(contextUsage + 0.03);
            } else if (role === "voice") {
                m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar">🎤</div><div class="assistant-message-content">${content}</div></div>`;
            } else {
                m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content">${content}</div></div>`;
            }
            messagesDiv.appendChild(m);
            if (!isUserScrolling) {
                scrollToBottom();
            }
        }

        function updateStreamingMessage(c) {
            const m = document.getElementById(streamingMessageId);
            if (m) {
                const isHtmlStream = c.trim().startsWith('<');
                m.innerHTML = isHtmlStream ? c : marked.parse(c);
                if (!isUserScrolling) {
                    scrollToBottom();
                }
            }
        }

        function finalizeStreamingMessage() {
            const m = document.getElementById(streamingMessageId);
            if (m) {
                const isHtmlFinal = currentStreamingContent.trim().startsWith('<');
                m.innerHTML = isHtmlFinal ? currentStreamingContent : marked.parse(currentStreamingContent);
                m.removeAttribute("id");
                if (!isUserScrolling) {
                    scrollToBottom();
                }
            }
        }

        function escapeHtml(t) {
            const d = document.createElement("div");
            d.textContent = t;
            return d.innerHTML;
        }

        function clearSuggestions() {
            suggestionsContainer.innerHTML = "";
        }

        function handleSuggestions(s) {
            const uniqueSuggestions = s.filter(
                (v, i, a) =>
                    i ===
                    a.findIndex(
                        (t) => t.text === v.text && t.context === v.context,
                    ),
            );
            suggestionsContainer.innerHTML = "";
            uniqueSuggestions.forEach((v) => {
                const b = document.createElement("button");
                b.textContent = v.text;
                b.className = "suggestion-button";
                b.onclick = () => {
                    setContext(v.context);
                    input.value = "";
                };
                suggestionsContainer.appendChild(b);
            });
        }

        let pendingContextChange = null;

        async function setContext(c) {
            try {
                const t = event?.target?.textContent || c;
                addMessage("user", t);
                const i = document.getElementById("messageInput");
                if (i) {
                    i.value = "";
                }
                if (ws && ws.readyState === WebSocket.OPEN) {
                    pendingContextChange = new Promise((r) => {
                        const h = (e) => {
                            const d = JSON.parse(e.data);
                            if (
                                d.message_type ===
                                MessageType.CONTEXT_CHANGE &&
                                d.context_name === c
                            ) {
                                ws.removeEventListener("message", h);
                                r();
                            }
                        };
                        ws.addEventListener("message", h);
                        const s = {
                            bot_id: currentBotId,
                            user_id: currentUserId,
                            session_id: currentSessionId,
                            channel: "web",
                            content: t,
                            message_type: MessageType.SUGGESTION,
                            is_suggestion: true,
                            context_name: c,
                            timestamp: new Date().toISOString(),
                        };
                        ws.send(JSON.stringify(s));
                    });
                    await pendingContextChange;
                    const x = document.getElementById("contextIndicator");
                    if (x) {
                        document.getElementById(
                            "contextPercentage",
                        ).textContent = c;
                    }
                } else {
                    console.warn(
                        "WebSocket não está conectado. Tentando reconectar...",
                    );
                    connectWebSocket();
                }
            } catch (err) {
                console.error("Failed to set context:", err);
            }
        }
