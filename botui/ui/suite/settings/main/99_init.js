/* Settings module 99: init, public module surface and boot. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var bindNavigation = S.bindNavigation;
  var initEditionVisibility = S.initEditionVisibility;
  var bindToggles = S.bindToggles;
  var bindThemeSelector = S.bindThemeSelector;
  var bindAvatarUpload = S.bindAvatarUpload;
  var bindFormValidation = S.bindFormValidation;
  var initLanguageSettings = S.initLanguageSettings;
  var loadSavedSettings = S.loadSavedSettings;
  var syncPreviewToggle = S.syncPreviewToggle;
  var initUserManagement = S.initUserManagement;
  var initGroupManagement = S.initGroupManagement;
  var loadCurrentUser = S.loadCurrentUser;
  var initOAuthLinking = S.initOAuthLinking;
  var initWallpapers = S.initWallpapers;
  var applyWallpaper = S.applyWallpaper;
  var getWallpaper = S.getWallpaper;
  var loadNotificationPreferences = S.loadNotificationPreferences;
  var showToast = S.showToast;
  var editUser = S.editUser;
  var deleteUser = S.deleteUser;
  var editGroup = S.editGroup;
  var deleteGroup = S.deleteGroup;
  var viewGroupMembers = S.viewGroupMembers;
  var removeGroupMember = S.removeGroupMember;
  var handleLogout = S.handleLogout;
  var applyPreviewSimplifying = S.applyPreviewSimplifying;
  var syncIndexAllDrive = S.syncIndexAllDrive;
  var initFromHash = S.initFromHash;

function init() {
  bindNavigation();
  initEditionVisibility();
  bindToggles();
  bindThemeSelector();
  bindAvatarUpload();
  bindFormValidation();
  initLanguageSettings();
  loadSavedSettings();
  syncPreviewToggle();
  initUserManagement();
  initGroupManagement();
  loadCurrentUser();
  initOAuthLinking();
  initWallpapers();
  applyWallpaper(getWallpaper());
  loadNotificationPreferences();
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
  (function(){ var __cb = function () {
    init();
    applyPreviewSimplifying();
    syncIndexAllDrive();
    initFromHash();
  }; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
} else {
  init();
  applyPreviewSimplifying();
  syncIndexAllDrive();
  initFromHash();
}
})(window);
