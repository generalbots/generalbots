// Chat Messages Module - Handles message rendering and display
function addMessage(role, content, messageId) {
    var messages = document.getElementById("messages");
    if (!messages) return;
    
    var messageDiv = document.createElement("div");
    messageDiv.className = "message message-" + role;
    if (messageId) {
        messageDiv.dataset.messageId = messageId;
    }
    
    if (role === "bot") {
        messageDiv.innerHTML = marked.parse(content);
    } else {
        messageDiv.textContent = content;
    }
    
    messages.appendChild(messageDiv);
    scrollToBottom(false);
}

function scrollToBottom(animate) {
    var messages = document.getElementById("messages");
    if (!messages) return;
    
    if (animate) {
        messages.scrollTo({
            top: messages.scrollHeight,
            behavior: "smooth"
        });
    } else {
        messages.scrollTop = messages.scrollHeight;
    }
}

function showThinking() {
    var messages = document.getElementById("messages");
    if (!messages) return;
    
    var thinking = document.createElement("div");
    thinking.className = "message message-bot thinking";
    thinking.id = "thinking-indicator";
    thinking.textContent = "Thinking...";
    messages.appendChild(thinking);
    scrollToBottom(false);
}

function hideThinking() {
    var thinking = document.getElementById("thinking-indicator");
    if (thinking) {
        thinking.remove();
    }
}
