// Chat Main Module - Initializes chat and coordinates between modules
(function () {
    "use strict";
    
    function notify(message, type) {
        type = type || "info";
        if (window.GBAlerts) {
            window.GBAlerts.show(message, type);
        } else {
            console.log("[" + type + "]", message);
        }
    }
    
    function initChat() {
        console.log("Chat module initialized");
        setupEventHandlers();
    }
    
    function setupEventHandlers() {
        var form = document.getElementById("chatForm");
        var input = document.getElementById("messageInput");
        
        if (form) {
            form.onsubmit = function (e) {
                e.preventDefault();
                sendMessage();
                return false;
            };
        }
        
        if (input) {
            input.addEventListener("input", handleMentionInput);
        }
        
        var scrollBtn = document.getElementById("scrollToBottom");
        if (scrollBtn) {
            scrollBtn.addEventListener("click", function() {
                scrollToBottom(true);
            });
        }
        
        document.addEventListener("click", function (e) {
            if (
                !e.target.closest("#mentionDropdown") &&
                !e.target.closest("#messageInput")
            ) {
                hideMentionDropdown();
            }
        });
    }
    
    function sendMessage() {
        var input = document.getElementById("messageInput");
        if (!input) return;
        
        var content = input.value.trim();
        if (!content) return;
        
        // Get active switchers
        var activeSwitcherIds = getActiveSwitcherIds();
        console.log('Sending message with active_switchers:', activeSwitcherIds);
        
        // Add user message
        addMessage("user", content);
        
        // Clear input
        input.value = "";
        input.focus();
        
        // Send via WebSocket
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
                bot_id: currentBotId,
                user_id: currentUserId,
                session_id: currentSessionId,
                channel: "web",
                content: content,
                message_type: MessageType.USER,
                active_switchers: activeSwitcherIds,
                timestamp: new Date().toISOString()
            }));
        } else {
            notify("Not connected to server. Message not sent.", "warning");
        }
    }
    
    // Expose to global scope
    window.sendMessage = sendMessage;
    window.initChat = initChat;
    
    // Initialize on load
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", initChat);
    } else {
        initChat();
    }
})();
