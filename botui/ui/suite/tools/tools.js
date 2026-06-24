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
   * Fix an issue
   */
  function fixIssue(issueId) {
    if (typeof htmx !== "undefined") {
      htmx
        .ajax("POST", `/api/compliance/fix/${issueId}`, {
          swap: "none",
        })
        .then(() => {
          // Refresh results
          const scanBtn = document.getElementById("scan-btn");
          if (scanBtn) scanBtn.click();
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
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  /**
   * Configure a protection tool
   */
  function configureProtectionTool(toolName) {
    const modal =
      document.getElementById("configure-modal") ||
      document.getElementById("tool-config-modal");
    if (modal) {
      const titleEl = modal.querySelector(".modal-title, h2, h3");
      if (titleEl) {
        titleEl.textContent = `Configure ${toolName}`;
      }
      modal.dataset.tool = toolName;
      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.remove("hidden");
        modal.style.display = "flex";
      }
    } else {
      showToast(`Opening configuration for ${toolName}...`, "info");
      fetch(`/api/tools/security/${toolName}/config`)
        .then((r) => r.json())
        .then((config) => {
          console.log(`${toolName} config:`, config);
          showToast(`${toolName} configuration loaded`, "success");
        })
        .catch((err) => {
          console.error(`Error loading ${toolName} config:`, err);
          showToast(`Failed to load ${toolName} configuration`, "error");
        });
    }
  }

  /**
   * Run a protection tool scan
   */
  function runProtectionTool(toolName) {
    showToast(`Running ${toolName} scan...`, "info");

    const statusEl = document.querySelector(
      `[data-tool="${toolName}"] .tool-status, #${toolName}-status`,
    );
    if (statusEl) {
      statusEl.textContent = "Running...";
      statusEl.classList.add("running");
    }

    fetch(`/api/tools/security/${toolName}/run`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    })
      .then((r) => r.json())
      .then((result) => {
        if (statusEl) {
          statusEl.textContent = "Completed";
          statusEl.classList.remove("running");
          statusEl.classList.add("completed");
        }
        showToast(`${toolName} scan completed`, "success");

        if (result.report_url) {
          viewReport(toolName);
        }
      })
      .catch((err) => {
        console.error(`Error running ${toolName}:`, err);
        if (statusEl) {
          statusEl.textContent = "Error";
          statusEl.classList.remove("running");
          statusEl.classList.add("error");
        }
        showToast(`Failed to run ${toolName}`, "error");
      });
  }

  /**
   * Update a protection tool
   */
  function updateProtectionTool(toolName) {
    showToast(`Updating ${toolName}...`, "info");

    fetch(`/api/tools/security/${toolName}/update`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    })
      .then((r) => r.json())
      .then((result) => {
        showToast(
          `${toolName} updated to version ${result.version || "latest"}`,
          "success",
        );

        const versionEl = document.querySelector(
          `[data-tool="${toolName}"] .tool-version, #${toolName}-version`,
        );
        if (versionEl && result.version) {
          versionEl.textContent = result.version;
        }
      })
      .catch((err) => {
        console.error(`Error updating ${toolName}:`, err);
        showToast(`Failed to update ${toolName}`, "error");
      });
  }

  /**
   * View report for a protection tool
   */
  function viewReport(toolName) {
    const reportModal =
      document.getElementById("report-modal") ||
      document.getElementById("view-report-modal");

    if (reportModal) {
      const titleEl = reportModal.querySelector(".modal-title, h2, h3");
      if (titleEl) {
        titleEl.textContent = `${toolName} Report`;
      }

      const contentEl = reportModal.querySelector(
        ".report-content, .modal-body",
      );
      if (contentEl) {
        contentEl.innerHTML = '<div class="loading">Loading report...</div>';
      }

      if (reportModal.showModal) {
        reportModal.showModal();
      } else {
        reportModal.classList.remove("hidden");
        reportModal.style.display = "flex";
      }

      fetch(`/api/tools/security/${toolName}/report`)
        .then((r) => r.json())
        .then((report) => {
          if (contentEl) {
            contentEl.innerHTML = renderReport(toolName, report);
          }
        })
        .catch((err) => {
          console.error(`Error loading ${toolName} report:`, err);
          if (contentEl) {
            contentEl.innerHTML =
              '<div class="error">Failed to load report</div>';
          }
        });
    } else {
      window.open(
        `/api/tools/security/${toolName}/report?format=html`,
        "_blank",
      );
    }
  }

  /**
   * Render a security tool report
   */
  function renderReport(toolName, report) {
    const findings = report.findings || [];
    const summary = report.summary || {};

    return `
            <div class="report-summary">
                <h4>Summary</h4>
                <div class="summary-stats">
                    <span class="stat critical">${summary.critical || 0} Critical</span>
                    <span class="stat high">${summary.high || 0} High</span>
                    <span class="stat medium">${summary.medium || 0} Medium</span>
                    <span class="stat low">${summary.low || 0} Low</span>
                </div>
                <p>Scan completed: ${report.completed_at || new Date().toISOString()}</p>
            </div>
            <div class="report-findings">
                <h4>Findings (${findings.length})</h4>
                ${findings.length === 0 ? '<p class="no-findings">No issues found</p>' : ""}
                ${findings
                  .map(
                    (f) => `
                    <div class="finding ${f.severity || "info"}">
                        <span class="severity-badge ${f.severity || "info"}">${f.severity || "info"}</span>
                        <span class="finding-title">${f.title || f.message || "Finding"}</span>
                        <p class="finding-description">${f.description || ""}</p>
                        ${f.remediation ? `<p class="finding-remediation"><strong>Fix:</strong> ${f.remediation}</p>` : ""}
                    </div>
                `,
                  )
                  .join("")}
            </div>
        `;
  }

  /**
   * Toggle auto action for a protection tool
   */
  function toggleAutoAction(toolName, btn) {
    const isEnabled =
      btn.classList.contains("active") ||
      btn.getAttribute("aria-pressed") === "true";
    const newState = !isEnabled;

    fetch(`/api/tools/security/${toolName}/auto`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ enabled: newState }),
    })
      .then((r) => r.json())
      .then((result) => {
        if (newState) {
          btn.classList.add("active");
          btn.setAttribute("aria-pressed", "true");
          showToast(`Auto-scan enabled for ${toolName}`, "success");
        } else {
          btn.classList.remove("active");
          btn.setAttribute("aria-pressed", "false");
          showToast(`Auto-scan disabled for ${toolName}`, "info");
        }
      })
      .catch((err) => {
        console.error(`Error toggling auto action for ${toolName}:`, err);
        showToast(`Failed to update ${toolName} settings`, "error");
      });
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

  /**
   * Show TSC (Trust Service Criteria) details
   */
  function showTscDetails(category) {
    const detailPanel =
      document.getElementById("tsc-detail-panel") ||
      document.querySelector(".tsc-details");

    if (detailPanel) {
      fetch(`/api/compliance/tsc/${category}`)
        .then((r) => r.json())
        .then((data) => {
          detailPanel.innerHTML = renderTscDetails(category, data);
          detailPanel.classList.add("open");
        })
        .catch((err) => {
          console.error(`Error loading TSC details for ${category}:`, err);
          showToast(`Failed to load ${category} details`, "error");
        });
    } else {
      showToast(`Viewing ${category} criteria...`, "info");
    }
  }

  /**
   * Render TSC details
   */
  function renderTscDetails(category, data) {
    const controls = data.controls || [];
    return `
            <div class="tsc-detail-header">
                <h3>${category.charAt(0).toUpperCase() + category.slice(1)} Criteria</h3>
                <button class="close-btn" onclick="document.querySelector('.tsc-details').classList.remove('open')">×</button>
            </div>
            <div class="tsc-controls">
                ${controls
                  .map(
                    (c) => `
                    <div class="control-item ${c.status || "pending"}">
                        <span class="control-id">${c.id}</span>
                        <span class="control-name">${c.name}</span>
                        <span class="control-status">${c.status || "Pending"}</span>
                    </div>
                `,
                  )
                  .join("")}
            </div>
        `;
  }

  /**
   * Show control remediation steps
   */
  function showControlRemediation(controlId) {
    const modal =
      document.getElementById("remediation-modal") ||
      document.getElementById("control-modal");

    if (modal) {
      const titleEl = modal.querySelector(".modal-title, h2, h3");
      if (titleEl) {
        titleEl.textContent = `Remediate ${controlId}`;
      }

      const contentEl = modal.querySelector(
        ".modal-body, .remediation-content",
      );
      if (contentEl) {
        contentEl.innerHTML =
          '<div class="loading">Loading remediation steps...</div>';
      }

      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.remove("hidden");
        modal.style.display = "flex";
      }

      fetch(`/api/compliance/controls/${controlId}/remediation`)
        .then((r) => r.json())
        .then((data) => {
          if (contentEl) {
            contentEl.innerHTML = `
                            <div class="remediation-steps">
                                <h4>Steps to Remediate</h4>
                                <ol>
                                    ${(data.steps || []).map((s) => `<li>${s}</li>`).join("")}
                                </ol>
                                ${data.documentation_url ? `<a href="${data.documentation_url}" target="_blank" class="btn btn-secondary">View Documentation</a>` : ""}
                            </div>
                        `;
          }
        })
        .catch((err) => {
          console.error(`Error loading remediation for ${controlId}:`, err);
          if (contentEl) {
            contentEl.innerHTML =
              '<div class="error">Failed to load remediation steps</div>';
          }
        });
    } else {
      showToast(`Loading remediation for ${controlId}...`, "info");
    }
  }

  // Expose for external use
  window.Tools = {
    updateStats,
    applyFilters,
    fixIssue,
    exportReport,
    showToast,
  };

  // Expose security tool functions globally
  window.configureProtectionTool = configureProtectionTool;
  window.runProtectionTool = runProtectionTool;
  window.updateProtectionTool = updateProtectionTool;
  window.viewReport = viewReport;
  window.toggleAutoAction = toggleAutoAction;
  window.reindexSource = reindexSource;
  window.showTscDetails = showTscDetails;
  window.showControlRemediation = showControlRemediation;
})();
