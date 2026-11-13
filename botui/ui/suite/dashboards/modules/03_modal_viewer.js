/* Dashboards app — split from dashboards/dashboards.js for #1731 */

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
        <div class="empty-widgets" role="status" aria-live="polite">
          <span class="empty-icon" aria-hidden="true">📈</span>
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
          <h4>${dashEscapeHtml(widget.title)}</h4>
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
    const cfg = widget.config || {};
    const chart = cfg.chart_config || {};
    const series = chart.series || [];
    const type = widget.widget_type || "line_chart";

    if (type === "kpi") {
      const kpi = cfg.kpi_config || {};
      const value = kpi.value != null ? kpi.value : "--";
      return `<div class="widget-kpi"><span class="widget-kpi-value">${dashEscapeHtml(String(value))}</span><span class="widget-kpi-label">${dashEscapeHtml(kpi.label || widget.title || "")}</span></div>`;
    }
    if (type === "text") {
      return `<div class="widget-text">${dashEscapeHtml(cfg.text_content || "")}</div>`;
    }
    if (type === "image") {
      return cfg.image_url
        ? `<img src="${dashEscapeHtml(cfg.image_url)}" alt="${dashEscapeHtml(widget.title)}" class="widget-image" />`
        : `<div class="widget-empty" role="status">No image configured</div>`;
    }
    if (type === "iframe") {
      return cfg.iframe_url
        ? `<iframe src="${dashEscapeHtml(cfg.iframe_url)}" class="widget-iframe" title="${dashEscapeHtml(widget.title)}"></iframe>`
        : `<div class="widget-empty" role="status">No URL configured</div>`;
    }
    if (type === "table") {
      const table = cfg.table_config || {};
      const columns = table.columns || [];
      if (!columns.length) return `<div class="widget-empty" role="status">No columns configured</div>`;
      return `<div class="widget-table-wrap"><table class="widget-table"><thead><tr>${columns.map(c => `<th>${dashEscapeHtml(c.label || c.field || "")}</th>`).join("")}</tr></thead><tbody></tbody></table></div>`;
    }

    // Charts: line, bar, area, pie, scatter, gauge
    const data = widget.data || (widget.latest_data && widget.latest_data.values) || [];
    const values = Array.isArray(data)
      ? data.map(d => {
          const field = series[1] ? series[1].field : (series[0] ? series[0].field : "value");
          return parseFloat(d && d[field]);
        }).filter(v => !isNaN(v))
      : [];

    if (!values.length) {
      return `<div class="widget-empty" role="status">No data yet — configure a data source and refresh.</div>`;
    }

    const max = Math.max(...values, 1);
    const bars = values.slice(0, 24).map(v => {
      const h = Math.round((v / max) * 100);
      return `<div class="widget-bar" style="height:${h}%" title="${v}"></div>`;
    }).join("");

    if (type === "pie_chart") {
      const total = values.reduce((a, b) => a + b, 0) || 1;
      const segments = values.map((v, i) => {
        const pct = (v / total) * 100;
        return `<div class="widget-pie-segment" style="background:${(chart.colors && chart.colors[i]) || "var(--accent,#d4f505)"};width:${pct}%"></div>`;
      }).join("");
      return `<div class="widget-pie">${segments}</div>`;
    }
    if (type === "gauge") {
      const pct = Math.min(100, Math.round((values[values.length - 1] || 0) / max * 100));
      return `<div class="widget-gauge"><span class="widget-gauge-value">${pct}%</span></div>`;
    }

    return `<div class="widget-chart">${bars}</div>`;
  }
