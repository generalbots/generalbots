/* Settings module 04: current user, role and admin visibility. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var API_ENDPOINTS = S.API_ENDPOINTS;
  var getAuthToken = S.getAuthToken;
  var currentUser = null;

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
        S.currentOrgId = user.organization_id;
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
    S.loadUsers();
    S.loadGroups();
  }
}

  S.loadCurrentUser = loadCurrentUser;
  S.updateUserDisplay = updateUserDisplay;
  S.getInitials = getInitials;
  S.checkAdminAccess = checkAdminAccess;
})(window);
