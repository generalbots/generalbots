(function () {
  "use strict";

  const APP_GUIDES = [
    {
      id: "local-files",
      label: "Local Files",
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                <path d="M12 11v6M9 14h6"/>
            </svg>`,
      hxGet: "/app/guides/local-files.html",
      description: "Access and manage files on your device",
    },
    {
      id: "native-settings",
      label: "App Settings",
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>`,
      hxGet: "/app/guides/native-settings.html",
      description: "Configure desktop app settings",
    },
  ];

  const APP_STYLES = `
        .app-grid-separator {
            grid-column: 1 / -1;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 0;
            margin-top: 0.5rem;
            border-top: 1px solid var(--border-color, #e0e0e0);
        }

        .app-grid-separator span {
            font-size: 0.75rem;
            color: var(--text-secondary, #666);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        .app-item.app-only {
            position: relative;
        }

        .app-item.app-only::after {
            content: '';
            position: absolute;
            top: 4px;
            right: 4px;
            width: 6px;
            height: 6px;
            background: var(--accent-color, #4a90d9);
            border-radius: 50%;
        }
    `;

  function injectStyles() {
    const styleEl = document.createElement("style");
    styleEl.id = "botapp-styles";
    styleEl.textContent = APP_STYLES;
    document.head.appendChild(styleEl);
  }

  function injectAppGuides() {
    const grid = document.querySelector(".app-grid");
    if (!grid) {
      setTimeout(injectAppGuides, 100);
      return;
    }

    if (document.querySelector(".app-grid-separator")) {
      return;
    }

    const separator = document.createElement("div");
    separator.className = "app-grid-separator";
    separator.innerHTML = "<span>Desktop Features</span>";
    grid.appendChild(separator);

    APP_GUIDES.forEach((guide) => {
      const item = document.createElement("a");
      item.className = "app-item app-only";
      item.href = `#${guide.id}`;
      item.dataset.section = guide.id;
      item.setAttribute("role", "menuitem");
      item.setAttribute("aria-label", guide.description || guide.label);
      item.setAttribute("hx-get", guide.hxGet);
      item.setAttribute("hx-target", "#main-content");
      item.setAttribute("hx-push-url", "true");
      item.innerHTML = `
                <div class="app-icon" aria-hidden="true">${guide.icon}</div>
                <span>${guide.label}</span>
            `;
      grid.appendChild(item);
    });

    if (window.htmx) {
      htmx.process(grid);
    }

    console.log("[BotApp] App guides injected successfully");
  }

  function setupTauriEvents() {
    if (!window.__TAURI__) {
      console.warn("[BotApp] Tauri API not available");
      return;
    }

    const { listen } = window.__TAURI__.event;

    listen("upload_progress", (event) => {
      const progress = event.payload;
      const progressEl = document.getElementById("upload-progress");
      if (progressEl) {
        progressEl.style.width = `${progress}%`;
        progressEl.textContent = `${Math.round(progress)}%`;
      }
    });

    console.log("[BotApp] Tauri event listeners registered");
  }

  function init() {
    console.log("[BotApp] Initializing app extensions...");
    injectStyles();
    injectAppGuides();
    setupTauriEvents();
    console.log("[BotApp] App extensions initialized");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  window.BotApp = {
    isApp: true,
    version: "6.1.0",
    guides: APP_GUIDES,

    invoke: async function (cmd, args) {
      if (!window.__TAURI__) {
        throw new Error("Tauri API not available");
      }
      return window.__TAURI__.core.invoke(cmd, args);
    },

    fs: {
      listFiles: (path) => window.BotApp.invoke("list_files", { path }),
      uploadFile: (srcPath, destPath) =>
        window.BotApp.invoke("upload_file", { srcPath, destPath }),
      createFolder: (path, name) =>
        window.BotApp.invoke("create_folder", { path, name }),
      deletePath: (path) => window.BotApp.invoke("delete_path", { path }),
      getHomeDir: () => window.BotApp.invoke("get_home_dir"),
    },

    notify: async function (title, body) {
      if (window.__TAURI__?.notification) {
        await window.__TAURI__.notification.sendNotification({ title, body });
      }
    },

    openFileDialog: async function (options = {}) {
      if (!window.__TAURI__?.dialog) {
        throw new Error("Dialog API not available");
      }
      return window.__TAURI__.dialog.open(options);
    },

    saveFileDialog: async function (options = {}) {
      if (!window.__TAURI__?.dialog) {
        throw new Error("Dialog API not available");
      }
      return window.__TAURI__.dialog.save(options);
    },
  };
})();
