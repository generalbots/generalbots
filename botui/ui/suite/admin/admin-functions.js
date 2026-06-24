/* =============================================================================
   ADMIN MODULE - Missing Function Handlers
   These functions are called by onclick handlers in admin HTML files
   ============================================================================= */

(function () {
  "use strict";

  // =============================================================================
  // MODAL HELPERS
  // =============================================================================

  function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.add("open");
        modal.style.display = "flex";
      }
    }
  }

  function hideModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      if (modal.close) {
        modal.close();
      } else {
        modal.classList.remove("open");
        modal.style.display = "none";
      }
    }
  }

  function showNotification(message, type) {
    if (typeof window.showNotification === "function") {
      window.showNotification(message, type);
    } else if (typeof window.GBAlerts !== "undefined") {
      if (type === "success") window.GBAlerts.success("Admin", message);
      else if (type === "error") window.GBAlerts.error("Admin", message);
      else if (type === "warning") window.GBAlerts.warning("Admin", message);
      else window.GBAlerts.info("Admin", message);
    } else {
      console.log(`[${type}] ${message}`);
    }
  }

  // =============================================================================
  // ACCOUNTS.HTML FUNCTIONS
  // =============================================================================

  function showSmtpModal() {
    showModal("smtp-modal");
  }

  function closeSmtpModal() {
    hideModal("smtp-modal");
  }

  function testSmtpConnection() {
    const host = document.getElementById("smtp-host")?.value;
    const port = document.getElementById("smtp-port")?.value;
    const username = document.getElementById("smtp-username")?.value;

    if (!host || !port) {
      showNotification("Please fill in SMTP host and port", "error");
      return;
    }

    showNotification("Testing SMTP connection...", "info");

    fetch("/api/settings/smtp/test", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ host, port: parseInt(port), username }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("SMTP connection successful!", "success");
        } else {
          showNotification(
            "SMTP connection failed: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => {
        showNotification("Connection test failed: " + err.message, "error");
      });
  }

  function connectAccount(provider) {
    showNotification(`Connecting to ${provider}...`, "info");
    // OAuth flow would redirect to provider
    window.location.href = `/api/auth/oauth/${provider}?redirect=/admin/accounts`;
  }

  function disconnectAccount(provider) {
    if (!confirm(`Disconnect ${provider} account?`)) return;

    fetch(`/api/settings/accounts/${provider}/disconnect`, { method: "POST" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification(`${provider} disconnected`, "success");
          location.reload();
        } else {
          showNotification("Failed to disconnect: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  // =============================================================================
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

  function sendInvitation() {
    const email = document.getElementById("invite-email")?.value;
    const role = document.getElementById("invite-role")?.value || "member";

    if (!email) {
      showNotification("Please enter an email address", "error");
      return;
    }

    fetch("/api/admin/invitations", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, role }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Invitation sent to " + email, "success");
          closeInviteMemberModal();
        } else {
          showNotification("Failed to send invitation: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function sendBulkInvitations() {
    const emailsText = document.getElementById("bulk-emails")?.value || "";
    const role = document.getElementById("bulk-role")?.value || "member";
    const emails = emailsText
      .split(/[\n,;]+/)
      .map((e) => e.trim())
      .filter((e) => e);

    if (emails.length === 0) {
      showNotification("Please enter at least one email address", "error");
      return;
    }

    fetch("/api/admin/invitations/bulk", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ emails, role }),
    })
      .then((response) => response.json())
      .then((data) => {
        showNotification(
          `${data.sent || emails.length} invitations sent`,
          "success",
        );
        closeBulkInviteModal();
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function resendInvitation(invitationId) {
    fetch(`/api/admin/invitations/${invitationId}/resend`, { method: "POST" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Invitation resent", "success");
        } else {
          showNotification("Failed to resend: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function cancelInvitation(invitationId) {
    if (!confirm("Cancel this invitation?")) return;

    fetch(`/api/admin/invitations/${invitationId}`, { method: "DELETE" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Invitation cancelled", "success");
          location.reload();
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  // =============================================================================
  // BILLING-DASHBOARD.HTML FUNCTIONS
  // =============================================================================

  function updateBillingPeriod(period) {
    const params = new URLSearchParams({ period });

    // Update dashboard stats via HTMX or fetch
    if (typeof htmx !== "undefined") {
      htmx.ajax("GET", `/api/admin/billing/stats?${params}`, "#billing-stats");
    } else {
      fetch(`/api/admin/billing/stats?${params}`)
        .then((r) => r.json())
        .then((data) => updateBillingStats(data))
        .catch((err) => console.error("Failed to update billing period:", err));
    }
  }

  function updateBillingStats(data) {
    if (data.totalRevenue) {
      const el = document.getElementById("total-revenue");
      if (el) el.textContent = formatCurrency(data.totalRevenue);
    }
    if (data.activeSubscriptions) {
      const el = document.getElementById("active-subscriptions");
      if (el) el.textContent = data.activeSubscriptions;
    }
  }

  function exportBillingReport() {
    const period = document.getElementById("billingPeriod")?.value || "current";
    showNotification("Generating billing report...", "info");

    fetch(`/api/admin/billing/export?period=${period}`)
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
        showNotification("Report downloaded", "success");
      })
      .catch((err) =>
        showNotification("Export failed: " + err.message, "error"),
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
          showNotification("Quota settings saved", "success");
          closeQuotaSettings();
        } else {
          showNotification("Failed to save: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
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
          showNotification("Alert settings saved", "success");
          closeAlertsConfig();
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
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
    showNotification("Exporting invoices...", "info");

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
        showNotification("Invoices exported", "success");
      })
      .catch((err) =>
        showNotification("Export failed: " + err.message, "error"),
      );
  }

  function contactSales() {
    window.open(
      "mailto:sales@example.com?subject=Enterprise Plan Inquiry",
      "_blank",
    );
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
      showNotification("Please select a plan", "error");
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
          showNotification("Plan upgraded successfully!", "success");
          closeUpgradeModal();
          location.reload();
        } else {
          showNotification("Upgrade failed: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
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
          showNotification("Subscription cancelled", "success");
          closeCancelModal();
          location.reload();
        } else {
          showNotification("Cancellation failed: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  // =============================================================================
  // COMPLIANCE-DASHBOARD.HTML FUNCTIONS
  // =============================================================================

  function updateFramework(framework) {
    // Update dashboard for selected compliance framework
    if (typeof htmx !== "undefined") {
      htmx.ajax(
        "GET",
        `/api/compliance/dashboard?framework=${framework}`,
        "#compliance-content",
      );
    } else {
      fetch(`/api/compliance/dashboard?framework=${framework}`)
        .then((r) => r.json())
        .then((data) => updateComplianceDashboard(data))
        .catch((err) => console.error("Failed to update framework:", err));
    }
  }

  function updateComplianceDashboard(data) {
    // Update various dashboard elements
    if (data.score) {
      const el = document.getElementById("compliance-score");
      if (el) el.textContent = data.score + "%";
    }
  }

  function generateComplianceReport() {
    const framework =
      document.getElementById("complianceFramework")?.value || "soc2";
    showNotification("Generating compliance report...", "info");

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
        showNotification("Report generated", "success");
      })
      .catch((err) =>
        showNotification("Report failed: " + err.message, "error"),
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
      showNotification("Please select a file", "error");
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
          showNotification("Evidence uploaded", "success");
          closeEvidenceUpload();
        } else {
          showNotification("Upload failed: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
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
    showNotification("Exporting audit log...", "info");

    fetch(`/api/compliance/audit-log/export?category=${category}`)
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
        showNotification("Audit log exported", "success");
      })
      .catch((err) =>
        showNotification("Export failed: " + err.message, "error"),
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
        htmx.ajax("GET", `/api/admin/groups/${groupId}`, "#panel-content");
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
      showNotification("Please enter a group name", "error");
      return;
    }

    fetch("/api/admin/groups", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, description }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Group created", "success");
          closeCreateGroup();
          location.reload();
        } else {
          showNotification("Failed to create group: " + data.error, "error");
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function deleteGroup(groupId) {
    if (!confirm("Delete this group? This action cannot be undone.")) return;

    fetch(`/api/admin/groups/${groupId}`, { method: "DELETE" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Group deleted", "success");
          closeDetailPanel();
          location.reload();
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  // =============================================================================
  // ROLE MANAGEMENT FUNCTIONS (roles.html)
  // =============================================================================

  let currentRole = null;
  let availablePermissions = [];
  let assignedPermissions = [];

  function selectRole(roleId, element) {
    // Update UI selection
    document
      .querySelectorAll(".role-item")
      .forEach((item) => item.classList.remove("selected"));
    if (element) element.classList.add("selected");

    // Show role detail
    document
      .getElementById("role-placeholder")
      ?.style.setProperty("display", "none");
    document
      .getElementById("role-detail")
      ?.style.setProperty("display", "block");

    // Fetch role details
    fetch(`/api/rbac/roles/${roleId}`)
      .then((response) => response.json())
      .then((data) => {
        currentRole = data;
        renderRoleDetail(data);
      })
      .catch((err) =>
        showNotification("Failed to load role: " + err.message, "error"),
      );
  }

  function renderRoleDetail(role) {
    document.getElementById("role-display-name").textContent =
      role.displayName || role.name;
    document.getElementById("role-name").textContent = role.name;

    const typeBadge = document.getElementById("role-type-badge");
    if (typeBadge) {
      typeBadge.textContent = role.isSystem ? "System" : "Custom";
      typeBadge.className =
        "role-type-badge " + (role.isSystem ? "system" : "custom");
    }

    // Enable/disable delete button based on system role
    const deleteBtn = document.getElementById("btn-delete-role");
    if (deleteBtn) deleteBtn.disabled = role.isSystem;

    // Load permissions
    loadRolePermissions(role.id);
  }

  function loadRolePermissions(roleId) {
    Promise.all([
      fetch("/api/rbac/permissions").then((r) => r.json()),
      fetch(`/api/rbac/roles/${roleId}/permissions`).then((r) => r.json()),
    ])
      .then(([allPerms, rolePerms]) => {
        assignedPermissions = rolePerms || [];
        availablePermissions = (allPerms || []).filter(
          (p) => !assignedPermissions.find((rp) => rp.id === p.id),
        );
        renderPermissionLists();
      })
      .catch((err) => console.error("Failed to load permissions:", err));
  }

  function renderPermissionLists() {
    const availableList = document.getElementById("available-permissions");
    const assignedList = document.getElementById("assigned-permissions");

    if (availableList) {
      availableList.innerHTML =
        availablePermissions
          .map(
            (p) => `
                <div class="permission-item" data-id="${p.id}" onclick="togglePermissionSelect(this)">
                    <span class="permission-name">${p.name}</span>
                    <span class="permission-scope">${p.scope || "global"}</span>
                </div>
            `,
          )
          .join("") ||
        '<div class="empty-state">No available permissions</div>';
    }

    if (assignedList) {
      assignedList.innerHTML =
        assignedPermissions
          .map(
            (p) => `
                <div class="permission-item" data-id="${p.id}" onclick="togglePermissionSelect(this)">
                    <span class="permission-name">${p.name}</span>
                    <span class="permission-scope">${p.scope || "global"}</span>
                </div>
            `,
          )
          .join("") || '<div class="empty-state">No assigned permissions</div>';
    }
  }

  function togglePermissionSelect(element) {
    element.classList.toggle("selected");
  }

  function assignSelected() {
    const selected = document.querySelectorAll(
      "#available-permissions .permission-item.selected",
    );
    selected.forEach((item) => {
      const id = item.dataset.id;
      const perm = availablePermissions.find((p) => p.id === id);
      if (perm) {
        availablePermissions = availablePermissions.filter((p) => p.id !== id);
        assignedPermissions.push(perm);
      }
    });
    renderPermissionLists();
  }

  function assignAll() {
    assignedPermissions = [...assignedPermissions, ...availablePermissions];
    availablePermissions = [];
    renderPermissionLists();
  }

  function removeSelected() {
    const selected = document.querySelectorAll(
      "#assigned-permissions .permission-item.selected",
    );
    selected.forEach((item) => {
      const id = item.dataset.id;
      const perm = assignedPermissions.find((p) => p.id === id);
      if (perm) {
        assignedPermissions = assignedPermissions.filter((p) => p.id !== id);
        availablePermissions.push(perm);
      }
    });
    renderPermissionLists();
  }

  function removeAll() {
    availablePermissions = [...availablePermissions, ...assignedPermissions];
    assignedPermissions = [];
    renderPermissionLists();
  }

  function savePermissions() {
    if (!currentRole) {
      showNotification("No role selected", "error");
      return;
    }

    const permissionIds = assignedPermissions.map((p) => p.id);

    fetch(`/api/rbac/roles/${currentRole.id}/permissions`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ permissions: permissionIds }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Permissions saved successfully", "success");
        } else {
          showNotification(
            "Failed to save permissions: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) =>
        showNotification("Error saving permissions: " + err.message, "error"),
      );
  }

  function resetPermissions() {
    if (!currentRole) return;
    if (
      !confirm(
        "Reset permissions to default? This will undo any unsaved changes.",
      )
    )
      return;
    loadRolePermissions(currentRole.id);
    showNotification("Permissions reset to saved state", "info");
  }

  function duplicateRole() {
    if (!currentRole) {
      showNotification("No role selected", "error");
      return;
    }

    const newName = prompt(
      "Enter name for the new role:",
      currentRole.name + "_copy",
    );
    if (!newName) return;

    fetch("/api/rbac/roles", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: newName,
        displayName: currentRole.displayName + " (Copy)",
        description: currentRole.description,
        permissions: assignedPermissions.map((p) => p.id),
      }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success || data.id) {
          showNotification("Role duplicated successfully", "success");
          location.reload();
        } else {
          showNotification(
            "Failed to duplicate role: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function confirmDeleteRole() {
    if (!currentRole) return;
    if (currentRole.isSystem) {
      showNotification("System roles cannot be deleted", "error");
      return;
    }
    showModal("delete-role-modal");
  }

  function deleteRole() {
    if (!currentRole) return;

    fetch(`/api/rbac/roles/${currentRole.id}`, { method: "DELETE" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Role deleted", "success");
          hideModal("delete-role-modal");
          location.reload();
        } else {
          showNotification(
            "Failed to delete role: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function assignUsersToRole() {
    const selectedUsers = Array.from(
      document.querySelectorAll("#user-assign-list input:checked"),
    ).map((input) => input.value);

    if (selectedUsers.length === 0) {
      showNotification("Please select at least one user", "error");
      return;
    }

    if (!currentRole) {
      showNotification("No role selected", "error");
      return;
    }

    fetch(`/api/rbac/roles/${currentRole.id}/users`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ userIds: selectedUsers }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification(
            `${selectedUsers.length} user(s) assigned to role`,
            "success",
          );
          hideModal("assign-users-modal");
        } else {
          showNotification(
            "Failed to assign users: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function assignGroupsToRole() {
    const selectedGroups = Array.from(
      document.querySelectorAll("#group-assign-list input:checked"),
    ).map((input) => input.value);

    if (selectedGroups.length === 0) {
      showNotification("Please select at least one group", "error");
      return;
    }

    if (!currentRole) {
      showNotification("No role selected", "error");
      return;
    }

    fetch(`/api/rbac/roles/${currentRole.id}/groups`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ groupIds: selectedGroups }),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification(
            `${selectedGroups.length} group(s) assigned to role`,
            "success",
          );
          hideModal("assign-groups-modal");
        } else {
          showNotification(
            "Failed to assign groups: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => showNotification("Error: " + err.message, "error"));
  }

  function filterRoles(type) {
    const items = document.querySelectorAll(".role-item");
    items.forEach((item) => {
      const isSystem = item.dataset.system === "true";
      if (type === "all") {
        item.style.display = "";
      } else if (type === "system") {
        item.style.display = isSystem ? "" : "none";
      } else if (type === "custom") {
        item.style.display = isSystem ? "none" : "";
      }
    });
  }

  // =============================================================================
  // BILLING ADMIN FUNCTIONS
  // =============================================================================

  function downloadInvoice(invoiceId) {
    showNotification(`Downloading invoice ${invoiceId}...`, "info");

    fetch(`/api/billing/invoices/${invoiceId}/download`)
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
        link.download = `${invoiceId}.pdf`;
        link.click();
        URL.revokeObjectURL(url);
        showNotification("Invoice downloaded", "success");
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

    fetch(`/api/compliance/evidence/${evidenceId}`)
      .then((response) => response.json())
      .then((data) => {
        // Show evidence in modal or new window
        if (data.url) {
          window.open(data.url, "_blank");
        } else {
          showModal("evidence-view-modal");
          const content = document.getElementById("evidence-content");
          if (content) {
            content.innerHTML = `
                            <h4>${data.name || evidenceId}</h4>
                            <p>${data.description || "No description available"}</p>
                            <div class="evidence-meta">
                                <span>Type: ${data.type || "Document"}</span>
                                <span>Uploaded: ${data.uploadedAt || "Unknown"}</span>
                            </div>
                        `;
          }
        }
      })
      .catch((err) =>
        showNotification("Failed to load evidence: " + err.message, "error"),
      );
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
  // ONBOARDING FUNCTIONS
  // =============================================================================

  let currentStep = 1;

  function nextStep(step) {
    const currentPanel = document.querySelector(
      `.onboarding-panel[data-step="${step}"]`,
    );
    const nextPanel = document.querySelector(
      `.onboarding-panel[data-step="${step + 1}"]`,
    );

    if (currentPanel) currentPanel.classList.remove("active");
    if (nextPanel) nextPanel.classList.add("active");

    currentStep = step + 1;
    updateStepIndicators();
  }

  function prevStep(step) {
    const currentPanel = document.querySelector(
      `.onboarding-panel[data-step="${step}"]`,
    );
    const prevPanel = document.querySelector(
      `.onboarding-panel[data-step="${step - 1}"]`,
    );

    if (currentPanel) currentPanel.classList.remove("active");
    if (prevPanel) prevPanel.classList.add("active");

    currentStep = step - 1;
    updateStepIndicators();
  }

  function updateStepIndicators() {
    document.querySelectorAll(".step-indicator").forEach((indicator, index) => {
      const stepNum = index + 1;
      indicator.classList.remove("active", "completed");
      if (stepNum < currentStep) {
        indicator.classList.add("completed");
      } else if (stepNum === currentStep) {
        indicator.classList.add("active");
      }
    });
  }

  function skipPayment() {
    showNotification("Skipping payment setup - Free plan selected", "info");
    nextStep(4);
  }

  function removeLogo() {
    const logoPreview = document.getElementById("logo-preview");
    const logoInput = document.getElementById("logo-input");

    if (logoPreview) {
      logoPreview.innerHTML =
        '<span class="placeholder-text">No logo uploaded</span>';
    }
    if (logoInput) {
      logoInput.value = "";
    }
    showNotification("Logo removed", "success");
  }

  function previewLogo(input) {
    if (input.files && input.files[0]) {
      const reader = new FileReader();
      reader.onload = function (e) {
        const preview = document.getElementById("logo-preview");
        if (preview) {
          preview.innerHTML = `<img src="${e.target.result}" alt="Logo preview" style="max-width: 128px; max-height: 128px;">`;
        }
      };
      reader.readAsDataURL(input.files[0]);
    }
  }

  // =============================================================================
  // ACCOUNTS LIST FUNCTIONS (sources/accounts-list.html)
  // =============================================================================

  function editAccount(accountId) {
    showNotification(`Loading account ${accountId}...`, "info");

    fetch(`/api/sources/accounts/${accountId}`)
      .then((response) => response.json())
      .then((data) => {
        showModal("edit-account-modal");
        // Populate form fields
        const form = document.getElementById("edit-account-form");
        if (form) {
          form
            .querySelector('[name="account-name"]')
            ?.setAttribute("value", data.name || "");
          form
            .querySelector('[name="account-email"]')
            ?.setAttribute("value", data.email || "");
        }
      })
      .catch((err) =>
        showNotification("Failed to load account: " + err.message, "error"),
      );
  }

  function syncAccount(accountId) {
    showNotification(`Syncing account ${accountId}...`, "info");

    fetch(`/api/sources/accounts/${accountId}/sync`, { method: "POST" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Account sync started", "success");
        } else {
          showNotification(
            "Sync failed: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => showNotification("Sync error: " + err.message, "error"));
  }

  // =============================================================================
  // SEARCH SETTINGS FUNCTIONS (admin/search-settings.html)
  // =============================================================================

  function openReindexModal() {
    const modal = document.getElementById("reindex-modal");
    if (modal) {
      modal.showModal();
    }
  }

  function closeReindexModal() {
    const modal = document.getElementById("reindex-modal");
    if (modal) {
      modal.close();
    }
  }

  function startReindex() {
    showNotification("Starting reindex operation...", "info");

    fetch("/api/ui/sources/kb/reindex", { method: "POST" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Reindex started successfully", "success");
          closeReindexModal();
          refreshStats();
        } else {
          showNotification(
            "Reindex failed: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) =>
        showNotification("Reindex error: " + err.message, "error"),
      );
  }

  function refreshStats() {
    showNotification("Refreshing search statistics...", "info");

    fetch("/api/ui/sources/kb/stats")
      .then((response) => response.json())
      .then((data) => {
        // Update stats display
        const totalDocs = document.getElementById("total-documents");
        const indexSize = document.getElementById("index-size");
        const lastIndexed = document.getElementById("last-indexed");

        if (totalDocs) totalDocs.textContent = data.totalDocuments || "0";
        if (indexSize) indexSize.textContent = data.indexSize || "0 MB";
        if (lastIndexed) lastIndexed.textContent = data.lastIndexed || "Never";

        showNotification("Statistics updated", "success");
      })
      .catch((err) =>
        showNotification("Failed to refresh stats: " + err.message, "error"),
      );
  }

  function saveSearchSettings() {
    const form = document.getElementById("search-settings-form");
    if (!form) {
      showNotification("Settings form not found", "error");
      return;
    }

    const formData = new FormData(form);
    const settings = Object.fromEntries(formData.entries());

    showNotification("Saving search settings...", "info");

    fetch("/api/settings/search", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(settings),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification("Search settings saved successfully", "success");
        } else {
          showNotification(
            "Failed to save settings: " + (data.error || "Unknown error"),
            "error",
          );
        }
      })
      .catch((err) => showNotification("Save error: " + err.message, "error"));
  }

  // =============================================================================
  // UTILITY FUNCTIONS
  // =============================================================================

  function formatCurrency(amount, currency = "USD") {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: currency,
    }).format(amount);
  }

  // =============================================================================
  // EXPORT TO WINDOW
  // =============================================================================

  // Accounts
  window.showSmtpModal = showSmtpModal;
  window.closeSmtpModal = closeSmtpModal;
  window.testSmtpConnection = testSmtpConnection;
  window.connectAccount = connectAccount;
  window.disconnectAccount = disconnectAccount;

  // Admin Dashboard
  window.showInviteMemberModal = showInviteMemberModal;
  window.closeInviteMemberModal = closeInviteMemberModal;
  window.showBulkInviteModal = showBulkInviteModal;
  window.closeBulkInviteModal = closeBulkInviteModal;
  window.sendInvitation = sendInvitation;
  window.sendBulkInvitations = sendBulkInvitations;
  window.resendInvitation = resendInvitation;
  window.cancelInvitation = cancelInvitation;

  // Billing Dashboard
  window.updateBillingPeriod = updateBillingPeriod;
  window.exportBillingReport = exportBillingReport;
  window.toggleBreakdownView = toggleBreakdownView;
  window.showQuotaSettings = showQuotaSettings;
  window.closeQuotaSettings = closeQuotaSettings;
  window.saveQuotaSettings = saveQuotaSettings;
  window.configureAlerts = configureAlerts;
  window.closeAlertsConfig = closeAlertsConfig;
  window.saveAlertSettings = saveAlertSettings;

  // Billing
  window.showUpgradeModal = showUpgradeModal;
  window.closeUpgradeModal = closeUpgradeModal;
  window.showCancelModal = showCancelModal;
  window.closeCancelModal = closeCancelModal;
  window.showAddPaymentModal = showAddPaymentModal;
  window.closeAddPaymentModal = closeAddPaymentModal;
  window.showEditAddressModal = showEditAddressModal;
  window.closeEditAddressModal = closeEditAddressModal;
  window.exportInvoices = exportInvoices;
  window.contactSales = contactSales;
  window.showDowngradeOptions = showDowngradeOptions;
  window.selectPlan = selectPlan;
  window.confirmUpgrade = confirmUpgrade;
  window.confirmCancellation = confirmCancellation;

  // Compliance Dashboard
  window.updateFramework = updateFramework;
  window.generateComplianceReport = generateComplianceReport;
  window.startAuditPrep = startAuditPrep;
  window.closeAuditPrep = closeAuditPrep;
  window.showEvidenceUpload = showEvidenceUpload;
  window.closeEvidenceUpload = closeEvidenceUpload;
  window.uploadEvidence = uploadEvidence;
  window.filterLogs = filterLogs;
  window.exportAuditLog = exportAuditLog;

  // Groups
  window.closeDetailPanel = closeDetailPanel;
  window.openDetailPanel = openDetailPanel;
  window.createGroup = createGroup;
  window.closeCreateGroup = closeCreateGroup;
  window.saveGroup = saveGroup;
  window.deleteGroup = deleteGroup;

  // Role Management
  window.selectRole = selectRole;
  window.assignSelected = assignSelected;
  window.assignAll = assignAll;
  window.removeSelected = removeSelected;
  window.removeAll = removeAll;
  window.savePermissions = savePermissions;
  window.resetPermissions = resetPermissions;
  window.duplicateRole = duplicateRole;
  window.confirmDeleteRole = confirmDeleteRole;
  window.deleteRole = deleteRole;
  window.assignUsersToRole = assignUsersToRole;
  window.assignGroupsToRole = assignGroupsToRole;
  window.filterRoles = filterRoles;

  // Billing Admin
  window.downloadInvoice = downloadInvoice;
  window.dismissAlert = dismissAlert;
  window.viewEvidence = viewEvidence;

  // Operations Dashboard
  window.toggleAutoRefresh = toggleAutoRefresh;
  window.showAlertConfig = showAlertConfig;
  window.closeAlertConfig = closeAlertConfig;
  window.showTraceDetail = showTraceDetail;
  window.refreshHealth = refreshHealth;

  // Onboarding
  window.nextStep = nextStep;
  window.prevStep = prevStep;
  window.skipPayment = skipPayment;
  window.removeLogo = removeLogo;
  window.previewLogo = previewLogo;

  // Account Management
  window.editAccount = editAccount;
  window.syncAccount = syncAccount;

  // Search Settings
  window.openReindexModal = openReindexModal;
  window.closeReindexModal = closeReindexModal;
  window.startReindex = startReindex;
  window.refreshStats = refreshStats;
  window.saveSearchSettings = saveSearchSettings;
})();
