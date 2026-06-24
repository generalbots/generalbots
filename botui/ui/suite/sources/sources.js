/**
 * Sources Module JavaScript
 * Repositories, Apps, Prompts, Templates, MCP Servers & AI Models
 * Provides @mention support for chat context
 */
(function () {
  "use strict";

  /**
   * Initialize the Sources module
   */
  function init() {
    setupTabNavigation();
    setupCategoryNavigation();
    setupViewToggle();
    setupKeyboardShortcuts();
    setupHTMXEvents();
    setupRepoCards();
    setupAppCards();
    setupMentionAutocomplete();
  }

  /**
   * Set active tab
   */
  window.setActiveTab = function (btn) {
    document.querySelectorAll(".tab-btn").forEach((t) => {
      t.classList.remove("active");
      t.setAttribute("aria-selected", "false");
    });
    btn.classList.add("active");
    btn.setAttribute("aria-selected", "true");
  };

  /**
   * Setup tab navigation
   */
  function setupTabNavigation() {
    document.querySelectorAll(".tab-btn").forEach((btn) => {
      btn.addEventListener("click", function () {
        setActiveTab(this);
      });
    });
  }

  /**
   * Setup category navigation
   */
  function setupCategoryNavigation() {
    document.addEventListener("click", function (e) {
      const categoryItem = e.target.closest(".category-item");
      if (categoryItem) {
        document
          .querySelectorAll(".category-item")
          .forEach((c) => c.classList.remove("active"));
        categoryItem.classList.add("active");
      }
    });
  }

  /**
   * Setup view toggle (grid/list)
   */
  function setupViewToggle() {
    document.addEventListener("click", function (e) {
      const viewBtn = e.target.closest(".view-btn");
      if (viewBtn) {
        const controls = viewBtn.closest(".view-controls");
        if (controls) {
          controls
            .querySelectorAll(".view-btn")
            .forEach((b) => b.classList.remove("active"));
          viewBtn.classList.add("active");

          const grid = document.querySelector(
            ".prompts-grid, .templates-grid, .servers-grid, .models-grid, .news-grid",
          );
          if (grid) {
            if (viewBtn.title === "List view") {
              grid.classList.add("list-view");
            } else {
              grid.classList.remove("list-view");
            }
          }
        }
      }
    });
  }

  /**
   * Setup keyboard shortcuts
   */
  function setupKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
      // Ctrl+K to focus search
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        const searchInput = document.querySelector(".search-box input");
        if (searchInput) searchInput.focus();
      }

      // Tab navigation with number keys
      if (
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        !e.target.matches("input, textarea")
      ) {
        const tabs = document.querySelectorAll(".tab-btn");
        const num = parseInt(e.key);
        if (num >= 1 && num <= tabs.length) {
          tabs[num - 1].click();
        }
      }

      // Escape to close modals
      if (e.key === "Escape") {
        closeModals();
      }
    });
  }

  /**
   * Setup HTMX events
   */
  function setupHTMXEvents() {
    if (typeof htmx === "undefined") return;

    document.body.addEventListener("htmx:beforeRequest", function (e) {
      if (e.detail.target && e.detail.target.id === "content-area") {
        e.detail.target.innerHTML = `
                    <div class="loading-spinner">
                        <div class="spinner"></div>
                        <p>Loading...</p>
                    </div>
                `;
      }
    });

    document.body.addEventListener("htmx:afterSwap", function (e) {
      // Re-initialize any dynamic content handlers after content swap
      setupPromptCards();
      setupServerCards();
      setupModelCards();
      setupRepoCards();
      setupAppCards();
    });
  }

  /**
   * Setup prompt card interactions
   */
  function setupPromptCards() {
    document.querySelectorAll(".prompt-card").forEach((card) => {
      card.addEventListener("click", function (e) {
        // Don't trigger if clicking on action buttons
        if (e.target.closest(".prompt-action-btn")) return;

        const promptId = this.dataset.id;
        if (promptId) {
          showPromptDetail(promptId);
        }
      });
    });

    document.querySelectorAll(".prompt-action-btn").forEach((btn) => {
      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        const action = this.title.toLowerCase();
        const card = this.closest(".prompt-card");
        const promptId = card?.dataset.id;

        switch (action) {
          case "use":
            usePrompt(promptId);
            break;
          case "copy":
            copyPrompt(promptId);
            break;
          case "save":
            savePrompt(promptId);
            break;
        }
      });
    });
  }

  /**
   * Setup server card interactions
   */
  function setupServerCards() {
    document.querySelectorAll(".server-card").forEach((card) => {
      card.addEventListener("click", function () {
        const serverId = this.dataset.id;
        if (serverId) {
          showServerDetail(serverId);
        }
      });
    });
  }

  /**
   * Setup model card interactions
   */
  function setupModelCards() {
    document.querySelectorAll(".model-card").forEach((card) => {
      card.addEventListener("click", function () {
        const modelId = this.dataset.id;
        if (modelId) {
          showModelDetail(modelId);
        }
      });
    });
  }

  /**
   * Show prompt detail modal/panel
   */
  function showPromptDetail(promptId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/prompts/${promptId}`, {
          target: "#prompt-detail-panel",
          swap: "innerHTML",
        })
        .then(() => {
          document
            .getElementById("prompt-detail-panel")
            ?.classList.remove("hidden");
        });
    }
  }

  /**
   * Use a prompt
   */
  function usePrompt(promptId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("POST", `/api/ui/sources/prompts/${promptId}/use`, {
          swap: "none",
        })
        .then(() => {
          // Navigate to the appropriate module
          window.location.hash = "#research";
        });
    }
  }

  /**
   * Copy prompt to clipboard
   */
  function copyPrompt(promptId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/prompts/${promptId}/content`, {
          swap: "none",
        })
        .then((response) => {
          // Parse response and copy to clipboard
          navigator.clipboard.writeText(response || "");
          showToast("Prompt copied to clipboard");
        });
    }
  }

  /**
   * Save prompt to collection
   */
  function savePrompt(promptId) {
    const collectionName = prompt("Enter collection name:");
    if (collectionName && typeof htmx !== "undefined") {
      htmx
        .ajax("POST", "/api/ui/sources/prompts/save", {
          values: {
            promptId,
            collection: collectionName,
          },
        })
        .then(() => {
          showToast("Prompt saved to collection");
        });
    }
  }

  /**
   * Setup repository card interactions
   */
  function setupRepoCards() {
    document.querySelectorAll(".repo-card").forEach((card) => {
      card.addEventListener("click", function (e) {
        if (e.target.closest(".repo-action-btn")) return;

        const repoId = this.dataset.id;
        if (repoId) {
          showRepoDetail(repoId);
        }
      });
    });

    document.querySelectorAll(".repo-action-btn").forEach((btn) => {
      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        const action = this.dataset.action;
        const card = this.closest(".repo-card");
        const repoId = card?.dataset.id;
        const repoName = card?.dataset.name;

        switch (action) {
          case "connect":
            connectRepo(repoId);
            break;
          case "disconnect":
            disconnectRepo(repoId);
            break;
          case "mention":
            insertMention("repo", repoName);
            break;
          case "browse":
            browseRepo(repoId);
            break;
        }
      });
    });
  }

  /**
   * Setup app card interactions
   */
  function setupAppCards() {
    document.querySelectorAll(".app-card").forEach((card) => {
      card.addEventListener("click", function (e) {
        if (e.target.closest(".app-action-btn")) return;

        const appId = this.dataset.id;
        if (appId) {
          showAppDetail(appId);
        }
      });
    });

    document.querySelectorAll(".app-action-btn").forEach((btn) => {
      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        const action = this.dataset.action;
        const card = this.closest(".app-card");
        const appId = card?.dataset.id;
        const appName = card?.dataset.name;

        switch (action) {
          case "open":
            openApp(appId);
            break;
          case "edit":
            editApp(appId);
            break;
          case "mention":
            insertMention("app", appName);
            break;
        }
      });
    });
  }

  /**
   * Setup @mention autocomplete for chat
   */
  function setupMentionAutocomplete() {
    // Listen for @ symbol in chat input
    document.addEventListener("input", function (e) {
      if (!e.target.matches(".chat-input, .message-input, #chat-input")) return;

      const input = e.target;
      const value = input.value;
      const cursorPos = input.selectionStart;

      // Find @ before cursor
      const textBeforeCursor = value.substring(0, cursorPos);
      const atMatch = textBeforeCursor.match(/@(\w*)$/);

      if (atMatch) {
        const query = atMatch[1];
        showMentionSuggestions(input, query);
      } else {
        hideMentionSuggestions();
      }
    });

    // Handle mention selection
    document.addEventListener("click", function (e) {
      const suggestion = e.target.closest(".mention-suggestion");
      if (suggestion) {
        const type = suggestion.dataset.type;
        const name = suggestion.dataset.name;
        applyMention(type, name);
      }
    });

    // Keyboard navigation for suggestions
    document.addEventListener("keydown", function (e) {
      const suggestions = document.querySelector(".mention-suggestions");
      if (!suggestions || suggestions.classList.contains("hidden")) return;

      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        navigateMentionSuggestions(e.key === "ArrowDown" ? 1 : -1);
      } else if (e.key === "Enter" || e.key === "Tab") {
        const active = suggestions.querySelector(".mention-suggestion.active");
        if (active) {
          e.preventDefault();
          active.click();
        }
      } else if (e.key === "Escape") {
        hideMentionSuggestions();
      }
    });
  }

  /**
   * Show mention suggestions dropdown
   */
  function showMentionSuggestions(input, query) {
    let suggestions = document.querySelector(".mention-suggestions");

    if (!suggestions) {
      suggestions = document.createElement("div");
      suggestions.className = "mention-suggestions";
      suggestions.style.cssText = `
                position: absolute;
                background: var(--surface);
                border: 1px solid var(--border);
                border-radius: 8px;
                box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                max-height: 300px;
                overflow-y: auto;
                z-index: 1000;
            `;
      document.body.appendChild(suggestions);
    }

    // Fetch matching repos and apps
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/mentions?q=${encodeURIComponent(query)}`, {
          swap: "none",
        })
        .then((response) => {
          try {
            const data = JSON.parse(response);
            renderMentionSuggestions(suggestions, data, input);
          } catch (e) {
            // Fallback to showing cached data
            renderMentionSuggestions(
              suggestions,
              getMockMentions(query),
              input,
            );
          }
        });
    } else {
      renderMentionSuggestions(suggestions, getMockMentions(query), input);
    }
  }

  /**
   * Get mock mentions for development
   */
  function getMockMentions(query) {
    const allMentions = [
      { type: "repo", name: "botserver", description: "Core API server" },
      { type: "repo", name: "botui", description: "Web UI components" },
      { type: "repo", name: "botbook", description: "Documentation" },
      { type: "repo", name: "bottemplates", description: "Bot templates" },
      { type: "app", name: "crm", description: "Customer management app" },
      { type: "app", name: "dashboard", description: "Analytics dashboard" },
      { type: "app", name: "myapp", description: "Custom application" },
    ];

    if (!query) return allMentions.slice(0, 5);

    return allMentions
      .filter((m) => m.name.toLowerCase().includes(query.toLowerCase()))
      .slice(0, 5);
  }

  /**
   * Render mention suggestions
   */
  function renderMentionSuggestions(container, data, input) {
    if (!data || data.length === 0) {
      container.classList.add("hidden");
      return;
    }

    const rect = input.getBoundingClientRect();
    container.style.top = `${rect.bottom + 4}px`;
    container.style.left = `${rect.left}px`;
    container.style.width = `${Math.min(rect.width, 320)}px`;

    container.innerHTML = data
      .map(
        (item, index) => `
            <div class="mention-suggestion ${index === 0 ? "active" : ""}"
                 data-type="${item.type}"
                 data-name="${item.name}">
                <div class="mention-suggestion-icon ${item.type}">
                    ${
                      item.type === "repo"
                        ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>'
                        : '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>'
                    }
                </div>
                <div class="mention-suggestion-info">
                    <div class="mention-suggestion-name">@${item.name}</div>
                    <div class="mention-suggestion-type">${item.type === "repo" ? "Repository" : "App"} • ${item.description || ""}</div>
                </div>
            </div>
        `,
      )
      .join("");

    container.classList.remove("hidden");
  }

  /**
   * Hide mention suggestions
   */
  function hideMentionSuggestions() {
    const suggestions = document.querySelector(".mention-suggestions");
    if (suggestions) {
      suggestions.classList.add("hidden");
    }
  }

  /**
   * Navigate mention suggestions with keyboard
   */
  function navigateMentionSuggestions(direction) {
    const suggestions = document.querySelectorAll(".mention-suggestion");
    const current = document.querySelector(".mention-suggestion.active");
    let index = Array.from(suggestions).indexOf(current);

    index += direction;
    if (index < 0) index = suggestions.length - 1;
    if (index >= suggestions.length) index = 0;

    suggestions.forEach((s) => s.classList.remove("active"));
    suggestions[index]?.classList.add("active");
    suggestions[index]?.scrollIntoView({ block: "nearest" });
  }

  /**
   * Apply selected mention to input
   */
  function applyMention(type, name) {
    const input = document.querySelector(
      ".chat-input, .message-input, #chat-input",
    );
    if (!input) return;

    const value = input.value;
    const cursorPos = input.selectionStart;
    const textBeforeCursor = value.substring(0, cursorPos);
    const textAfterCursor = value.substring(cursorPos);

    // Replace @query with @name
    const newTextBefore = textBeforeCursor.replace(/@\w*$/, `@${name} `);
    input.value = newTextBefore + textAfterCursor;
    input.selectionStart = input.selectionEnd = newTextBefore.length;
    input.focus();

    hideMentionSuggestions();

    // Store context for the task
    storeTaskContext(type, name);
  }

  /**
   * Insert mention from Sources page
   */
  function insertMention(type, name) {
    // Navigate to chat and insert mention
    const chatInput = document.querySelector(
      ".chat-input, .message-input, #chat-input",
    );
    if (chatInput) {
      chatInput.value += `@${name} `;
      chatInput.focus();
      storeTaskContext(type, name);
      showToast(`Added @${name} to chat context`);
    } else {
      // Store for next chat session
      sessionStorage.setItem("pendingMention", JSON.stringify({ type, name }));
      showToast(`@${name} will be added when you open chat`);
    }
  }

  /**
   * Store context for autonomous tasks
   */
  function storeTaskContext(type, name) {
    let context = JSON.parse(sessionStorage.getItem("taskContext") || "[]");

    // Avoid duplicates
    if (!context.find((c) => c.type === type && c.name === name)) {
      context.push({ type, name, addedAt: Date.now() });
      sessionStorage.setItem("taskContext", JSON.stringify(context));
    }
  }

  /**
   * Get current task context
   */
  window.getTaskContext = function () {
    return JSON.parse(sessionStorage.getItem("taskContext") || "[]");
  };

  /**
   * Clear task context
   */
  window.clearTaskContext = function () {
    sessionStorage.removeItem("taskContext");
  };

  /**
   * Show repository detail
   */
  function showRepoDetail(repoId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/repositories/${repoId}`, {
          target: "#repo-detail-panel",
          swap: "innerHTML",
        })
        .then(() => {
          document
            .getElementById("repo-detail-panel")
            ?.classList.remove("hidden");
        });
    }
  }

  /**
   * Connect a repository
   */
  function connectRepo(repoId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("POST", `/api/ui/sources/repositories/${repoId}/connect`, {
          swap: "none",
        })
        .then(() => {
          showToast("Repository connected");
          // Refresh the repo card
          htmx.ajax("GET", "/api/ui/sources/repositories", {
            target: "#content-area",
            swap: "innerHTML",
          });
        });
    }
  }

  /**
   * Disconnect a repository
   */
  function disconnectRepo(repoId) {
    if (confirm("Disconnect this repository?")) {
      if (typeof htmx !== "undefined") {
        htmx
          .ajax("DELETE", `/api/ui/sources/repositories/${repoId}/connect`, {
            swap: "none",
          })
          .then(() => {
            showToast("Repository disconnected");
            htmx.ajax("GET", "/api/ui/sources/repositories", {
              target: "#content-area",
              swap: "innerHTML",
            });
          });
      }
    }
  }

  /**
   * Browse repository files
   */
  function browseRepo(repoId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/repositories/${repoId}/files`, {
          target: "#repo-browser-panel",
          swap: "innerHTML",
        })
        .then(() => {
          document
            .getElementById("repo-browser-panel")
            ?.classList.remove("hidden");
        });
    }
  }

  /**
   * Show app detail
   */
  function showAppDetail(appId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/apps/${appId}`, {
          target: "#app-detail-panel",
          swap: "innerHTML",
        })
        .then(() => {
          document
            .getElementById("app-detail-panel")
            ?.classList.remove("hidden");
        });
    }
  }

  /**
   * Open an app in new tab
   */
  function openApp(appId) {
    window.open(`https://${appId}.gb.solutions`, "_blank");
  }

  /**
   * Edit an app (opens in Tasks with context)
   */
  function editApp(appId) {
    // Store app context and navigate to tasks
    storeTaskContext("app", appId);
    window.location.hash = "#tasks";
    showToast(`Editing @${appId} - describe your changes`);
  }

  /**
   * Show server detail
   */
  function showServerDetail(serverId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/mcp-servers/${serverId}`, {
          target: "#server-detail-panel",
          swap: "innerHTML",
        })
        .then(() => {
          document
            .getElementById("server-detail-panel")
            ?.classList.remove("hidden");
        });
    }
  }

  /**
   * Show model detail
   */
  function showModelDetail(modelId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("GET", `/api/ui/sources/models/${modelId}`, {
          target: "#model-detail-panel",
          swap: "innerHTML",
        })
        .then(() => {
          document
            .getElementById("model-detail-panel")
            ?.classList.remove("hidden");
        });
    }
  }

  /**
   * Close all modals
   */
  function closeModals() {
    document.querySelectorAll(".modal, .detail-panel").forEach((modal) => {
      modal.classList.add("hidden");
    });
  }

  /**
   * Filter MCP catalog by category
   */
  function filterMcpCategory(btn, category) {
    document
      .querySelectorAll("#mcp-category-filter .category-btn")
      .forEach((b) => {
        b.classList.remove("active");
        b.style.background = "#f5f5f5";
        b.style.color = "#333";
      });
    btn.classList.add("active");
    btn.style.background = "#2196F3";
    btn.style.color = "white";

    document.querySelectorAll(".server-card").forEach((card) => {
      if (category === "all" || card.dataset.category === category) {
        card.style.display = "";
      } else {
        card.style.display = "none";
      }
    });
  }

  /**
   * Add MCP server from catalog
   */
  function addCatalogServer(id, name) {
    if (confirm('Add "' + name + '" to your MCP configuration?')) {
      if (typeof htmx !== "undefined") {
        htmx.ajax("POST", "/api/ui/sources/mcp/add-from-catalog", {
          values: { server_id: id },
          target: "#mcp-grid",
          swap: "innerHTML",
        });
        showToast('Server "' + name + '" added to configuration');
      }
    }
  }

  /**
   * Show toast notification
   */
  function showToast(message, type = "success") {
    const toast = document.createElement("div");
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    document.body.appendChild(toast);

    // Trigger animation
    requestAnimationFrame(() => {
      toast.classList.add("show");
    });

    // Remove after delay
    setTimeout(() => {
      toast.classList.remove("show");
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }

  // Initialize on DOM ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  // Expose for external use
  window.Sources = {
    setActiveTab,
    showPromptDetail,
    usePrompt,
    copyPrompt,
    savePrompt,
    showToast,
    showRepoDetail,
    connectRepo,
    disconnectRepo,
    browseRepo,
    showAppDetail,
    openApp,
    editApp,
    insertMention,
    filterMcpCategory,
    addCatalogServer,
    getTaskContext: window.getTaskContext,
    clearTaskContext: window.clearTaskContext,
  };

  // Expose globally for inline onclick handlers
  window.filterMcpCategory = filterMcpCategory;
  window.addCatalogServer = addCatalogServer;
  window.installSkill = installSkill;

  /**
   * Featured skills for the Skills Library modal (Issue #481)
   */
  const FEATURED_SKILLS = [
    {
      id: "email-campaigner",
      name: "Email Campaigner",
      description:
        "Create and manage email marketing campaigns with automated follow-ups and analytics.",
      icon: "✉️",
    },
    {
      id: "whatsapp-broadcaster",
      name: "WhatsApp Broadcaster",
      description:
        "Send broadcast messages, manage templates, and track delivery via WhatsApp Business API.",
      icon: "💬",
    },
    {
      id: "report-generator",
      name: "Report Generator",
      description:
        "Generate rich PDF and Excel reports from database queries with scheduling support.",
      icon: "📊",
    },
    {
      id: "crm-assistant",
      name: "CRM Assistant",
      description:
        "Manage leads, contacts, and deals with AI-powered follow-up suggestions and pipeline tracking.",
      icon: "👥",
    },
    {
      id: "analytics-dashboard",
      name: "Analytics Dashboard",
      description:
        "Visual dashboards with real-time metrics, custom widgets, and data export capabilities.",
      icon: "📈",
    },
    {
      id: "social-media-poster",
      name: "Social Media Poster",
      description:
        "Schedule and publish content across multiple social platforms with media management.",
      icon: "📱",
    },
  ];

  /**
   * Install a skill via the backend API
   */
  async function installSkill(skillId, skillName) {
    const botId =
      (typeof ChatState !== "undefined" && ChatState.currentBotId) ||
      document.body.dataset.botId ||
      "default";

    const installBtn = document.querySelector(
      `[data-skill-id="${skillId}"] .btn-install`,
    );
    if (installBtn) {
      installBtn.disabled = true;
      installBtn.textContent = "Installing...";
    }

    try {
      const response = await fetch("/api/ui/skills/install", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: skillId, bot_id: botId }),
      });

      if (!response.ok) {
        const err = await response.text();
        throw new Error(err || `HTTP ${response.status}`);
      }

      showToast(`"${skillName}" installed successfully!`, "success");
    } catch (err) {
      console.error("Skill install failed:", err);
      showToast(
        `Failed to install "${skillName}": ${err.message}`,
        "error",
      );
    } finally {
      if (installBtn) {
        installBtn.disabled = false;
        installBtn.textContent = "Install";
      }
    }
  }

  /**
   * Filter skills grid by search query
   */
  window.filterSkills = function () {
    const input = document.getElementById("skills-search-input");
    if (!input) return;
    const q = input.value.toLowerCase();
    document.querySelectorAll(".skill-card").forEach((card) => {
      const name = card.dataset.name?.toLowerCase() || "";
      const desc = card.dataset.description?.toLowerCase() || "";
      card.style.display =
        !q || name.includes(q) || desc.includes(q) ? "" : "none";
    });
  };

  /**
   * Open Skills Library modal (Issue #481)
   */
  window.openSkillsLibrary = function () {
    let modal = document.getElementById("skills-library-modal");

    if (!modal) {
      modal = document.createElement("div");
      modal.id = "skills-library-modal";
      modal.className = "modal";
      modal.style.display = "none";

      modal.innerHTML = `
        <div class="modal-backdrop" onclick="hideSkillsLibrary()"></div>
        <div class="modal-content modal-content--wide">
          <div class="modal-header">
            <h3>Skills Library</h3>
            <div class="modal-header-actions">
              <span class="skills-count">${FEATURED_SKILLS.length} skills</span>
              <button class="modal-close" onclick="hideSkillsLibrary()">&times;</button>
            </div>
          </div>
          <div class="modal-body">
            <div class="skills-search">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
              </svg>
              <input
                type="text"
                id="skills-search-input"
                placeholder="Search skills..."
                oninput="filterSkills()"
              />
            </div>
            <div class="skills-grid" id="skills-grid">
              ${FEATURED_SKILLS.map(
                (s) => `
                <div class="skill-card" data-skill-id="${s.id}" data-name="${s.name}" data-description="${s.description}">
                  <div class="skill-card-icon">${s.icon}</div>
                  <div class="skill-card-body">
                    <div class="skill-card-name">${s.name}</div>
                    <div class="skill-card-description">${s.description}</div>
                  </div>
                  <div class="skill-card-action">
                    <button class="btn-install" onclick="installSkill('${s.id}', '${s.name}')">Install</button>
                  </div>
                </div>
              `,
              ).join("")}
            </div>
            <p class="skills-footer-note">
              More skills available at <a href="https://skills.sh" target="_blank" rel="noopener">skills.sh</a>
            </p>
          </div>
        </div>
      `;

      // Inject supporting styles
      const style = document.createElement("style");
      style.textContent = `
        .modal-content--wide { max-width: 640px; }
        .modal-header-actions { display: flex; align-items: center; gap: 0.75rem; }
        .skills-count { font-size: 0.75rem; color: var(--text-secondary); background: var(--bg-secondary); padding: 0.2rem 0.6rem; border-radius: 1rem; }
        .skills-search { position: relative; margin-bottom: 1.25rem; }
        .skills-search svg { position: absolute; left: 12px; top: 50%; transform: translateY(-50%); color: var(--text-secondary); }
        .skills-search input {
          width: 100%; padding: 0.65rem 0.75rem 0.65rem 2.25rem;
          border: 1px solid var(--border); border-radius: 0.5rem;
          background: var(--bg-secondary); color: var(--text-primary);
          font-size: 0.875rem; outline: none; transition: border-color 0.2s;
        }
        .skills-search input:focus { border-color: var(--primary); }
        .skills-search input::placeholder { color: var(--text-secondary); }
        .skills-grid { display: flex; flex-direction: column; gap: 0.75rem; max-height: 50vh; overflow-y: auto; }
        .skill-card {
          display: flex; align-items: center; gap: 1rem;
          padding: 1rem; background: var(--bg-secondary);
          border: 1px solid var(--border); border-radius: 0.75rem;
          transition: all 0.2s;
        }
        .skill-card:hover { border-color: var(--primary); }
        .skill-card-icon {
          width: 44px; height: 44px; border-radius: 0.5rem;
          display: flex; align-items: center; justify-content: center;
          background: var(--primary-light, rgba(212,245,5,0.1));
          font-size: 1.35rem; flex-shrink: 0;
        }
        .skill-card-body { flex: 1; min-width: 0; }
        .skill-card-name { font-weight: 600; font-size: 0.9375rem; color: var(--text-primary); }
        .skill-card-description { font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.15rem; }
        .skill-card-action { flex-shrink: 0; }
        .btn-install {
          padding: 0.45rem 1rem; border: none; border-radius: 0.375rem;
          background: var(--primary); color: var(--primary-text, #000);
          font-size: 0.8125rem; font-weight: 500; cursor: pointer;
          transition: all 0.2s; white-space: nowrap;
        }
        .btn-install:hover { opacity: 0.85; }
        .btn-install:disabled { opacity: 0.5; cursor: not-allowed; }
        .skills-footer-note { margin-top: 1rem; font-size: 0.75rem; color: var(--text-secondary); text-align: center; }
        .skills-footer-note a { color: var(--primary); text-decoration: none; }
        .skills-footer-note a:hover { text-decoration: underline; }
        .toast-error { background: #dc3545; color: white; }
      `;
      document.head.appendChild(style);
      document.body.appendChild(modal);
    }

    modal.style.display = "flex";
    const input = document.getElementById("skills-search-input");
    if (input) {
      input.value = "";
      input.focus();
    }
    filterSkills();
  };

  /**
   * Hide the Skills Library modal
   */
  window.hideSkillsLibrary = function () {
    const modal = document.getElementById("skills-library-modal");
    if (modal) modal.style.display = "none";
  };

  /**
   * Close skills modal on Escape
   */
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape") {
      hideSkillsLibrary();
    }
  });
})();
