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

// Filter visible apps based on enabled list
function filterAppsByConfig(enabledApps) {
  const enabledSet = new Set(enabledApps.map((a) => a.toLowerCase()));

  // Hide nav items for disabled apps
  document.querySelectorAll("[data-app]").forEach((el) => {
    const appName = el.getAttribute("data-app").toLowerCase();
    if (!enabledSet.has(appName)) {
      el.style.display = "none";
    }
  });

  // Hide app items in dropdown
  document.querySelectorAll(".app-item").forEach((el) => {
    const href = el.getAttribute("href") || el.getAttribute("hx-get") || "";
    const appMatch = href.match(
      /\/(chat|mail|calendar|drive|tasks|docs|paper|sheet|slides|meet|research|sources|analytics|admin|monitoring|settings)/i,
    );
    if (appMatch) {
      const appName = appMatch[1].toLowerCase();
      if (!enabledSet.has(appName)) {
        el.style.display = "none";
      }
    }
  });
}

// Check if an app is enabled
function isAppEnabled(appName) {
  if (!window.productConfig || !window.productConfig.apps) return true;
  return window.productConfig.apps
    .map((a) => a.toLowerCase())
    .includes(appName.toLowerCase());
}

// Load product config on page load
loadProductConfig();

if (appsBtn) {
  appsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = appsDropdown.classList.toggle("show");
    appsBtn.setAttribute("aria-expanded", isOpen);
    settingsPanel.classList.remove("show");
  });
}

if (settingsBtn) {
  settingsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = settingsPanel.classList.toggle("show");
    settingsBtn.setAttribute("aria-expanded", isOpen);
    appsDropdown.classList.remove("show");
    if (userMenu) {
      userMenu.style.display = "none";
      userAvatarBtn.setAttribute("aria-expanded", "false");
    }
  });
}

if (userAvatarBtn) {
  userAvatarBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen =
      userMenu.style.display === "none" || userMenu.style.display === "";
    userMenu.style.display = isOpen ? "block" : "none";
    userAvatarBtn.setAttribute("aria-expanded", isOpen);
    if (appsDropdown) appsDropdown.classList.remove("show");
    if (settingsPanel) settingsPanel.classList.remove("show");
    if (appsBtn) appsBtn.setAttribute("aria-expanded", "false");
    if (settingsBtn) settingsBtn.setAttribute("aria-expanded", "false");
  });
}

document.addEventListener("click", (e) => {
  if (
    appsDropdown &&
    !appsDropdown.contains(e.target) &&
    !appsBtn.contains(e.target)
  ) {
    appsDropdown.classList.remove("show");
    appsBtn.setAttribute("aria-expanded", "false");
  }
  if (
    settingsPanel &&
    !settingsPanel.contains(e.target) &&
    !settingsBtn.contains(e.target)
  ) {
    settingsPanel.classList.remove("show");
    settingsBtn.setAttribute("aria-expanded", "false");
  }
  if (
    userMenu &&
    !userMenu.contains(e.target) &&
    !userAvatarBtn.contains(e.target)
  ) {
    userMenu.style.display = "none";
    userAvatarBtn.setAttribute("aria-expanded", "false");
  }
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (appsDropdown) appsDropdown.classList.remove("show");
    if (settingsPanel) settingsPanel.classList.remove("show");
    if (userMenu) userMenu.style.display = "none";
    if (appsBtn) appsBtn.setAttribute("aria-expanded", "false");
    if (settingsBtn) settingsBtn.setAttribute("aria-expanded", "false");
    if (userAvatarBtn) userAvatarBtn.setAttribute("aria-expanded", "false");
  }
});

document.addEventListener("keydown", (e) => {
  if (e.altKey && !e.ctrlKey && !e.shiftKey) {
    const shortcuts = {
      1: "chat",
      2: "drive",
      3: "tasks",
      4: "mail",
      5: "calendar",
      6: "meet",
      7: "paper",
      8: "research",
      9: "sources",
      0: "analytics",
      a: "admin",
      m: "monitoring",
    };
    if (shortcuts[e.key]) {
      e.preventDefault();
      const link = document.querySelector(`a[href="#${shortcuts[e.key]}"]`);
      if (link) link.click();
      if (appsDropdown) appsDropdown.classList.remove("show");
    }
    if (e.key === ",") {
      e.preventDefault();
      if (settingsPanel) settingsPanel.classList.toggle("show");
    }
    if (e.key === "s") {
      e.preventDefault();
      const settingsLink = document.querySelector(`a[href="#settings"]`);
      if (settingsLink) settingsLink.click();
    }
  }
});

document.body.addEventListener("htmx:afterSwap", (e) => {
  if (e.detail.target.id === "main-content") {
    // Close settings panel on navigation
    if (settingsPanel) settingsPanel.classList.remove("show");
    // Note: Active class handling is done in htmx-app.js updateActiveNav()
  }
});

const themeOptions = document.querySelectorAll(".theme-option");
const savedTheme = localStorage.getItem("gb-theme") || "sentient";
document.body.setAttribute("data-theme", savedTheme);
document
  .querySelector(`.theme-option[data-theme="${savedTheme}"]`)
  ?.classList.add("active");

function updateThemeColor(theme) {
  const themeColors = {
    dark: "#3b82f6",
    light: "#3b82f6",
    blue: "#0ea5e9",
    purple: "#a855f7",
    green: "#22c55e",
    orange: "#f97316",
    sentient: "#d4f505",
    cyberpunk: "#ff00ff",
    retrowave: "#ff6b9d",
    vapordream: "#a29bfe",
    y2kglow: "#00ff00",
    arcadeflash: "#ffff00",
    discofever: "#ff1493",
    grungeera: "#8b4513",
    jazzage: "#d4af37",
    mellowgold: "#daa520",
    midcenturymod: "#e07b39",
    polaroidmemories: "#e6b89c",
    saturdaycartoons: "#ff6347",
    seasidepostcard: "#20b2aa",
    typewriter: "#2f2f2f",
    "3dbevel": "#0000ff",
    xeroxui: "#4a86cf",
    xtreegold: "#ffff00",
  };
  const metaTheme = document.querySelector('meta[name="theme-color"]');
  if (metaTheme) {
    metaTheme.setAttribute("content", themeColors[theme] || "#d4f505");
  }
}
updateThemeColor(savedTheme);

themeOptions.forEach((option) => {
  option.addEventListener("click", () => {
    const theme = option.getAttribute("data-theme");
    document.body.setAttribute("data-theme", theme);
    localStorage.setItem("gb-theme", theme);
    themeOptions.forEach((o) => o.classList.remove("active"));
    option.classList.add("active");
    updateThemeColor(theme);
  });
});

window.setTheme = function (theme, element) {
  document.body.setAttribute("data-theme", theme);
  localStorage.setItem("gb-theme", theme);

  // Update theme options in header settings panel
  themeOptions.forEach((o) => {
    o.classList.toggle("active", o.getAttribute("data-theme") === theme);
  });

  // Update theme options in settings page (if present)
  document.querySelectorAll(".theme-option").forEach((opt) => {
    opt.classList.toggle("active", opt.getAttribute("data-theme") === theme);
  });

  // If element was passed, ensure it's marked active
  if (element) {
    element.classList.add("active");
  }

  updateThemeColor(theme);

  // Show toast notification if available
  if (window.showNotification) {
    window.showNotification("Theme updated", "success");
  }
};

function toggleQuickSetting(el) {
  el.classList.toggle("active");
  const setting = el.id.replace("toggle-", "");
  localStorage.setItem(`gb-${setting}`, el.classList.contains("active"));
}

["notifications", "sound", "compact"].forEach((setting) => {
  const saved = localStorage.getItem(`gb-${setting}`);
  const toggle = document.getElementById(`toggle-${setting}`);
  if (toggle && saved !== null) {
    toggle.classList.toggle("active", saved === "true");
  }
});

function showKeyboardShortcuts() {
  window.showNotification(
    "Alt+1-9,0 for apps, Alt+A Admin, Alt+M Monitoring, Alt+S Settings, Alt+, quick settings",
    "info",
    8000,
  );
}

function announceToScreenReader(message) {
  const liveRegion = document.getElementById("aria-live");
  if (liveRegion) {
    liveRegion.textContent = message;
    setTimeout(() => {
      liveRegion.textContent = "";
    }, 1000);
  }
}

document.body.addEventListener("htmx:beforeRequest", function (e) {
  const target = e.detail.target;
  if (target && target.id === "main-content") {
    target.setAttribute("aria-busy", "true");
    announceToScreenReader("Loading content...");
  }
});

document.body.addEventListener("htmx:afterSwap", function (e) {
  const target = e.detail.target;
  if (target && target.id === "main-content") {
    target.setAttribute("aria-busy", "false");
    target.focus();
    announceToScreenReader("Content loaded");
  }
});

document.body.addEventListener("htmx:responseError", function (e) {
  const target = e.detail.target;
  if (target) {
    target.setAttribute("aria-busy", "false");
  }
  announceToScreenReader("Error loading content. Please try again.");
});

document.addEventListener("keydown", function (e) {
  const appsGrid = document.querySelector(".apps-grid");
  if (!appsGrid || !appsGrid.closest(".show")) return;

  const items = Array.from(appsGrid.querySelectorAll(".app-item"));
  const currentIndex = items.findIndex(
    (item) => item === document.activeElement,
  );

  if (currentIndex === -1) return;

  let newIndex = currentIndex;
  const columns = 3;

  switch (e.key) {
    case "ArrowRight":
      newIndex = Math.min(currentIndex + 1, items.length - 1);
      break;
    case "ArrowLeft":
      newIndex = Math.max(currentIndex - 1, 0);
      break;
    case "ArrowDown":
      newIndex = Math.min(currentIndex + columns, items.length - 1);
      break;
    case "ArrowUp":
      newIndex = Math.max(currentIndex - columns, 0);
      break;
    case "Home":
      newIndex = 0;
      break;
    case "End":
      newIndex = items.length - 1;
      break;
    default:
      return;
  }

  if (newIndex !== currentIndex) {
    e.preventDefault();
    items[newIndex].focus();
  }
});

window.showNotification = function (message, type = "info", duration = 5000) {
  const container = document.getElementById("notifications");
  if (!container) return;

  const notification = document.createElement("div");
  notification.className = `notification ${type}`;
  notification.innerHTML = `
        <div class="notification-content">
            <div class="notification-message">${message}</div>
        </div>
        <button class="notification-close" onclick="this.parentElement.remove()" style="background:none;border:none;color:var(--text-secondary);cursor:pointer;font-size:1.25rem;">×</button>
    `;
  container.appendChild(notification);
  if (duration > 0) {
    setTimeout(() => notification.remove(), duration);
  }
};

const htmxRetryConfig = {
  maxRetries: 3,
  retryDelay: 1000,
  retryCount: new Map(),
};

function getRetryKey(elt) {
  return (
    elt.getAttribute("hx-get") ||
    elt.getAttribute("hx-post") ||
    elt.getAttribute("hx-put") ||
    elt.getAttribute("hx-delete") ||
    elt.id ||
    Math.random().toString(36)
  );
}

function showErrorState(target, errorMessage, retryCallback) {
  const errorHtml = `
        <div class="error-state">
            <svg class="error-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <div class="error-state-title">Something went wrong</div>
            <div class="error-state-message">${errorMessage}</div>
            <div class="error-state-actions">
                <button class="btn-retry" onclick="window.retryLastRequest(this)">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M23 4v6h-6"></path>
                        <path d="M1 20v-6h6"></path>
                        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"></path>
                        <path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"></path>
                    </svg>
                    Try Again
                </button>
            </div>
        </div>
    `;
  target.innerHTML = errorHtml;
  target.dataset.retryCallback = retryCallback;
}

window.retryLastRequest = function (btn) {
  const target = btn.closest(".error-state").parentElement;
  const retryCallback = target.dataset.retryCallback;

  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Retrying...';

  if (retryCallback && window[retryCallback]) {
    window[retryCallback]();
  } else {
    const triggers = target.querySelectorAll("[hx-get], [hx-post]");
    if (triggers.length > 0) {
      htmx.trigger(triggers[0], "htmx:trigger");
    } else {
      const activeApp = document.querySelector(".app-item.active");
      if (activeApp) {
        activeApp.click();
      }
    }
  }
};

document.body.addEventListener("htmx:responseError", function (e) {
  const target = e.detail.target;
  const xhr = e.detail.xhr;
  const retryKey = getRetryKey(e.detail.elt);

  let currentRetries = htmxRetryConfig.retryCount.get(retryKey) || 0;

  if (
    (xhr.status === 0 || xhr.status >= 500) &&
    currentRetries < htmxRetryConfig.maxRetries
  ) {
    htmxRetryConfig.retryCount.set(retryKey, currentRetries + 1);
    const delay = htmxRetryConfig.retryDelay * Math.pow(2, currentRetries);

    window.showNotification(
      `Request failed. Retrying in ${delay / 1000}s... (${currentRetries + 1}/${htmxRetryConfig.maxRetries})`,
      "warning",
      delay,
    );

    setTimeout(() => {
      htmx.trigger(e.detail.elt, "htmx:trigger");
    }, delay);
  } else {
    htmxRetryConfig.retryCount.delete(retryKey);

    let errorMessage = "We couldn't load the content.";
    if (xhr.status === 401) {
      errorMessage = "Your session has expired. Please log in again.";
    } else if (xhr.status === 403) {
      errorMessage = "You don't have permission to access this resource.";
    } else if (xhr.status === 404) {
      errorMessage = "The requested content was not found.";
    } else if (xhr.status >= 500) {
      errorMessage =
        "The server is experiencing issues. Please try again later.";
    } else if (xhr.status === 0) {
      errorMessage =
        "Unable to connect. Please check your internet connection.";
    }

    if (target && target.id === "main-content") {
      showErrorState(target, errorMessage);
    } else {
      window.showNotification(errorMessage, "error", 8000);
    }
  }
});

document.body.addEventListener("htmx:afterRequest", function (e) {
  if (e.detail.successful) {
    const retryKey = getRetryKey(e.detail.elt);
    htmxRetryConfig.retryCount.delete(retryKey);
  }
});

document.body.addEventListener("htmx:timeout", function (e) {
  window.showNotification(
    "Request timed out. Please try again.",
    "warning",
    5000,
  );
});

document.body.addEventListener("htmx:sendError", function (e) {
  window.showNotification(
    "Network error. Please check your connection.",
    "error",
    5000,
  );
});
