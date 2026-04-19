// Simple initialization for HTMX app
// Note: Chat module is self-contained in chat.html
// ==========================================
// SUITE CORE (Plugin Architecture)
// ==========================================
window.Suite = {
  apps: new Map(),

  registerApp(id, config) {
    console.log(`[Suite] Registering app: ${id}`);
    this.apps.set(id, {
      id,
      icon: "📦",
      title: id,
      description: "",
      actions: [],
      searchable: true,
      ...config,
    });

    // Trigger UI update if Omnibox is initialized
    if (typeof Omnibox !== "undefined" && Omnibox.isActive) {
      Omnibox.updateActions();
    }
  },

  getApp(id) {
    return this.apps.get(id);
  },

  getAllApps() {
    return Array.from(this.apps.values());
  },

  // Helper to get actions for a specific context
  getContextActions(contextId) {
    const app = this.apps.get(contextId);
    return app ? app.actions : null;
  },
};

// ==========================================
// OMNIBOX (Search + Chat) Functionality
// ==========================================
const Omnibox = {
  isActive: false,
  isChatMode: false,
  chatHistory: [],
  selectedIndex: 0,

  init() {
    this.omnibox = document.getElementById("omnibox");
    this.input = document.getElementById("omniboxInput");
    this.panel = document.getElementById("omniboxPanel");
    this.backdrop = document.getElementById("omniboxBackdrop");
    this.results = document.getElementById("omniboxResults");
    this.chat = document.getElementById("omniboxChat");
    this.chatMessages = document.getElementById("omniboxChatMessages");
    this.chatInput = document.getElementById("omniboxChatInput");
    this.modeToggle = document.getElementById("omniboxModeToggle");

    // Only bind events if all required elements exist
    if (this.input && this.backdrop) {
      this.bindEvents();
    }
  },

  bindEvents() {
    // Defensive: ensure elements exist before binding
    if (!this.input || !this.backdrop) {
      console.warn("[Omnibox] Required elements not found, skipping event binding");
      return;
    }

    // Input focus/blur
    this.input.addEventListener("focus", () => this.open());
    this.backdrop.addEventListener("click", () => this.close());

    // Input typing
    this.input.addEventListener("input", (e) =>
      this.handleInput(e.target.value),
    );

    // Keyboard navigation
    this.input.addEventListener("keydown", (e) => this.handleKeydown(e));
    this.chatInput?.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        this.sendMessage();
      }
    });

    // Mode toggle
    this.modeToggle.addEventListener("click", (e) => {
      e.stopPropagation();
      this.toggleChatMode();
    });

    // Action buttons
    document.querySelectorAll(".omnibox-action").forEach((btn) => {
      btn.addEventListener("click", () => this.handleAction(btn));
    });

    // Send button
    document
      .getElementById("omniboxSendBtn")
      ?.addEventListener("click", () => this.sendMessage());

    // Back button
    document
      .getElementById("omniboxBackBtn")
      ?.addEventListener("click", () => this.showResults());

    // Expand button
    document
      .getElementById("omniboxExpandBtn")
      ?.addEventListener("click", () => this.expandToFullChat());

    // Global shortcut (Cmd+K / Ctrl+K)
    document.addEventListener("keydown", (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        this.input.focus();
        this.open();
      }
      if (e.key === "Escape" && this.isActive) {
        this.close();
      }
    });
  },

  open() {
    this.isActive = true;
    this.omnibox.classList.add("active");
    this.updateActions();
  },

  close() {
    this.isActive = false;
    this.omnibox.classList.remove("active");
    this.input.blur();
  },

  handleInput(value) {
    if (value.trim()) {
      this.searchContent(value);
    } else {
      this.showDefaultActions();
    }
  },

  handleKeydown(e) {
    const actions = document.querySelectorAll(
      '.omnibox-action:not([style*="display: none"])',
    );

    if (e.key === "ArrowDown") {
      e.preventDefault();
      this.selectedIndex = Math.min(this.selectedIndex + 1, actions.length - 1);
      this.updateSelection(actions);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
      this.updateSelection(actions);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const selected = actions[this.selectedIndex];
      if (selected) {
        this.handleAction(selected);
      } else if (this.input.value.trim()) {
        // Start chat with the query
        this.startChatWithQuery(this.input.value);
      }
    }
  },

  updateSelection(actions) {
    actions.forEach((a, i) => {
      a.classList.toggle("selected", i === this.selectedIndex);
    });
  },

  updateActions() {
    const currentApp = this.getCurrentApp();
    const actionsContainer = document.getElementById("omniboxActions");

    const contextActions = {
      chat: [
        {
          icon: "💬",
          text: "New conversation",
          action: "chat",
        },
        {
          icon: "📋",
          text: "View history",
          action: "navigate",
          target: "chat",
        },
      ],
      mail: [
        {
          icon: "✉️",
          text: "Compose email",
          action: "chat",
          query: "Help me compose an email",
        },
        {
          icon: "📥",
          text: "Check inbox",
          action: "navigate",
          target: "mail",
        },
      ],
      tasks: [
        {
          icon: "✅",
          text: "Create task",
          action: "chat",
          query: "Create a new task",
        },
        {
          icon: "📋",
          text: "Show my tasks",
          action: "navigate",
          target: "tasks",
        },
      ],
      calendar: [
        {
          icon: "📅",
          text: "Schedule event",
          action: "chat",
          query: "Schedule a meeting",
        },
        {
          icon: "📆",
          text: "View calendar",
          action: "navigate",
          target: "calendar",
        },
      ],
      sheet: [
        {
          icon: "📊",
          text: "Analyze data",
          action: "chat",
          query: "Analyze this data",
        },
        {
          icon: "📈",
          text: "Create chart",
          action: "chat",
          query: "Create a chart for this data",
        },
        {
          icon: "💲",
          text: "Format currency",
          action: "chat",
          query: "Format as currency",
        },
      ],
      paper: [
        {
          icon: "📝",
          text: "Summarize",
          action: "chat",
          query: "Summarize this document",
        },
        {
          icon: "✨",
          text: "Fix grammar",
          action: "chat",
          query: "Fix grammar and spelling",
        },
        {
          icon: "👔",
          text: "Make formal",
          action: "chat",
          query: "Make the tone more formal",
        },
      ],
      default: [
        {
          icon: "💬",
          text: "Chat with Bot",
          action: "chat",
        },
        {
          icon: "🔍",
          text: "Search everywhere",
          action: "search",
        },
      ],
    };

    // Check for plugin actions first
    const pluginActions = window.Suite.getContextActions(currentApp);

    let actions;
    if (pluginActions && pluginActions.length > 0) {
      actions = pluginActions;
    } else {
      actions = contextActions[currentApp] || contextActions.default;
    }

    // Add navigation shortcuts
    const navActions = [
      {
        icon: "📱",
        text: "Open Chat",
        action: "navigate",
        target: "chat",
      },
      {
        icon: "✉️",
        text: "Open Mail",
        action: "navigate",
        target: "mail",
      },
      {
        icon: "✓",
        text: "Open Tasks",
        action: "navigate",
        target: "tasks",
      },
      {
        icon: "📅",
        text: "Open Calendar",
        action: "navigate",
        target: "calendar",
      },
    ];

    actionsContainer.innerHTML = actions
      .concat(navActions)
      .map(
        (a, i) => `
                        <button class="omnibox-action ${i === 0 ? "selected" : ""}"
                                data-action="${a.action}"
                                data-target="${a.target || ""}"
                                data-query="${a.query || ""}">
                            <span class="action-icon">${a.icon}</span>
                            <span class="action-text">${a.text}</span>
                            ${i === 0 ? "<kbd>↵</kbd>" : ""}
                        </button>
                    `,
      )
      .join("");

    // Rebind events
    actionsContainer.querySelectorAll(".omnibox-action").forEach((btn) => {
      btn.addEventListener("click", () => this.handleAction(btn));
    });

    this.selectedIndex = 0;
  },

  getCurrentApp() {
    const hash = window.location.hash.replace("#", "");
    return hash || "default";
  },

  handleAction(btn) {
    const action = btn.dataset.action;
    const target = btn.dataset.target;
    const query = btn.dataset.query;

    if (action === "chat") {
      if (query) {
        this.startChatWithQuery(query);
      } else {
        this.showChat();
      }
    } else if (action === "navigate" && target) {
      this.navigateTo(target);
    } else if (action === "search") {
      this.input.focus();
    }
  },

  navigateTo(target) {
    this.close();
    const link = document.querySelector(`a[data-section="${target}"]`);
    if (link) {
      link.click();
    }
  },

  toggleChatMode() {
    this.isChatMode = !this.isChatMode;
    this.omnibox.classList.toggle("chat-mode", this.isChatMode);

    if (this.isChatMode) {
      this.input.placeholder = "Ask me anything...";
      this.showChat();
    } else {
      this.input.placeholder = "Search or ask anything...";
      this.showResults();
    }
  },

  showChat() {
    this.results.style.display = "none";
    this.chat.style.display = "flex";
    this.isChatMode = true;
    this.omnibox.classList.add("chat-mode");
    this.chatInput?.focus();
  },

  showResults() {
    this.chat.style.display = "none";
    this.results.style.display = "block";
    this.isChatMode = false;
    this.omnibox.classList.remove("chat-mode");
    this.input.focus();
  },

  showDefaultActions() {
    document.getElementById("searchResultsSection").style.display = "none";
    this.updateActions();
  },

  searchContent(query) {
    // Show search results section
    const resultsSection = document.getElementById("searchResultsSection");
    const resultsList = document.getElementById("searchResultsList");

    resultsSection.style.display = "block";

    // Update first action to be "Ask about: query"
    const actionsContainer = document.getElementById("omniboxActions");
    const firstAction = actionsContainer.querySelector(".omnibox-action");
    if (firstAction) {
      firstAction.dataset.action = "chat";
      firstAction.dataset.query = query;
      firstAction.querySelector(".action-icon").textContent = "💬";
      firstAction.querySelector(".action-text").textContent =
        `Ask: "${query.substring(0, 30)}${query.length > 30 ? "..." : ""}"`;
    }

    // Simple client-side search of navigation items
    const searchResults = this.performSearch(query);
    resultsList.innerHTML =
      searchResults
        .map(
          (r) => `
                        <button class="omnibox-result" data-action="navigate" data-target="${r.target}">
                            <span class="result-icon">${r.icon}</span>
                            <div class="result-content">
                                <span class="result-title">${r.title}</span>
                                <span class="result-desc">${r.description}</span>
                            </div>
                        </button>
                    `,
        )
        .join("") ||
      '<div class="no-results">No results found. Try asking the bot!</div>';

    // Bind click events
    resultsList.querySelectorAll(".omnibox-result").forEach((btn) => {
      btn.addEventListener("click", () => this.navigateTo(btn.dataset.target));
    });
  },

  performSearch(query) {
    const q = query.toLowerCase();
    const items = [
      {
        target: "chat",
        icon: "💬",
        title: "Chat",
        description: "Chat with the bot",
      },
      {
        target: "mail",
        icon: "✉️",
        title: "Mail",
        description: "Email inbox",
      },
      {
        target: "tasks",
        icon: "✓",
        title: "Tasks",
        description: "Task management",
      },
      {
        target: "calendar",
        icon: "📅",
        title: "Calendar",
        description: "Schedule and events",
      },
      {
        target: "drive",
        icon: "📁",
        title: "Drive",
        description: "File storage",
      },
      {
        target: "paper",
        icon: "📄",
        title: "Documents",
        description: "Document editor",
      },
      {
        target: "sheet",
        icon: "📊",
        title: "Sheet",
        description: "Spreadsheets",
      },
      {
        target: "slides",
        icon: "📽️",
        title: "Slides",
        description: "Presentations",
      },
      {
        target: "editor",
        icon: "📝",
        title: "Editor",
        description: "Code & text editor",
      },
      {
        target: "designer",
        icon: "🔷",
        title: "Designer",
        description: "Visual .bas editor",
      },
      {
        target: "meet",
        icon: "📹",
        title: "Meet",
        description: "Video meetings",
      },
      {
        target: "research",
        icon: "🔍",
        title: "Research",
        description: "Research assistant",
      },
      {
        target: "analytics",
        icon: "📊",
        title: "Analytics",
        description: "Data analytics",
      },
      {
        target: "settings",
        icon: "⚙️",
        title: "Settings",
        description: "App settings",
      },
    ];

    // Add plugin apps
    const pluginApps = window.Suite.getAllApps()
      .filter((app) => app.searchable)
      .map((app) => ({
        target: app.id,
        icon: app.icon || "📦",
        title: app.title || app.id,
        description: app.description || "App plugin",
      }));

    const allItems = items.concat(pluginApps);

    return allItems.filter(
      (item) =>
        item.title.toLowerCase().includes(q) ||
        item.description.toLowerCase().includes(q),
    );
  },

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
  },
};

// Initialize Omnibox when DOM is ready
document.addEventListener("DOMContentLoaded", () => {
  // Detect bot name from pathname (e.g., /bot/cristo -> bot_name = "cristo", /edu -> bot_name = "edu")
  const detectBotFromPath = () => {
    const pathname = window.location.pathname;
    // Remove leading/trailing slashes and split
    const segments = pathname.replace(/^\/|\/$/g, "").split("/");

    // Handle /bot/{bot_name} pattern
    if (segments[0] === "bot" && segments[1]) {
      return segments[1];
    }

    // For other patterns, use first segment if it's not a known route
    const firstSegment = segments[0];
    const knownRoutes = ["suite", "auth", "api", "static", "public", "bot"];
    if (firstSegment && !knownRoutes.includes(firstSegment)) {
      return firstSegment;
    }
    return "default";
  };

  // Set global bot name
  window.__INITIAL_BOT_NAME__ = detectBotFromPath();
  console.log(`🤖 Bot detected from path: ${window.__INITIAL_BOT_NAME__}`);

  // Check if bot is public to skip authentication
  const checkBotPublicStatus = async () => {
    try {
      const botName = window.__INITIAL_BOT_NAME__;
      if (!botName || botName === "default") return;

      const response = await fetch(
        `/api/bot/config?bot_name=${encodeURIComponent(botName)}`,
      );
      if (response.ok) {
        const config = await response.json();
        if (config.public === true) {
          window.__BOT_IS_PUBLIC__ = true;
          console.log(
            `✅ Bot '${botName}' is public - authentication not required`,
          );
        }
      }
    } catch (e) {
      console.warn("Failed to check bot public status:", e);
    }
  };

  Omnibox.init();
  console.log("🚀 Initializing General Bots with HTMX...");

  // Check bot public status early
  checkBotPublicStatus();

  // Provide a global function to hide the loading overlay
  window.hideLoadingOverlay = function() {
    const loadingOverlay = document.getElementById("loadingOverlay");
    if (loadingOverlay && !loadingOverlay.classList.contains("hidden")) {
      loadingOverlay.classList.add("hidden");
    }
  };

  // Failsafe: hide after 10 seconds if no message arrives
  setTimeout(window.hideLoadingOverlay, 10000);

  // Simple apps menu handling
  const appsBtn = document.getElementById("appsButton");
  const appsDropdown = document.getElementById("appsDropdown");
  const settingsBtn = document.getElementById("settingsBtn");
  const settingsPanel = document.getElementById("settingsPanel");

  if (appsBtn && appsDropdown) {
    appsBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      const isOpen = appsDropdown.classList.toggle("show");
      appsBtn.setAttribute("aria-expanded", isOpen);
      // Close settings panel
      if (settingsPanel) settingsPanel.classList.remove("show");
    });

    document.addEventListener("click", (e) => {
      if (!appsDropdown.contains(e.target) && !appsBtn.contains(e.target)) {
        appsDropdown.classList.remove("show");
        appsBtn.setAttribute("aria-expanded", "false");
      }
    });
  }

  // Settings panel handling
  if (settingsBtn && settingsPanel) {
    settingsBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      const isOpen = settingsPanel.classList.toggle("show");
      settingsBtn.setAttribute("aria-expanded", isOpen);
      // Close apps dropdown
      if (appsDropdown) appsDropdown.classList.remove("show");
    });

    document.addEventListener("click", (e) => {
      if (
        !settingsPanel.contains(e.target) &&
        !settingsBtn.contains(e.target)
      ) {
        settingsPanel.classList.remove("show");
        settingsBtn.setAttribute("aria-expanded", "false");
      }
    });
  }

  // Theme selection handling
  const themeOptions = document.querySelectorAll(".theme-option");
  const savedTheme = localStorage.getItem("gb-theme") || "sentient";

  // Apply saved theme
  document.body.setAttribute("data-theme", savedTheme);
  document
    .querySelector(`.theme-option[data-theme="${savedTheme}"]`)
    ?.classList.add("active");

  themeOptions.forEach((option) => {
    option.addEventListener("click", () => {
      const theme = option.getAttribute("data-theme");
      document.body.setAttribute("data-theme", theme);
      localStorage.setItem("gb-theme", theme);
      themeOptions.forEach((o) => o.classList.remove("active"));
      option.classList.add("active");

      // Update theme-color meta tag
      const themeColors = {
        dark: "#3b82f6",
        light: "#3b82f6",
        purple: "#a855f7",
        green: "#22c55e",
        orange: "#f97316",
        sentient: "#d4f505",
      };
      const metaTheme = document.querySelector('meta[name="theme-color"]');
      if (metaTheme) {
        metaTheme.setAttribute("content", themeColors[theme] || "#d4f505");
      }
    });
  });

  // List of sections that appear in header tabs
  const headerTabSections = [
    "chat",
    "paper",
    "sheet",
    "slides",
    "mail",
    "calendar",
    "drive",
    "tasks",
  ];

  // Update active states for navigation (single selection only)
  function updateNavigationActive(section) {
    // Remove all active states first
    document
      .querySelectorAll(".app-tab")
      .forEach((t) => t.classList.remove("active"));
    document
      .querySelectorAll(".app-item")
      .forEach((i) => i.classList.remove("active"));
    appsBtn.classList.remove("active");

    // Check if section is in header tabs
    const isInHeaderTabs = headerTabSections.includes(section);

    // Activate the matching app-tab if in header
    if (isInHeaderTabs) {
      const headerTab = document.querySelector(
        `.app-tab[data-section="${section}"]`,
      );
      if (headerTab) {
        headerTab.classList.add("active");
      }
    } else {
      // Section is NOT in header tabs, activate apps button
      appsBtn.classList.add("active");
    }

    // Always mark the app-item in dropdown as active
    const appItem = document.querySelector(
      `.app-item[data-section="${section}"]`,
    );
    if (appItem) {
      appItem.classList.add("active");
    }
  }

  // Handle app item clicks - update active state
  document.querySelectorAll(".app-item").forEach((item) => {
    item.addEventListener("click", function () {
      const section = this.getAttribute("data-section");
      updateNavigationActive(section);
      appsDropdown.classList.remove("show");
      appsBtn.setAttribute("aria-expanded", "false");
    });
  });

  // Handle app tab clicks
  document.querySelectorAll(".app-tab").forEach((tab) => {
    tab.addEventListener("click", function () {
      const section = this.getAttribute("data-section");
      updateNavigationActive(section);
    });
  });

  // Track currently loaded section to prevent duplicate loads
  let currentLoadedSection = null;
  let isLoadingSection = false;
  let pendingLoadTimeout = null;

  // Handle hash navigation
  function handleHashChange(fromHtmxSwap = false) {
    const hash = window.location.hash.slice(1) || "chat";

    // Skip if already loaded this section or currently loading
    if (currentLoadedSection === hash || isLoadingSection) {
      return;
    }

    // If this was triggered by HTMX swap, just update tracking
    if (fromHtmxSwap) {
      currentLoadedSection = hash;
      return;
    }

    updateNavigationActive(hash);

    // Abort any pending load timeout
    if (pendingLoadTimeout) {
      clearTimeout(pendingLoadTimeout);
      pendingLoadTimeout = null;
    }

    // Abort any in-flight HTMX requests for main-content
    const mainContent = document.getElementById("main-content");
    if (mainContent) {
      try {
        htmx.trigger(mainContent, "htmx:abort");
      } catch (e) {
        // Ignore abort errors
      }
    }

    // Validate target exists before triggering HTMX load
    if (!mainContent) {
      console.warn("handleHashChange: #main-content not found, skipping load");
      return;
    }

    // Check if main-content is in the DOM
    if (!document.body.contains(mainContent)) {
      console.warn("handleHashChange: #main-content not in DOM, skipping load");
      return;
    }

    // Verify main-content has a valid parent (prevents insertBefore errors)
    if (!mainContent.parentNode) {
      console.warn(
        "handleHashChange: #main-content has no parent, skipping load",
      );
      return;
    }

    // Debounce the load to prevent rapid double-requests
    pendingLoadTimeout = setTimeout(() => {
      // Re-check if section changed during debounce
      const currentHash = window.location.hash.slice(1) || "chat";
      if (currentLoadedSection === currentHash) {
        return;
      }

      // Trigger HTMX load
      const appItem = document.querySelector(
        `.app-item[data-section="${currentHash}"]`,
      );
      if (appItem) {
        const hxGet = appItem.getAttribute("hx-get");
        if (hxGet) {
          try {
            isLoadingSection = true;
            currentLoadedSection = currentHash;
            htmx.ajax("GET", hxGet, {
              target: "#main-content",
              swap: "innerHTML",
            });
          } catch (e) {
            console.warn("handleHashChange: HTMX ajax error:", e);
            currentLoadedSection = null;
            isLoadingSection = false;
          }
        }
      }
    }, 50);
  }

  // Listen for HTMX swaps to track loaded sections and prevent duplicates
  document.body.addEventListener("htmx:afterSwap", (event) => {
    if (event.detail.target && event.detail.target.id === "main-content") {
      const hash = window.location.hash.slice(1) || "chat";
      currentLoadedSection = hash;
      isLoadingSection = false;
    }
  });

  // Reset tracking on swap errors
  document.body.addEventListener("htmx:swapError", (event) => {
    if (event.detail.target && event.detail.target.id === "main-content") {
      isLoadingSection = false;
    }
  });

  // Also listen for response errors
  document.body.addEventListener("htmx:responseError", (event) => {
    if (event.detail.target && event.detail.target.id === "main-content") {
      isLoadingSection = false;
      currentLoadedSection = null;
    }
  });

  // Load initial content based on hash or default to chat
  window.addEventListener("hashchange", handleHashChange);

  // Initial load - wait for HTMX to be ready
  function initialLoad() {
    if (typeof htmx !== "undefined" && htmx.ajax) {
      handleHashChange();
    } else {
      setTimeout(initialLoad, 50);
    }
  }

  // Skip SPA initialization on auth pages (login, register, etc.) and desktop
  if (window.location.pathname.startsWith("/auth/")) {
    console.log("[SPA] Skipping initialization on auth page");
  } else if (document.getElementById('desktop-content')) {
    console.log("[SPA] Skipping initialization on desktop page");
  } else if (document.readyState === "complete") {
    setTimeout(initialLoad, 50);
  } else {
    window.addEventListener("load", () => {
      setTimeout(initialLoad, 50);
    });
  }

  // ==========================================================================
  // GBAlerts - Global Notification System
  // ==========================================================================
  window.GBAlerts = (function () {
    const notifications = [];
    const badge = document.getElementById("notificationsBadge");
    const list = document.getElementById("notificationsList");
    const btn = document.getElementById("notificationsBtn");
    const panel = document.getElementById("notificationsPanel");
    const clearBtn = document.getElementById("clearNotificationsBtn");

    function updateBadge() {
      if (badge) {
        if (notifications.length > 0) {
          badge.textContent =
            notifications.length > 99 ? "99+" : notifications.length;
          badge.style.display = "flex";
        } else {
          badge.style.display = "none";
        }
      }
    }

    function renderList() {
      if (!list) return;
      if (notifications.length === 0) {
        list.innerHTML =
          '<div class="notifications-empty"><span>🔔</span><p>No notifications</p></div>';
      } else {
        list.innerHTML = notifications
          .map(
            (n, i) => `
                                <div class="notification-item notification-${n.type}" data-index="${i}">
                                    <div class="notification-icon">${n.icon || "📢"}</div>
                                    <div class="notification-content">
                                        <div class="notification-title">${n.title}</div>
                                        <div class="notification-message">${n.message || ""}</div>
                                        <div class="notification-time">${n.time}</div>
                                    </div>
                                    ${n.action ? `<button class="notification-action" onclick="GBAlerts.handleAction(${i})">${n.actionText || "Open"}</button>` : ""}
                                    <button class="notification-dismiss" onclick="GBAlerts.dismiss(${i})">×</button>
                                </div>
                            `,
          )
          .join("");
      }
    }

    function add(notification) {
      notifications.unshift({
        ...notification,
        time: new Date().toLocaleTimeString(),
      });
      updateBadge();
      renderList();
      // Auto-open panel when new notification arrives
      if (panel) {
        panel.classList.add("show");
        if (btn) btn.setAttribute("aria-expanded", "true");
      }
    }

    function dismiss(index) {
      notifications.splice(index, 1);
      updateBadge();
      renderList();
    }

    function clearAll() {
      notifications.length = 0;
      updateBadge();
      renderList();
    }

    function handleAction(index) {
      const n = notifications[index];
      if (n && n.action) {
        if (typeof n.action === "function") {
          n.action();
        } else if (typeof n.action === "string") {
          window.open(n.action, "_blank");
        }
      }
      dismiss(index);
    }

    // Toggle panel
    if (btn && panel) {
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const isOpen = panel.classList.toggle("show");
        btn.setAttribute("aria-expanded", isOpen);
      });

      document.addEventListener("click", (e) => {
        if (!panel.contains(e.target) && !btn.contains(e.target)) {
          panel.classList.remove("show");
          btn.setAttribute("aria-expanded", "false");
        }
      });
    }

    if (clearBtn) {
      clearBtn.addEventListener("click", clearAll);
    }

    // Convenience methods for common notifications
    return {
      add,
      dismiss,
      clearAll,
      handleAction,
      taskCompleted: function (title, url) {
        add({
          type: "success",
          icon: "✅",
          title: "Task Completed",
          message: title,
          action: url,
          actionText: "Open App",
        });
      },
      taskFailed: function (title, error) {
        add({
          type: "error",
          icon: "❌",
          title: "Task Failed",
          message: title + (error ? ": " + error : ""),
        });
      },
      info: function (title, message) {
        add({
          type: "info",
          icon: "ℹ️",
          title: title,
          message: message,
        });
      },
      warning: function (title, message) {
        add({
          type: "warning",
          icon: "⚠️",
          title: title,
          message: message,
        });
      },
      connectionStatus: function (status, message) {
        add({
          type:
            status === "connected"
              ? "success"
              : status === "disconnected"
                ? "error"
                : "warning",
          icon:
            status === "connected"
              ? "🟢"
              : status === "disconnected"
                ? "🔴"
                : "🟡",
          title:
            "Connection " + status.charAt(0).toUpperCase() + status.slice(1),
          message: message || "",
        });
      },
    };
  })();

  // Keyboard shortcuts
  document.addEventListener("keydown", (e) => {
    // Alt + number for quick app switching
    if (e.altKey && !e.ctrlKey && !e.shiftKey) {
      const num = parseInt(e.key);
      if (num >= 1 && num <= 9) {
        const items = document.querySelectorAll(".app-item");
        if (items[num - 1]) {
          items[num - 1].click();
          e.preventDefault();
        }
      }
    }

    // Alt + A to open apps menu
    if (e.altKey && e.key.toLowerCase() === "a") {
      appsBtn.click();
      e.preventDefault();
    }
  });
});

// User Profile Loading
(function () {
  function updateUserUI(user) {
    if (!user) return;

    const userName = document.getElementById("userName");
    const userEmail = document.getElementById("userEmail");
    const userAvatar = document.getElementById("userAvatar");
    const userAvatarLarge = document.getElementById("userAvatarLarge");
    const authAction = document.getElementById("authAction");
    const authText = document.getElementById("authText");
    const authIcon = document.getElementById("authIcon");
    const settingsBtn = document.getElementById("settingsBtn");
    const appsButton = document.getElementById("appsButton");
    const notificationsBtn = document.getElementById("notificationsBtn");

    const displayName =
      user.display_name || user.first_name || user.username || "User";
    const email = user.email || "";
    const initial = (displayName.charAt(0) || "U").toUpperCase();

    console.log("Updating user UI:", displayName, email);

    if (userName) userName.textContent = displayName;
    if (userEmail) userEmail.textContent = email;
    if (userAvatar) {
      const avatarSpan = userAvatar.querySelector("span");
      if (avatarSpan) avatarSpan.textContent = initial;
    }
    if (userAvatarLarge) userAvatarLarge.textContent = initial;

    if (authAction) {
      authAction.href = "#";
      authAction.onclick = function (e) {
        e.preventDefault();
        fetch("/api/auth/logout", {
          method: "POST",
        }).finally(function () {
          localStorage.removeItem("gb-access-token");
          localStorage.removeItem("gb-refresh-token");
          localStorage.removeItem("gb-user-data");
          sessionStorage.removeItem("gb-access-token");
          window.location.href = "/auth/login.html";
        });
      };
      authAction.style.color = "var(--error)";
    }
    if (authText) authText.textContent = "Sign out";
    if (authIcon) {
      authIcon.innerHTML =
        '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path><polyline points="16 17 21 12 16 7"></polyline><line x1="21" y1="12" x2="9" y2="12"></line>';
    }

    if (settingsBtn) settingsBtn.style.display = "";
    if (appsButton) appsButton.style.display = "";
    if (notificationsBtn) notificationsBtn.style.display = "";

    // Show omnibox (search bar) when signed in
    const omnibox = document.getElementById("omnibox");
    if (omnibox) omnibox.style.display = "";

    // Show Drive, Tasks, CRM, and Calendar navigation when signed in (all instances)
    const driveTabs = document.querySelectorAll('[data-section="drive"]');
    const tasksTabs = document.querySelectorAll('[data-section="tasks"]');
    const crmTabs = document.querySelectorAll('[data-section="crm"]');
    const calendarTabs = document.querySelectorAll('[data-section="calendar"]');

    driveTabs.forEach(tab => tab.style.display = "");
    tasksTabs.forEach(tab => tab.style.display = "");
    crmTabs.forEach(tab => tab.style.display = "");
    calendarTabs.forEach(tab => tab.style.display = "");
  }

  function loadUserProfile() {
    var token =
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token");
    
    if (!token) {
      console.log("No auth token found - user is signed out");
      updateSignedOutUI();
      return;
    }

    console.log(
      "Loading user profile with token:",
      token.substring(0, 10) + "...",
    );

    fetch("/api/auth/me", {
      headers: { Authorization: "Bearer " + token },
    })
      .then(function (res) {
        if (!res.ok) {
          console.log("User not authenticated");
          updateSignedOutUI();
          throw new Error("Not authenticated");
        }
        return res.json();
      })
      .then(function (user) {
        console.log("User profile loaded:", user);
        updateUserUI(user);
        localStorage.setItem("gb-user-data", JSON.stringify(user));
      })
      .catch(function (err) {
        console.log("Failed to load user profile:", err);
        updateSignedOutUI();
      });
  }

  function updateSignedOutUI() {
    const userName = document.getElementById("userName");
    const userEmail = document.getElementById("userEmail");
    const userAvatar = document.getElementById("userAvatar");
    const userAvatarLarge = document.getElementById("userAvatarLarge");
    const authAction = document.getElementById("authAction");
    const authText = document.getElementById("authText");
    const authIcon = document.getElementById("authIcon");
    const settingsBtn = document.getElementById("settingsBtn");
    const appsButton = document.getElementById("appsButton");
    const notificationsBtn = document.getElementById("notificationsBtn");

    if (userName) userName.textContent = "User";
    if (userEmail) userEmail.textContent = "user@example.com";
    if (userAvatar) {
      const avatarSpan = userAvatar.querySelector("span");
      if (avatarSpan) avatarSpan.textContent = "U";
    }
    if (userAvatarLarge) userAvatarLarge.textContent = "U";

    if (authAction) {
      authAction.href = "/auth/login.html";
      authAction.removeAttribute("onclick");
      authAction.style.color = "var(--primary)";
    }
    if (authText) authText.textContent = "Sign in";
    if (authIcon) {
      authIcon.innerHTML =
        '<path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path><polyline points="10 17 15 12 21 12"></polyline><line x1="15" y1="12" x2="3" y2="12"></line>';
    }

    if (settingsBtn) settingsBtn.style.display = "none";
    if (appsButton) appsButton.style.display = "none";
    if (notificationsBtn) notificationsBtn.style.display = "none";

    // Hide omnibox (search bar) when signed out
    const omnibox = document.getElementById("omnibox");
    if (omnibox) omnibox.style.display = "none";

    // Hide Drive, Tasks, CRM, and Calendar navigation when signed out (all instances)
    const driveTabs = document.querySelectorAll('[data-section="drive"]');
    const tasksTabs = document.querySelectorAll('[data-section="tasks"]');
    const crmTabs = document.querySelectorAll('[data-section="crm"]');
    const calendarTabs = document.querySelectorAll('[data-section="calendar"]');

    driveTabs.forEach(tab => tab.style.display = "none");
    tasksTabs.forEach(tab => tab.style.display = "none");
    crmTabs.forEach(tab => tab.style.display = "none");
    calendarTabs.forEach(tab => tab.style.display = "none");
  }

  // Try to load cached user first
  var cachedUser = localStorage.getItem("gb-user-data");
  if (cachedUser) {
    try {
      var user = JSON.parse(cachedUser);
      if (user && user.email) {
        updateUserUI(user);
      }
    } catch (e) {}
  }

  // Always fetch fresh user data
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", loadUserProfile);
  } else {
    loadUserProfile();
  }
})();
