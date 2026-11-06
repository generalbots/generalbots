/* Dashboards app — split from dashboards/dashboards.js for #1731 */

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
          <h3>${dashEscapeHtml(dashboard.name)}</h3>
          <p>${dashEscapeHtml(dashboard.description || "No description")}</p>
          <div class="dashboard-meta">
            <span class="category">${dashEscapeHtml(dashboard.category || "General")}</span>
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
