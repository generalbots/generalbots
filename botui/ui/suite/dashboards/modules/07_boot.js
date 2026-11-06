/* Dashboards app — split from dashboards/dashboards.js for #1731 */

  // =============================================================================
  // UTILITIES
  // =============================================================================

  function dashEscapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
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
    (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    init();
  }
