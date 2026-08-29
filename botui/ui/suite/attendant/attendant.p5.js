
            function emojiHasLabel(emoji, q) {
                var labels = {
                    "😀": "smile happy grin",
                    "😂": "laugh cry joy",
                    "❤️": "heart love",
                    "👍": "thumbs up yes",
                    "👎": "thumbs down no",
                    "🙏": "pray please thanks",
                    "🎉": "party celebrate",
                    "👏": "clap applause",
                    "💯": "hundred percent",
                    "✅": "check done ok",
                    "❌": "cross no error",
                    "⚠️": "warning",
                    "🔥": "fire hot",
                    "✨": "sparkle shine",
                    "💡": "idea lightbulb",
                    "📎": "clip attachment",
                    "📦": "package box delivery",
                    "🔒": "lock secure",
                    "📞": "phone call",
                    "✉️": "email mail",
                    "💰": "money cash payment",
                    "💳": "card credit payment",
                    "🧾": "receipt invoice",
                    "⭐": "star rating",
                    "🏆": "trophy award winner",
                    "🥇": "gold medal",
                    "💪": "strong muscle",
                    "🤝": "handshake deal agree",
                    "🚀": "rocket launch ship",
                    "⏰": "clock time alarm",
                    "📅": "calendar date schedule",
                    "📌": "pin location",
                    "📍": "location map pin",
                    "🔍": "search magnifier",
                    "📈": "chart growth increase",
                    "📉": "chart decline decrease",
                    "📊": "bar chart stats",
                    "🗂️": "folder organize",
                    "🖥️": "computer desktop",
                    "💻": "laptop computer",
                    "📱": "phone mobile",
                    "🔧": "wrench fix repair",
                    "🔨": "hammer tool",
                    "⚙️": "gear settings",
                    "🧪": "test lab experiment",
                    "🦠": "virus bug health",
                    "💊": "pill medicine health",
                    "🏥": "hospital health",
                    "👨‍⚕️": "doctor medic",
                    "🤖": "robot ai",
                    "👋": "wave hello bye",
                    "👀": "eyes watch see",
                    "💬": "chat message talk",
                    "📝": "note write edit",
                    "🗑️": "trash delete remove",
                    "🔔": "bell notify alert",
                    "🌧️": "rain weather",
                    "☀️": "sun weather",
                    "❄️": "snow cold winter",
                    "🌈": "rainbow",
                    "🎂": "cake birthday",
                    "🎁": "gift present",
                    "🍕": "pizza food",
                    "☕": "coffee drink",
                    "🍺": "beer drink",
                    "🥂": "toast cheers drink",
                    "🚗": "car vehicle",
                    "✈️": "plane flight travel",
                    "🏠": "home house",
                    "🏢": "building office",
                    "🌍": "world earth global",
                    "🇧🇷": "brazil",
                    "🇺🇸": "united states usa",
                    "🇬🇧": "united kingdom uk england",
                    "🇪🇸": "spain",
                    "🇫🇷": "france",
                    "🇩🇪": "germany",
                    "🇯🇵": "japan",
                    "🇨🇳": "china",
                    "🇰🇷": "korea south",
                    "🇲🇽": "mexico",
                    "🇵🇹": "portugal",
                    "🇦🇷": "argentina",
                    "🇨🇴": "colombia",
                    "🇵🇪": "peru",
                    "🇺🇾": "uruguay",
                    "🇨🇱": "chile",
                };
                var label = labels[emoji] || "";
                return label.indexOf(q) !== -1;
            }

            function pickEmoji(emoji) {
                var input = document.getElementById("chatInput");
                if (input) {
                    var start = input.selectionStart || input.value.length;
                    var end = input.selectionEnd || input.value.length;
                    input.value =
                        input.value.slice(0, start) +
                        emoji +
                        input.value.slice(end);
                    input.focus();
                    input.selectionStart = input.selectionEnd = start + emoji.length;
                    input.dispatchEvent(new Event("input"));
                }
                rememberEmoji(emoji);
                var picker = document.getElementById("emojiPicker");
                if (picker) picker.style.display = "none";
            }

            function escapeHtml(text) {
                var div = document.createElement("div");
                div.textContent = text == null ? "" : String(text);
                return div.innerHTML;
            }

            function loadHistoricalConversation(id) {
                var box = document.getElementById("conversationHistory");
                if (!box) return;
                if (!id) {
                    box.innerHTML =
                        '<div class="history-item"><div class="history-summary">Select a customer to view their history</div></div>';
                    return;
                }
                box.innerHTML = '<div class="loading-spinner"></div>';
                fetch("/api/attendant/sessions/" + encodeURIComponent(id))
                    .then(function (r) {
                        return r.json();
                    })
                    .then(function (data) {
                        var session = data && data.session ? data.session : null;
                        var messages =
                            data && Array.isArray(data.messages) ? data.messages : [];
                        if (!messages.length) {
                            box.innerHTML =
                                '<div class="history-item"><div class="history-summary">No previous conversations</div></div>';
                            return;
                        }
                        box.innerHTML = messages
                            .map(function (m) {
                                var when = m.created_at
                                    ? new Date(m.created_at).toLocaleString()
                                    : "";
                                var who = m.sender_type
                                    ? m.sender_type
                                    : m.agent_id
                                      ? "agent"
                                      : "customer";
                                return (
                                    '<div class="history-item" onclick="loadHistoricalConversation(\'' +
                                    id +
                                    '\')"><div class="history-header"><span class="history-date">' +
                                    escapeHtml(when) +
                                    '</span></div><div class="history-summary">' +
                                    escapeHtml(m.content || "") +
                                    '</div><div class="history-meta">' +
                                    escapeHtml(who) +
                                    "</div></div>"
                                );
                            })
                            .join("");
                    })
                    .catch(function () {
                        box.innerHTML =
                            '<div class="history-item"><div class="history-summary">Could not load history</div></div>';
                    });
            }

            // Periodic refresh (every 30 seconds if WebSocket not connected)
            var __pollQueue = function () {
                if (currentAttendantStatus === "online") {
                    if (!ws || ws.readyState !== WebSocket.OPEN) {
                        loadQueue();
                    }
                }
            };
            if (window.GBAppLifecycle) {
                GBAppLifecycle.interval("attendant", __pollQueue, 30000);
            } else {
                setInterval(__pollQueue, 30000);
            }

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
