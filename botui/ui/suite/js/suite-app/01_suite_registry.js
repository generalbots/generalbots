/* Suite app — plugin registry (1/7). */
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
