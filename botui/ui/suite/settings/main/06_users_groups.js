/* Settings module 06: user and group management. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var API_ENDPOINTS = S.API_ENDPOINTS;
  var apiUrl = S.apiUrl;
  var getAuthToken = S.getAuthToken;
  var getInitials = S.getInitials;
  var escapeHtml = S.escapeHtml;
  var debounce = S.debounce;
  var showToast = S.showToast;
  var usersData = [];
  var groupsData = [];

function initUserManagement() {
  var addUserBtn = document.getElementById("add-user-btn");
  if (addUserBtn) addUserBtn.addEventListener("click", openAddUserDialog);
  var userSearchInput = document.getElementById("user-search");
  if (userSearchInput) {
    userSearchInput.addEventListener(
      "input",
      debounce(function () {
        loadUsers(this.value);
      }, 300),
    );
  }
  var addUserForm = document.getElementById("add-user-form");
  if (addUserForm) addUserForm.addEventListener("submit", handleAddUser);
}

function initGroupManagement() {
  var addGroupBtn = document.getElementById("add-group-btn");
  if (addGroupBtn) addGroupBtn.addEventListener("click", openAddGroupDialog);
  var groupSearchInput = document.getElementById("group-search");
  if (groupSearchInput) {
    groupSearchInput.addEventListener(
      "input",
      debounce(function () {
        loadGroups(this.value);
      }, 300),
    );
  }
  var addGroupForm = document.getElementById("add-group-form");
  if (addGroupForm) addGroupForm.addEventListener("submit", handleAddGroup);
}

function loadUsers(search) {
  var container = document.getElementById("users-list");
  if (!container) return;
  container.innerHTML =
    '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
  var params = new URLSearchParams();
  params.append("per_page", "50");
  if (search) params.append("search", search);
  if (S.currentOrgId) params.append("organization_id", S.currentOrgId);
  var token = getAuthToken();
  fetch(apiUrl(API_ENDPOINTS.USERS_LIST) + "?" + params.toString(), {
    headers: { Authorization: "Bearer " + token },
  })
    .then(function (r) {
      if (!r.ok) throw new Error("Failed");
      return r.json();
    })
    .then(function (data) {
      usersData = data.users || [];
      renderUsers(container);
    })
    .catch(function () {
      container.innerHTML =
        '<div class="error-state"><p>Failed to load users.</p></div>';
    });
}

function renderUsers(container) {
  if (usersData.length === 0) {
    container.innerHTML =
      '<div class="empty-state"><p>No users found</p></div>';
    return;
  }
  var html =
    '<table class="data-table users-table"><thead><tr><th>User</th><th>Email</th><th>Organization</th><th>Roles</th><th>Status</th><th>Actions</th></tr></thead><tbody>';
  usersData.forEach(function (user) {
    var initials = getInitials(user.username || user.email);
    var displayName = user.display_name || user.username || user.email;
    var isActive =
      user.state === "active" || user.state === "USER_STATE_ACTIVE";
    var roles = (user.roles || []).join(", ") || "user";
    var orgId = user.organization_id || "-";
    html += "<tr>";
    html +=
      '<td class="user-cell"><div class="user-avatar">' +
      initials +
      '</div><div class="user-info"><span class="user-name">' +
      escapeHtml(displayName) +
      '</span><span class="user-username">@' +
      escapeHtml(user.username || "") +
      "</span></div></td>";
    html += "<td>" + escapeHtml(user.email || "") + "</td>";
    html += "<td>" + escapeHtml(orgId) + "</td>";
    html += "<td>" + escapeHtml(roles) + "</td>";
    html +=
      '<td><span class="status-badge ' +
      (isActive ? "status-active" : "status-inactive") +
      '">' +
      (isActive ? "Active" : "Inactive") +
      "</span></td>";
    html += '<td class="actions-cell">';
    html +=
      '<button class="btn-icon" onclick="SettingsModule.editUser(\'' +
      user.id +
      '\')" title="Edit"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg></button>';
    html +=
      '<button class="btn-icon btn-danger" onclick="SettingsModule.deleteUser(\'' +
      user.id +
      '\')" title="Delete"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></button>';
    html += "</td></tr>";
  });
  html += "</tbody></table>";
  container.innerHTML = html;
}

function loadGroups(search) {
  var container = document.getElementById("groups-list");
  if (!container) return;
  container.innerHTML =
    '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
  var params = new URLSearchParams();
  params.append("per_page", "50");
  if (search) params.append("search", search);
  var token = getAuthToken();
  fetch(apiUrl(API_ENDPOINTS.GROUPS_LIST) + "?" + params.toString(), {
    headers: { Authorization: "Bearer " + token },
  })
    .then(function (r) {
      if (!r.ok) throw new Error("Failed");
      return r.json();
    })
    .then(function (data) {
      groupsData = data.groups || [];
      renderGroups(container);
    })
    .catch(function () {
      container.innerHTML =
        '<div class="error-state"><p>Failed to load groups.</p></div>';
    });
}

function renderGroups(container) {
  if (groupsData.length === 0) {
    container.innerHTML =
      '<div class="empty-state"><p>No groups found</p></div>';
    return;
  }
  var html = '<div class="groups-grid">';
  groupsData.forEach(function (group) {
    html += '<div class="group-card" data-group-id="' + group.id + '">';
    html +=
      '<div class="group-header"><h3 class="group-name">' +
      escapeHtml(group.name) +
      '</h3><span class="member-count">' +
      (group.member_count || 0) +
      " members</span></div>";
    if (group.description)
      html +=
        '<p class="group-description">' +
        escapeHtml(group.description) +
        "</p>";
    html += '<div class="group-actions">';
    html +=
      '<button class="btn-secondary btn-sm" onclick="SettingsModule.viewGroupMembers(\'' +
      group.id +
      "')\">View Members</button>";
    html +=
      '<button class="btn-icon" onclick="SettingsModule.editGroup(\'' +
      group.id +
      '\')" title="Edit"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg></button>';
    html +=
      '<button class="btn-icon btn-danger" onclick="SettingsModule.deleteGroup(\'' +
      group.id +
      '\')" title="Delete"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></button>';
    html += "</div></div>";
  });
  html += "</div>";
  container.innerHTML = html;
}

function openAddUserDialog() {
  var dialog = document.getElementById("add-user-dialog");
  if (dialog) dialog.showModal();
}

function closeAddUserDialog() {
  var dialog = document.getElementById("add-user-dialog");
  if (dialog) {
    dialog.close();
    var f = document.getElementById("add-user-form");
    if (f) f.reset();
  }
}

function openAddGroupDialog() {
  var dialog = document.getElementById("add-group-dialog");
  if (dialog) dialog.showModal();
}

function closeAddGroupDialog() {
  var dialog = document.getElementById("add-group-dialog");
  if (dialog) {
    dialog.close();
    var f = document.getElementById("add-group-form");
    if (f) f.reset();
  }
}

function handleAddUser(e) {
  e.preventDefault();
  var form = e.target,
    btn = form.querySelector('button[type="submit"]'),
    orig = btn.textContent;
  btn.disabled = true;
  btn.textContent = "Creating...";

  var userData = {
    username: form.username.value,
    email: form.email.value,
    password: form.password ? form.password.value : null,
    first_name: form.first_name.value,
    last_name: form.last_name.value,
    role: form.role ? form.role.value : "user",
    organization_id:
      S.currentOrgId ||
      (form.organization_id ? form.organization_id.value : null),
    roles: form.roles ? [form.roles.value] : ["user"],
  };

  fetch(apiUrl(API_ENDPOINTS.USERS_CREATE), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: "Bearer " + getAuthToken(),
    },
    body: JSON.stringify(userData),
  })
    .then(function (r) {
      if (!r.ok)
        return r.json().then(function (err) {
          throw new Error(err.error || "Failed");
        });
      return r.json();
    })
    .then(function () {
      showToast("User created and added to organization", "success");
      closeAddUserDialog();
      loadUsers();
    })
    .catch(function (e) {
      showToast(e.message, "error");
    })
    .finally(function () {
      btn.disabled = false;
      btn.textContent = orig;
    });
}

function handleAddGroup(e) {
  e.preventDefault();
  var form = e.target,
    btn = form.querySelector('button[type="submit"]'),
    orig = btn.textContent;
  btn.disabled = true;
  btn.textContent = "Creating...";
  var groupData = {
    name: form.name.value,
    description: form.description ? form.description.value : "",
  };
  fetch(apiUrl(API_ENDPOINTS.GROUPS_CREATE), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: "Bearer " + getAuthToken(),
    },
    body: JSON.stringify(groupData),
  })
    .then(function (r) {
      if (!r.ok)
        return r.json().then(function (err) {
          throw new Error(err.error || "Failed");
        });
      return r.json();
    })
    .then(function () {
      showToast("Group created successfully", "success");
      closeAddGroupDialog();
      loadGroups();
    })
    .catch(function (e) {
      showToast(e.message, "error");
    })
    .finally(function () {
      btn.disabled = false;
      btn.textContent = orig;
    });
}

function editUser(userId) {
  var user = usersData.find(function (u) {
    return u.id === userId;
  });
  if (!user) return;
  var d = document.getElementById("edit-user-dialog");
  if (d) d.showModal();
}

function deleteUser(userId) {
  if (!confirm("Delete this user?")) return;
  fetch(apiUrl(API_ENDPOINTS.USERS_DELETE.replace(":user_id", userId)), {
    method: "DELETE",
    headers: { Authorization: "Bearer " + getAuthToken() },
  })
    .then(function (r) {
      if (!r.ok) throw new Error("Failed");
      showToast("User deleted", "success");
      loadUsers();
    })
    .catch(function (e) {
      showToast(e.message, "error");
    });
}

function editGroup(groupId) {
  var group = groupsData.find(function (g) {
    return g.id === groupId;
  });
  if (!group) return;
  var d = document.getElementById("edit-group-dialog");
  if (d) d.showModal();
}

function deleteGroup(groupId) {
  if (!confirm("Delete this group?")) return;
  fetch(apiUrl(API_ENDPOINTS.GROUPS_DELETE.replace(":group_id", groupId)), {
    method: "DELETE",
    headers: { Authorization: "Bearer " + getAuthToken() },
  })
    .then(function (r) {
      if (!r.ok) throw new Error("Failed");
      showToast("Group deleted", "success");
      loadGroups();
    })
    .catch(function (e) {
      showToast(e.message, "error");
    });
}

function viewGroupMembers(groupId) {
  var d = document.getElementById("group-members-dialog");
  if (d) d.showModal();
}

function removeGroupMember(groupId, userId) {
  if (!confirm("Remove member?")) return;
  fetch(
    apiUrl(API_ENDPOINTS.GROUPS_REMOVE_MEMBER.replace(":group_id", groupId)),
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + getAuthToken(),
      },
      body: JSON.stringify({ user_id: userId }),
    },
  )
    .then(function (r) {
      if (!r.ok) throw new Error("Failed");
      showToast("Member removed", "success");
      loadGroups();
    })
    .catch(function (e) {
      showToast(e.message, "error");
    });
}

function handleLogout() {
  var token = getAuthToken();
  if (token)
    fetch(API_ENDPOINTS.LOGOUT, {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    }).catch(function () {});
  localStorage.removeItem("gb-access-token");
  localStorage.removeItem("gb-refresh-token");
  localStorage.removeItem("gb-token-expires");
  localStorage.removeItem("gb-user-data");
  window.location.href = window.GB_LOGIN_URL || "/login";
}

  S.initUserManagement = initUserManagement;
  S.initGroupManagement = initGroupManagement;
  S.loadUsers = loadUsers;
  S.loadGroups = loadGroups;
  S.editUser = editUser;
  S.deleteUser = deleteUser;
  S.editGroup = editGroup;
  S.deleteGroup = deleteGroup;
  S.viewGroupMembers = viewGroupMembers;
  S.removeGroupMember = removeGroupMember;
  S.handleLogout = handleLogout;
})(window);
