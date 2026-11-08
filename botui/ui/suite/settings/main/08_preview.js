/* Settings module 08: preview simplifying and section routing. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var showToast = S.showToast;

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

window.togglePreviewMode = function (el) {
  if (window.GBPreviewMode) {
    GBPreviewMode.set(el.checked);
  }
};

function syncPreviewToggle() {
  var el = document.getElementById("settings-preview-toggle");
  if (el && window.GBPreviewMode) el.checked = GBPreviewMode.isOn();
}
window.addEventListener("gb-preview-changed", syncPreviewToggle);

// #1545/#1813: Drive-as-KB setting, persisted locally and sent by the chat
// client as `index_all_drive`. Default is on; only an explicit "false" disables
// it, matching chat-send.js.
window.toggleIndexAllDrive = function (el) {
  var enabled = !!el.checked;
  try {
    localStorage.setItem("gb-index-all-drive", enabled ? "true" : "false");
  } catch (e) {}
  showToast(
    enabled
      ? "Drive files enabled as chat context"
      : "Drive files removed from chat context",
  );
  window.dispatchEvent(
    new CustomEvent("gb-index-all-drive-changed", {
      detail: { enabled: enabled },
    }),
  );
};
function syncIndexAllDrive() {
  var el = document.getElementById("settings-index-all-drive");
  if (!el) return;
  try {
    el.checked = localStorage.getItem("gb-index-all-drive") !== "false";
  } catch (e) {
    el.checked = true;
  }
}

// BBSIA #1541 / #1882 / #1889: the tree is shaped by settings/nav-registry.js
// in ONE pass. Depth (essential/advanced) lives on the node, RBAC decides
// administration before preview does, and edition comes from the product file.
// The hard-coded PREVIEW_ESSENTIAL_NAV / PREVIEW_HIDDEN_SECTIONS arrays are
// gone: adding or moving a section is a registry edit.
function applyPreviewSimplifying() {
  if (window.GBNavRegistry) {
    window.GBNavRegistry.applyNavVisibility();
  }
  // Sections follow their nav item so the registry drives both surfaces.
  document.querySelectorAll(".settings-section").forEach(function (section) {
    var id = String(section.id || "");
    if (!/-section$/.test(id)) { return; }
    var navId = id.replace(/-section$/, "");
    var nav = document.querySelector('.settings-nav a[href="#' + navId + '"]');
    if (!nav) { return; }
    var visible = !nav.hidden;
    section.style.display = visible ? "" : "none";
    if (!visible) { section.classList.remove("active"); }
  });
  var active = document.querySelector(".settings-nav .nav-item.active");
  if (!active || active.hidden) {
    var profile = document.querySelector('.settings-nav .nav-item[href="#profile"]');
    if (profile) showSection("profile", profile);
  }
}
window.addEventListener("gb-preview-changed", applyPreviewSimplifying);
// HTMX swaps bring partials whose nav nodes must be shaped by the same pass.
document.addEventListener("htmx:load", applyPreviewSimplifying);
document.addEventListener("htmx:afterSwap", applyPreviewSimplifying);
document.addEventListener("htmx:load", syncPreviewToggle);
  S.showSection = showSection;
  S.initFromHash = initFromHash;
  S.syncPreviewToggle = syncPreviewToggle;
  S.syncIndexAllDrive = syncIndexAllDrive;
  S.applyPreviewSimplifying = applyPreviewSimplifying;
})(window);
