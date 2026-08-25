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
                    // Backend route: /api/attendance/ws (NOT /ws/attendant).
                    // Token required for the WS upgrade handshake.
                    const wsToken =
                        localStorage.getItem("gb-access-token") || "";
                    ws = new WebSocket(
                        `${protocol}//${window.location.host}/api/attendance/ws?attendant_id=${encodeURIComponent(currentAttendantId)}&token=${encodeURIComponent(wsToken)}`,
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

            // ------------------------------------------------------------------
            // File attachments
            // ------------------------------------------------------------------

            // Pending attachments for the active composer (reset on send).
            var pendingAttachments = [];
            var MAX_ATTACHMENT_MB = 25;

            // Attachment types that render inline as image previews.
            function isImageType(contentType) {
                return contentType && contentType.indexOf("image/") === 0;
            }

            function attachFile() {
                var input = document.getElementById("attachFileInput");
                if (!input) return;
                input.value = "";
                input.click();
            }

            async function handleFilesSelected(fileList) {
                if (!currentSessionId) {
                    showToast("Select a conversation first", "error");
                    return;
                }
                var files = Array.prototype.slice.call(fileList || []);
                for (var i = 0; i < files.length; i++) {
                    await uploadAttachment(files[i]);
                }
            }

            async function uploadAttachment(file) {
                if (file.size > MAX_ATTACHMENT_MB * 1024 * 1024) {
                    showToast(
                        file.name + " exceeds " + MAX_ATTACHMENT_MB + " MiB limit",
                        "error",
                    );
                    return;
                }

                var formData = new FormData();
                formData.append("file", file);

                showToast("Uploading " + file.name + "...", "info");

                try {
                    var resp = await fetch(
                        API_BASE +
                            "/api/attendant/sessions/" +
                            currentSessionId +
                            "/attachments",
                        {
                            method: "POST",
                            body: formData,
                        },
                    );
                    if (!resp.ok) {
                        var errBody = await resp.json().catch(() => ({}));
                        throw new Error(errBody.error || "Upload failed");
                    }
                    var meta = await resp.json();
                    pendingAttachments.push(meta);
                    renderPendingAttachments();
                    showToast(file.name + " attached", "success");
                } catch (e) {
                    showToast("Upload failed: " + e.message, "error");
                }
            }

            function renderPendingAttachments() {
                var area = document.getElementById("attachmentPreviewArea");
                if (!area) return;
                if (!pendingAttachments.length) {
                    area.style.display = "none";
                    area.innerHTML = "";
                    return;
                }
                area.style.display = "";
                area.innerHTML = pendingAttachments
                    .map(function (a) {
                        return (
                            '<div class="attachment-chip" data-id="' +
                            escapeHtml(a.id) +
                            '">' +
                            (isImageType(a.content_type)
                                ? '<span class="chip-icon">🖼️</span>'
                                : '<span class="chip-icon">📎</span>') +
                            '<span class="chip-name">' +
                            escapeHtml(a.name) +
                            "</span>" +
                            '<button class="chip-remove" onclick="removePendingAttachment(\'' +
                            escapeHtml(a.id) +
                            "')" + "\">×</button>" +
                            "</div>"
                        );
                    })
                    .join("");
            }

            function removePendingAttachment(id) {
                pendingAttachments = pendingAttachments.filter(function (a) {
                    return a.id !== id;
                });
                renderPendingAttachments();
            }

            // ------------------------------------------------------------------
            // Emoji picker
            // ------------------------------------------------------------------

            // Base emoji set grouped for quick picking. Entries support an
            // optional `tones` array of skin-tone modifier suffix codes.
                        var EMOJI_SET = [
                "😀","😃","😄","😁","😆","😅","😂","🤣","😊","😇","🙂","😉","😍","😘","😜","🤪","🤗","🤔","🤨","😐","😑","😶","😏","😒","🙄","😬","😮","😯","😴","🤤","😪","😷","🤒","🤕","🤢","🤮","🥳","😎","🤓","🧐","😕","😟","🙁","😖","😞","😤","😢","😭","😱","😳","🤯","😰","😥","😓","🤩","😡","😠","🤬","😈","💀","👻","👽","🤖","💩","🙏","👏","👍","👎","👊","✊","🤛","🤜","🤞","✌️","🤟","🤘","👌","🤌","🤏","👈","👉","👆","👇","☝️","✋","🤚","🖐️","🖖","👋","🤝","💪","🫶","✍️","💅","🤳","👀","👂","👃","🧠","🦷","👅","💋","❤️","🧡","💛","💚","💙","💜","🖤","🤍","💔","❣️","💕","💞","💓","💗","💖","💘","💝","💯","💢","💥","💫","💦","💨","💣","💬","💭","💤","🔥","✨","⭐","🌟","⚡","☄️","🌈","☀️","🌤️","⛅","🌧️","⛈️","🌨️","❄️","☃️","🌊","🌍","🌎","🌏","🌕","🌙","🪐","🍎","🍐","🍊","🍋","🍌","🍉","🍇","🍓","🫐","🍒","🍑","🥭","🍍","🥥","🥝","🍅","🥑","🥦","🥕","🌽","🍞","🥐","🧀","🍳","🥞","🍔","🍟","🍕","🌭","🥪","🌮","🌯","🍜","🍣","🍤","🍦","🍩","🍪","🎂","🍰","🧁","🍫","🍬","🍭","🍺","🍻","🥂","☕","🍵","🥤","🍾","⚽","🏀","🏈","⚾","🎾","🏐","🏉","🎱","🏓","🏸","🥅","🎯","🏹","🎮","🕹️","🎲","🎰","🎳","🎧","🎤","🎸","🎹","🥁","🎺","🎻","🎬","🎨","🎭","🏆","🥇","🥈","🥉","🏅","🚗","🚕","🚙","🚌","🚎","🏎️","🚓","🚑","🚒","🚐","🛴","🚲","🛵","🏍️","✈️","🚀","🛸","🚁","⛵","🚤","🛳️","🚦","🗺️","⌚","📱","💻","⌨️","🖥️","🖨️","🖱️","💽","💾","💿","📀","📷","📸","📹","🎥","📞","☎️","📟","📠","📺","📻","🧭","⏰","🌡️","🔋","🔌","💡","🔦","🕯️","💎","🗝️","🔑","🔨","🪛","🔧","⚙️","🔩","🧲","💉","🩹","💊","📎","📌","📍","📏","📐","✂️","🗃️","📁","📂","🗂️","📅","📆","🗒️","📇","📈","📉","📊","📋","📝","✏️","🖊️","🖋️","✒️","🖌️","🖍️","📚","📖","📕","📗","📘","📙","📰","🗞️","🏷️","🔖","💰","💸","💵","💴","💶","💷","💳","🧾","✉️","📧","📨","📩","📤","📥","📦","📫","📪","📬","📭","📮","🗳️","✅","❌","❓","❗","‼️","⁉️","⭕","🔴","🟠","🟡","🟢","🔵","🟣","⚫","⚪","🟤","🔺","🔻","🔸","🔹","🔶","🔷","🔔","🔕","🎵","🎶","🔇","🔈","🔉","🔊","📢","📣","🚫","⛔","🚧","⚠️","🚸","♻️","🔱","⚜️","🔰","🏁","🎌","🏳️","🏳️🌈","🇧🇷","🇺🇸","🇬🇧","🇪🇸","🇫🇷","🇩🇪","🇮🇹","🇯🇵","🇨🇳","🇰🇷","🇲🇽","🇵🇹","🇦🇷","🇨🇱","🇨🇴","🇵🇪","🇺🇾","👑","🎓","🎒","🧳","🎁","🎉","🎊","🎈","🎀","🪄","🔮","🧿","🔭","🔬","🧪","🧫","🧬","🩺","⚕️","🏥","🦠","🧼","🧽","🧴","🪥","🪒","🧷","👕","👖","🧥","🥼","👔","👗","👙","👘","🥻","👟","🥾","👠","👡","🩰","🧦","🧤","🧣","🧢","🎩","👒","🪖","⛑️","💄","💍","🕶️","🥽","🥷","🦸","🦸♀️","🦸♂️","🦹","🧙","🧙♀️","🧚","🧛","🧜","🧝","🧞","🧟","🦄","🐉","🦖","🦕","🦈","🐋","🐳","🐬","🐟","🐠","🐡","🦐","🦞","🦀","🐙","🦑","🐚","🐌","🦋","🐛","🐜","🐝","🐞","🦗","🕷️","🕸️","🦂","🦟","🐢","🐍","🦎","🐸","🐊","🐆","🐅","🦓","🦍","🦧","🐘","🦛","🦏","🐫","🦒","🦘","🐃","🐂","🐄","🐎","🐖","🐏","🐑","🐐","🦌","🐕","🐩","🦮","🐈","🦜","🦚","🦉","🦢","🦩","🕊️","🐇","🦝","🦨","🦡","🦫","🦦","🦥","🐁","🐭","🐹","🐰","🐻","🐨","🐼","🐯","🦁","🐮","🐷","🐸","🐵","🙈","🙉","🙊","🐒","🦆","🐓","🦃","🦅","🦇","🦔","🐿️","🐾","👶","🧒","👦","👧","🧑","👨","👩","🧓","👴","👵","👨👩👧👦","🧑🤝🧑","👭","👫","👬","💑","👩❤️👨","💏","👪","🌱","🌿","☘️","🍀","🌵","🌴","🌳","🌲","🍁","🍂","🍃","🌸","🌺","🌻","🌞","🌝","🌛","🌜","🌚","🌙","🌐","🏔️","⛰️","🌋","🗻","🏕️","🏖️","🏜️","🏝️","🏞️","🏟️","🏛️","🏠","🏡","🏢","🏣","🏤","🏥","🏦","🏨","🏩","🏪","🏫","🏬","🏭","🏯","🏰","💒","🗼","🗽","⛲","⛺","🌁","🌃","🏙️","🌄","🌅","🌆","🌇","🌉","♨️","🌌","🎠","🎡","🎢","🎪","🎭","🎨","🎰","🚃","🚋","🚝","🚂","🚆","🚇","🚉","🚊","🚏","🛤️","🛣️","🛫","🛬","🛩️","💺","🧮","🖇️","🪝","🪢","🪡","🧵","🪶","🧶","🧷","🎗️","🎟️","🎫","🎼","🎙️","🎚️","🎛️","🎷","🪘","🪗","🪕","👾","🎴","🃏","🀄","♠️","♥️","♦️","♣️","🎲","🧩","♟️","🎖️","🥎","🥏","🏏","🏑","🏒","🥍","🥊","🥋","⛳","⛸️","🎣","🤿","🎽","🎿","🛷","🥌","🪀","🪁","🔫","🧨","⚔️","🛡️","🏺","📿","🫧","🛎️","🚪","🪑","🛋️","🛏️","🛌","🧸","🪆","🖼️","🪞","🪟","🛍️","🛒","🎏","🎐","💌","📯","🧴","🪒","🩲","🩳","🥻","🩴","🩰","🪖","🦸♀️","🦸♂️","🦹♀️","🦹♂️","🧙♀️","🧙♂️","🧚♀️","🧚♂️","🧛♀️","🧛♂️","🧜♀️","🧜♂️","🧝♀️","🧝♂️","🧞♀️","🧞♂️","🧟♀️","🧟♂️","🐈⬛","🐻❄️","🫏","🫎","🫘","🫙","🪹","🪺","🫃","🫄","🫅","🫠","🫡","🫢","🫣","🫤","🫥","🫦","🫧","🫨","🫰","🫱","🫲","🫳","🫴","🫵","🫶","🫷","🫸","🩷","🩵","🩶"
            ];

            // Skin tone modifiers for emojis that support them.
            var SKIN_TONES = ["🏻", "🏼", "🏽", "🏾", "🏿"];

            function getRecentEmojis() {
                try {
                    var raw = localStorage.getItem("gb-attendant-recent-emojis");
                    return raw ? JSON.parse(raw) : [];
                } catch (e) {
                    return [];
                }
            }

            function rememberEmoji(emoji) {
                var recent = getRecentEmojis().filter(function (e) {
                    return e !== emoji;
                });
                recent.unshift(emoji);
                if (recent.length > 24) recent = recent.slice(0, 24);
                try {
                    localStorage.setItem(
                        "gb-attendant-recent-emojis",
                        JSON.stringify(recent),
                    );
                } catch (e) { /* storage unavailable */ }
            }

            function insertEmoji() {
                var picker = document.getElementById("emojiPicker");
                if (!picker) return;
                if (picker.style.display !== "none") {
                    picker.style.display = "none";
                    return;
                }
                renderEmojiPicker(picker, "");
                picker.style.display = "";
            }

            function renderEmojiPicker(picker, query) {
                var recent = getRecentEmojis();
                var q = query.trim().toLowerCase();
                var list = EMOJI_SET.filter(function (e) {
                    return !q || e.indexOf(q) !== -1 || emojiHasLabel(e, q);
                });
                if (q && !list.length) {
                    list = EMOJI_SET;
                }

                var html =
                    '<div class="emoji-picker-search">' +
                    '<input type="text" id="emojiSearch" placeholder="Search emoji..." value="' +
                    escapeHtml(query) +
                    '">' +
                    "</div>" +
                    (recent.length
                        ? '<div class="emoji-picker-section">Recent</div>' +
                          '<div class="emoji-picker-grid">' +
                          recent
                              .map(function (e) {
                                  return '<button class="emoji-item" onclick="pickEmoji(\'' +
                                      escapeHtml(e) +
                                      "')" + "\">" +
                                      e +
                                      "</button>";
                              })
                              .join("") +
                          "</div>"
                        : "") +
                    '<div class="emoji-picker-section">All</div>' +
                    '<div class="emoji-picker-grid">' +
                    list
                        .map(function (e) {
                            return '<button class="emoji-item" onclick="pickEmoji(\'' +
                                escapeHtml(e) +
                                "')" + "\">" +
                                e +
                                "</button>";
                        })
                        .join("") +
                    "</div>";
                picker.innerHTML = html;
                var search = document.getElementById("emojiSearch");
                if (search) {
                    search.addEventListener("input", function () {
                        renderEmojiPicker(picker, search.value);
                    });
                    search.focus();
                }
            }
