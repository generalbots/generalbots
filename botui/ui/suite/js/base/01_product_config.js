const appsBtn = document.getElementById("apps-btn");
const appsDropdown = document.getElementById("apps-dropdown");
const settingsBtn = document.getElementById("settings-btn");
const settingsPanel = document.getElementById("settings-panel");
const userAvatarBtn = document.getElementById("userAvatar");
const userMenu = document.getElementById("userMenu");

// Global product configuration
window.productConfig = null;

// Load product configuration and apply white-label settings
async function loadProductConfig() {
  try {
    const response = await fetch("/api/product");
    if (response.ok) {
      window.productConfig = await response.json();
      // A profile's app set is authoritative (#1596/#1811): never inject
      // editor/sheet or any other app. The `.product` file alone decides what
      // is always visible, preview or hidden.
      if (!Array.isArray(window.productConfig.apps)) {
        window.productConfig.apps = [];
      }
      applyProductConfig(window.productConfig);
    }
  } catch (e) {
    console.warn("Failed to load product config:", e);
  }
}

// Apply product configuration to the UI
function applyProductConfig(config) {
  if (!config) return;

  // Update page title with product name
  if (config.name && config.name !== "General Bots") {
    document.title = document.title.replace("General Bots", config.name);
    // Update any visible branding text
    document.querySelectorAll("[data-brand]").forEach((el) => {
      el.textContent = el.textContent.replace("General Bots", config.name);
    });
  }

  // Apply default theme if specified and user hasn't set one
  if (config.theme && !localStorage.getItem("gb-theme")) {
    document.body.setAttribute("data-theme", config.theme);
    localStorage.setItem("gb-theme", config.theme);
  }

  // Filter apps based on enabled list
  if (config.apps && Array.isArray(config.apps)) {
    let effectiveApps = config.apps;

    // Check if we have compiled_features info to filter even further
    // This ensures we don't show apps that are enabled in config but not compiled in binary
    if (config.compiled_features && Array.isArray(config.compiled_features)) {
      const compiledSet = new Set(
        config.compiled_features.map((f) => f.toLowerCase()),
      );
      effectiveApps = effectiveApps.filter(
        (app) =>
          compiledSet.has(app.toLowerCase()) ||
          app.toLowerCase() === "settings" ||
          app.toLowerCase() === "auth" ||
          app.toLowerCase() === "admin", // Admin usually contains settings which is always there
      );

      // Also call a helper to hide UI elements for non-compiled features explicitly
      // This handles features that might not be "apps" but are UI sections
      hideNonCompiledUI(compiledSet);
    }

    filterAppsByConfig(effectiveApps);

    // Check if there are any visible apps after filtering
    const hasVisibleApps = effectiveApps.length > 0;

    // Hide apps menu button if menu launcher is disabled or if there are no apps to show
    if (config.menu_launcher_enabled === false || !hasVisibleApps) {
      const appsButton = document.getElementById("appsButton");
      if (appsButton) {
        appsButton.style.display = "none";
      }
      const appsMenuContainer = document.querySelector(".apps-menu-container");
      if (appsMenuContainer) {
        appsMenuContainer.style.display = "none";
      }
    }
  }

  // #1841 — a build without the agent VM cannot host the terminal. Hide the
  // entry points instead of offering an app that can only fail, and keep the
  // same answer in isAppEnabled() for pins and quick actions.
  if (config.terminal_available === false) {
    document
      .querySelectorAll('[data-app-id="terminal"], [data-terminal-shortcut]')
      .forEach((el) => {
        el.style.display = "none";
        el.classList.add("hidden-uncompiled");
      });
  }

  // Hide omnibox if search is disabled
  if (config.search_enabled === false) {
    const omnibox = document.getElementById("omnibox");
    if (omnibox) {
      omnibox.style.display = "none";
    }
    const headerCenter = document.querySelector(".header-center");
    if (headerCenter) {
      headerCenter.style.display = "none";
    }
  }

  // Hide sidebar if disabled in product config
  if (config.sidebar === false) {
    const sidebar = document.getElementById("chatSidebar");
    if (sidebar) {
      sidebar.style.display = "none";
    }
  }

  // Apply custom logo
  if (config.logo) {
    const logoEl = document.querySelector(".header-logo img, .logo img");
    if (logoEl) logoEl.src = config.logo;
  }

  // Apply custom primary color
  if (config.primary_color) {
    document.documentElement.style.setProperty(
      "--primary",
      config.primary_color,
    );
  }

  // Update docs URL
  if (config.docs_url) {
    document.querySelectorAll('a[href*="docs.pragmatismo"]').forEach((el) => {
      el.href = config.docs_url;
    });
  }

  // Update copyright
  if (config.copyright) {
    document.querySelectorAll(".copyright, .footer-copyright").forEach((el) => {
      el.textContent = config.copyright;
    });
  }

  // #1596 — the config arrives asynchronously, after the sidebar and the
  // launchers may already have rendered. Announce it so surfaces that depend
  // on `isAppEnabled()` (quick actions, pins) re-render with the real app set.
  try {
    document.dispatchEvent(new CustomEvent("gb-product-config", { detail: config }));
  } catch (e) {
    console.warn("Could not announce product config:", e);
  }
}

// Hide UI elements that require features not compiled in the binary
function hideNonCompiledUI(compiledSet) {
  // Hide elements with data-feature attribute that aren't in compiled set
  document.querySelectorAll("[data-feature]").forEach((el) => {
    const feature = el.getAttribute("data-feature").toLowerCase();
    // Allow settings/admin as they are usually core
    if (
      !compiledSet.has(feature) &&
      feature !== "settings" &&
      feature !== "admin"
    ) {
      el.style.display = "none";
      el.classList.add("hidden-uncompiled");
    }
  });

  // Also look for specific sections that might map to features
  // e.g. .feature-mail, .feature-meet classes
  compiledSet.forEach((feature) => {
    // This loop defines what IS available.
    // Logic should be inverse: find all feature- classes and hide if not in set
    // But scanning all classes is expensive.
    // Better to rely on data-feature or explicit app hiding which filterAppsByConfig does.
  });
}

// #1614/#1615 — a product profile may hide an app that is still compiled and
// routed (the desktop edition hides Integrations, Automations and their Data
// Sources tab). Hiding is explicit and profile-scoped, so other editions keep
// the surface without a code change.
function isAppHidden(appName) {
  if (!appName) return false;
  const hidden = (window.productConfig && window.productConfig.hidden_apps) || [];
  const id = String(appName).toLowerCase();
  return hidden.some((entry) => String(entry).toLowerCase() === id);
}
window.isAppHidden = isAppHidden;

// Filter visible apps based on enabled list
function filterAppsByConfig(enabledApps) {
  const enabledSet = new Set(
    enabledApps.map((a) => a.toLowerCase()).filter((a) => !isAppHidden(a)),
  );

  // Hide nav items for disabled apps (data-app or data-section attribute)
  document.querySelectorAll("[data-app], [data-section]").forEach((el) => {
    const appName = (
      el.getAttribute("data-app") || el.getAttribute("data-section")
    ).toLowerCase();
    if (!enabledSet.has(appName) || isAppHidden(appName)) {
      el.style.display = "none";
    }
  });

  // Hide app items in dropdown by href matching
  document.querySelectorAll(".app-item").forEach((el) => {
    const href = el.getAttribute("href") || el.getAttribute("hx-get") || "";
    // Match both /app and #app patterns (hash and path-based links)
    const appMatch = href.match(
      /[\/#](chat|mail|calendar|drive|tasks|docs|paper|sheet|slides|meet|research|sources|analytics|admin|monitoring|settings|integrations|automations|connectors)/i,
    );
    if (appMatch) {
      const appName = appMatch[1].toLowerCase();
      if (!enabledSet.has(appName) || isAppHidden(appName)) {
        el.style.display = "none";
      }
    }
  });
}

// Check if an app is enabled
function isAppEnabled(appName) {
  if (isAppHidden(appName)) return false;
  if (
    String(appName || "").toLowerCase() === "terminal" &&
    window.productConfig &&
    window.productConfig.terminal_available === false
  ) {
    return false;
  }
  if (!window.productConfig || !window.productConfig.apps) return true;
  return window.productConfig.apps
    .map((a) => a.toLowerCase())
    .includes(appName.toLowerCase());
}
