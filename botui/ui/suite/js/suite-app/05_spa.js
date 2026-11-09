/* Suite app — SPA init: bot detection, panels, theme, hash routing (5/7). */
// Initialize Omnibox when DOM is ready
(function(){ var __cb = () => {
  // Detect bot name from pathname (e.g., /bot/cristo -> bot_name = "cristo", /edu -> bot_name = "edu")
  const detectBotFromPath = () => {
    const pathname = window.location.pathname;
    // Remove leading/trailing slashes and split
    const segments = pathname.replace(/^\/|\/$/g, "").split("/");

    // Handle /bot/{bot_name} pattern
    if (segments[0] === "bot" && segments[1]) {
      return segments[1];
    }

    // For other patterns, use first segment if it's not a known route
    const firstSegment = segments[0];
    const knownRoutes = ["suite", "auth", "api", "static", "public", "bot", "wizard"];
    // Suite apps are routes, not bot names — keep the domain-resolved bot the
    // server already injected (e.g. chat.pragmatismo.com.br/vibe -> pragmatismo).
    const suiteApps = ["vibe", "project", "database", "drive", "chat", "tasks", "admin", "mail", "calendar", "meet", "docs", "sheet", "slides", "paper", "research", "sources", "learn", "analytics", "dashboards", "monitoring", "governance", "people", "crm", "tickets", "billing", "products", "video", "player", "canvas", "social", "goals", "workspace", "designer", "integrations", "erp", "fraud", "settings", "about", "tools", "attendant", "banking", "biometry", "brazil", "browser", "campaigns", "compliance", "desktop", "email", "handoff", "hr", "itsm", "kyc", "lists", "o365", "minutes", "plan", "plugins", "pos", "retail", "tax", "templates", "templates-app", "terminal", "timeclock", "vision"];
    if (firstSegment && !knownRoutes.includes(firstSegment) && !suiteApps.includes(firstSegment)) {
      return firstSegment;
    }
    return window.__INITIAL_BOT_NAME__ || "default";
  };

  // Set global bot name
  window.__INITIAL_BOT_NAME__ = detectBotFromPath();
  console.log(`🤖 Bot detected from path: ${window.__INITIAL_BOT_NAME__}`);

  // Check if bot is public to skip authentication
  window.__checkBotPublicStatusPromise = (async () => {
    try {
      const botName = window.__INITIAL_BOT_NAME__;
      if (!botName || botName === "default") return;

      // Fetch without any credentials (no Authorization header).
      // GBSecurity.fetch interceptor strips tokens from public endpoints,
      // so this request is anonymous even when a stale session token
      // exists in localStorage from a previous login.
      const url = `/api/bot/public?bot_name=${encodeURIComponent(botName)}&_=${Date.now()}`;
      let response = await fetch(url);

      // The interceptor can still attach a bad token in edge cases
      // (e.g. the fetch polyfill bypasses our wrapper).  Retry once
      // with an explicit no-auth fetch if the first attempt 401s.
      if (response.status === 401) {
        console.warn("[GBSecurity] Bot public check 401 — retrying without auth");
        response = await fetch(url, { headers: { "Authorization": "" } });
      }

      if (response.ok) {
        const config = await response.json();
        if (config.is_public === "true") {
          window.__BOT_IS_PUBLIC__ = true;
          console.log(
            `✅ Bot '${botName}' is public - authentication not required`,
          );
        }
      }
    } catch (e) {
      console.warn("Failed to check bot public status:", e);
    }
  })();

  Omnibox.init();
  console.log("🚀 Initializing General Bots with HTMX...");

  // Check bot public status early
  // Already started via the promise above

  // Provide a global function to hide the loading overlay
  window.hideLoadingOverlay = function() {
    const loadingOverlay = document.getElementById("loadingOverlay");
    if (loadingOverlay && !loadingOverlay.classList.contains("hidden")) {
      loadingOverlay.classList.add("hidden");
    }
  };

  // Failsafe: hide after 10 seconds if no message arrives
  setTimeout(window.hideLoadingOverlay, 10000);

  // Simple apps menu handling
  const appsBtn = document.getElementById("appsButton");
  const appsDropdown = document.getElementById("appsDropdown");
  const settingsBtn = document.getElementById("settingsBtn");
  const settingsPanel = document.getElementById("settingsPanel");

  if (appsBtn && appsDropdown) {
    appsBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      const isOpen = appsDropdown.classList.toggle("show");
      appsBtn.setAttribute("aria-expanded", isOpen);
      // Close settings panel
      if (settingsPanel) settingsPanel.classList.remove("show");
    });

    document.addEventListener("click", (e) => {
      if (!appsDropdown.contains(e.target) && !appsBtn.contains(e.target)) {
        appsDropdown.classList.remove("show");
        appsBtn.setAttribute("aria-expanded", "false");
      }
    });
  }

  // Settings panel handling
  if (settingsBtn && settingsPanel) {
    settingsBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      const isOpen = settingsPanel.classList.toggle("show");
      settingsBtn.setAttribute("aria-expanded", isOpen);
      // Close apps dropdown
      if (appsDropdown) appsDropdown.classList.remove("show");
    });

    document.addEventListener("click", (e) => {
      if (
        !settingsPanel.contains(e.target) &&
        !settingsBtn.contains(e.target)
      ) {
        settingsPanel.classList.remove("show");
        settingsBtn.setAttribute("aria-expanded", "false");
      }
    });
  }

  // Theme selection handling
  const themeOptions = document.querySelectorAll(".theme-option");
  const savedTheme = localStorage.getItem("gb-theme") || "sentient";

  // Apply saved theme
  document.body.setAttribute("data-theme", savedTheme);
  document
    .querySelector(`.theme-option[data-theme="${savedTheme}"]`)
    ?.classList.add("active");

  themeOptions.forEach((option) => {
    option.addEventListener("click", () => {
      const theme = option.getAttribute("data-theme");
      document.body.setAttribute("data-theme", theme);
      localStorage.setItem("gb-theme", theme);
      themeOptions.forEach((o) => o.classList.remove("active"));
      option.classList.add("active");

      // Update theme-color meta tag
      const themeColors = {
        dark: "#3b82f6",
        light: "#3b82f6",
        purple: "#a855f7",
        green: "#22c55e",
        orange: "#f97316",
        sentient: "#d4f505",
      };
      const metaTheme = document.querySelector('meta[name="theme-color"]');
      if (metaTheme) {
        metaTheme.setAttribute("content", themeColors[theme] || "#d4f505");
      }
    });
  });

  // List of sections that appear in header tabs
  const headerTabSections = [
    "chat",
    "paper",
    "sheet",
    "slides",
    "mail",
    "calendar",
    "drive",
    "tasks",
  ];

  // Update active states for navigation (single selection only)
  function updateNavigationActive(section) {
    // Remove all active states first
    document
      .querySelectorAll(".app-tab")
      .forEach((t) => t.classList.remove("active"));
    document
      .querySelectorAll(".app-item")
      .forEach((i) => i.classList.remove("active"));
    appsBtn.classList.remove("active");

    // Check if section is in header tabs
    const isInHeaderTabs = headerTabSections.includes(section);

    // Activate the matching app-tab if in header
    if (isInHeaderTabs) {
      const headerTab = document.querySelector(
        `.app-tab[data-section="${section}"]`,
      );
      if (headerTab) {
        headerTab.classList.add("active");
      }
    } else {
      // Section is NOT in header tabs, activate apps button
      appsBtn.classList.add("active");
    }

    // Always mark the app-item in dropdown as active
    const appItem = document.querySelector(
      `.app-item[data-section="${section}"]`,
    );
    if (appItem) {
      appItem.classList.add("active");
    }
  }

  // Handle app item clicks - update active state
  document.querySelectorAll(".app-item").forEach((item) => {
    item.addEventListener("click", function () {
      const section = this.getAttribute("data-section");
      updateNavigationActive(section);
      appsDropdown.classList.remove("show");
      appsBtn.setAttribute("aria-expanded", "false");
    });
  });

  // Handle app tab clicks
  document.querySelectorAll(".app-tab").forEach((tab) => {
    tab.addEventListener("click", function () {
      const section = this.getAttribute("data-section");
      updateNavigationActive(section);
    });
  });

  // Track currently loaded section to prevent duplicate loads
  let currentLoadedSection = null;
  let isLoadingSection = false;
  let pendingLoadTimeout = null;

  // Handle hash navigation
  function handleHashChange(fromHtmxSwap = false) {
    const hash = window.location.hash.slice(1) || "chat";

    // Skip if already loaded this section or currently loading
    if (currentLoadedSection === hash || isLoadingSection) {
      return;
    }

    // If this was triggered by HTMX swap, just update tracking
    if (fromHtmxSwap) {
      currentLoadedSection = hash;
      return;
    }

    updateNavigationActive(hash);

    // Abort any pending load timeout
    if (pendingLoadTimeout) {
      clearTimeout(pendingLoadTimeout);
      pendingLoadTimeout = null;
    }

    // Abort any in-flight HTMX requests for main-content
    const mainContent = document.getElementById("main-content");
    if (mainContent) {
      try {
        htmx.trigger(mainContent, "htmx:abort");
      } catch (e) {
        // Ignore abort errors
      }
    }

    // Validate target exists before triggering HTMX load
    if (!mainContent) {
      console.warn("handleHashChange: #main-content not found, skipping load");
      return;
    }

    // Check if main-content is in the DOM
    if (!document.body.contains(mainContent)) {
      console.warn("handleHashChange: #main-content not in DOM, skipping load");
      return;
    }

    // Verify main-content has a valid parent (prevents insertBefore errors)
    if (!mainContent.parentNode) {
      console.warn(
        "handleHashChange: #main-content has no parent, skipping load",
      );
      return;
    }

    // Debounce the load to prevent rapid double-requests
    pendingLoadTimeout = setTimeout(() => {
      // Re-check if section changed during debounce
      const currentHash = window.location.hash.slice(1) || "chat";
      if (currentLoadedSection === currentHash) {
        return;
      }

      // Trigger HTMX load
      const appItem = document.querySelector(
        `.app-item[data-section="${currentHash}"]`,
      );
      if (appItem) {
        const hxGet = appItem.getAttribute("hx-get");
        if (hxGet) {
          try {
            isLoadingSection = true;
            currentLoadedSection = currentHash;
            htmx.ajax("GET", hxGet, {
              target: "#main-content",
              swap: "innerHTML",
            });
          } catch (e) {
            console.warn("handleHashChange: HTMX ajax error:", e);
            currentLoadedSection = null;
            isLoadingSection = false;
          }
        }
      }
    }, 50);
  }

  // Listen for HTMX swaps to track loaded sections and prevent duplicates
  document.body.addEventListener("htmx:afterSwap", (event) => {
    if (event.detail.target && event.detail.target.id === "main-content") {
      const hash = window.location.hash.slice(1) || "chat";
      currentLoadedSection = hash;
      isLoadingSection = false;
    }
  });

  // Reset tracking on swap errors
  document.body.addEventListener("htmx:swapError", (event) => {
    if (event.detail.target && event.detail.target.id === "main-content") {
      isLoadingSection = false;
    }
  });

  // Also listen for response errors
  document.body.addEventListener("htmx:responseError", (event) => {
    if (event.detail.target && event.detail.target.id === "main-content") {
      isLoadingSection = false;
      currentLoadedSection = null;
    }
  });

  // Load initial content based on hash or default to chat
  window.addEventListener("hashchange", handleHashChange);

  // Initial load - wait for HTMX to be ready
  function initialLoad() {
    if (typeof htmx !== "undefined" && htmx.ajax) {
      handleHashChange();
    } else {
      setTimeout(initialLoad, 50);
    }
  }

  // Skip SPA initialization on auth pages (login, register, etc.) and desktop
  if (window.location.pathname.startsWith("/auth/")) {
    console.log("[SPA] Skipping initialization on auth page");
  } else if (document.getElementById('desktop-content')) {
    console.log("[SPA] Skipping initialization on desktop page");
  } else if (document.readyState === "complete") {
    setTimeout(initialLoad, 50);
  } else {
    window.addEventListener("load", () => {
      setTimeout(initialLoad, 50);
    });
  }
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
