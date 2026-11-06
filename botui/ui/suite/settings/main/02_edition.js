/* Settings module 02: edition-aware section visibility. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;

function isSlimEdition(config) {
  if (!config) { return false; }
  if (config.no_security === true) { return true; }
  var apps = Array.isArray(config.apps) ? config.apps : [];
  return apps.indexOf("chat") !== -1 &&
    apps.indexOf("drive") !== -1 &&
    apps.indexOf("billing") === -1 &&
    apps.indexOf("vibe") === -1;
}

function hideSettingsSection(id) {
  var section = document.getElementById(id + "-section");
  if (section) { section.style.display = "none"; }
  var nav = document.querySelector('.settings-nav a[href="#' + id + '"]');
  if (nav) { nav.style.display = "none"; }
}

function applyEditionVisibility(config) {
  if (!isSlimEdition(config)) { return; }
  ["billing", "vibe"].forEach(hideSettingsSection);
  document.querySelectorAll('[data-admin-only="true"]').forEach(function (node) {
    node.style.display = "none";
  });
  var active = document.querySelector(".settings-section.active");
  if (active && ["billing-section", "vibe-section"].indexOf(active.id) !== -1) {
    var profile = document.querySelector('.settings-nav a[href="#profile"]');
    if (profile) { profile.click(); }
  }
}

function initEditionVisibility() {
  if (window.productConfig) { applyEditionVisibility(window.productConfig); return; }
  var tries = 0;
  var timer = setInterval(function () {
    tries += 1;
    if (window.productConfig) {
      clearInterval(timer);
      applyEditionVisibility(window.productConfig);
    } else if (tries > 40) {
      clearInterval(timer);
    }
  }, 100);
}

  S.isSlimEdition = isSlimEdition;
  S.hideSettingsSection = hideSettingsSection;
  S.applyEditionVisibility = applyEditionVisibility;
  S.initEditionVisibility = initEditionVisibility;
})(window);
