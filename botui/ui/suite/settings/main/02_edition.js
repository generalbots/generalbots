/* Settings module 02: edition-aware section visibility. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;

// #1889 — the product file declares the deployment edition. The explicit
// value always wins over the flag soup below (no_security used to imply three
// different editions at once).
function declaredEdition(config) {
  var value = config && typeof config.edition === "string"
    ? config.edition.toLowerCase()
    : "";
  return value === "cloud" || value === "desktop" || value === "os" ? value : "";
}

function currentEdition(config) {
  if (window.GBNavRegistry && typeof window.GBNavRegistry.edition === "function") {
    return window.GBNavRegistry.edition();
  }
  var declared = declaredEdition(config);
  if (declared) { return declared; }
  if (window.__TAURI__ || window.__TAURI_INTERNALS__) { return "desktop"; }
  if (config && (config.shell_mode === true || config.no_security === true)) { return "os"; }
  return "cloud";
}

function isDesktopEdition(config) {
  var declared = declaredEdition(config);
  if (declared) { return declared !== "cloud"; }
  if (window.__TAURI__ || window.__TAURI_INTERNALS__) { return true; }
  if (!config) { return false; }
  if (config.desktop === true || config.desktop_local === true || config.is_desktop === true) { return true; }
  // Desktop-local product config ships with no_security; treat as desktop edition.
  if (config.no_security === true) { return true; }
  return false;
}

function isSlimEdition(config) {
  if (!config) { return false; }
  if (config.no_security === true) { return true; }
  var apps = Array.isArray(config.apps) ? config.apps : [];
  return apps.indexOf("chat") !== -1 &&
    apps.indexOf("drive") !== -1 &&
    apps.indexOf("billing") === -1 &&
    apps.indexOf("vibe") === -1;
}

// #1852 — shell mode is the Tauri desktop app, the desktop-local build or the
// General Bots OS kiosk image. Only there do host-level settings make sense;
// a normal browser/cloud deployment must never show the Operating System panel.
function isShellMode(config) {
  var declared = declaredEdition(config);
  if (declared) { return declared !== "cloud"; }
  if (window.__GB_SHELL_MODE__ === true || window.__GB_SHELL_MODE__ === "true") {
    return true;
  }
  if (window.__TAURI__ || window.__TAURI_INTERNALS__) { return true; }
  if (!config) { return false; }
  return config.shell_mode === true ||
    config.desktop === true ||
    config.desktop_local === true ||
    config.is_desktop === true ||
    config.no_security === true;
}

// #1887 — one security story per deployment. A no-security / single-user OS
// install has no directory: password, 2FA and session listings are absent and
// a single "Local data" card takes their place. Directory-backed clouds keep
// the session/device lists and let settings-actions.js hide the local password
// card once an issuer is known.
function applySecurityStory(config) {
  var noSecurity = !!(config && config.no_security === true) || currentEdition(config) === "os";
  var password = document.getElementById("security-password-card");
  var twofa = document.getElementById("security-2fa-card");
  var sessions = document.getElementById("security-sessions-card");
  var devices = document.getElementById("security-devices-card");
  var local = document.getElementById("security-local-card");
  if (noSecurity) {
    [password, twofa, sessions, devices].forEach(function (card) {
      if (card) { card.style.display = "none"; }
    });
    if (local) {
      local.hidden = false;
      local.style.display = "";
    }
    return;
  }
  if (local) {
    local.hidden = true;
    local.style.display = "none";
  }
}

function applyShellModeVisibility(config) {
  var shell = isShellMode(config);
  if (document.body) {
    document.body.classList.toggle("gb-shell-mode", shell);
  }
  applySecurityStory(config);
  document.querySelectorAll('[data-shell-only="true"]').forEach(function (node) {
    if (shell) {
      node.removeAttribute("hidden");
    } else {
      node.setAttribute("hidden", "");
    }
  });
  // #1877 — panels that only make sense in shell mode (OS, Shared Links)
  // lazy-load their data. They are revealed after productConfig arrives, so
  // announce the transition and let each panel refresh itself.
  window.dispatchEvent(new CustomEvent("gb-shell-mode-changed", { detail: { shell: shell } }));
  if (shell) { return; }
  var active = document.querySelector(".settings-section.active");
  if (active && active.id === "os-section") {
    var profile = document.querySelector('.settings-nav a[href="#profile"]');
    if (profile) { profile.click(); }
  }
}

function hideSettingsSection(id) {
  var section = document.getElementById(id + "-section");
  if (section) { section.style.display = "none"; }
  var nav = document.querySelector('.settings-nav a[href="#' + id + '"]');
  if (nav) { nav.style.display = "none"; }
}

function hideSettingControl(control) {
  if (!control) { return; }
  var row = control.closest ? control.closest(".setting-row, .form-group") : null;
  if (row) { row.style.display = "none"; }
}

function applyDesktopEditionVisibility() {
  // #1838: DNS Management does not apply to the desktop edition.
  hideSettingsSection("dns");
  var dnsModal = document.getElementById("register-dns-modal");
  if (dnsModal) { dnsModal.style.display = "none"; }

  // #1837: Billing does not make sense in the desktop edition.
  hideSettingsSection("billing");

  // #1815: remove Marketing/news opt-in from desktop notifications.
  hideSettingControl(document.querySelector('input[name="email_marketing"]'));

  // #1837: reformulate Privacy for desktop (local, single-user context).
  var privacy = document.getElementById("privacy-section");
  if (privacy) {
    var subtitle = privacy.querySelector(".section-header .subtitle");
    if (subtitle) { subtitle.textContent = "Manage local privacy and data on this device"; }

    var cards = privacy.querySelectorAll(".setting-card");
    if (cards[0]) {
      var h = cards[0].querySelector(".card-header h2");
      if (h) { h.textContent = "Local Privacy"; }
      hideSettingControl(cards[0].querySelector('[name="profile_visibility"]'));
      var descs = cards[0].querySelectorAll(".setting-desc");
      descs.forEach(function (d) {
        var t = d.textContent || "";
        if (t.indexOf("Show when") !== -1) { d.textContent = "Show your status in local conversations"; }
        if (t.indexOf("read messages") !== -1) { d.textContent = "Let others know when you've read messages in local conversations"; }
      });
    }
    if (cards[1]) {
      var exportBtn = cards[1].querySelector('button[onclick*="downloadDataExport"], [hx-post="/api/user/data/export"]');
      if (exportBtn) { exportBtn.removeAttribute("hx-post"); exportBtn.removeAttribute("hx-swap"); exportBtn.removeAttribute("hx-on::after-request"); exportBtn.textContent = "Export Local Data"; exportBtn.title = "Export data stored on this device"; }
      var exportDesc = cards[1].querySelector(".setting-desc");
      if (exportDesc) { exportDesc.textContent = "Export data stored on this device"; }
      var exportHeadP = cards[1].querySelector(".card-header p");
      if (exportHeadP) { exportHeadP.textContent = "Export data stored on this device"; }
    }
    if (cards[2]) { cards[2].style.display = "none"; }
  }

  var active = document.querySelector(".settings-section.active");
  if (active && ["billing-section", "dns-section"].indexOf(active.id) !== -1) {
    var profile = document.querySelector('.settings-nav a[href="#profile"]');
    if (profile) { profile.click(); }
  }
}

function applyEditionVisibility(config) {
  if (isDesktopEdition(config)) { applyDesktopEditionVisibility(config); }
  if (!isSlimEdition(config)) { return; }
  ["billing", "vibe"].forEach(hideSettingsSection);
  document.querySelectorAll('[data-admin-only="true"]').forEach(function (node) {
    // #1852 — the Operating System panel is admin-gated too, but it must stay
    // reachable in the slim desktop-local profile. Its shell-mode switch owns
    // visibility; RBAC hides it again for non-admin users.
    if (node.getAttribute("data-shell-only") === "true") { return; }
    node.style.display = "none";
  });
  var active = document.querySelector(".settings-section.active");
  if (active && ["billing-section", "vibe-section"].indexOf(active.id) !== -1) {
    var profile = document.querySelector('.settings-nav a[href="#profile"]');
    if (profile) { profile.click(); }
  }
}

function initEditionVisibility() {
  var resolvedDesktop = false;
  function run() {
    applyShellModeVisibility(window.productConfig);
    if (window.productConfig) {
      resolvedDesktop = isDesktopEdition(window.productConfig);
      applyEditionVisibility(window.productConfig);
      return true;
    }
    // Desktop shell may be detectable before productConfig loads.
    if (window.__TAURI__ || window.__TAURI_INTERNALS__) {
      resolvedDesktop = true;
      applyDesktopEditionVisibility();
    }
    return false;
  }
  function reapply() {
    applyShellModeVisibility(window.productConfig);
    if (window.GBNavRegistry) { window.GBNavRegistry.applyNavVisibility(); }
    if (resolvedDesktop || isDesktopEdition(window.productConfig)) {
      resolvedDesktop = true;
      applyDesktopEditionVisibility();
    }
    if (window.productConfig) { applyEditionVisibility(window.productConfig); }
  }
  if (!run()) {
    var tries = 0;
    var timer = setInterval(function () {
      tries += 1;
      if (run() || tries > 40) { clearInterval(timer); }
    }, 100);
  }
  // Settings sections load via HTMX partials; reapply when content arrives.
  document.body.addEventListener("htmx:afterSwap", reapply);
  document.body.addEventListener("htmx:load", reapply);
}

  S.currentEdition = currentEdition;
  S.applySecurityStory = applySecurityStory;
  S.isDesktopEdition = isDesktopEdition;
  S.isSlimEdition = isSlimEdition;
  S.isShellMode = isShellMode;
  S.hideSettingsSection = hideSettingsSection;
  S.applyDesktopEditionVisibility = applyDesktopEditionVisibility;
  S.applyShellModeVisibility = applyShellModeVisibility;
  S.applyEditionVisibility = applyEditionVisibility;
  S.initEditionVisibility = initEditionVisibility;
})(window);
