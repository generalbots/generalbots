/* Suite app — Omnibox state, init and actions (2/7). */
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
};
