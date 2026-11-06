/* Admin modals — role management */
(function (window) {
  "use strict";
  var A = (window.__GBAdminModals = window.__GBAdminModals || {});
  var showModal = A.showModal;
  var hideModal = A.hideModal;
  var showNotification = A.showNotification;

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

  A.selectRole = selectRole;
  A.renderRoleDetail = renderRoleDetail;
  A.loadRolePermissions = loadRolePermissions;
  A.renderPermissionLists = renderPermissionLists;
  A.togglePermissionSelect = togglePermissionSelect;
  A.assignSelected = assignSelected;
  A.assignAll = assignAll;
  A.removeSelected = removeSelected;
  A.removeAll = removeAll;
  A.savePermissions = savePermissions;
  A.resetPermissions = resetPermissions;
  A.duplicateRole = duplicateRole;
  A.confirmDeleteRole = confirmDeleteRole;
  A.deleteRole = deleteRole;
  A.assignUsersToRole = assignUsersToRole;
  A.assignGroupsToRole = assignGroupsToRole;
  A.filterRoles = filterRoles;
})(window);
