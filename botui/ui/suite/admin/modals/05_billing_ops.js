/* Admin modals — billing admin and operations dashboard */
(function (window) {
  "use strict";
  var A = (window.__GBAdminModals = window.__GBAdminModals || {});
  var showModal = A.showModal;
  var hideModal = A.hideModal;
  var showNotification = A.showNotification;

// BILLING ADMIN FUNCTIONS
// =============================================================================

function downloadInvoice(invoiceId) {
  showNotification("Exporting invoices...", "info");

  fetch("/api/ui/billing/invoices/export")
    .then((response) => {
      if (response.ok) {
        return response.blob();
      }
      throw new Error("Download failed");
    })
    .then((blob) => {
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "invoices-export.csv";
      link.click();
      URL.revokeObjectURL(url);
      showNotification("Invoices exported", "success");
    })
    .catch((err) =>
      showNotification("Failed to download: " + err.message, "error"),
    );
}

function dismissAlert(button) {
  const alertItem = button.closest(".alert-item");
  if (alertItem) {
    alertItem.style.opacity = "0";
    alertItem.style.transform = "translateX(100%)";
    setTimeout(() => alertItem.remove(), 300);
  }
}

function viewEvidence(evidenceId) {
  showNotification(`Loading evidence: ${evidenceId}...`, "info");

  // Load the real evidence list fragment into the modal
  showModal("evidence-view-modal");
  const content = document.getElementById("evidence-content");
  if (content) {
    fetch("/api/compliance/dashboard/evidence")
      .then((response) => response.text())
      .then((html) => {
        content.innerHTML = html;
      })
      .catch((err) =>
        showNotification("Failed to load evidence: " + err.message, "error"),
      );
  }
}

// =============================================================================
// OPERATIONS DASHBOARD FUNCTIONS
// =============================================================================

let autoRefreshEnabled = true;
let autoRefreshInterval = null;

function toggleAutoRefresh() {
  autoRefreshEnabled = !autoRefreshEnabled;
  const label = document.getElementById("autoRefreshLabel");
  if (label) {
    label.textContent = `Auto-refresh: ${autoRefreshEnabled ? "ON" : "OFF"}`;
  }

  if (autoRefreshEnabled) {
    startAutoRefresh();
  } else {
    stopAutoRefresh();
  }
}

function startAutoRefresh() {
  if (autoRefreshInterval) clearInterval(autoRefreshInterval);
  autoRefreshInterval = setInterval(() => {
    refreshHealth();
  }, 30000);
}

function stopAutoRefresh() {
  if (autoRefreshInterval) {
    clearInterval(autoRefreshInterval);
    autoRefreshInterval = null;
  }
}

function showAlertConfig() {
  showModal("alert-config-modal");
}

function closeAlertConfig() {
  hideModal("alert-config-modal");
}

function showTraceDetail(traceId) {
  showNotification(`Loading trace ${traceId}...`, "info");

  fetch(`/api/ops/traces/${traceId}`)
    .then((response) => response.json())
    .then((data) => {
      showModal("trace-detail-modal");
      const content = document.getElementById("trace-detail-content");
      if (content) {
        content.innerHTML = `
                      <div class="trace-header">
                          <h4>${data.name || traceId}</h4>
                          <span class="trace-status ${data.status || "success"}">${data.status || "Success"}</span>
                      </div>
                      <div class="trace-info-grid">
                          <div class="trace-info-item">
                              <label>Trace ID</label>
                              <span>${data.traceId || traceId}</span>
                          </div>
                          <div class="trace-info-item">
                              <label>Duration</label>
                              <span>${data.duration || "0"}ms</span>
                          </div>
                          <div class="trace-info-item">
                              <label>Spans</label>
                              <span>${data.spanCount || 0}</span>
                          </div>
                          <div class="trace-info-item">
                              <label>Service</label>
                              <span>${data.service || "unknown"}</span>
                          </div>
                      </div>
                      <div class="trace-spans">
                          ${(data.spans || [])
                            .map(
                              (span) => `
                              <div class="span-item ${span.status || "success"}">
                                  <span class="span-name">${span.name}</span>
                                  <span class="span-duration">${span.duration}ms</span>
                              </div>
                          `,
                            )
                            .join("")}
                      </div>
                  `;
      }
    })
    .catch((err) =>
      showNotification("Failed to load trace: " + err.message, "error"),
    );
}

function refreshHealth() {
  const healthGrid = document.querySelector(".health-grid");
  if (healthGrid) {
    healthGrid.innerHTML =
      '<div class="loading-state"><div class="spinner"></div></div>';
  }

  fetch("/api/ops/health")
    .then((response) => response.json())
    .then((data) => {
      if (healthGrid) {
        healthGrid.innerHTML =
          (data.services || [])
            .map(
              (service) => `
                      <div class="health-item ${service.status}">
                          <div class="health-status">
                              <span class="status-indicator ${service.status}"></span>
                              <span class="service-name">${service.name}</span>
                          </div>
                          <div class="health-metrics">
                              <span>Latency: ${service.latency || 0}ms</span>
                              <span>Uptime: ${service.uptime || "100%"}</span>
                          </div>
                      </div>
                  `,
            )
            .join("") || '<div class="empty-state">No services found</div>';
      }
    })
    .catch((err) => {
      if (healthGrid) {
        healthGrid.innerHTML =
          '<div class="error-state">Failed to load health data</div>';
      }
    });
}

// =============================================================================

  A.downloadInvoice = downloadInvoice;
  A.dismissAlert = dismissAlert;
  A.viewEvidence = viewEvidence;
  A.toggleAutoRefresh = toggleAutoRefresh;
  A.startAutoRefresh = startAutoRefresh;
  A.stopAutoRefresh = stopAutoRefresh;
  A.showAlertConfig = showAlertConfig;
  A.closeAlertConfig = closeAlertConfig;
  A.showTraceDetail = showTraceDetail;
  A.refreshHealth = refreshHealth;
})(window);
