if (typeof window.WindowManager === "undefined") {
  class WindowManager {
    constructor() {
      this.openWindows = [];
      this.activeWindowId = null;
      this.zIndexCounter = 100;
      // Will fetch dynamically in open() since script runs before DOM is ready
      this.workspace = null;
      this.taskbarApps = null;
    }

    open(id, title, htmlContent) {
      // Lazy load the container elements to avoid head script loading issues
      if (!this.workspace)
        this.workspace =
          document.getElementById("desktop-content") || document.body;
      if (!this.taskbarApps)
        this.taskbarApps = document.getElementById("taskbar-apps");

      // If window already exists, focus it
      const existingWindow = this.openWindows.find((w) => w.id === id);
      if (existingWindow) {
        this.focus(id);
        return;
      }

      // Create new window
      const windowData = {
        id,
        title,
        isMinimized: false,
        isMaximized: false,
        previousState: null,
      };
      this.openWindows.push(windowData);

      // Generate DOM structure
      const windowEl = document.createElement("div");
      windowEl.id = `window-${id}`;
      // Add random slight offset for cascade effect
      const offset = (this.openWindows.length * 20) % 100;
      const top = 100 + offset;
      const left = 150 + offset;

      windowEl.className = "window-element";
      windowEl.style.top = `${top}px`;
      windowEl.style.left = `${left}px`;
      windowEl.style.zIndex = this.zIndexCounter++;

      windowEl.innerHTML = `
                <!-- Header (Draggable) -->
                <div class="window-header">
                    <div class="font-mono text-xs font-bold text-brand-600 tracking-wide">${title}</div>
                    <div class="flex space-x-3 text-gray-400">
                        <button class="btn-minimize hover:text-gray-600" onclick="window.WindowManager.toggleMinimize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"></line></svg></button>
                        <button class="btn-maximize hover:text-gray-600" onclick="window.WindowManager.toggleMaximize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect></svg></button>
                        <button class="btn-close hover:text-red-500" onclick="window.WindowManager.close('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg></button>
                    </div>
                </div>
                <!-- Body (HTMX target) -->
                <div id="window-body-${id}" class="window-body relative flex-1 overflow-y-auto bg-[#fafdfa]"></div>
            `;

      this.workspace.appendChild(windowEl);

      // Inject content into the window body
      const windowBody = windowEl.querySelector(`#window-body-${id}`);
      if (windowBody) {
        this.injectContentWithScripts(windowBody, htmlContent);
      }

      // Add to taskbar
      if (this.taskbarApps) {
        const taskbarIcon = document.createElement("div");
        taskbarIcon.id = `taskbar-item-${id}`;
        taskbarIcon.className = "taskbar-item taskbar-icon";
        taskbarIcon.onclick = () => this.toggleMinimize(id);

        let iconHtml =
          '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect></svg>';
        if (id === "vibe")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>';
        else if (id === "tasks")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>';
        else if (id === "chat")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>';
        else if (id === "terminal")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>';
        else if (id === "drive")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>';
        else if (id === "editor")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>';
        else if (id === "browser")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/></svg>';
        else if (id === "mail")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg>';
        else if (id === "settings")
          iconHtml =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>';

        taskbarIcon.innerHTML = `
                    <div class="app-icon w-8 h-8 rounded-md flex items-center justify-center text-xs shadow-sm" style="color: var(--text, #374151);">
                        ${iconHtml}
                    </div>
                `;
        this.taskbarApps.appendChild(taskbarIcon);
      }

      this.makeDraggable(windowEl);
      this.makeResizable(windowEl);
      this.focus(id);

      // Tell HTMX to process the new content
      if (window.htmx) {
        htmx.process(windowEl);
      }
    }

    focus(id) {
      this.activeWindowId = id;
      const windowEl = document.getElementById(`window-${id}`);
      if (windowEl) {
        windowEl.style.zIndex = this.zIndexCounter++;
      }

      // Update document title
      const windowObj = this.openWindows.find((w) => w.id === id);
      if (windowObj) {
        document.title = `${windowObj.title} - General Bots`;
      }

      // Highlight taskbar icon
      if (this.taskbarApps) {
        const icons = this.taskbarApps.querySelectorAll(".taskbar-icon");
        icons.forEach((icon) => {
          icon.classList.add("border-transparent");
        });
        const activeIcon = document.getElementById(`taskbar-item-${id}`);
        if (activeIcon) {
          activeIcon.classList.remove("border-transparent");
        }
      }
    }

    close(id) {
      const windowEl = document.getElementById(`window-${id}`);
      if (windowEl) {
        windowEl.remove();
      }
      const taskbarIcon = document.getElementById(`taskbar-item-${id}`);
      if (taskbarIcon) {
        taskbarIcon.remove();
      }
      this.openWindows = this.openWindows.filter((w) => w.id !== id);
      if (this.activeWindowId === id) {
        this.activeWindowId = null;
        // Reset title to default when all windows are closed
        if (this.openWindows.length === 0) {
          document.title = "General Bots Desktop";
        }
      }
    }

    toggleMinimize(id) {
      const windowObj = this.openWindows.find((w) => w.id === id);
      if (!windowObj) return;

      const windowEl = document.getElementById(`window-${id}`);
      if (!windowEl) return;

      if (windowObj.isMinimized) {
        // Restore
        windowEl.style.display = "flex";
        windowObj.isMinimized = false;
        this.focus(id);
      } else {
        // Minimize
        windowEl.style.display = "none";
        windowObj.isMinimized = true;
        if (this.activeWindowId === id) {
          this.activeWindowId = null;
        }
      }
    }

    toggleMaximize(id) {
      const windowObj = this.openWindows.find((w) => w.id === id);
      if (!windowObj) return;

      const windowEl = document.getElementById(`window-${id}`);
      if (!windowEl) return;

      if (windowObj.isMaximized) {
        // Restore
        windowEl.style.width = windowObj.previousState.width;
        windowEl.style.height = windowObj.previousState.height;
        windowEl.style.top = windowObj.previousState.top;
        windowEl.style.left = windowObj.previousState.left;
        windowObj.isMaximized = false;

        // Check if any other windows are still maximized
        const anyMaximized = this.openWindows.some((w) => w.isMaximized);
        if (!anyMaximized) {
          document.body.classList.remove("window-maximized");
        }
      } else {
        // Maximize
        windowObj.previousState = {
          width: windowEl.style.width,
          height: windowEl.style.height,
          top: windowEl.style.top,
          left: windowEl.style.left,
        };

        // Adjust for taskbar height (assuming taskbar is at bottom)
        const taskbarHeight = document.getElementById("taskbar")
          ? document.getElementById("taskbar").offsetHeight
          : 0;

        // Adjust for minibar height (fixed 28px at top)
        const minibarHeight = 28;

        windowEl.style.width = "100%";
        windowEl.style.height = `calc(100% - ${minibarHeight}px)`;
        windowEl.style.top = `${minibarHeight}px`;
        windowEl.style.left = "0px";
        windowObj.isMaximized = true;

        // Add CSS class to body to hide background content
        document.body.classList.add("window-maximized");
      }
      this.focus(id);
    }

    makeDraggable(windowEl) {
      const header = windowEl.querySelector(".window-header");
      if (!header) return;

      let isDragging = false;
      let startX, startY, initialLeft, initialTop;

      const onMouseDown = (e) => {
        // Don't drag if clicking buttons
        if (
          e.target.tagName.toLowerCase() === "button" ||
          e.target.closest("button")
        )
          return;

        isDragging = true;
        startX = e.clientX;
        startY = e.clientY;
        initialLeft = parseInt(windowEl.style.left || 0, 10);
        initialTop = parseInt(windowEl.style.top || 0, 10);

        this.focus(windowEl.id.replace("window-", ""));

        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mouseup", onMouseUp);
      };

      const onMouseMove = (e) => {
        if (!isDragging) return;

        // Allow animation frame optimization here in a real implementation
        requestAnimationFrame(() => {
          const dx = e.clientX - startX;
          const dy = e.clientY - startY;

          // Add basic boundaries
          let newLeft = initialLeft + dx;
          let newTop = initialTop + dy;

          // Prevent dragging completely out
          newTop = Math.max(0, newTop);

          windowEl.style.left = `${newLeft}px`;
          windowEl.style.top = `${newTop}px`;
        });
      };

      const onMouseUp = () => {
        isDragging = false;
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
      };

      header.addEventListener("mousedown", onMouseDown);

      header.addEventListener("dblclick", (e) => {
        if (e.target.tagName.toLowerCase() === "button" || e.target.closest("button")) return;
        this.toggleMaximize(windowEl.id.replace("window-", ""));
      });

      // Add focus listener to the whole window
      windowEl.addEventListener("mousedown", () => {
        this.focus(windowEl.id.replace("window-", ""));
      });
    }

    injectContentWithScripts(container, htmlContent) {
      // Create a temporary div to parse the HTML
      const tempDiv = document.createElement("div");
      tempDiv.innerHTML = htmlContent;

      // Extract all script tags
      const scripts = tempDiv.querySelectorAll("script");
      const scriptsToExecute = [];

      scripts.forEach((originalScript) => {
        const scriptClone = document.createElement("script");
        Array.from(originalScript.attributes).forEach((attr) => {
          scriptClone.setAttribute(attr.name, attr.value);
        });
        scriptClone.textContent = originalScript.textContent;
        scriptsToExecute.push(scriptClone);
        originalScript.remove(); // Remove from tempDiv so innerHTML doesn't include it
      });

      // Inject HTML content without scripts
      container.innerHTML = tempDiv.innerHTML;

      // Execute each script
      scriptsToExecute.forEach((script) => {
        container.appendChild(script);
      });
    }

    makeResizable(windowEl) {
      // Implement simple bottom-right resize for now
      // In a full implementation, you'd add invisible handles
      windowEl.style.resize = "both";
      // Note: CSS resize creates conflicts with custom dragging/resizing if not careful.
      // For a true "WinBox" feel, custom handles (divs) on all 8 edges/corners are needed.
    }
  }

  // Initialize globally
  window.WindowManager = new WindowManager();
}
