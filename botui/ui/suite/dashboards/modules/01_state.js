/* Dashboards app — split from dashboards/dashboards.js for #1731 */

/* =============================================================================
   DASHBOARDS MODULE - Business Intelligence Dashboards
   ============================================================================= */
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
