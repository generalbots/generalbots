/* Suite app — Omnibox chat helpers (4/7). */
Object.assign(Omnibox, {
  startChatWithQuery(query) {
    this.showChat();
    this.input.value = "";
    setTimeout(() => {
      this.addMessage(query, "user");
      this.sendToBot(query);
    }, 100);
  },

  async sendMessage() {
    const message = this.chatInput?.value.trim();
    if (!message) return;

    this.chatInput.value = "";
    this.addMessage(message, "user");
    await this.sendToBot(message);
  },

  async sendToBot(message) {
    // Show typing indicator
    const typingId = this.addTypingIndicator();

    try {
      // Call the bot API
      const response = await fetch("/api/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message,
          context: this.getCurrentApp(),
        }),
      });

      this.removeTypingIndicator(typingId);

      if (response.ok) {
        const data = await response.json();
        this.addMessage(
          data.reply || data.message || "I received your message.",
          "bot",
        );

        // Handle any actions the bot suggests
        if (data.action) {
          this.handleBotAction(data.action);
        }
      } else {
        this.addMessage(
          "Sorry, I encountered an error. Please try again.",
          "bot",
        );
      }
    } catch (error) {
      this.removeTypingIndicator(typingId);
      // Fallback response when API is not available
      this.addMessage(this.getFallbackResponse(message), "bot");
    }
  },

  getFallbackResponse(message) {
    const msg = message.toLowerCase();
    if (msg.includes("help")) {
      return "I can help you navigate the app, search for content, manage tasks, compose emails, and more. What would you like to do?";
    } else if (msg.includes("task") || msg.includes("todo")) {
      return "Would you like me to open Tasks for you? You can create, view, and manage your tasks there.";
    } else if (msg.includes("email") || msg.includes("mail")) {
      return "I can help with email! Would you like to compose a new message or check your inbox?";
    } else if (
      msg.includes("calendar") ||
      msg.includes("meeting") ||
      msg.includes("schedule")
    ) {
      return "I can help with scheduling. Would you like to create an event or view your calendar?";
    }
    return (
      "I understand you're asking about: \"" +
      message +
      '". How can I assist you further?'
    );
  },

  addMessage(text, sender) {
    const msgDiv = document.createElement("div");
    msgDiv.className = `omnibox-message ${sender}`;
    msgDiv.innerHTML = `
                        <div class="message-avatar">${sender === "user" ? "👤" : "🤖"}</div>
                        <div class="message-content">${this.escapeHtml(text)}</div>
                    `;
    this.chatMessages.appendChild(msgDiv);
    this.chatMessages.scrollTop = this.chatMessages.scrollHeight;

    this.chatHistory.push({ role: sender, content: text });
  },

  addTypingIndicator() {
    const id = "typing-" + Date.now();
    const typingDiv = document.createElement("div");
    typingDiv.id = id;
    typingDiv.className = "omnibox-message bot typing";
    typingDiv.innerHTML = `
                        <div class="message-avatar">🤖</div>
                        <div class="message-content typing-indicator">
                            <span></span><span></span><span></span>
                        </div>
                    `;
    this.chatMessages.appendChild(typingDiv);
    this.chatMessages.scrollTop = this.chatMessages.scrollHeight;
    return id;
  },

  removeTypingIndicator(id) {
    const el = document.getElementById(id);
    if (el) el.remove();
  },

  handleBotAction(action) {
    if (action.navigate) {
      setTimeout(() => this.navigateTo(action.navigate), 1000);
    }
  },

  expandToFullChat() {
    this.close();
    const chatLink = document.querySelector('a[data-section="chat"]');
    if (chatLink) chatLink.click();
  },

  escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }
});
