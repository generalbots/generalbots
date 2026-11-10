/* Admin modals — admin/billing dashboards and billing page */
(function (window) {
  "use strict";
  // #2093 — translated admin message with an authored English fallback.
  function t(key, fallback, params) {
    var value = window.gbT ? window.gbT(key, fallback, params) : fallback;
    if (value !== fallback || !params) return value;
    return Object.keys(params).reduce(function (text, name) {
      return text.replace(new RegExp("\\{\\s*" + name + "\\s*\\}", "g"), params[name]);
    }, fallback);
  }
  var A = (window.__GBAdminModals = window.__GBAdminModals || {});
  var showModal = A.showModal;
  var hideModal = A.hideModal;
  var showNotification = A.showNotification;

// ADMIN-DASHBOARD.HTML FUNCTIONS
// =============================================================================

function showInviteMemberModal() {
  showModal("invite-member-modal");
}

function closeInviteMemberModal() {
  hideModal("invite-member-modal");
}

function showBulkInviteModal() {
  showModal("bulk-invite-modal");
}

function closeBulkInviteModal() {
  hideModal("bulk-invite-modal");
}

function getCurrentOrgId() {
  return localStorage.getItem("currentOrganizationId") || "";
}

function refreshInvitations() {
  if (typeof htmx !== "undefined") {
    htmx.ajax("GET", "/api/admin/dashboard/invitations", ".invitation-list");
  } else {
    location.reload();
  }
}

function sendInvitation() {
  const email = document.getElementById("invite-email")?.value;
  const role = document.getElementById("invite-role")?.value || "member";
  const orgId = getCurrentOrgId();

  if (!orgId) {
    showNotification(t("admin-notify-select-an-organization-first", "Select an organization first"), "error");
    return;
  }
  if (!email) {
    showNotification(t("admin-notify-please-enter-an-email-address", "Please enter an email address"), "error");
    return;
  }

  fetch("/api/admin/invitations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ org_id: orgId, email, role }),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification("Invitation sent to " + email, "success");
        closeInviteMemberModal();
        refreshInvitations();
      } else {
        showNotification(t("admin-notify-failed-to-send-invitation-reason", "Failed to send invitation: {reason}", { reason: (data.error || "unknown error") }), "error",
        );
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function sendBulkInvitations() {
  const emailsText = document.getElementById("bulk-emails")?.value || "";
  const role = document.getElementById("bulk-role")?.value || "member";
  const orgId = getCurrentOrgId();
  const emails = emailsText
    .split(/[\n,;]+/)
    .map((e) => e.trim())
    .filter((e) => e);

  if (!orgId) {
    showNotification(t("admin-notify-select-an-organization-first", "Select an organization first"), "error");
    return;
  }
  if (emails.length === 0) {
    showNotification(t("admin-notify-please-enter-at-least-one-email-address", "Please enter at least one email address"), "error");
    return;
  }

  fetch("/api/admin/invitations/bulk", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ org_id: orgId, emails, role }),
  })
    .then((response) => response.json())
    .then((data) => {
      const sent = data.sent || 0;
      const failed = data.failed || 0;
      if (failed > 0) {
        showNotification(
          `${sent} sent, ${failed} failed`,
          "warning",
        );
      } else {
        showNotification(`${sent} invitations sent`, "success");
      }
      closeBulkInviteModal();
      refreshInvitations();
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function resendInvitation(invitationId) {
  fetch(`/api/admin/invitations/resend/${invitationId}`, { method: "POST" })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-invitation-resent", "Invitation resent"), "success");
        refreshInvitations();
      } else {
        showNotification(t("admin-notify-failed-to-resend-reason", "Failed to resend: {reason}", { reason: (data.error || "unknown error") }), "error",
        );
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function cancelInvitation(invitationId) {
  if (!confirm(t("admin-notify-cancel-invitation-confirm", "Cancel this invitation?"))) return;

  fetch(`/api/admin/invitations/${invitationId}`, { method: "DELETE" })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-invitation-cancelled", "Invitation cancelled"), "success");
        refreshInvitations();
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

// =============================================================================
// BILLING-DASHBOARD.HTML FUNCTIONS
// =============================================================================

function updateBillingPeriod(period) {
  // The metrics fragment is the real data source; re-request it on period change
  if (typeof htmx !== "undefined") {
    htmx.ajax(
      "GET",
      "/api/ui/billing/dashboard/metrics",
      ".metrics-overview",
      { swap: "innerHTML" },
    );
  }
}

function exportBillingReport() {
  const period = document.getElementById("billingPeriod")?.value || "current";
  showNotification(t("admin-notify-generating-billing-report", "Generating billing report..."), "info");

  fetch(`/api/ui/billing/invoices/export?period=${period}`)
    .then((response) => {
      if (response.ok) return response.blob();
      throw new Error("Export failed");
    })
    .then((blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `billing-report-${period}.csv`;
      a.click();
      URL.revokeObjectURL(url);
      showNotification(t("admin-notify-report-downloaded", "Report downloaded"), "success");
    })
    .catch((err) =>
      showNotification(t("admin-notify-export-failed-reason", "Export failed: {reason}", { reason: err.message }), "error"),
    );
}

function toggleBreakdownView() {
  const chart = document.getElementById("breakdown-chart");
  const table = document.getElementById("breakdown-table");

  if (chart && table) {
    const showingChart = !chart.classList.contains("hidden");
    chart.classList.toggle("hidden", showingChart);
    table.classList.toggle("hidden", !showingChart);
  }
}

function showQuotaSettings() {
  showModal("quota-settings-modal");
}

function closeQuotaSettings() {
  hideModal("quota-settings-modal");
}

function saveQuotaSettings() {
  const form = document.getElementById("quota-form");
  if (!form) return;

  const formData = new FormData(form);
  const quotas = Object.fromEntries(formData);

  fetch("/api/admin/billing/quotas", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(quotas),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-quota-settings-saved", "Quota settings saved"), "success");
        closeQuotaSettings();
      } else {
        showNotification(t("admin-notify-failed-to-save-reason", "Failed to save: {reason}", { reason: data.error }), "error");
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function configureAlerts() {
  showModal("alerts-config-modal");
}

function closeAlertsConfig() {
  hideModal("alerts-config-modal");
}

function saveAlertSettings() {
  const form = document.getElementById("alerts-form");
  if (!form) return;

  const formData = new FormData(form);
  const settings = Object.fromEntries(formData);

  fetch("/api/admin/billing/alerts", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-alert-settings-saved", "Alert settings saved"), "success");
        closeAlertsConfig();
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

// =============================================================================
// BILLING.HTML FUNCTIONS
// =============================================================================

function showUpgradeModal() {
  showModal("upgrade-modal");
}

function closeUpgradeModal() {
  hideModal("upgrade-modal");
}

function showCancelModal() {
  showModal("cancel-modal");
}

function closeCancelModal() {
  hideModal("cancel-modal");
}

function showAddPaymentModal() {
  showModal("add-payment-modal");
}

function closeAddPaymentModal() {
  hideModal("add-payment-modal");
}

function showEditAddressModal() {
  showModal("edit-address-modal");
}

function closeEditAddressModal() {
  hideModal("edit-address-modal");
}

function exportInvoices() {
  showNotification(t("admin-notify-exporting-invoices", "Exporting invoices..."), "info");

  fetch("/api/billing/invoices/export")
    .then((response) => {
      if (response.ok) return response.blob();
      throw new Error("Export failed");
    })
    .then((blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "invoices.csv";
      a.click();
      URL.revokeObjectURL(url);
      showNotification(t("admin-notify-invoices-exported", "Invoices exported"), "success");
    })
    .catch((err) =>
      showNotification(t("admin-notify-export-failed-reason", "Export failed: {reason}", { reason: err.message }), "error"),
    );
}

// E-mail surface removed (#1546): no mailto actions in the UI.
function contactSales() {
  if (typeof showUpgradeModal === "function") {
    showUpgradeModal();
  } else if (typeof showToast === "function") {
    showToast(t("admin-notify-contact-sales-is-disabled", "Contact sales is disabled"), "info");
  }
}

function showDowngradeOptions() {
  closeCancelModal();
  showUpgradeModal();
  // Focus on lower-tier plans
  const planSelector = document.querySelector(".plan-options");
  if (planSelector) {
    planSelector.scrollIntoView({ behavior: "smooth" });
  }
}

function selectPlan(planId) {
  document.querySelectorAll(".plan-option").forEach((el) => {
    el.classList.toggle("selected", el.dataset.plan === planId);
  });
}

function confirmUpgrade() {
  const selectedPlan = document.querySelector(".plan-option.selected");
  if (!selectedPlan) {
    showNotification(t("admin-notify-please-select-a-plan", "Please select a plan"), "error");
    return;
  }

  const planId = selectedPlan.dataset.plan;

  fetch("/api/billing/subscription/upgrade", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ plan_id: planId }),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-plan-upgraded-successfully", "Plan upgraded successfully!"), "success");
        closeUpgradeModal();
        location.reload();
      } else {
        showNotification(t("admin-notify-upgrade-failed-reason", "Upgrade failed: {reason}", { reason: data.error }), "error");
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function confirmCancellation() {
  const reason = document.getElementById("cancel-reason")?.value;

  fetch("/api/billing/subscription/cancel", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ reason }),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-subscription-cancelled", "Subscription cancelled"), "success");
        closeCancelModal();
        location.reload();
      } else {
        showNotification(t("admin-notify-cancellation-failed-reason", "Cancellation failed: {reason}", { reason: data.error }), "error");
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

// =============================================================================

  A.showInviteMemberModal = showInviteMemberModal;
  A.closeInviteMemberModal = closeInviteMemberModal;
  A.showBulkInviteModal = showBulkInviteModal;
  A.closeBulkInviteModal = closeBulkInviteModal;
  A.getCurrentOrgId = getCurrentOrgId;
  A.refreshInvitations = refreshInvitations;
  A.sendInvitation = sendInvitation;
  A.sendBulkInvitations = sendBulkInvitations;
  A.resendInvitation = resendInvitation;
  A.cancelInvitation = cancelInvitation;
  A.updateBillingPeriod = updateBillingPeriod;
  A.exportBillingReport = exportBillingReport;
  A.toggleBreakdownView = toggleBreakdownView;
  A.showQuotaSettings = showQuotaSettings;
  A.closeQuotaSettings = closeQuotaSettings;
  A.saveQuotaSettings = saveQuotaSettings;
  A.configureAlerts = configureAlerts;
  A.closeAlertsConfig = closeAlertsConfig;
  A.saveAlertSettings = saveAlertSettings;
  A.showUpgradeModal = showUpgradeModal;
  A.closeUpgradeModal = closeUpgradeModal;
  A.showCancelModal = showCancelModal;
  A.closeCancelModal = closeCancelModal;
  A.showAddPaymentModal = showAddPaymentModal;
  A.closeAddPaymentModal = closeAddPaymentModal;
  A.showEditAddressModal = showEditAddressModal;
  A.closeEditAddressModal = closeEditAddressModal;
  A.exportInvoices = exportInvoices;
  A.contactSales = contactSales;
  A.showDowngradeOptions = showDowngradeOptions;
  A.selectPlan = selectPlan;
  A.confirmUpgrade = confirmUpgrade;
  A.confirmCancellation = confirmCancellation;
})(window);
