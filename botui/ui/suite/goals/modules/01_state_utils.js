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

  /**
   * Turns a value that reached the UI as data into readable text (#1979).
   * Objectives, key results and run values are authored by API callers and
   * agents, so a description or a value can arrive as a JSON object, a JSON
   * string or a plain scalar. The UI must show that as words, never as raw
   * JSON: `{ "owner": "ana", "quarter": "Q3" }` becomes
   * `owner: ana · quarter: Q3`.
   */
  function humanizeText(value) {
    if (value === null || value === undefined) return "";
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    if (Array.isArray(value)) {
      return value.map((entry) => humanizeText(entry)).filter(Boolean).join(", ");
    }
    if (typeof value === "object") {
      return Object.keys(value)
        .filter((key) => value[key] !== null && value[key] !== undefined && value[key] !== "")
        .map((key) => {
          const label = key.replace(/[_-]+/g, " ").replace(/^\w/, (c) => c.toUpperCase());
          return `${label}: ${humanizeText(value[key])}`;
        })
        .join(" \u00b7 ");
    }
    const text = String(value).trim();
    if (text.startsWith("{") || text.startsWith("[")) {
      try {
        return humanizeText(JSON.parse(text));
      } catch {
        // Not JSON after all — show the text itself.
      }
    }
    return text;
  }

  function showNotification(message, type) {
    message = humanizeText(message);
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

  Object.assign(app, { state, escapeHtml, formatDate, humanizeText, showNotification });
})(window.__gbGoals = window.__gbGoals || {});
