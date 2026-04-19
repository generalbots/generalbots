/* =============================================================================
   DASHBOARDS MODULE - Business Intelligence Dashboards
   ============================================================================= */

(function () {
  "use strict";

  // =============================================================================
  // STATE
  // =============================================================================

  const state = {
    dashboards: [],
    dataSources: [],
    currentDashboard: null,
    selectedWidgetType: null,
    isEditing: false,
    widgets: [],
    filters: {
      category: "all",
      search: "",
    },
  };

  // =============================================================================
  // INITIALIZATION
  // =============================================================================

  function init() {
    loadDashboards();
    loadDataSources();
    bindEvents();
    console.log("Dashboards module initialized");
  }

  function bindEvents() {
    // Search input
    const searchInput = document.getElementById("dashboard-search");
    if (searchInput) {
      searchInput.addEventListener("input", (e) => {
        state.filters.search = e.target.value;
        filterDashboards();
      });
    }

    // Category filter
    const categorySelect = document.getElementById("category-filter");
    if (categorySelect) {
      categorySelect.addEventListener("change", (e) => {
        state.filters.category = e.target.value;
        filterDashboards();
      });
    }

    // Dashboard card clicks
    document.addEventListener("click", (e) => {
      const card = e.target.closest(".dashboard-card");
      if (card && !e.target.closest("button")) {
        const dashboardId = card.dataset.id;
        if (dashboardId) {
          openDashboard(dashboardId);
        }
      }
    });
  }

  // =============================================================================
  // DASHBOARD CRUD
  // =============================================================================

  async function loadDashboards() {
    try {
      const response = await fetch("/api/dashboards");
      if (response.ok) {
        const data = await response.json();
        state.dashboards = data.dashboards || [];
        renderDashboardList();
      }
    } catch (e) {
      console.error("Failed to load dashboards:", e);
    }
  }

  function renderDashboardList() {
    const container = document.getElementById("dashboards-grid");
    if (!container) return;

    const filtered = state.dashboards.filter((d) => {
      const matchesSearch =
        !state.filters.search ||
        d.name.toLowerCase().includes(state.filters.search.toLowerCase());
      const matchesCategory =
        state.filters.category === "all" ||
        d.category === state.filters.category;
      return matchesSearch && matchesCategory;
    });

    if (filtered.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="empty-icon">📊</span>
          <h3>No dashboards found</h3>
          <p>Create your first dashboard to visualize your data</p>
          <button class="btn-primary" onclick="showCreateDashboardModal()">
            Create Dashboard
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = filtered
      .map(
        (dashboard) => `
      <div class="dashboard-card" data-id="${dashboard.id}">
        <div class="dashboard-preview">
          <div class="preview-placeholder">📊</div>
        </div>
        <div class="dashboard-info">
          <h3>${escapeHtml(dashboard.name)}</h3>
          <p>${escapeHtml(dashboard.description || "No description")}</p>
          <div class="dashboard-meta">
            <span class="category">${escapeHtml(dashboard.category || "General")}</span>
            <span class="updated">Updated ${formatRelativeTime(dashboard.updated_at)}</span>
          </div>
        </div>
        <div class="dashboard-actions">
          <button class="btn-icon" onclick="event.stopPropagation(); editDashboardById('${dashboard.id}')" title="Edit">✏️</button>
          <button class="btn-icon" onclick="event.stopPropagation(); duplicateDashboard('${dashboard.id}')" title="Duplicate">📋</button>
          <button class="btn-icon" onclick="event.stopPropagation(); deleteDashboard('${dashboard.id}')" title="Delete">🗑️</button>
        </div>
      </div>
    `,
      )
      .join("");
  }

  function filterDashboards() {
    renderDashboardList();
  }

  // =============================================================================
  // CREATE DASHBOARD MODAL
  // =============================================================================

  function showCreateDashboardModal() {
    const modal = document.getElementById("createDashboardModal");
    if (modal) {
      modal.style.display = "flex";
      // Reset form
      const form = modal.querySelector("form");
      if (form) form.reset();
    }
  }

  function closeCreateDashboardModal() {
    const modal = document.getElementById("createDashboardModal");
    if (modal) {
      modal.style.display = "none";
    }
  }

  async function createDashboard(formData) {
    const data = {
      name: formData.get("name") || "Untitled Dashboard",
      description: formData.get("description") || "",
      category: formData.get("category") || "general",
      is_public: formData.get("is_public") === "on",
    };

    try {
      const response = await fetch("/api/dashboards", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      if (response.ok) {
        const result = await response.json();
        showNotification("Dashboard created", "success");
        closeCreateDashboardModal();
        loadDashboards();

        // Open the new dashboard for editing
        if (result.id) {
          openDashboard(result.id);
        }
      } else {
        showNotification("Failed to create dashboard", "error");
      }
    } catch (e) {
      console.error("Failed to create dashboard:", e);
      showNotification("Failed to create dashboard", "error");
    }
  }

  // Handle form submission
  document.addEventListener("submit", (e) => {
    if (e.target.id === "createDashboardForm") {
      e.preventDefault();
      createDashboard(new FormData(e.target));
    } else if (e.target.id === "addDataSourceForm") {
      e.preventDefault();
      addDataSource(new FormData(e.target));
    } else if (e.target.id === "addWidgetForm") {
      e.preventDefault();
      addWidget(new FormData(e.target));
    }
  });

  // =============================================================================
  // DASHBOARD VIEWER
  // =============================================================================

  async function openDashboard(dashboardId) {
    try {
      const response = await fetch(`/api/dashboards/${dashboardId}`);
      if (response.ok) {
        const dashboard = await response.json();
        state.currentDashboard = dashboard;
        state.widgets = dashboard.widgets || [];
        showDashboardViewer(dashboard);
      }
    } catch (e) {
      console.error("Failed to open dashboard:", e);
      showNotification("Failed to load dashboard", "error");
    }
  }

  function showDashboardViewer(dashboard) {
    const viewer = document.getElementById("dashboard-viewer");
    const list = document.getElementById("dashboards-list");

    if (viewer) viewer.classList.remove("hidden");
    if (list) list.classList.add("hidden");

    // Update title
    const titleEl = document.getElementById("viewer-dashboard-name");
    if (titleEl) titleEl.textContent = dashboard.name;

    // Render widgets
    renderWidgets(dashboard.widgets || []);
  }

  function closeDashboardViewer() {
    const viewer = document.getElementById("dashboard-viewer");
    const list = document.getElementById("dashboards-list");

    if (viewer) viewer.classList.add("hidden");
    if (list) list.classList.remove("hidden");

    state.currentDashboard = null;
    state.isEditing = false;
  }

  function renderWidgets(widgets) {
    const container = document.getElementById("widgets-grid");
    if (!container) return;

    if (widgets.length === 0) {
      container.innerHTML = `
        <div class="empty-widgets">
          <span class="empty-icon">📈</span>
          <h3>No widgets yet</h3>
          <p>Add widgets to visualize your data</p>
          <button class="btn-primary" onclick="showAddWidgetModal()">
            Add Widget
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = widgets
      .map(
        (widget) => `
      <div class="widget" data-id="${widget.id}" style="grid-column: span ${widget.width || 1}; grid-row: span ${widget.height || 1};">
        <div class="widget-header">
          <h4>${escapeHtml(widget.title)}</h4>
          <div class="widget-actions">
            <button class="btn-icon btn-sm" onclick="editWidget('${widget.id}')" title="Edit">⚙️</button>
            <button class="btn-icon btn-sm" onclick="removeWidget('${widget.id}')" title="Remove">✕</button>
          </div>
        </div>
        <div class="widget-content" id="widget-content-${widget.id}">
          ${renderWidgetContent(widget)}
        </div>
      </div>
    `,
      )
      .join("");
  }

  function renderWidgetContent(widget) {
    // Placeholder rendering - in production, this would render actual charts
    const icons = {
      line_chart: "📈",
      bar_chart: "📊",
      pie_chart: "🥧",
      area_chart: "📉",
      scatter_plot: "⚬",
      kpi: "🎯",
      table: "📋",
      gauge: "⏲️",
      map: "🗺️",
      text: "📝",
    };

    return `
      <div class="widget-placeholder">
        <span class="widget-icon">${icons[widget.widget_type] || "📊"}</span>
        <span class="widget-type">${widget.widget_type}</span>
      </div>
    `;
  }

  // =============================================================================
  // DASHBOARD ACTIONS
  // =============================================================================

  async function refreshDashboard() {
    if (!state.currentDashboard) return;
    showNotification("Refreshing dashboard...", "info");
    await openDashboard(state.currentDashboard.id);
    showNotification("Dashboard refreshed", "success");
  }

  function editDashboard() {
    if (!state.currentDashboard) return;
    state.isEditing = true;

    const viewer = document.getElementById("dashboard-viewer");
    if (viewer) {
      viewer.classList.add("editing");
    }

    showNotification("Edit mode enabled", "info");
  }

  function editDashboardById(dashboardId) {
    openDashboard(dashboardId).then(() => {
      editDashboard();
    });
  }

  function shareDashboard() {
    if (!state.currentDashboard) return;

    const shareUrl = `${window.location.origin}/dashboards/${state.currentDashboard.id}`;
    navigator.clipboard.writeText(shareUrl).then(() => {
      showNotification("Share link copied to clipboard", "success");
    });
  }

  function exportDashboard() {
    if (!state.currentDashboard) return;

    // Export as JSON
    const data = JSON.stringify(state.currentDashboard, null, 2);
    const blob = new Blob([data], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.download = `${state.currentDashboard.name}.json`;
    link.href = url;
    link.click();
    URL.revokeObjectURL(url);

    showNotification("Dashboard exported", "success");
  }

  async function duplicateDashboard(dashboardId) {
    try {
      const response = await fetch(`/api/dashboards/${dashboardId}/duplicate`, {
        method: "POST",
      });

      if (response.ok) {
        showNotification("Dashboard duplicated", "success");
        loadDashboards();
      } else {
        showNotification("Failed to duplicate dashboard", "error");
      }
    } catch (e) {
      console.error("Failed to duplicate dashboard:", e);
      showNotification("Failed to duplicate dashboard", "error");
    }
  }

  async function deleteDashboard(dashboardId) {
    if (!confirm("Delete this dashboard? This cannot be undone.")) return;

    try {
      const response = await fetch(`/api/dashboards/${dashboardId}`, {
        method: "DELETE",
      });

      if (response.ok) {
        showNotification("Dashboard deleted", "success");
        loadDashboards();
        if (state.currentDashboard?.id === dashboardId) {
          closeDashboardViewer();
        }
      } else {
        showNotification("Failed to delete dashboard", "error");
      }
    } catch (e) {
      console.error("Failed to delete dashboard:", e);
      showNotification("Failed to delete dashboard", "error");
    }
  }

  // =============================================================================
  // DATA SOURCES
  // =============================================================================

  async function loadDataSources() {
    try {
      const response = await fetch("/api/dashboards/data-sources");
      if (response.ok) {
        const data = await response.json();
        state.dataSources = data.data_sources || [];
        renderDataSourcesList();
      }
    } catch (e) {
      console.error("Failed to load data sources:", e);
    }
  }

  function renderDataSourcesList() {
    const container = document.getElementById("data-sources-list");
    if (!container) return;

    if (state.dataSources.length === 0) {
      container.innerHTML = `
        <p class="empty-message">No data sources configured</p>
      `;
      return;
    }

    container.innerHTML = state.dataSources
      .map(
        (source) => `
      <div class="data-source-item" data-id="${source.id}">
        <span class="source-icon">${getSourceIcon(source.source_type)}</span>
        <div class="source-info">
          <span class="source-name">${escapeHtml(source.name)}</span>
          <span class="source-type">${source.source_type}</span>
        </div>
        <button class="btn-icon btn-sm" onclick="removeDataSource('${source.id}')" title="Remove">✕</button>
      </div>
    `,
      )
      .join("");
  }

  function getSourceIcon(sourceType) {
    const icons = {
      postgresql: "🐘",
      mysql: "🐬",
      api: "🔌",
      csv: "📄",
      json: "📋",
      elasticsearch: "🔍",
      mongodb: "🍃",
    };
    return icons[sourceType] || "📊";
  }

  function showAddDataSourceModal() {
    const modal = document.getElementById("addDataSourceModal");
    if (modal) {
      modal.style.display = "flex";
      const form = modal.querySelector("form");
      if (form) form.reset();
    }
  }

  function closeAddDataSourceModal() {
    const modal = document.getElementById("addDataSourceModal");
    if (modal) {
      modal.style.display = "none";
    }
  }

  async function testDataSourceConnection() {
    const form = document.getElementById("addDataSourceForm");
    if (!form) return;

    const formData = new FormData(form);
    showNotification("Testing connection...", "info");

    try {
      const response = await fetch("/api/dashboards/data-sources/test", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          source_type: formData.get("source_type"),
          connection_string: formData.get("connection_string"),
        }),
      });

      if (response.ok) {
        showNotification("Connection successful!", "success");
      } else {
        showNotification("Connection failed", "error");
      }
    } catch (e) {
      showNotification("Connection test failed", "error");
    }
  }

  async function addDataSource(formData) {
    const data = {
      name: formData.get("name"),
      source_type: formData.get("source_type"),
      connection_string: formData.get("connection_string"),
    };

    try {
      const response = await fetch("/api/dashboards/data-sources", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      if (response.ok) {
        showNotification("Data source added", "success");
        closeAddDataSourceModal();
        loadDataSources();
      } else {
        showNotification("Failed to add data source", "error");
      }
    } catch (e) {
      console.error("Failed to add data source:", e);
      showNotification("Failed to add data source", "error");
    }
  }

  async function removeDataSource(sourceId) {
    if (!confirm("Remove this data source?")) return;

    try {
      const response = await fetch(`/api/dashboards/data-sources/${sourceId}`, {
        method: "DELETE",
      });

      if (response.ok) {
        showNotification("Data source removed", "success");
        loadDataSources();
      }
    } catch (e) {
      console.error("Failed to remove data source:", e);
    }
  }

  // =============================================================================
  // WIDGETS
  // =============================================================================

  function showAddWidgetModal() {
    const modal = document.getElementById("addWidgetModal");
    if (modal) {
      modal.style.display = "flex";
      state.selectedWidgetType = null;
      updateWidgetTypeSelection();
    }
  }

  function closeAddWidgetModal() {
    const modal = document.getElementById("addWidgetModal");
    if (modal) {
      modal.style.display = "none";
    }
  }

  function selectWidgetType(widgetType) {
    state.selectedWidgetType = widgetType;
    updateWidgetTypeSelection();
  }

  function updateWidgetTypeSelection() {
    document.querySelectorAll(".widget-option").forEach((btn) => {
      btn.classList.toggle(
        "selected",
        btn.dataset.type === state.selectedWidgetType,
      );
    });

    // Show/hide configuration section
    const configSection = document.getElementById("widget-config-section");
    if (configSection) {
      configSection.style.display = state.selectedWidgetType ? "block" : "none";
    }
  }

  async function addWidget(formData) {
    if (!state.currentDashboard || !state.selectedWidgetType) {
      showNotification("Please select a widget type", "error");
      return;
    }

    const data = {
      dashboard_id: state.currentDashboard.id,
      widget_type: state.selectedWidgetType,
      title: formData.get("widget_title") || "Untitled Widget",
      data_source_id: formData.get("data_source_id"),
      config: {
        width: parseInt(formData.get("width")) || 1,
        height: parseInt(formData.get("height")) || 1,
      },
    };

    try {
      const response = await fetch(
        `/api/dashboards/${state.currentDashboard.id}/widgets`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        },
      );

      if (response.ok) {
        showNotification("Widget added", "success");
        closeAddWidgetModal();
        openDashboard(state.currentDashboard.id);
      } else {
        showNotification("Failed to add widget", "error");
      }
    } catch (e) {
      console.error("Failed to add widget:", e);
      showNotification("Failed to add widget", "error");
    }
  }

  function editWidget(widgetId) {
    // TODO: Implement widget editing modal
    showNotification("Widget editing coming soon", "info");
  }

  async function removeWidget(widgetId) {
    if (!confirm("Remove this widget?")) return;

    try {
      const response = await fetch(`/api/dashboards/widgets/${widgetId}`, {
        method: "DELETE",
      });

      if (response.ok) {
        showNotification("Widget removed", "success");
        if (state.currentDashboard) {
          openDashboard(state.currentDashboard.id);
        }
      }
    } catch (e) {
      console.error("Failed to remove widget:", e);
    }
  }

  // =============================================================================
  // UTILITIES
  // =============================================================================

  function escapeHtml(text) {
    if (!text) return "";
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function formatRelativeTime(dateString) {
    if (!dateString) return "Never";
    try {
      const date = new Date(dateString);
      const now = new Date();
      const diffMs = now - date;
      const diffMin = Math.floor(diffMs / 60000);
      const diffHour = Math.floor(diffMin / 60);
      const diffDay = Math.floor(diffHour / 24);

      if (diffMin < 1) return "just now";
      if (diffMin < 60) return `${diffMin}m ago`;
      if (diffHour < 24) return `${diffHour}h ago`;
      if (diffDay < 7) return `${diffDay}d ago`;
      return date.toLocaleDateString();
    } catch {
      return dateString;
    }
  }

  function showNotification(message, type) {
    if (typeof window.showNotification === "function") {
      window.showNotification(message, type);
    } else if (typeof window.GBAlerts !== "undefined") {
      if (type === "success") window.GBAlerts.success("Dashboards", message);
      else if (type === "error") window.GBAlerts.error("Dashboards", message);
      else window.GBAlerts.info("Dashboards", message);
    } else {
      console.log(`[${type}] ${message}`);
    }
  }

  // =============================================================================
  // EXPORT TO WINDOW
  // =============================================================================

  // Create Dashboard Modal
  window.showCreateDashboardModal = showCreateDashboardModal;
  window.closeCreateDashboardModal = closeCreateDashboardModal;

  // Dashboard Viewer
  window.openDashboard = openDashboard;
  window.closeDashboardViewer = closeDashboardViewer;
  window.refreshDashboard = refreshDashboard;
  window.editDashboard = editDashboard;
  window.editDashboardById = editDashboardById;
  window.shareDashboard = shareDashboard;
  window.exportDashboard = exportDashboard;
  window.duplicateDashboard = duplicateDashboard;
  window.deleteDashboard = deleteDashboard;

  // Data Sources
  window.showAddDataSourceModal = showAddDataSourceModal;
  window.closeAddDataSourceModal = closeAddDataSourceModal;
  window.testDataSourceConnection = testDataSourceConnection;
  window.removeDataSource = removeDataSource;

  // Widgets
  window.showAddWidgetModal = showAddWidgetModal;
  window.closeAddWidgetModal = closeAddWidgetModal;
  window.selectWidgetType = selectWidgetType;
  window.editWidget = editWidget;
  window.removeWidget = removeWidget;

  // =============================================================================
  // INITIALIZE
  // =============================================================================

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
