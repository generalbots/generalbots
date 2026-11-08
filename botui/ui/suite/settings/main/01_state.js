/* Settings Module JavaScript */
/* Settings module 01: storage keys, API endpoints and shared helpers. */
(function (window) {
  "use strict";
  var S = (window.__GBSettingsMain = window.__GBSettingsMain || {});


var STORAGE_KEYS = {
  LOCALE: "gb-locale",
  DATE_FORMAT: "gb-date-format",
  TIME_FORMAT: "gb-time-format",
  THEME: "gb-theme",
  WALLPAPER: "gb-wallpaper",
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

var organizationsData = [];

function getAuthToken() {
  return localStorage.getItem("gb-access-token");
}

function apiUrl(endpoint) {
  return "/api/directory" + endpoint;
}


/* #1628 — settings must match the edition. A product profile that ships
   billing/vibe-less (no_security, chat+drive) must not offer sections that
   need services it does not have. #1675 — Integrations is NO LONGER hidden
   in the slim/desktop edition: MCP server management ships there too
   (Claude-Desktop-style Extensions panel), while the full provider catalog
   window stays available through the section header. */

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

  S.STORAGE_KEYS = STORAGE_KEYS;
  S.API_ENDPOINTS = API_ENDPOINTS;
  S.getAuthToken = getAuthToken;
  S.apiUrl = apiUrl;
  S.escapeHtml = escapeHtml;
  S.debounce = debounce;
  S.showToast = showToast;
})(window);

window.downloadDataExport = async function downloadDataExport() {
  var token = (window.SettingsApp && SettingsApp.getAuthToken) ? SettingsApp.getAuthToken() : (localStorage.getItem('gb_token') || '');
  try {
    (SettingsApp.showToast || window.showToast || function(){})('Preparing export…');
    var res = await fetch('/api/user/data/export', {
      method: 'POST',
      headers: token ? { 'Authorization': 'Bearer ' + token } : {}
    });
    var ct = (res.headers.get('Content-Type') || '').toLowerCase();
    if (!res.ok || ct.indexOf('application/zip') === -1) {
      var msg = 'Export failed';
      try { var j = await res.json(); if (j && j.message) msg = j.message; } catch (e) {}
      (SettingsApp.showToast || window.showToast || alert)(msg, 'error');
      return;
    }
    var blob = await res.blob();
    var cd = res.headers.get('Content-Disposition') || '';
    var m = /filename="?([^";]+)"?/.exec(cd);
    var name = m ? m[1] : 'generalbots-export.zip';
    var a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = name;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(a.href);
    (SettingsApp.showToast || window.showToast || function(){})('Export downloaded');
  } catch (e) {
    (SettingsApp.showToast || window.showToast || alert)('Export failed', 'error');
  }
};
