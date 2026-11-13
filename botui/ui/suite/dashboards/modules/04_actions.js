/* Dashboards app — split from dashboards/dashboards.js for #1731 */

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

  function gbDashConfirm(message, title) {
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

  async function deleteDashboard(dashboardId) {
    if (!(await gbDashConfirm("Delete this dashboard? This cannot be undone.", "Confirm"))) return;

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
