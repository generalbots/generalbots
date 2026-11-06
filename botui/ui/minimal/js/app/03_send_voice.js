        async function sendMessage() {
            console.log("=== sendMessage called ===");
            console.log("input element:", input);
            console.log(
                "input.value:",
                input ? input.value : "input is null",
            );

            if (pendingContextChange) {
                await pendingContextChange;
                pendingContextChange = null;
            }

            if (!input) {
                console.error("Input element is null!");
                return;
            }

            const m = input.value.trim();
            console.log(
                "Attempting to send message:",
                m,
                "WS state:",
                ws ? ws.readyState : "no ws",
                "WebSocket.OPEN value:",
                WebSocket.OPEN,
            );

            if (!m) {
                console.log("Message is empty, not sending");
                return;
            }

            if (!ws || ws.readyState !== WebSocket.OPEN) {
                console.log(
                    "WebSocket not connected, attempting reconnect",
                );
                if (!ws || ws.readyState !== WebSocket.OPEN) {
                    showWarning(
                        "Conexão não disponível. Tentando reconectar...",
                    );
                    connectWebSocket();
                }
                return;
            }
            if (isThinking) {
                hideThinkingIndicator();
            }

            console.log("Adding message to UI");
            addMessage("user", m);

            console.log("Building message data object");
            console.log("currentBotId:", currentBotId);
            console.log("currentUserId:", currentUserId);
            console.log("currentSessionId:", currentSessionId);

            const d = {
                bot_id: currentBotId || "default_bot",
                user_id: currentUserId,
                session_id: currentSessionId,
                channel: "web",
                content: m,
                message_type: MessageType.USER,
                media_url: null,
                timestamp: new Date().toISOString(),
            };
            console.log("Message data object:", JSON.stringify(d, null, 2));

            try {
                const messageString = JSON.stringify(d);
                console.log("Stringified message:", messageString);
                console.log("About to call ws.send()");
                ws.send(messageString);
                console.log("ws.send() completed successfully");
            } catch (error) {
                console.error("Error sending message:", error);
                console.error("Error stack:", error.stack);
            }

            console.log("Clearing input field");
            input.value = "";
            input.focus();
            console.log("=== sendMessage completed ===");
        }

        async function toggleVoiceMode() {
            isVoiceMode = !isVoiceMode;
            const v = document.getElementById("voiceToggle");
            if (isVoiceMode) {
                v.textContent = "🔴 Stop Voice";
                v.classList.add("recording");
                await startVoiceSession();
            } else {
                v.textContent = "🎤 Voice Mode";
                v.classList.remove("recording");
                await stopVoiceSession();
            }
        }

        async function startVoiceSession() {
            if (!currentSessionId) return;
            try {
                const r = await fetch(`${BOTSERVER_URL}/api/voice/start`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        session_id: currentSessionId,
                        user_id: currentUserId,
                    }),
                }),
                    d = await r.json();
                if (d.token) {
                    await connectToVoiceRoom(d.token);
                    startVoiceRecording();
                }
            } catch (e) {
                console.error("Failed to start voice session:", e);
                showWarning("Falha ao iniciar modo de voz");
            }
        }

        async function stopVoiceSession() {
            if (!currentSessionId) return;
            try {
                await fetch(`${BOTSERVER_URL}/api/voice/stop`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ session_id: currentSessionId }),
                });
                if (voiceRoom) {
                    voiceRoom.disconnect();
                    voiceRoom = null;
                }
                if (mediaRecorder && mediaRecorder.state === "recording") {
                    mediaRecorder.stop();
                }
            } catch (e) {
                console.error("Failed to stop voice session:", e);
            }
        }

        async function connectToVoiceRoom(t) {
            try {
                const r = new LiveKitClient.Room(),
                    u = `${BOTSERVER_WS_URL}/voice`;
                await r.connect(u, t);

                voiceRoom = r;
                r.on("dataReceived", (d) => {
                    const dc = new TextDecoder(),
                        m = dc.decode(d);
                    try {
                        const j = JSON.parse(m);
                        if (j.type === "voice_response") {
                            addMessage("assistant", j.text);
                        }
                    } catch (e) {
                        console.log("Voice data:", m);
                    }
                });
                const l = await LiveKitClient.createLocalTracks({
                    audio: true,
                    video: false,
                });
                for (const k of l) {
                    await r.localParticipant.publishTrack(k);
                }
            } catch (e) {
                console.error("Failed to connect to voice room:", e);
                showWarning("Falha na conexão de voz");
            }
        }

        function startVoiceRecording() {
            if (!navigator.mediaDevices) {
                console.log("Media devices not supported");
                return;
            }
            navigator.mediaDevices
                .getUserMedia({ audio: true })
                .then((s) => {
                    mediaRecorder = new MediaRecorder(s);
                    audioChunks = [];
                    mediaRecorder.ondataavailable = (e) => {
                        audioChunks.push(e.data);
                    };
                    mediaRecorder.onstop = () => {
                        const a = new Blob(audioChunks, {
                            type: "audio/wav",
                        });
                        simulateVoiceTranscription();
                    };
                    mediaRecorder.start();
                    setTimeout(() => {
                        if (
                            mediaRecorder &&
                            mediaRecorder.state === "recording"
                        ) {
                            mediaRecorder.stop();
                            setTimeout(() => {
                                if (isVoiceMode) {
                                    startVoiceRecording();
                                }
                            }, 1000);
                        }
                    }, 5000);
                })
                .catch((e) => {
                    console.error("Error accessing microphone:", e);
                    showWarning("Erro ao acessar microfone");
                });
        }

        function simulateVoiceTranscription() {
            const p = [
                "Olá, como posso ajudá-lo hoje?",
                "Entendo o que você está dizendo",
                "Esse é um ponto interessante",
                "Deixe-me pensar sobre isso",
                "Posso ajudá-lo com isso",
                "O que você gostaria de saber?",
                "Isso parece ótimo",
                "Estou ouvindo sua voz",
            ],
                r = p[Math.floor(Math.random() * p.length)];
            if (voiceRoom) {
                const m = {
                    type: "voice_input",
                    content: r,
                    timestamp: new Date().toISOString(),
                };
                voiceRoom.localParticipant.publishData(
                    new TextEncoder().encode(JSON.stringify(m)),
                    LiveKitClient.DataPacketKind.RELIABLE,
                );
            }
            addMessage("voice", `🎤 ${r}`);
        }

        function scrollToBottom() {
            messagesDiv.scrollTop = messagesDiv.scrollHeight;
            isUserScrolling = false;
            scrollToBottomBtn.classList.remove("visible");
        }

        window.addEventListener("load", () => {
            console.log("Page loaded, initializing...");

            // Re-get elements after DOM is ready
            messagesDiv = document.getElementById("messages");
            input = document.getElementById("messageInput");
            sendBtn = document.getElementById("sendBtn");
            voiceBtn = document.getElementById("voiceBtn");

            console.log("After load - input:", !!input);
            console.log("After load - sendBtn:", !!sendBtn);

            // Attach event listeners after DOM is ready
            if (sendBtn) {
                sendBtn.onclick = () => {
                    console.log("Send button clicked!");
                    sendMessage();
                };
                console.log("sendBtn.onclick attached");
            } else {
                console.error("sendBtn element not found!");
            }

            if (input) {
                input.addEventListener("keypress", (e) => {
                    console.log("Key pressed:", e.key);
                    if (e.key === "Enter") {
                        console.log("Enter key detected, sending message");
                        sendMessage();
                    }
                });
                console.log("input keypress listener attached");
            } else {
                console.error("input element not found!");
            }

            initializeAuth();
        });
        window.addEventListener("focus", function () {
            if (!ws || ws.readyState !== WebSocket.OPEN) {
                connectWebSocket();
            }
        });
