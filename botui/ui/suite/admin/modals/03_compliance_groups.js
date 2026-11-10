/* Admin modals — compliance dashboard and groups */
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

// COMPLIANCE-DASHBOARD.HTML FUNCTIONS
// =============================================================================

function updateFramework(framework) {
  // The registered dashboard fragment is the real data source
  if (typeof htmx !== "undefined") {
    htmx.ajax(
      "GET",
      "/api/compliance/dashboard/overview",
      "#compliance-content",
    );
  }
}

function generateComplianceReport() {
  const framework =
    document.getElementById("complianceFramework")?.value || "soc2";
  showNotification(t("admin-notify-generating-compliance-report", "Generating compliance report..."), "info");

  fetch(`/api/compliance/report?framework=${framework}`)
    .then((response) => {
      if (response.ok) return response.blob();
      throw new Error("Report generation failed");
    })
    .then((blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `compliance-report-${framework}.pdf`;
      a.click();
      URL.revokeObjectURL(url);
      showNotification(t("admin-notify-report-generated", "Report generated"), "success");
    })
    .catch((err) =>
      showNotification(t("admin-notify-report-failed-reason", "Report failed: {reason}", { reason: err.message }), "error"),
    );
}

function startAuditPrep() {
  showModal("audit-prep-modal");
}

function closeAuditPrep() {
  hideModal("audit-prep-modal");
}

function showEvidenceUpload() {
  showModal("evidence-upload-modal");
}

function closeEvidenceUpload() {
  hideModal("evidence-upload-modal");
}

function uploadEvidence() {
  const fileInput = document.getElementById("evidence-file");
  const category = document.getElementById("evidence-category")?.value;

  if (!fileInput?.files?.length) {
    showNotification(t("admin-notify-please-select-a-file", "Please select a file"), "error");
    return;
  }

  const formData = new FormData();
  formData.append("file", fileInput.files[0]);
  formData.append("category", category);

  fetch("/api/compliance/evidence", {
    method: "POST",
    body: formData,
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-evidence-uploaded", "Evidence uploaded"), "success");
        closeEvidenceUpload();
      } else {
        showNotification(t("admin-notify-upload-failed-reason", "Upload failed: {reason}", { reason: data.error }), "error");
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function filterLogs() {
  const category = document.getElementById("logCategory")?.value || "all";

  if (typeof htmx !== "undefined") {
    htmx.ajax(
      "GET",
      `/api/compliance/audit-log?category=${category}`,
      "#audit-log-list",
    );
  }
}

function exportAuditLog() {
  const category = document.getElementById("logCategory")?.value || "all";
  showNotification(t("admin-notify-exporting-audit-log", "Exporting audit log..."), "info");

  fetch(`/api/compliance/export?category=${category}`)
    .then((response) => {
      if (response.ok) return response.blob();
      throw new Error("Export failed");
    })
    .then((blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "audit-log.csv";
      a.click();
      URL.revokeObjectURL(url);
      showNotification(t("admin-notify-audit-log-exported", "Audit log exported"), "success");
    })
    .catch((err) =>
      showNotification(t("admin-notify-export-failed-reason", "Export failed: {reason}", { reason: err.message }), "error"),
    );
}

// =============================================================================
// GROUPS.HTML FUNCTIONS
// =============================================================================

function closeDetailPanel() {
  const panel = document.getElementById("detail-panel");
  if (panel) {
    panel.classList.remove("open");
  }
}

function openDetailPanel(groupId) {
  const panel = document.getElementById("detail-panel");
  if (panel) {
    panel.classList.add("open");
    // Load group details
    if (typeof htmx !== "undefined") {
      htmx.ajax(
        "GET",
        `/api/directory/groups/${groupId}`,
        "#panel-content",
      );
    }
  }
}

function createGroup() {
  showModal("create-group-modal");
}

function closeCreateGroup() {
  hideModal("create-group-modal");
}

function saveGroup() {
  const name = document.getElementById("group-name")?.value;
  const description = document.getElementById("group-description")?.value;

  if (!name) {
    showNotification(t("admin-notify-please-enter-a-group-name", "Please enter a group name"), "error");
    return;
  }

  const params = new URLSearchParams();
  params.append("name", name);
  params.append("display_name", name);
  if (description) params.append("description", description);

  fetch("/api/directory/groups/create", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: params,
  })
    .then((response) => {
      if (response.ok) {
        showNotification(t("admin-notify-group-created", "Group created"), "success");
        closeCreateGroup();
        location.reload();
      } else {
        return response.text().then((text) => {
          throw new Error(text || "Failed to create group");
        });
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

function deleteGroup(groupId) {
  if (!confirm(t("admin-notify-delete-group-confirm", "Delete this group? This action cannot be undone."))) return;

  fetch(`/api/directory/groups/${groupId}/delete`, { method: "DELETE" })
    .then((response) => response.json())
    .then((data) => {
      if (data.ok || data.success) {
        showNotification(t("admin-notify-group-deleted", "Group deleted"), "success");
        closeDetailPanel();
        location.reload();
      } else {
        showNotification(t("admin-notify-failed-to-delete-group", "Failed to delete group"), "error");
      }
    })
    .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
}

// =============================================================================

  A.updateFramework = updateFramework;
  A.generateComplianceReport = generateComplianceReport;
  A.startAuditPrep = startAuditPrep;
  A.closeAuditPrep = closeAuditPrep;
  A.showEvidenceUpload = showEvidenceUpload;
  A.closeEvidenceUpload = closeEvidenceUpload;
  A.uploadEvidence = uploadEvidence;
  A.filterLogs = filterLogs;
  A.exportAuditLog = exportAuditLog;
  A.closeDetailPanel = closeDetailPanel;
  A.openDetailPanel = openDetailPanel;
  A.createGroup = createGroup;
  A.closeCreateGroup = closeCreateGroup;
  A.saveGroup = saveGroup;
  A.deleteGroup = deleteGroup;
})(window);
