const sections = {
  drive: "drive/drive.html",
  tasks: "tasks/tasks.html",
  mail: "mail/mail.html",
  chat: "chat/chat.html",
};
const sectionCache = {};

function getBasePath() {
  // All static assets (HTML, CSS, JS) are served from the site root.
  // Returning empty string for relative paths when served from same directory
  return "";
}

// Preload chat CSS to avoid flash on first load
function preloadChatCSS() {
  const chatCssPath = getBasePath() + "chat/chat.css";
  const existing = document.querySelector(`link[href="${chatCssPath}"]`);
  if (!existing) {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = chatCssPath;
    document.head.appendChild(link);
  }
}

async function loadSectionHTML(path) {
  const fullPath = getBasePath() + path;
  const response = await fetch(fullPath);
  if (!response.ok) throw new Error("Failed to load section: " + fullPath);
  return await response.text();
}

async function loadScript(jsPath) {
  return new Promise((resolve, reject) => {
    const existingScript = document.querySelector(`script[src="${jsPath}"]`);
    if (existingScript) {
      console.log(`Script already loaded: ${jsPath}`);
      resolve();
      return;
    }

    const script = document.createElement("script");
    script.src = jsPath;
    script.onload = () => {
      console.log(`✓ Script loaded: ${jsPath}`);
      resolve();
    };
    script.onerror = (err) => {
      console.error(`✗ Script failed to load: ${jsPath}`, err);
      reject(err);
    };
    document.body.appendChild(script);
  });
}

async function switchSection(section) {
  const mainContent = document.getElementById("main-content");

  // Validate section exists
  if (!sections[section]) {
    console.warn(`Section "${section}" does not exist, defaulting to chat`);
    section = "chat";
  }

  // Clean up any existing WebSocket connections from chat
  if (
    window.chatAppInstance &&
    typeof window.chatAppInstance.cleanup === "function"
  ) {
    window.chatAppInstance.cleanup();
  }

  try {
    const htmlPath = sections[section];
    console.log("Loading section:", section, "from", htmlPath);
    const cssPath = getBasePath() + htmlPath.replace(".html", ".css");
    const jsPath = getBasePath() + htmlPath.replace(".html", ".js");

    // Preload chat CSS if the target is chat
    if (section === "chat") {
      preloadChatCSS();
    }

    // Remove any existing section CSS
    document
      .querySelectorAll("link[data-section-css]")
      .forEach((link) => link.remove());

    // Load CSS first (skip if already loaded)
    let cssLink = document.querySelector(`link[href="${cssPath}"]`);
    if (!cssLink) {
      cssLink = document.createElement("link");
      cssLink.rel = "stylesheet";
      cssLink.href = cssPath;
      cssLink.setAttribute("data-section-css", "true");
      document.head.appendChild(cssLink);
    }

    // Hide previously loaded sections and show the requested one
    // Ensure a container exists for sections
    let container = document.getElementById("section-container");
    if (!container) {
      container = document.createElement("div");
      container.id = "section-container";
      mainContent.appendChild(container);
    }

    const targetDiv = document.getElementById(`section-${section}`);

    if (targetDiv) {
      // Section already loaded: hide others, show this one
      container.querySelectorAll(".section").forEach((div) => {
        div.style.display = "none";
      });
      targetDiv.style.display = "block";
    } else {
      // Remove any existing loading divs first
      container.querySelectorAll(".loading").forEach((div) => {
        div.remove();
      });

      // Show loading placeholder inside the container
      const loadingDiv = document.createElement("div");
      loadingDiv.className = "loading";
      loadingDiv.textContent = "Loading…";
      container.appendChild(loadingDiv);

      // For Alpine sections, load JavaScript FIRST before HTML
      const isAlpineSection = ["drive", "tasks", "mail"].includes(section);

      if (isAlpineSection) {
        console.log(`Loading JS before HTML for Alpine section: ${section}`);
        await loadScript(jsPath);

        // Wait for the component function to be registered
        const appFunctionName = section + "App";
        let retries = 0;
        while (typeof window[appFunctionName] !== "function" && retries < 100) {
          await new Promise((resolve) => setTimeout(resolve, 100));
          retries++;
        }

        if (typeof window[appFunctionName] !== "function") {
          console.error(`${appFunctionName} function not found after waiting!`);
          throw new Error(
            `Component function ${appFunctionName} not available`,
          );
        }

        console.log(`✓ Component function registered: ${appFunctionName}`);
      }

      // Load HTML
      const html = await loadSectionHTML(htmlPath);

      // Create wrapper for the new section
      const wrapper = document.createElement("div");
      wrapper.id = `section-${section}`;
      wrapper.className = "section";

      // For Alpine sections, mark for manual initialization
      if (isAlpineSection) {
        wrapper.setAttribute("x-ignore", "");
      }

      wrapper.innerHTML = html;

      // Hide any existing sections
      container.querySelectorAll(".section").forEach((div) => {
        div.style.display = "none";
        // Dispatch a custom event to notify sections they're being hidden
        div.dispatchEvent(new CustomEvent("section-hidden"));
      });

      // Remove loading placeholder if it still exists
      if (loadingDiv && loadingDiv.parentNode) {
        container.removeChild(loadingDiv);
      }

      // Add the new section to the container and cache it
      container.appendChild(wrapper);
      sectionCache[section] = wrapper;

      // For Alpine sections, initialize after DOM insertion
      if (isAlpineSection && window.Alpine) {
        console.log(`Initializing Alpine for section: ${section}`);

        // Remove x-ignore to allow Alpine to process
        wrapper.removeAttribute("x-ignore");

        // Verify component function is available
        const appFunctionName = section + "App";
        if (typeof window[appFunctionName] !== "function") {
          console.error(`${appFunctionName} not available during Alpine init!`);
          throw new Error(`Component ${appFunctionName} missing`);
        }

        // Small delay to ensure DOM is ready
        await new Promise((resolve) => setTimeout(resolve, 100));

        try {
          console.log(`Calling Alpine.initTree for ${section}`);
          window.Alpine.initTree(wrapper);
          console.log(`✓ Alpine initialized for ${section}`);
        } catch (err) {
          console.error(`Error initializing Alpine for ${section}:`, err);
        }
      } else if (!isAlpineSection) {
        // For non-Alpine sections (like chat), load JS after HTML
        await loadScript(jsPath);
        await new Promise((resolve) => setTimeout(resolve, 100));
      }

      // Dispatch a custom event to notify the section it's being shown
      wrapper.dispatchEvent(new CustomEvent("section-shown"));

      // Ensure the new section is visible with a fast GSAP fade-in
      gsap.fromTo(
        wrapper,
        { opacity: 0 },
        { opacity: 1, duration: 0.15, ease: "power2.out" },
      );
    }

    window.history.pushState({}, "", `#${section}`);

    const inputEl = document.getElementById("messageInput");
    if (inputEl) {
      inputEl.focus();
    }
  } catch (err) {
    console.error("Error loading section:", err);
    mainContent.innerHTML = `<div class="error">Failed to load ${section} section</div>`;
  }
}

// Handle initial load based on URL hash
function getInitialSection() {
  // 1️⃣ Prefer hash fragment (e.g., #chat)
  let section = window.location.hash.substring(1);
  // 2️⃣ Fallback to pathname segments (e.g., /chat)
  if (!section) {
    const parts = window.location.pathname.split("/").filter((p) => p);
    const last = parts[parts.length - 1];
    if (["drive", "tasks", "mail", "chat"].includes(last)) {
      section = last;
    }
  }
  // 3️⃣ As a last resort, inspect the full URL for known sections
  if (!section) {
    const match = window.location.href.match(
      /\/(drive|tasks|mail|chat)(?:\.html)?(?:[?#]|$)/i,
    );
    if (match) {
      section = match[1].toLowerCase();
    }
  }
  // Default to chat if nothing matches
  return section || "chat";
}

window.addEventListener("DOMContentLoaded", () => {
  console.log("DOM Content Loaded");

  const initApp = () => {
    const section = getInitialSection();
    console.log(`Initializing app with section: ${section}`);

    // Ensure valid section
    if (!sections[section]) {
      console.warn(`Invalid section: ${section}, defaulting to chat`);
      window.location.hash = "#chat";
      switchSection("chat");
    } else {
      switchSection(section);
    }
  };

  // Check if Alpine sections might be needed and wait for Alpine
  const hash = window.location.hash.substring(1);
  if (["drive", "tasks", "mail"].includes(hash)) {
    console.log(`Waiting for Alpine to load for section: ${hash}`);

    const waitForAlpine = () => {
      if (window.Alpine) {
        console.log("Alpine is ready");
        setTimeout(initApp, 100);
      } else {
        console.log("Waiting for Alpine...");
        setTimeout(waitForAlpine, 100);
      }
    };

    // Also listen for alpine:init event
    document.addEventListener("alpine:init", () => {
      console.log("Alpine initialized via event");
    });

    waitForAlpine();
  } else {
    // For chat, don't need to wait for Alpine
    setTimeout(initApp, 100);
  }
});

// Handle browser back/forward navigation
window.addEventListener("popstate", () => {
  const section = getInitialSection();
  // Ensure valid section
  if (!sections[section]) {
    window.location.hash = "#chat";
    switchSection("chat");
  } else {
    switchSection(section);
  }
});

// Make switchSection globally accessible
window.switchSection = switchSection;
