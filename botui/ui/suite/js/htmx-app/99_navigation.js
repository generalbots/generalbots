(function (window, document) {
  "use strict";
  var N = document.__gb_mod_htmx_app;
  if (!N) return;
  var initHTMX = N.initHTMX;
  var showNotification = N.showNotification;
  var appendMessage = N.appendMessage;
  var updateConnectionStatus = N.updateConnectionStatus;
  var config = N.config;
  function initNavigation() {
    // Update active nav item on page change
    document.addEventListener("htmx:pushedIntoHistory", (event) => {
      const path = event.detail.path;
      updateActiveNav(path);
    });

    // Also listen for htmx:afterSwap to catch all navigation
    document.addEventListener("htmx:afterSwap", (event) => {
      setTimeout(() => {
        const path = window.location.hash || window.location.pathname;
        updateActiveNav(path);
      }, 10);
    });

    // Handle hash change
    window.addEventListener("hashchange", (event) => {
      updateActiveNav(window.location.hash);
    });

    // Handle browser back/forward
    window.addEventListener("popstate", (event) => {
      updateActiveNav(window.location.hash || window.location.pathname);
    });

    // Handle direct clicks on app tabs and app items
    document.addEventListener("click", (event) => {
      const appTab = event.target.closest(".app-tab");
      const appItem = event.target.closest(".app-item");

      if (appTab || appItem) {
        const element = appTab || appItem;
        const href = element.getAttribute("href");
        if (href) {
          // Immediately update active state on click
          updateActiveNav(href);
        }
      }
    });
  }

  // Get current section from URL
  function getCurrentSection() {
    const hash = window.location.hash;
    if (hash && hash.length > 1) {
      // Handle both #section and /#section formats
      return hash.replace(/^#\/?/, "").split("/")[0].split("?")[0];
    }
    return "chat";
  }

  // Update active navigation item and page title
  function updateActiveNav(path) {
    // Extract section name from path
    // Handles: "/#chat", "#chat", "/chat", "chat", "/#paper", "#paper"
    let section;
    if (path && path.length > 0) {
      // Remove leading /, #, or /# combinations
      section = path
        .replace(/^[/#]+/, "")
        .split("/")[0]
        .split("?")[0];
    }

    // Fallback to current URL hash if section is empty
    if (!section) {
      section = getCurrentSection();
    }

    // Set data-app attribute on body for CSS targeting (e.g., docs app styling)
    if (section) {
      document.body.setAttribute("data-app", section);
    }

    // First, remove ALL active classes from all tabs, items, and apps button
    document.querySelectorAll(".app-tab.active").forEach((item) => {
      item.classList.remove("active");
    });
    document.querySelectorAll(".app-item.active").forEach((item) => {
      item.classList.remove("active");
    });

    // Remove active from apps button
    const appsButton = document.getElementById("appsButton");
    if (appsButton) {
      appsButton.classList.remove("active");
    }

    // Check if section exists in the main header tabs
    let foundInHeaderTabs = false;
    document.querySelectorAll(".app-tab").forEach((item) => {
      const dataSection = item.getAttribute("data-section");
      const href = item.getAttribute("href");
      const itemSection = dataSection || (href ? href.replace(/^#/, "") : "");
      if (itemSection === section) {
        item.classList.add("active");
        foundInHeaderTabs = true;
      }
    });

    // Update app items in launcher dropdown (always mark the current section)
    document.querySelectorAll(".app-item").forEach((item) => {
      const href = item.getAttribute("href");
      const dataSection = item.getAttribute("data-section");
      const itemSection = dataSection || (href ? href.replace(/^#/, "") : "");
      if (itemSection === section) {
        item.classList.add("active");
      }
    });

    // If section is NOT in header tabs, select the apps button instead
    if (!foundInHeaderTabs && appsButton) {
      appsButton.classList.add("active");
    }

    // Update page title
    const sectionName = section.charAt(0).toUpperCase() + section.slice(1);
    document.title = sectionName + " - General Bots";
  }

  // Initialize keyboard shortcuts
  function initKeyboardShortcuts() {
    document.addEventListener("keydown", (e) => {
      // Send message on Enter (when in input)
      if (e.key === "Enter" && !e.shiftKey) {
        const input = document.getElementById("messageInput");
        if (input && document.activeElement === input) {
          e.preventDefault();
          const form = input.closest("form");
          if (form) {
            htmx.trigger(form, "submit");
          }
        }
      }

      // Focus input on /
      if (e.key === "/" && document.activeElement.tagName !== "INPUT") {
        e.preventDefault();
        const input = document.getElementById("messageInput");
        if (input) input.focus();
      }

      // Escape to blur input
      if (e.key === "Escape") {
        const input = document.getElementById("messageInput");
        if (input && document.activeElement === input) {
          input.blur();
        }
      }
    });
  }

  // Initialize scroll behavior
  function initScrollBehavior() {
    const scrollBtn = document.getElementById("scrollToBottom");
    const messages = document.getElementById("messages");

    if (scrollBtn && messages) {
      // Show/hide scroll button
      messages.addEventListener("scroll", () => {
        const isAtBottom =
          messages.scrollHeight - messages.scrollTop <=
          messages.clientHeight + 100;
        scrollBtn.style.display = isAtBottom ? "none" : "flex";
      });

      // Scroll to bottom on click
      scrollBtn.addEventListener("click", () => {
        messages.scrollTo({
          top: messages.scrollHeight,
          behavior: "smooth",
        });
      });
    }
  }

  // Initialize theme if ThemeManager exists
  function initTheme() {
    if (window.ThemeManager) {
      ThemeManager.init();
    }
  }

  // Main initialization
  function init() {
    console.log("Initializing HTMX application...");

    // Initialize HTMX
    initHTMX();

    // Initialize navigation
    initNavigation();

    // Initialize keyboard shortcuts
    initKeyboardShortcuts();

    // Initialize scroll behavior
    initScrollBehavior();

    // Initialize theme
    initTheme();

    // Set initial active nav based on hash or default to chat
    updateActiveNav(window.location.hash || "#chat");

    console.log("HTMX application initialized");
  }

  // Wait for DOM and HTMX to be ready
  if (document.readyState === "loading") {
    (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    init();
  }

  // Expose public API
  window.BotServerApp = {
    showNotification,
    appendMessage,
    updateConnectionStatus,
    config,
  };

  N.initNavigation = initNavigation;
  N.getCurrentSection = getCurrentSection;
  N.updateActiveNav = updateActiveNav;
  N.initKeyboardShortcuts = initKeyboardShortcuts;
  N.initScrollBehavior = initScrollBehavior;
  N.initTheme = initTheme;
  N.init = init;
  delete document.__gb_mod_htmx_app;
})(window, document);
