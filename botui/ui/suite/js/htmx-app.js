// HTMX-based application initialization
// NOTE: Auth headers are now handled centrally by security-bootstrap.js
(function () {
  "use strict";

  // Configuration
  const config = {
    wsUrl: "/ws",
    apiBase: "/api",
    reconnectDelay: 3000,
    maxReconnectAttempts: 5,
  };

  // State
  let reconnectAttempts = 0;
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
      reconnectAttempts = 0;
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
    if (reconnectAttempts >= config.maxReconnectAttempts) {
      showNotification("Connection lost. Please refresh the page.", "error");
      return;
    }

    reconnectAttempts++;
    setTimeout(() => {
      console.log(`Reconnection attempt ${reconnectAttempts}...`);
      htmx.trigger(document.body, "htmx:wsReconnect");
    }, config.reconnectDelay);
  }

  // Utility: Escape HTML
  function escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
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
  function initNavigation() {
    // Update active nav item on page change
    document.addEventListener("htmx:pushedIntoHistory", (event) => {
      const path = event.detail.path;
      updateActiveNav(path);
    });

    // Also listen for htmx:afterSwap to catch all navigation
    document.addEventListener("htmx:afterSwap", (event) => {
      setTimeout(() => {
        const path = window.location.hash || window.location.pathname;
        updateActiveNav(path);
      }, 10);
    });

    // Handle hash change
    window.addEventListener("hashchange", (event) => {
      updateActiveNav(window.location.hash);
    });

    // Handle browser back/forward
    window.addEventListener("popstate", (event) => {
      updateActiveNav(window.location.hash || window.location.pathname);
    });

    // Handle direct clicks on app tabs and app items
    document.addEventListener("click", (event) => {
      const appTab = event.target.closest(".app-tab");
      const appItem = event.target.closest(".app-item");

      if (appTab || appItem) {
        const element = appTab || appItem;
        const href = element.getAttribute("href");
        if (href) {
          // Immediately update active state on click
          updateActiveNav(href);
        }
      }
    });
  }

  // Get current section from URL
  function getCurrentSection() {
    const hash = window.location.hash;
    if (hash && hash.length > 1) {
      // Handle both #section and /#section formats
      return hash.replace(/^#\/?/, "").split("/")[0].split("?")[0];
    }
    return "chat";
  }

  // Update active navigation item and page title
  function updateActiveNav(path) {
    // Extract section name from path
    // Handles: "/#chat", "#chat", "/chat", "chat", "/#paper", "#paper"
    let section;
    if (path && path.length > 0) {
      // Remove leading /, #, or /# combinations
      section = path
        .replace(/^[/#]+/, "")
        .split("/")[0]
        .split("?")[0];
    }

    // Fallback to current URL hash if section is empty
    if (!section) {
      section = getCurrentSection();
    }

    // Set data-app attribute on body for CSS targeting (e.g., docs app styling)
    if (section) {
      document.body.setAttribute("data-app", section);
    }

    // First, remove ALL active classes from all tabs, items, and apps button
    document.querySelectorAll(".app-tab.active").forEach((item) => {
      item.classList.remove("active");
    });
    document.querySelectorAll(".app-item.active").forEach((item) => {
      item.classList.remove("active");
    });

    // Remove active from apps button
    const appsButton = document.getElementById("appsButton");
    if (appsButton) {
      appsButton.classList.remove("active");
    }

    // Check if section exists in the main header tabs
    let foundInHeaderTabs = false;
    document.querySelectorAll(".app-tab").forEach((item) => {
      const dataSection = item.getAttribute("data-section");
      const href = item.getAttribute("href");
      const itemSection = dataSection || (href ? href.replace(/^#/, "") : "");
      if (itemSection === section) {
        item.classList.add("active");
        foundInHeaderTabs = true;
      }
    });

    // Update app items in launcher dropdown (always mark the current section)
    document.querySelectorAll(".app-item").forEach((item) => {
      const href = item.getAttribute("href");
      const dataSection = item.getAttribute("data-section");
      const itemSection = dataSection || (href ? href.replace(/^#/, "") : "");
      if (itemSection === section) {
        item.classList.add("active");
      }
    });

    // If section is NOT in header tabs, select the apps button instead
    if (!foundInHeaderTabs && appsButton) {
      appsButton.classList.add("active");
    }

    // Update page title
    const sectionName = section.charAt(0).toUpperCase() + section.slice(1);
    document.title = sectionName + " - General Bots";
  }

  // Initialize keyboard shortcuts
  function initKeyboardShortcuts() {
    document.addEventListener("keydown", (e) => {
      // Send message on Enter (when in input)
      if (e.key === "Enter" && !e.shiftKey) {
        const input = document.getElementById("messageInput");
        if (input && document.activeElement === input) {
          e.preventDefault();
          const form = input.closest("form");
          if (form) {
            htmx.trigger(form, "submit");
          }
        }
      }

      // Focus input on /
      if (e.key === "/" && document.activeElement.tagName !== "INPUT") {
        e.preventDefault();
        const input = document.getElementById("messageInput");
        if (input) input.focus();
      }

      // Escape to blur input
      if (e.key === "Escape") {
        const input = document.getElementById("messageInput");
        if (input && document.activeElement === input) {
          input.blur();
        }
      }
    });
  }

  // Initialize scroll behavior
  function initScrollBehavior() {
    const scrollBtn = document.getElementById("scrollToBottom");
    const messages = document.getElementById("messages");

    if (scrollBtn && messages) {
      // Show/hide scroll button
      messages.addEventListener("scroll", () => {
        const isAtBottom =
          messages.scrollHeight - messages.scrollTop <=
          messages.clientHeight + 100;
        scrollBtn.style.display = isAtBottom ? "none" : "flex";
      });

      // Scroll to bottom on click
      scrollBtn.addEventListener("click", () => {
        messages.scrollTo({
          top: messages.scrollHeight,
          behavior: "smooth",
        });
      });
    }
  }

  // Initialize theme if ThemeManager exists
  function initTheme() {
    if (window.ThemeManager) {
      ThemeManager.init();
    }
  }

  // Main initialization
  function init() {
    console.log("Initializing HTMX application...");

    // Initialize HTMX
    initHTMX();

    // Initialize navigation
    initNavigation();

    // Initialize keyboard shortcuts
    initKeyboardShortcuts();

    // Initialize scroll behavior
    initScrollBehavior();

    // Initialize theme
    initTheme();

    // Set initial active nav based on hash or default to chat
    updateActiveNav(window.location.hash || "#chat");

    console.log("HTMX application initialized");
  }

  // Wait for DOM and HTMX to be ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  // Expose public API
  window.BotServerApp = {
    showNotification,
    appendMessage,
    updateConnectionStatus,
    config,
  };
})();
