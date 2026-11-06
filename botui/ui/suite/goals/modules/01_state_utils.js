/* Goals app — module 1/6: state and shared helpers (split from goals/goals.js for #1731)
   =============================================================================
   GOALS/OKR MODULE - Objectives & Key Results
   ============================================================================= */

(function (app) {
  "use strict";

  // =============================================================================
  // STATE
  // =============================================================================

  const state = {
    currentView: "dashboard",
    objectives: [],
    selectedObjective: null,
    filters: {
      status: "all",
      owner: "all",
      period: "current",
    },
  };

  // =============================================================================
  // UTILITIES
  // =============================================================================

  function escapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function formatDate(dateString) {
    if (!dateString) return "Not set";
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString();
    } catch {
      return dateString;
    }
  }

  function showNotification(message, type) {
    if (typeof window.showNotification === "function") {
      window.showNotification(message, type);
    } else if (typeof window.GBAlerts !== "undefined") {
      if (type === "success") window.GBAlerts.success("Goals", message);
      else if (type === "error") window.GBAlerts.error("Goals", message);
      else window.GBAlerts.info("Goals", message);
    } else {
      console.log(`[${type}] ${message}`);
    }
  }

  Object.assign(app, { state, escapeHtml, formatDate, showNotification });
})(window.__gbGoals = window.__gbGoals || {});
