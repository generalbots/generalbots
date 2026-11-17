/* Settings module 06a: shared helpers, user management and session logout.
   #2307 — all DOM access is null-safe: a missing id degrades to a status
   message instead of throwing and killing the rest of the callback.
   Shared helpers are published on S._ugCommon for 06_groups.js. */
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
  var pendingConfirm = null;

  function t(key, fallback, params) {
    return typeof window.gbT === "function"
      ? window.gbT(key, fallback, params)
      : fallback;
  }

  function byId(id) {
    return document.getElementById(id);
  }

  function setStatus(id, message, isError) {
    var el = byId(id);
    if (!el) return;
    el.textContent = message || "";
    el.setAttribute("role", isError ? "alert" : "status");
    el.setAttribute("aria-live", isError ? "assertive" : "polite");
  }

  function fieldValue(form, name) {
    if (!form || !form.elements) return null;
    var field = form.elements[name];
    if (!field || field.value === undefined || field.value === null) return null;
    return String(field.value).trim();
  }

  function openDialog(id) {
    var dialog = byId(id);
    if (!dialog || typeof dialog.showModal !== "function") return;
    if (!dialog.open) dialog.showModal();
  }

  function closeDialog(id) {
    var dialog = byId(id);
    if (dialog && dialog.open && typeof dialog.close === "function") {
      dialog.close();
    }
  }

  function parseResponse(response) {
    if (!response.ok) {
      return response.json().catch(function () { return {}; }).then(function (body) {
        var message =
          (body && (body.error || body.message)) ||
          "HTTP " + response.status;
        throw new Error(message);
      });
    }
    return response.json().catch(function () { return {}; });
  }

  function askConfirm(message, onYes) {
    var dialog = byId("gb-confirm-dialog");
    if (!dialog || typeof dialog.showModal !== "function") return;
    var messageEl = byId("gb-confirm-message");
    if (messageEl) messageEl.textContent = message;
    pendingConfirm = onYes;
    if (!dialog.open) dialog.showModal();
  }

  function bindConfirmDialog() {
    var dialog = byId("gb-confirm-dialog");
    if (!dialog || dialog.dataset.gb2307Bound) return;
    dialog.dataset.gb2307Bound = "1";
    var cancel = byId("gb-confirm-cancel");
    var accept = byId("gb-confirm-accept");
    if (cancel) {
      cancel.addEventListener("click", function () {
        pendingConfirm = null;
        if (dialog.open) dialog.close();
      });
    }
    if (accept) {
      accept.addEventListener("click", function () {
        var action = pendingConfirm;
        pendingConfirm = null;
        if (dialog.open) dialog.close();
        if (typeof action === "function") action();
      });
    }
  }

  function loadingMarkup() {
    return '<div class="loading-state" role="status" aria-live="polite">' +
      '<div class="spinner" aria-hidden="true"></div><p>' +
      escapeHtml(t("admin-loading", "Loading…")) + "</p></div>";
  }

  function errorMarkup(message) {
    return '<div class="error-state" role="alert" aria-live="assertive"><p>' +
      escapeHtml(message) + "</p></div>";
  }

  function resetForm(id) {
    var form = byId(id);
    if (form && typeof form.reset === "function") form.reset();
  }

  function withSubmitState(form, busy, original) {
    var button = form ? form.querySelector('button[type="submit"]') : null;
    if (!button) return;
    button.disabled = busy;
    button.textContent = busy ? t("admin-creating", "Creating…") : original;
  }

  S._ugCommon = {
    t: t,
    byId: byId,
    setStatus: setStatus,
    fieldValue: fieldValue,
    openDialog: openDialog,
    closeDialog: closeDialog,
    parseResponse: parseResponse,
    askConfirm: askConfirm,
    bindConfirmDialog: bindConfirmDialog,
    loadingMarkup: loadingMarkup,
    errorMarkup: errorMarkup,
    resetForm: resetForm,
    withSubmitState: withSubmitState,
    showToast: showToast,
    escapeHtml: escapeHtml,
    apiUrl: apiUrl,
    getAuthToken: getAuthToken,
    API_ENDPOINTS: API_ENDPOINTS,
  };

  function initUserManagement() {
    bindConfirmDialog();
    var addUserBtn = byId("add-user-btn");
    if (addUserBtn) addUserBtn.addEventListener("click", openAddUserDialog);
    var userSearchInput = byId("user-search");
    if (userSearchInput) {
      userSearchInput.addEventListener(
        "input",
        debounce(function () { loadUsers(this.value); }, 300),
      );
    }
    var addUserForm = byId("add-user-form");
    if (addUserForm) addUserForm.addEventListener("submit", handleAddUser);
    // The HTMX list loader was removed with the old admin partial; the JS
    // module is now the only owner of the users list.
    loadUsers();
  }

  function loadUsers(search) {
    var container = byId("users-list");
    if (!container) return;
    container.setAttribute("aria-busy", "true");
    container.innerHTML = loadingMarkup();
    var params = new URLSearchParams();
    params.append("per_page", "50");
    if (search) params.append("search", search);
    if (S.currentOrgId) params.append("organization_id", S.currentOrgId);
    fetch(apiUrl(API_ENDPOINTS.USERS_LIST) + "?" + params.toString(), {
      headers: { Authorization: "Bearer " + getAuthToken() },
    })
      .then(function (r) { return parseResponse(r); })
      .then(function (data) {
        usersData = data && Array.isArray(data.users) ? data.users : [];
        renderUsers(container);
        container.setAttribute("aria-busy", "false");
      })
      .catch(function () {
        container.setAttribute("aria-busy", "false");
        container.innerHTML = errorMarkup(
          t("admin-failed-load-users", "Failed to load users."),
        );
      });
  }

  function renderUsers(container) {
    if (!container) return;
    if (usersData.length === 0) {
      container.innerHTML = '<div class="empty-state"><p>' +
        escapeHtml(t("admin-no-users", "No users found")) + "</p></div>";
      return;
    }
    var keys = [
      "admin-col-user", "admin-col-email", "admin-col-organization",
      "admin-col-roles", "admin-col-status", "admin-col-actions",
    ];
    var labels = ["User", "Email", "Organization", "Roles", "Status", "Actions"];
    var html = '<table class="data-table users-table"><thead><tr>';
    keys.forEach(function (key, i) {
      html += "<th>" + escapeHtml(t(key, labels[i])) + "</th>";
    });
    html += "</tr></thead><tbody>";
    usersData.forEach(function (user) {
      if (!user) return;
      var initials = escapeHtml(getInitials(user.username || user.email));
      var name = user.display_name || user.username || user.email || "";
      var active = user.state === "active" || user.state === "USER_STATE_ACTIVE";
      var roles = (Array.isArray(user.roles) ? user.roles : []).join(", ") || "user";
      var status = active
        ? t("admin-status-active", "Active")
        : t("admin-status-inactive", "Inactive");
      var edit = escapeHtml(t("admin-action-edit", "Edit"));
      var del = escapeHtml(t("admin-action-delete", "Delete"));
      var id = escapeHtml(user.id || "");
      html += "<tr>";
      html += '<td class="user-cell"><div class="user-avatar">' + initials +
        '</div><div class="user-info"><span class="user-name">' +
        escapeHtml(name) + '</span><span class="user-username">@' +
        escapeHtml(user.username || "") + "</span></div></td>";
      html += "<td>" + escapeHtml(user.email || "") + "</td>";
      html += "<td>" + escapeHtml(user.organization_id || "-") + "</td>";
      html += "<td>" + escapeHtml(roles) + "</td>";
      html += '<td><span class="status-badge ' +
        (active ? "status-active" : "status-inactive") + '">' +
        escapeHtml(status) + "</span></td>";
      html += '<td class="actions-cell">';
      html += '<button class="btn-icon" title="' + edit +
        '" onclick="SettingsModule.editUser(\'' + id +
        '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg></button>';
      html += '<button class="btn-icon btn-danger" title="' + del +
        '" onclick="SettingsModule.deleteUser(\'' + id +
        '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></button>';
      html += "</td></tr>";
    });
    html += "</tbody></table>";
    container.innerHTML = html;
  }

  function openAddUserDialog() {
    resetForm("add-user-form");
    setStatus("add-user-status", "");
    openDialog("create-user-modal");
  }

  function closeAddUserDialog() {
    closeDialog("create-user-modal");
    resetForm("add-user-form");
    setStatus("add-user-status", "");
  }

  function handleAddUser(event) {
    if (event && typeof event.preventDefault === "function") event.preventDefault();
    var form = event ? event.target : null;
    if (!form) return;
    var submit = form.querySelector('button[type="submit"]');
    var original = submit ? submit.textContent : "";
    withSubmitState(form, true, original);
    setStatus("add-user-status", "");
    var payload = {
      username: fieldValue(form, "username"),
      email: fieldValue(form, "email"),
      display_name: fieldValue(form, "display_name"),
      password: fieldValue(form, "password"),
      role: fieldValue(form, "role") || "user",
      organization_id: S.currentOrgId || fieldValue(form, "organization_id"),
    };
    fetch(apiUrl(API_ENDPOINTS.USERS_CREATE), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + getAuthToken(),
      },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return parseResponse(r); })
      .then(function () {
        var ok = t("admin-user-created", "User created and added to organization");
        setStatus("add-user-status", ok);
        showToast(ok, "success");
        closeAddUserDialog();
        loadUsers();
      })
      .catch(function (error) {
        var message = error && error.message
          ? error.message
          : t("admin-failed-create-user", "Failed to create user.");
        setStatus("add-user-status", message, true);
      })
      .finally(function () { withSubmitState(form, false, original); });
  }

  function editUser(userId) {
    var user = usersData.find(function (u) { return u && u.id === userId; });
    if (!user) return;
    openDialog("edit-user-dialog");
  }

  function deleteUser(userId) {
    askConfirm(t("admin-confirm-delete-user", "Delete this user?"), function () {
      fetch(apiUrl(API_ENDPOINTS.USERS_DELETE.replace(":user_id", userId)), {
        method: "DELETE",
        headers: { Authorization: "Bearer " + getAuthToken() },
      })
        .then(function (r) { return parseResponse(r); })
        .then(function () {
          showToast(t("admin-user-deleted", "User deleted"), "success");
          loadUsers();
        })
        .catch(function (error) { showToast(error.message, "error"); });
    });
  }

  function handleLogout() {
    var token = getAuthToken();
    if (token) {
      fetch(API_ENDPOINTS.LOGOUT, {
        method: "POST",
        headers: { Authorization: "Bearer " + token },
      }).catch(function (err) {
        /* Best-effort server logout; local session is cleared below. */
        console.debug("[gb-settings] Server logout request failed:", err);
      });
    }
    localStorage.removeItem("gb-access-token");
    localStorage.removeItem("gb-refresh-token");
    localStorage.removeItem("gb-token-expires");
    localStorage.removeItem("gb-user-data");
    window.location.href = window.GB_LOGIN_URL || "/login";
  }

  S.initUserManagement = initUserManagement;
  S.loadUsers = loadUsers;
  S.editUser = editUser;
  S.deleteUser = deleteUser;
  S.openAddUserDialog = openAddUserDialog;
  S.closeAddUserDialog = closeAddUserDialog;
  S.handleAddUser = handleAddUser;
  S.handleLogout = handleLogout;
})(window);
