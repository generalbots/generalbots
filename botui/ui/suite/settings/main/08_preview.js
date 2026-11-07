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

// BBSIA #1541: preview OFF is a minimal settings surface. Only Profile,
// Appearance, Language, About and Versions stay visible; administrative,
// billing, Vibe, storage and integrations sections collapse into preview
// mode. RBAC still applies on top for the sections that remain.
// #1675 — Integrations stays reachable in minimal/preview mode: the
// desktop edition exposes MCP management there (the Integrations APP stays
// hidden; this is a Settings section).
var PREVIEW_ESSENTIAL_NAV = ["profile", "appearance", "language", "about", "versions", "integrations", "shared-links"];
var PREVIEW_HIDDEN_SECTIONS = [
  "security-section", "notifications-section", "storage-section",
  "vibe-section", "privacy-section",
  "billing-section", "users-section", "groups-section", "bots-section",
  "dns-section", "audit-section", "organizations-section",
];

function applyPreviewSimplifying() {
  var on = !!(window.GBPreviewMode && GBPreviewMode.isOn());
  document.querySelectorAll(".settings-nav .nav-item").forEach(function (item) {
    var href = (item.getAttribute("href") || "").replace("#", "");
    var essential = PREVIEW_ESSENTIAL_NAV.indexOf(href) !== -1;
    item.style.display = on || essential ? "" : "none";
  });
  document.querySelectorAll(".settings-nav .nav-divider").forEach(function (divider) {
    divider.style.display = on ? "" : "none";
  });
  PREVIEW_HIDDEN_SECTIONS.forEach(function (id) {
    var section = document.getElementById(id);
    if (section) section.style.display = on ? "" : "none";
  });
  if (!on) {
    var active = document.querySelector(".settings-nav .nav-item.active");
    var activeHref = active ? (active.getAttribute("href") || "").replace("#", "") : "";
    if (PREVIEW_ESSENTIAL_NAV.indexOf(activeHref) === -1) {
      var profile = document.querySelector('.settings-nav .nav-item[href="#profile"]');
      if (profile) showSection("profile", profile);
    }
  }
}
window.addEventListener("gb-preview-changed", applyPreviewSimplifying);
  S.showSection = showSection;
  S.initFromHash = initFromHash;
  S.syncPreviewToggle = syncPreviewToggle;
  S.syncIndexAllDrive = syncIndexAllDrive;
  S.applyPreviewSimplifying = applyPreviewSimplifying;
})(window);
