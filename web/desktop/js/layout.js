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
    // Resolve CSS path relative to the base directory.
    const cssPath = getBasePath() + htmlPath.replace(".html", ".css");

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

      // Load HTML
      const html = await loadSectionHTML(htmlPath);
      // Create wrapper for the new section
      const wrapper = document.createElement("div");
      wrapper.id = `section-${section}`;
      wrapper.className = "section";
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

      // Dispatch a custom event to notify the section it's being shown
      wrapper.dispatchEvent(new CustomEvent("section-shown"));

      // Ensure the new section is visible with a fast GSAP fade-in
      gsap.fromTo(
        wrapper,
        { opacity: 0 },
        { opacity: 1, duration: 0.15, ease: "power2.out" },
      );
    }

    // Then load JS after HTML is inserted (skip if already loaded)
    // Resolve JS path relative to the base directory.
    const jsPath = getBasePath() + htmlPath.replace(".html", ".js");
    const existingScript = document.querySelector(`script[src="${jsPath}"]`);

    if (!existingScript) {
      // Create script and wait for it to load before initializing Alpine
      const script = document.createElement("script");
      script.src = jsPath;
      script.defer = true;

      // Wait for script to load before initializing Alpine
      await new Promise((resolve, reject) => {
        script.onload = resolve;
        script.onerror = reject;
        document.body.appendChild(script);
      });
    }

    window.history.pushState({}, "", `#${section}`);

    // Start Alpine on first load, then just init the tree for new sections
    if (typeof window.startAlpine === "function") {
      window.startAlpine();
      delete window.startAlpine;
    } else if (window.Alpine) {
      window.Alpine.initTree(mainContent);
    }

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
  // Small delay to ensure all resources are loaded
  setTimeout(() => {
    const section = getInitialSection();
    // Ensure valid section
    if (!sections[section]) {
      window.location.hash = "#chat";
      switchSection("chat");
    } else {
      switchSection(section);
    }
  }, 50);
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
