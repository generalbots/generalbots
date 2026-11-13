/* Dashboards app — split from dashboards/dashboards.js for #1731 */

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

  function gbDashPrompt(message, defaultValue, title) {
    if (window.GBModal && typeof window.GBModal.prompt === "function") {
      return new Promise((resolve) => {
        try {
          window.GBModal.prompt(message, defaultValue == null ? "" : String(defaultValue), title || "Input", (value) => {
            resolve(value == null ? null : String(value));
          });
        } catch (e) {
          console.error("GBModal.prompt failed", e);
          resolve(null);
        }
      });
    }
    return Promise.resolve(null);
  }

  function gbDashWidgetConfirm(message, title) {
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
      return new Promise((resolve) => {
        try {
          window.GBModal.confirm(message, title || "Confirm", () => resolve(true), () => resolve(false));
        } catch (e) {
          console.error("GBModal.confirm failed", e);
          resolve(false);
        }
      });
    }
    return Promise.resolve(false);
  }

  async function editWidget(widgetId) {
    if (!state.currentDashboard) return;
    const widget = (state.widgets || []).find((w) => w.id === widgetId);
    if (!widget) {
      showNotification("Widget not found", "error");
      return;
    }
    // Reuse the suite edit modal when present; otherwise fall back to an
    // inline prompt-driven flow so editing always works from this module.
    const modal = document.getElementById("editWidgetModal");
    if (modal) {
      document.getElementById("editWidgetId").value = widget.id;
      document.getElementById("editWidgetTitle").value = widget.title || "";
      document.getElementById("editWidgetType").value =
        widget.widget_type || "line_chart";
      const pos = widget.position || {};
      document.getElementById("editWidgetWidth").value = pos.width || 4;
      document.getElementById("editWidgetHeight").value = pos.height || 2;
      const cfg = widget.config || {};
      const chart = cfg.chart_config || {};
      const series = chart.series || [];
      document.getElementById("editWidgetXAxis").value =
        chart.x_axis || (series[0] ? series[0].field : "");
      document.getElementById("editWidgetYAxis").value =
        chart.y_axis || (series[1] ? series[1].field : "");
      document.getElementById("editWidgetDataSource").value =
        (widget.data_query && widget.data_query.source_id) || "";
      modal.style.display = "flex";
      return;
    }
    const title = await gbDashPrompt("Widget title", widget.title || "", "Edit Widget");
    if (title === null) return;
    const widthRaw = await gbDashPrompt("Width (columns 1-12)", String((widget.position && widget.position.width) || 4), "Edit Widget");
    if (widthRaw === null) return;
    const width = parseInt(widthRaw, 10);
    if (isNaN(width)) return;
    const heightRaw = await gbDashPrompt("Height (rows 1-12)", String((widget.position && widget.position.height) || 2), "Edit Widget");
    if (heightRaw === null) return;
    const height = parseInt(heightRaw, 10);
    if (isNaN(height)) return;
    const data = {
      title: title || "Untitled Widget",
      position: {
        x: (widget.position && widget.position.x) || 0,
        y: (widget.position && widget.position.y) || 0,
        width,
        height,
      },
    };
    try {
      const response = await fetch(
        `/api/dashboards/${state.currentDashboard.id}/widgets/${widgetId}`,
        {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        },
      );
      if (response.ok) {
        showNotification("Widget updated", "success");
        openDashboard(state.currentDashboard.id);
      } else {
        const body = await response.json().catch(() => null);
        showNotification((body && body.error) || "Failed to update widget", "error");
      }
    } catch (e) {
      console.error("Failed to update widget:", e);
      showNotification("Failed to update widget", "error");
    }
  }

  function closeEditWidgetModal() {
    const modal = document.getElementById("editWidgetModal");
    if (modal) modal.style.display = "none";
  }

  async function saveWidgetEdit() {
    if (!state.currentDashboard) return;
    const widgetId = document.getElementById("editWidgetId").value;
    if (!widgetId) {
      showNotification("Widget not found", "error");
      return;
    }
    const width = parseInt(document.getElementById("editWidgetWidth").value, 10);
    const height = parseInt(document.getElementById("editWidgetHeight").value, 10);
    if (isNaN(width) || isNaN(height) || width < 1 || height < 1) {
      showNotification("Width and height must be positive numbers", "error");
      return;
    }
    const widget = (state.widgets || []).find((w) => w.id === widgetId);
    const pos = (widget && widget.position) || { x: 0, y: 0, width: 4, height: 2 };
    const series = [
      {
        name: document.getElementById("editWidgetYAxis").value || "Value",
        field: document.getElementById("editWidgetYAxis").value || "value",
      },
    ];
    const data = {
      title: document.getElementById("editWidgetTitle").value || "Untitled Widget",
      position: {
        x: pos.x || 0,
        y: pos.y || 0,
        width,
        height,
      },
      config: {
        chart_config: {
          x_axis: document.getElementById("editWidgetXAxis").value || null,
          y_axis: document.getElementById("editWidgetYAxis").value || null,
          series,
        },
      },
    };
    try {
      const response = await fetch(
        `/api/dashboards/${state.currentDashboard.id}/widgets/${widgetId}`,
        {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        },
      );
      if (response.ok) {
        showNotification("Widget updated", "success");
        closeEditWidgetModal();
        openDashboard(state.currentDashboard.id);
      } else {
        const body = await response.json().catch(() => null);
        showNotification((body && body.error) || "Failed to update widget", "error");
      }
    } catch (e) {
      console.error("Failed to update widget:", e);
      showNotification("Failed to update widget", "error");
    }
  }

  async function removeWidget(widgetId) {
    if (!(await gbDashWidgetConfirm("Remove this widget?", "Confirm"))) return;
    if (!state.currentDashboard) return;

    try {
      const response = await fetch(
        `/api/dashboards/${state.currentDashboard.id}/widgets/${widgetId}`,
        {
          method: "DELETE",
        },
      );

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
