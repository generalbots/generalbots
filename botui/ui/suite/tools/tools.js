/**
 * Tools Module JavaScript
 * Compliance, Analytics, and Developer Tools
 */
(function () {
  "use strict";

  /**
   * Initialize the Tools module
   */
  function init() {
    setupBotSelector();
    setupFilters();
    setupKeyboardShortcuts();
    setupHTMXEvents();
    updateStats();
  }

  /**
   * Setup bot chip selection
   */
  function setupBotSelector() {
    document.addEventListener("click", function (e) {
      const chip = e.target.closest(".bot-chip");
      if (chip) {
        // Toggle selection
        chip.classList.toggle("selected");

        // Update hidden checkbox
        const checkbox = chip.querySelector('input[type="checkbox"]');
        if (checkbox) {
          checkbox.checked = chip.classList.contains("selected");
        }

        // Handle "All Bots" logic
        if (chip.querySelector('input[value="all"]')) {
          if (chip.classList.contains("selected")) {
            // Deselect all other chips
            document
              .querySelectorAll(".bot-chip:not([data-all])")
              .forEach((c) => {
                c.classList.remove("selected");
                const cb = c.querySelector('input[type="checkbox"]');
                if (cb) cb.checked = false;
              });
          }
        } else {
          // Deselect "All Bots" when selecting individual bots
          const allChip = document
            .querySelector('.bot-chip input[value="all"]')
            ?.closest(".bot-chip");
          if (allChip) {
            allChip.classList.remove("selected");
            const cb = allChip.querySelector('input[type="checkbox"]');
            if (cb) cb.checked = false;
          }
        }
      }
    });
  }

  /**
   * Setup filter controls
   */
  function setupFilters() {
    // Filter select changes
    document.querySelectorAll(".filter-select").forEach((select) => {
      select.addEventListener("change", function () {
        applyFilters();
      });
    });

    // Search input
    const searchInput = document.querySelector(
      '.filter-input[name="filter-search"]',
    );
    if (searchInput) {
      let debounceTimer;
      searchInput.addEventListener("input", function () {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => applyFilters(), 300);
      });
    }
  }

  /**
   * Apply filters to results
   */
  function applyFilters() {
    const severity = document.getElementById("filter-severity")?.value || "all";
    const type = document.getElementById("filter-type")?.value || "all";
    const search =
      document
        .querySelector('.filter-input[name="filter-search"]')
        ?.value.toLowerCase() || "";

    const rows = document.querySelectorAll("#results-body tr");
    let visibleCount = 0;

    rows.forEach((row) => {
      let visible = true;

      // Filter by severity
      if (severity !== "all") {
        const badge = row.querySelector(".severity-badge");
        if (badge && !badge.classList.contains(severity)) {
          visible = false;
        }
      }

      // Filter by type
      if (type !== "all" && visible) {
        const issueIcon = row.querySelector(".issue-icon");
        if (issueIcon && !issueIcon.classList.contains(type)) {
          visible = false;
        }
      }

      // Filter by search
      if (search && visible) {
        const text = row.textContent.toLowerCase();
        if (!text.includes(search)) {
          visible = false;
        }
      }

      row.style.display = visible ? "" : "none";
      if (visible) visibleCount++;
    });

    // Update results count
    const countEl = document.getElementById("results-count");
    if (countEl) {
      countEl.textContent = `${visibleCount} issues found`;
    }
  }

  /**
   * Setup keyboard shortcuts
   */
  function setupKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
      // Ctrl+Enter to run scan
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        document.getElementById("scan-btn")?.click();
      }

      // Escape to close any open modals
      if (e.key === "Escape") {
        closeModals();
      }

      // Ctrl+E to export report
      if ((e.ctrlKey || e.metaKey) && e.key === "e") {
        e.preventDefault();
        exportReport();
      }
    });
  }

  /**
   * Setup HTMX events
   */
  function setupHTMXEvents() {
    if (typeof htmx === "undefined") return;

    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.detail.target.id === "scan-results") {
        updateStats();
      }
    });
  }

  /**
   * Update statistics from results
   */
  function updateStats() {
    const rows = document.querySelectorAll("#results-body tr");
    let stats = { critical: 0, high: 0, medium: 0, low: 0, info: 0 };

    rows.forEach((row) => {
      if (row.style.display === "none") return;

      const badge = row.querySelector(".severity-badge");
      if (badge) {
        if (badge.classList.contains("critical")) stats.critical++;
        else if (badge.classList.contains("high")) stats.high++;
        else if (badge.classList.contains("medium")) stats.medium++;
        else if (badge.classList.contains("low")) stats.low++;
        else if (badge.classList.contains("info")) stats.info++;
      }
    });

    // Update stat cards
    const updateStat = (id, value) => {
      const el = document.getElementById(id);
      if (el) el.textContent = value;
    };

    updateStat("stat-critical", stats.critical);
    updateStat("stat-high", stats.high);
    updateStat("stat-medium", stats.medium);
    updateStat("stat-low", stats.low);
    updateStat("stat-info", stats.info);

    // Update total count
    const total =
      stats.critical + stats.high + stats.medium + stats.low + stats.info;
    const countEl = document.getElementById("results-count");
    if (countEl) {
      countEl.textContent = `${total} issues found`;
    }
  }

  /**
   * Export compliance report
   */
  function exportReport() {
    if (typeof htmx !== "undefined") {
      htmx.ajax("GET", "/api/compliance/export", {
        swap: "none",
      });
    }
  }

  /**
   * Close all modals
   */
  function closeModals() {
    document.querySelectorAll(".modal").forEach((modal) => {
      modal.classList.add("hidden");
    });
  }

  /**
   * Show toast notification
   */
  function showToast(message, type = "success") {
    const toast = document.createElement("div");
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    document.body.appendChild(toast);

    requestAnimationFrame(() => {
      toast.classList.add("show");
    });

    setTimeout(() => {
      toast.classList.remove("show");
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }

  // Initialize on DOM ready
  if (document.readyState === "loading") {
    (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    init();
  }

  /**
   * Reindex a data source for search
   */
  function reindexSource(sourceName) {
    showToast(`Reindexing ${sourceName}...`, "info");

    fetch(`/api/search/reindex/${sourceName}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    })
      .then((r) => r.json())
      .then((result) => {
        showToast(
          `${sourceName} reindexing started. ${result.documents || 0} documents queued.`,
          "success",
        );
      })
      .catch((err) => {
        console.error(`Error reindexing ${sourceName}:`, err);
        showToast(`Failed to reindex ${sourceName}`, "error");
      });
  }

  // Expose for external use
  window.Tools = {
    updateStats,
    applyFilters,
    exportReport,
    showToast,
  };

  window.reindexSource = reindexSource;
})();
