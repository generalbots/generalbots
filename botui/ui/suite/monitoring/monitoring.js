/* Monitoring module - shared/base JavaScript */

(function () {
  "use strict";

  function setActiveNav(element) {
    document.querySelectorAll(".monitoring-nav .nav-item").forEach((item) => {
      item.classList.remove("active");
    });
    element.classList.add("active");

    // Update page title
    const title = element.querySelector(
      "span:not(.alert-badge):not(.health-indicator)",
    ).textContent;
    document.getElementById("page-title").textContent = title;
  }

  function updateTimeRange(range) {
    // Store selected time range
    localStorage.setItem("monitoring-time-range", range);

    // Trigger refresh of current view
    htmx.trigger("#monitoring-content", "refresh");
  }

  function refreshMonitoring() {
    htmx.trigger("#monitoring-content", "refresh");

    // Visual feedback
    const btn = event.currentTarget;
    btn.classList.add("active");
    setTimeout(() => btn.classList.remove("active"), 500);
  }

  // Guard against duplicate declarations on HTMX reload
  if (typeof window.monitoringModuleInitialized === "undefined") {
    window.monitoringModuleInitialized = true;
    var autoRefresh = true;
  }

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;
    const btn = document.getElementById("auto-refresh-btn");
    btn.classList.toggle("active", autoRefresh);

    if (autoRefresh) {
      // Re-enable polling by refreshing the page content
      htmx.trigger("#monitoring-content", "refresh");
    }
  }

  function exportData() {
    const timeRange = document.getElementById("time-range").value;
    window.open(`/api/monitoring/export?range=${timeRange}`, "_blank");
  }

  // Initialize
  document.addEventListener("DOMContentLoaded", function () {
    // Restore time range preference
    const savedRange = localStorage.getItem("monitoring-time-range");
    if (savedRange) {
      const timeRangeEl = document.getElementById("time-range");
      if (timeRangeEl) timeRangeEl.value = savedRange;
    }

    // Set auto-refresh button state
    const autoRefreshBtn = document.getElementById("auto-refresh-btn");
    if (autoRefreshBtn) autoRefreshBtn.classList.toggle("active", autoRefresh);
  });

  // Handle HTMX events for loading states
  document.body.addEventListener("htmx:beforeRequest", function (evt) {
    if (evt.target.id === "monitoring-content") {
      evt.target.innerHTML =
        '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    }
  });

  function initViewToggle() {
    var toggleBtn = document.getElementById("view-toggle");
    if (toggleBtn) {
      toggleBtn.addEventListener("click", function () {
        var liveView = document.getElementById("live-view");
        var gridView = document.getElementById("grid-view");

        if (liveView && gridView) {
          if (liveView.style.display === "none") {
            liveView.style.display = "flex";
            gridView.style.display = "none";
          } else {
            liveView.style.display = "none";
            gridView.style.display = "grid";
          }
        }
      });
    }
  }

  function updateServiceStatusDots(event) {
    if (event.detail.target.id === "service-status-container") {
      try {
        var data = JSON.parse(event.detail.target.textContent);
        Object.entries(data).forEach(function (entry) {
          var service = entry[0];
          var status = entry[1];
          var dot = document.querySelector('[data-status="' + service + '"]');
          if (dot) {
            dot.classList.remove("running", "warning", "stopped");
            dot.classList.add(status);
          }
        });
      } catch (e) {
        // Silent fail for non-JSON responses
      }
    }
  }

  function handleKeyboardShortcuts(e) {
    if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") {
      return;
    }

    if (e.key === "r" && !e.ctrlKey && !e.metaKey) {
      if (typeof htmx !== "undefined") {
        htmx.trigger(document.body, "htmx:load");
      }
    }
    if (e.key === "v" && !e.ctrlKey && !e.metaKey) {
      var toggleBtn = document.getElementById("view-toggle");
      if (toggleBtn) {
        toggleBtn.click();
      }
    }
  }

  function initMonitoring() {
    initViewToggle();
    document.body.addEventListener("htmx:afterSwap", updateServiceStatusDots);
    document.addEventListener("keydown", handleKeyboardShortcuts);
  }

  window.setActiveNav = setActiveNav;
  window.updateTimeRange = updateTimeRange;
  window.refreshMonitoring = refreshMonitoring;
  window.toggleAutoRefresh = toggleAutoRefresh;
  window.exportData = exportData;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initMonitoring);
  } else {
    initMonitoring();
  }

  document.body.addEventListener("htmx:afterSwap", function (evt) {
    if (evt.detail.target && evt.detail.target.id === "main-content") {
      if (document.querySelector(".monitoring-container")) {
        initMonitoring();
      }
    }
  });
})();
