/* Settings Module JavaScript */

(function () {
  "use strict";

  var STORAGE_KEYS = {
    LOCALE: "gb-locale",
    DATE_FORMAT: "gb-date-format",
    TIME_FORMAT: "gb-time-format",
    THEME: "gb-theme",
    COMPACT_MODE: "gb-compact-mode",
    ANIMATIONS: "gb-animations",
  };

  var API_ENDPOINTS = {
    USERS_LIST: "/users/list",
    USERS_CREATE: "/users/create",
    USERS_UPDATE: "/users/:user_id/update",
    USERS_DELETE: "/users/:user_id/delete",
    USERS_PROFILE: "/users/:user_id/profile",
    USERS_ASSIGN_ORG: "/users/:user_id/organization",
    USERS_MEMBERSHIPS: "/users/:user_id/memberships",
    GROUPS_LIST: "/groups/list",
    GROUPS_CREATE: "/groups/create",
    GROUPS_UPDATE: "/groups/:group_id/update",
    GROUPS_DELETE: "/groups/:group_id/delete",
    GROUPS_MEMBERS: "/groups/:group_id/members",
    GROUPS_ADD_MEMBER: "/groups/:group_id/members/add",
    GROUPS_REMOVE_MEMBER: "/groups/:group_id/members/remove",
    ORGS_LIST: "/organizations/list",
    CURRENT_USER: "/api/auth/me",
    LOGOUT: "/api/auth/logout",
  };

  var currentUser = null;
  var currentOrgId = null;
  var usersData = [];
  var groupsData = [];
  var organizationsData = [];

  function getAuthToken() {
    return localStorage.getItem("gb-access-token");
  }

  function apiUrl(endpoint) {
    return "/api/directory" + endpoint;
  }

  function init() {
    bindNavigation();
    bindToggles();
    bindThemeSelector();
    bindAvatarUpload();
    bindFormValidation();
    initLanguageSettings();
    loadSavedSettings();
    initUserManagement();
    initGroupManagement();
    loadCurrentUser();
  }

  function loadCurrentUser() {
    var token = getAuthToken();
    if (!token) return;

    fetch(API_ENDPOINTS.CURRENT_USER, {
      headers: { Authorization: "Bearer " + token },
    })
      .then(function (r) {
        return r.ok ? r.json() : null;
      })
      .then(function (user) {
        if (user) {
          currentUser = user;
          currentOrgId = user.organization_id;
          updateUserDisplay();
          checkAdminAccess();
        }
      })
      .catch(function (e) {
        console.warn("Failed to load user:", e);
      });
  }

  function updateUserDisplay() {
    if (!currentUser) return;
    var displayNameEl = document.getElementById("user-display-name");
    var emailEl = document.getElementById("user-email");
    if (displayNameEl) {
      displayNameEl.textContent =
        currentUser.display_name || currentUser.username || "";
    }
    if (emailEl) {
      emailEl.textContent = currentUser.email || "";
    }
  }

  function getInitials(name) {
    if (!name) return "??";
    var parts = name.split(" ");
    if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase();
    return name.substring(0, 2).toUpperCase();
  }

  function checkAdminAccess() {
    if (!currentUser || !currentUser.roles) return;
    var isAdmin = currentUser.roles.some(function (r) {
      var rl = r.toLowerCase();
      return rl.indexOf("admin") !== -1 || rl.indexOf("super") !== -1;
    });
    var adminSections = document.querySelectorAll(
      '[data-admin-only="true"], .admin-only',
    );
    var adminNavItems = document.querySelectorAll(
      '.nav-item[href="#users"], .nav-item[href="#groups"]',
    );
    adminSections.forEach(function (s) {
      s.style.display = isAdmin ? "" : "none";
    });
    adminNavItems.forEach(function (i) {
      i.style.display = isAdmin ? "" : "none";
    });
    if (isAdmin) {
      loadUsers();
      loadGroups();
    }
  }

  function bindNavigation() {
    document.querySelectorAll(".settings-nav-item").forEach(function (item) {
      item.addEventListener("click", function (e) {
        e.preventDefault();
        document.querySelectorAll(".settings-nav-item").forEach(function (i) {
          i.classList.remove("active");
        });
        this.classList.add("active");
      });
    });
  }

  function bindToggles() {
    document
      .querySelectorAll(".toggle-switch input")
      .forEach(function (toggle) {
        toggle.addEventListener("change", function () {
          saveSetting(this.dataset.setting, this.checked);
        });
      });
  }

  function bindThemeSelector() {
    document.querySelectorAll(".theme-option input").forEach(function (option) {
      option.addEventListener("change", function () {
        document.body.setAttribute("data-theme", this.value);
        saveSetting("theme", this.value);
      });
    });
  }

  function bindAvatarUpload() {
    var avatarInput = document.getElementById("avatar-input");
    if (avatarInput) {
      avatarInput.addEventListener("change", function (e) {
        var file = e.target.files[0];
        if (file) {
          var reader = new FileReader();
          reader.onload = function (ev) {
            var preview = document.querySelector(".avatar-preview img");
            if (preview) preview.src = ev.target.result;
          };
          reader.readAsDataURL(file);
        }
      });
    }
  }

  function bindFormValidation() {
    document.querySelectorAll(".settings-form").forEach(function (form) {
      form.addEventListener("submit", function (e) {
        var inputs = form.querySelectorAll("[required]");
        var valid = true;
        inputs.forEach(function (input) {
          if (!input.value.trim()) {
            valid = false;
            input.classList.add("error");
          } else {
            input.classList.remove("error");
          }
        });
        if (!valid) e.preventDefault();
      });
    });
  }

  function initLanguageSettings() {
    var languageSelect = document.getElementById("language-select");
    var savedLocale = localStorage.getItem(STORAGE_KEYS.LOCALE) || "en";
    if (languageSelect) languageSelect.value = savedLocale;
    document.querySelectorAll(".language-option").forEach(function (opt) {
      opt.classList.toggle("active", opt.dataset.locale === savedLocale);
    });
  }

  function loadSavedSettings() {
    var theme = localStorage.getItem(STORAGE_KEYS.THEME);
    if (theme) {
      document.body.setAttribute("data-theme", theme);
      var themeOption = document.querySelector(
        '.theme-option[data-theme="' + theme + '"]',
      );
      if (themeOption) themeOption.classList.add("active");
    }
    var compactMode =
      localStorage.getItem(STORAGE_KEYS.COMPACT_MODE) === "true";
    var compactToggle = document.querySelector('[name="compact_mode"]');
    if (compactToggle) {
      compactToggle.checked = compactMode;
      if (compactMode) document.body.classList.add("compact-mode");
    }
    var animations = localStorage.getItem(STORAGE_KEYS.ANIMATIONS) !== "false";
    var animationsToggle = document.querySelector('[name="animations"]');
    if (animationsToggle) {
      animationsToggle.checked = animations;
      if (!animations) document.body.classList.add("no-animations");
    }
  }

  function saveSetting(key, value) {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch (e) {
      console.warn("Failed to save setting:", key, e);
    }
  }

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
    if (currentOrgId) params.append("organization_id", currentOrgId);
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
        currentOrgId ||
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
    window.location.href = "/auth/login.html";
  }

  function escapeHtml(text) {
    if (!text) return "";
    var d = document.createElement("div");
    d.textContent = text;
    return d.innerHTML;
  }
  function debounce(func, wait) {
    var timeout;
    return function () {
      var ctx = this,
        args = arguments;
      clearTimeout(timeout);
      timeout = setTimeout(function () {
        func.apply(ctx, args);
      }, wait);
    };
  }

  function showToast(message, type) {
    type = type || "success";
    var existing = document.querySelector(".toast");
    if (existing) existing.remove();
    var toast = document.createElement("div");
    toast.className = "toast toast-" + type;
    toast.innerHTML =
      '<span class="toast-message">' +
      message +
      '</span><button class="toast-close" onclick="this.parentElement.remove()"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg></button>';
    document.body.appendChild(toast);
    requestAnimationFrame(function () {
      toast.classList.add("show");
    });
    setTimeout(function () {
      toast.classList.remove("show");
      setTimeout(function () {
        toast.remove();
      }, 300);
    }, 3000);
  }

  window.changeLanguage = function (locale) {
    localStorage.setItem(STORAGE_KEYS.LOCALE, locale);
    document.documentElement.lang = locale;
    showToast("Language changed");
    setTimeout(function () {
      location.reload();
    }, 500);
  };
  window.selectLanguage = function (locale, element) {
    document.querySelectorAll(".language-option").forEach(function (o) {
      o.classList.remove("active");
    });
    if (element) element.classList.add("active");
    changeLanguage(locale);
  };
  window.changeDateFormat = function (format) {
    localStorage.setItem(STORAGE_KEYS.DATE_FORMAT, format);
    showToast("Date format updated");
  };
  window.changeTimeFormat = function (format) {
    localStorage.setItem(STORAGE_KEYS.TIME_FORMAT, format);
    showToast("Time format updated");
  };
  window.setTheme = function (theme, element) {
    document.body.setAttribute("data-theme", theme);
    localStorage.setItem(STORAGE_KEYS.THEME, theme);
    document.querySelectorAll(".theme-option").forEach(function (o) {
      o.classList.remove("active");
    });
    if (element) element.classList.add("active");
    showToast("Theme updated");
  };
  window.toggleCompactMode = function (checkbox) {
    var enabled = checkbox.checked;
    localStorage.setItem(STORAGE_KEYS.COMPACT_MODE, enabled);
    document.body.classList.toggle("compact-mode", enabled);
    showToast(enabled ? "Compact mode enabled" : "Compact mode disabled");
  };
  window.toggleAnimations = function (checkbox) {
    var enabled = checkbox.checked;
    localStorage.setItem(STORAGE_KEYS.ANIMATIONS, enabled);
    document.body.classList.toggle("no-animations", !enabled);
    showToast(enabled ? "Animations enabled" : "Animations disabled");
  };
  window.showSection = function (sectionId, navElement) {
    document.querySelectorAll(".settings-section").forEach(function (s) {
      s.classList.remove("active");
    });
    document.querySelectorAll(".nav-item").forEach(function (i) {
      i.classList.remove("active");
    });
    var section = document.getElementById(sectionId + "-section");
    if (section) section.classList.add("active");
    if (navElement) navElement.classList.add("active");
    history.replaceState(null, "", "#" + sectionId);
  };
  window.showToast = showToast;
  window.previewAvatar = function (input) {
    if (input.files && input.files[0]) {
      var reader = new FileReader();
      reader.onload = function (e) {
        var avatar = document.getElementById("current-avatar");
        if (avatar)
          avatar.innerHTML = '<img src="' + e.target.result + '" alt="Avatar">';
      };
      reader.readAsDataURL(input.files[0]);
    }
  };
  window.removeAvatar = function () {
    var avatar = document.getElementById("current-avatar");
    if (avatar) avatar.innerHTML = "<span>JD</span>";
    showToast("Avatar removed");
  };

  function initFromHash() {
    var hash = window.location.hash.slice(1);
    if (hash) {
      var navItem = document.querySelector('.nav-item[href="#' + hash + '"]');
      if (navItem) showSection(hash, navItem);
    }
  }

  window.SettingsModule = {
    init: init,
    changeLanguage: window.changeLanguage,
    changeDateFormat: window.changeDateFormat,
    changeTimeFormat: window.changeTimeFormat,
    setTheme: window.setTheme,
    showToast: showToast,
    editUser: editUser,
    deleteUser: deleteUser,
    editGroup: editGroup,
    deleteGroup: deleteGroup,
    viewGroupMembers: viewGroupMembers,
    removeGroupMember: removeGroupMember,
    logout: handleLogout,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      init();
      initFromHash();
    });
  } else {
    init();
    initFromHash();
  }
})();
