/* Suite app — Omnibox navigation and search (3/7). */
Object.assign(Omnibox, {
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
  }
});
