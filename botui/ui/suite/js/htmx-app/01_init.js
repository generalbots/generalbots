// HTMX-based application initialization
// NOTE: Auth headers are now handled centrally by security-bootstrap.js
(function (window, document) {
  "use strict";
  var N = (document.__gb_mod_htmx_app = {});
  var S = (N.s = { reconnectAttempts: 0 });
  // Configuration
  const config = {
    wsUrl: "/ws",
    apiBase: "/api",
    reconnectDelay: 3000,
    maxReconnectAttempts: 5,
  };

  // State
  let wsConnection = null;

  // Initialize HTMX extensions
  function initHTMX() {
    // Configure HTMX
    htmx.config.defaultSwapStyle = "innerHTML";
    htmx.config.defaultSettleDelay = 100;
    htmx.config.timeout = 10000;

    // Handle errors globally
    document.body.addEventListener("htmx:responseError", (event) => {
      console.error("HTMX Error:", event.detail);
      showNotification("Connection error. Please try again.", "error");
    });

    // Handle before swap to prevent errors when target doesn't exist
    document.body.addEventListener("htmx:beforeSwap", (event) => {
      const target = event.detail.target;
      const status = event.detail.xhr?.status;
      const response = event.detail.serverResponse;
      const swapStyle = event.detail.swapStyle || "innerHTML";

      // If target doesn't exist or response is 404, prevent the swap
      if (!target || status === 404) {
        event.detail.shouldSwap = false;
        console.warn("HTMX swap prevented: target not found or 404 response");
        return;
      }

      // Check if target is actually in the DOM (prevents insertBefore errors)
      if (!document.body.contains(target)) {
        event.detail.shouldSwap = false;
        console.warn("HTMX swap prevented: target not in DOM");
        return;
      }

      // Check if target has a parent (required for most swap operations)
      if (!target.parentNode) {
        event.detail.shouldSwap = false;
        console.warn("HTMX swap prevented: target has no parent");
        return;
      }

      // Additional check: verify parentNode is still in DOM (race condition protection)
      if (
        !document.body.contains(target.parentNode) &&
        target.parentNode !== document.body &&
        target.parentNode !== document.documentElement
      ) {
        event.detail.shouldSwap = false;
        console.warn("HTMX swap prevented: target parent not in DOM");
        return;
      }

      // For swap styles that use insertBefore, verify the parent can accept children
      const insertBasedSwaps = [
        "outerHTML",
        "beforebegin",
        "afterbegin",
        "beforeend",
        "afterend",
      ];
      if (insertBasedSwaps.includes(swapStyle)) {
        try {
          // Verify we can actually perform DOM operations on the target
          if (
            swapStyle === "outerHTML" &&
            (!target.parentNode || !target.parentNode.contains(target))
          ) {
            event.detail.shouldSwap = false;
            console.warn(
              "HTMX swap prevented: outerHTML target detached from parent",
            );
            return;
          }
        } catch (e) {
          event.detail.shouldSwap = false;
          console.warn("HTMX swap prevented: DOM access error", e);
          return;
        }
      }

      // For empty responses, set empty content to prevent insertBefore errors
      if (!response || response.trim() === "") {
        event.detail.serverResponse = "<!-- empty -->";
        return;
      }

      // Validate that response is valid HTML before swapping
      // This prevents "Unexpected end of input" errors
      try {
        const trimmedResponse = response.trim();

        // Skip validation for comments
        if (
          trimmedResponse.startsWith("<!--") &&
          trimmedResponse.endsWith("-->")
        ) {
          return;
        }

        // Try to parse the response as HTML
        const parser = new DOMParser();
        const doc = parser.parseFromString(response, "text/html");

        // Check for parsing errors
        const parseError = doc.querySelector("parsererror");
        if (parseError) {
          console.warn(
            "HTMX swap: Response contains invalid HTML, wrapping in div",
          );
          event.detail.serverResponse = "<div>" + response + "</div>";
        }

        // Check if body is empty (happens with malformed HTML)
        if (
          doc.body &&
          doc.body.children.length === 0 &&
          doc.body.textContent.trim() === ""
        ) {
          if (trimmedResponse.length > 0) {
            console.warn(
              "HTMX swap: Response produced empty DOM, preserving as text",
            );
            event.detail.serverResponse = "<div>" + response + "</div>";
          }
        }
      } catch (e) {
        console.warn("HTMX swap: Error validating response HTML:", e);
        // Wrap potentially malformed content
        event.detail.serverResponse = "<div>" + response + "</div>";
      }
    });

    // Handle swap errors gracefully
    document.body.addEventListener("htmx:swapError", (event) => {
      console.error("HTMX swap error:", event.detail);
      // Don't show notification for swap errors - they're usually timing issues
      // Prevent the error from propagating
      event.preventDefault();
    });

    // Catch any uncaught HTMX errors related to DOM manipulation
    document.body.addEventListener("htmx:afterRequest", (event) => {
      // Clean up any orphaned requests
      const target = event.detail.target;
      if (target && !document.body.contains(target)) {
        console.warn(
          "HTMX afterRequest: target no longer in DOM, cleanup performed",
        );
      }
    });

    // Handle HTMX errors more gracefully
    document.body.addEventListener("htmx:onLoadError", (event) => {
      console.error("HTMX load error:", event.detail);
    });

    // Handle successful swaps
    document.body.addEventListener("htmx:afterSwap", (event) => {
      // Auto-scroll messages if in chat
      const messages = document.getElementById("messages");
      if (messages && event.detail.target === messages) {
        messages.scrollTop = messages.scrollHeight;
      }
    });

    // Handle WebSocket messages
    document.body.addEventListener("htmx:wsMessage", (event) => {
      handleWebSocketMessage(JSON.parse(event.detail.message));
    });

    // Handle WebSocket connection events
    document.body.addEventListener("htmx:wsConnecting", () => {
      updateConnectionStatus("connecting");
    });

    document.body.addEventListener("htmx:wsOpen", () => {
      updateConnectionStatus("connected");
    });

    document.body.addEventListener("htmx:wsClose", () => {
      updateConnectionStatus("disconnected");
      attemptReconnect();
    });
  }

  // Handle WebSocket messages
  function handleWebSocketMessage(message) {
    const messageType = message.type || message.event;

    if (messageType === "connected") {
      S.reconnectAttempts = 0;
    }

    // Debug logging
    console.log("handleWebSocketMessage called with:", { messageType, message });

    // Hide initial loading overlay when first bot message arrives
    if (window.hideLoadingOverlay) {
      setTimeout(window.hideLoadingOverlay, 300);
    }

    // Handle suggestions array from BotResponse
    if (message.suggestions && Array.isArray(message.suggestions) && message.suggestions.length > 0) {
      clearSuggestions();
      message.suggestions.forEach(suggestion => {
        addSuggestionButton(suggestion.text, suggestion.value || suggestion.text);
      });
    }

    switch (messageType) {
      case "message":
        appendMessage(message);
        break;
      case "notification":
        showNotification(message.text, message.severity);
        break;
      case "status":
        updateStatus(message);
        break;
      case "suggestion":
        addSuggestion(message.text);
        break;
      case "change_theme":
        console.log("Processing change_theme event, not appending to chat");
        if (message.data) {
          ThemeManager.setThemeFromServer(message.data);
        }
        return; // Don't append theme events to chat
      default:
        // Only append unknown message types to chat if they have text content
        if (message.text || message.content) {
          console.log("Unknown message type, treating as chat message:", messageType);
          appendMessage(message);
        } else {
          console.log("Unknown message type:", messageType, message);
        }
    }
  }

  // Clear all suggestions
  function clearSuggestions() {
    const suggestionsEl = document.getElementById("suggestions");
    if (suggestionsEl) {
      suggestionsEl.innerHTML = '';
      suggestionsEl.classList.remove('has-bot-suggestions');
    }
  }

  // Add suggestion button with value
  function addSuggestionButton(text, value) {
    const suggestionsEl = document.getElementById("suggestions");
    if (!suggestionsEl) return;

    const chip = document.createElement("button");
    chip.className = "suggestion-chip";
    chip.textContent = text;
    chip.setAttribute("hx-post", "/api/sessions/current/message");
    chip.setAttribute("hx-vals", JSON.stringify({ content: value }));
    chip.setAttribute("hx-target", "#messages");
    chip.setAttribute("hx-swap", "beforeend");

    suggestionsEl.appendChild(chip);
    suggestionsEl.classList.add('has-bot-suggestions');
    htmx.process(chip);
  }

  // Append message to chat
  function appendMessage(message) {
    const messagesEl = document.getElementById("messages");
    if (!messagesEl) return;

    const messageEl = document.createElement("div");
    messageEl.className = `message ${message.sender === "user" ? "user" : "bot"}`;
    
    // Render HTML if present, otherwise escape plain text
    const isHtml = /<[a-z][\s\S]*>/i.test(message.text);
    const content = isHtml ? message.text : escapeHtml(message.text);
    
    messageEl.innerHTML = `
            <div class="message-content">
                <span class="sender">${message.sender}</span>
                <span class="text">${content}</span>
                <span class="time">${formatTime(message.timestamp)}</span>
            </div>
        `;

    messagesEl.appendChild(messageEl);
    messagesEl.scrollTop = messagesEl.scrollHeight;
  }

  // Add suggestion chip
  function addSuggestion(text) {
    const suggestionsEl = document.getElementById("suggestions");
    if (!suggestionsEl) return;

    const chip = document.createElement("button");
    chip.className = "suggestion-chip";
    chip.textContent = text;
    chip.setAttribute("hx-post", "/api/sessions/current/message");
    chip.setAttribute("hx-vals", JSON.stringify({ content: text }));
    chip.setAttribute("hx-target", "#messages");
    chip.setAttribute("hx-swap", "beforeend");

    suggestionsEl.appendChild(chip);
    htmx.process(chip);
  }

  // Update connection status
  function updateConnectionStatus(status) {
    const statusEl = document.getElementById("connectionStatus");
    if (!statusEl) return;

    statusEl.className = `connection-status ${status}`;
    statusEl.textContent = status.charAt(0).toUpperCase() + status.slice(1);
  }

  // Update general status
  function updateStatus(message) {
    const statusEl = document.getElementById("status-" + message.id);
    if (statusEl) {
      statusEl.textContent = message.text;
      statusEl.className = `status ${message.severity}`;
    }
  }

  // Show notification
  function showNotification(text, type = "info") {
    const notification = document.createElement("div");
    notification.className = `notification ${type}`;
    notification.textContent = text;

    const container = document.getElementById("notifications") || document.body;
    container.appendChild(notification);

    setTimeout(() => {
      notification.classList.add("fade-out");
      setTimeout(() => notification.remove(), 300);
    }, 3000);
  }

  // Attempt to reconnect WebSocket
  function attemptReconnect() {
    if (S.reconnectAttempts >= config.maxReconnectAttempts) {
      showNotification("Connection lost. Please refresh the page.", "error");
      return;
    }

    S.reconnectAttempts++;
    setTimeout(() => {
      console.log(`Reconnection attempt ${S.reconnectAttempts}...`);
      htmx.trigger(document.body, "htmx:wsReconnect");
    }, config.reconnectDelay);
  }

  // Utility: Escape HTML
  function escapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  // Utility: Format timestamp
  function formatTime(timestamp) {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    return date.toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });
  }

  // Handle navigation

  N.config = config;
  N.initHTMX = initHTMX;
  N.handleWebSocketMessage = handleWebSocketMessage;
  N.clearSuggestions = clearSuggestions;
  N.addSuggestionButton = addSuggestionButton;
  N.appendMessage = appendMessage;
  N.addSuggestion = addSuggestion;
  N.updateConnectionStatus = updateConnectionStatus;
  N.updateStatus = updateStatus;
  N.showNotification = showNotification;
  N.attemptReconnect = attemptReconnect;
  N.escapeHtml = escapeHtml;
  N.formatTime = formatTime;
})(window, document);
