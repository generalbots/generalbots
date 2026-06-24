const ProgressPanel = {
  manifest: null,
  wsConnection: null, // Deprecated - now uses singleton from tasks.js
  startTime: null,
  runtimeInterval: null,
  _boundHandler: null, // Store bound handler for cleanup

  init(taskId) {
    // Clean up any existing handler before registering a new one
    // This prevents duplicate handlers if init is called multiple times
    if (
      this._boundHandler &&
      typeof unregisterTaskProgressHandler === "function"
    ) {
      unregisterTaskProgressHandler(this._boundHandler);
      this._boundHandler = null;
    }

    this.taskId = taskId;
    this.startTime = Date.now();
    this.startRuntimeCounter();
    this.connectWebSocket(taskId);
  },

  connectWebSocket(taskId) {
    // Instead of creating our own WebSocket, register with the singleton from tasks.js
    // This prevents the "2 receivers" problem where manifest_update goes to one connection
    // while the browser UI is listening on another

    console.log("[ProgressPanel] Using singleton WebSocket for task:", taskId);

    // Create bound handler that filters for our task
    this._boundHandler = (data) => {
      // Only process messages for our task
      if (data.task_id && String(data.task_id) !== String(taskId)) {
        return;
      }
      this.handleProgressUpdate(data);
    };

    // Register with the global singleton WebSocket
    if (typeof registerTaskProgressHandler === "function") {
      registerTaskProgressHandler(this._boundHandler);
      console.log("[ProgressPanel] Registered with singleton WebSocket");
    } else {
      // Fallback: wait for tasks.js to load and retry
      console.log(
        "[ProgressPanel] Waiting for tasks.js singleton to be available...",
      );
      setTimeout(() => this.connectWebSocket(taskId), 500);
    }
  },

  handleProgressUpdate(data) {
    // Skip manifest_update - already handled by tasks.js renderManifestProgress()
    // Processing it here would cause duplicate updates and race conditions
    if (
      data.type === "manifest_update" ||
      data.event_type === "manifest_update"
    ) {
      // Don't process here - tasks.js handles this via handleWebSocketMessage()
      // which calls renderManifestProgress() with proper normalized ID handling
      return;
    }

    if (data.type === "section_update") {
      this.updateSection(data.section_id, data.status, data.progress);
    } else if (data.type === "item_update") {
      this.updateItem(
        data.section_id,
        data.item_id,
        data.status,
        data.duration,
      );
    } else if (data.type === "terminal_line") {
      this.addTerminalLine(data.content, data.line_type);
    } else if (data.type === "stats_update") {
      this.updateStats(data.stats);
    } else if (data.type === "task_progress") {
      this.handleTaskProgress(data);
    }
  },

  handleTaskProgress(data) {
    // Check for manifest in activity
    if (data.activity && data.activity.manifest) {
      this.manifest = data.activity.manifest;
      this.render();
    }

    // Also check for manifest in details (manifest_update events)
    if (
      data.details &&
      (data.step === "manifest_update" || data.event_type === "manifest_update")
    ) {
      try {
        const parsed =
          typeof data.details === "string"
            ? JSON.parse(data.details)
            : data.details;
        if (parsed && parsed.sections) {
          this.manifest = parsed;
          this.render();
        }
      } catch (e) {
        // Not a manifest JSON, might be terminal output
        console.debug("Details is not manifest JSON:", e.message);
      }
    }

    if (data.step && data.step !== "manifest_update") {
      this.updateCurrentAction(data.message || data.step);
    }

    // Only add non-manifest details as terminal lines
    if (
      data.details &&
      data.step !== "manifest_update" &&
      data.event_type !== "manifest_update"
    ) {
      this.addTerminalLine(data.details, "info");
    }
  },

  startRuntimeCounter() {
    this.runtimeInterval = setInterval(() => {
      const elapsed = Math.floor((Date.now() - this.startTime) / 1000);
      const runtimeEl = document.getElementById("status-runtime");
      if (runtimeEl) {
        runtimeEl.textContent = this.formatDuration(elapsed);
      }
    }, 1000);
  },

  stopRuntimeCounter() {
    if (this.runtimeInterval) {
      clearInterval(this.runtimeInterval);
      this.runtimeInterval = null;
    }
  },

  formatDuration(seconds) {
    if (seconds < 60) {
      return `${seconds} sec`;
    } else if (seconds < 3600) {
      const mins = Math.floor(seconds / 60);
      return `${mins} min`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const mins = Math.floor((seconds % 3600) / 60);
      return `${hours} hr ${mins} min`;
    }
  },

  render() {
    if (!this.manifest) return;

    this.renderStatus();
    this.renderProgressLog();
    this.renderTerminal();
  },

  renderStatus() {
    const titleEl = document.getElementById("status-title");
    if (titleEl) {
      titleEl.textContent = this.manifest.description || this.manifest.app_name;
    }

    const estimatedEl = document.getElementById("estimated-time");
    if (estimatedEl && this.manifest.estimated_seconds) {
      estimatedEl.textContent = this.formatDuration(
        this.manifest.estimated_seconds,
      );
    }

    const currentAction = this.getCurrentAction();
    const actionEl = document.getElementById("current-action");
    if (actionEl && currentAction) {
      actionEl.textContent = currentAction;
    }

    this.updateDecisionPoint();
  },

  getCurrentAction() {
    if (!this.manifest || !this.manifest.sections) return null;

    for (const section of this.manifest.sections) {
      if (section.status === "Running") {
        for (const child of section.children || []) {
          if (child.status === "Running") {
            for (const item of child.items || []) {
              if (item.status === "Running") {
                return item.name;
              }
            }
            return child.name;
          }
        }
        return section.name;
      }
    }
    return null;
  },

  updateCurrentAction(action) {
    const actionEl = document.getElementById("current-action");
    if (actionEl) {
      actionEl.textContent = action;
    }
  },

  updateDecisionPoint() {
    const decisionStepEl = document.getElementById("decision-step");
    const decisionTotalEl = document.getElementById("decision-total");

    if (decisionStepEl && this.manifest) {
      decisionStepEl.textContent = this.manifest.completed_steps || 0;
    }
    if (decisionTotalEl && this.manifest) {
      decisionTotalEl.textContent = this.manifest.total_steps || 0;
    }
  },

  renderProgressLog() {
    const container = document.getElementById("progress-log-content");
    if (!container || !this.manifest || !this.manifest.sections) return;

    container.innerHTML = "";

    for (const section of this.manifest.sections) {
      const sectionEl = this.createSectionElement(section);
      container.appendChild(sectionEl);
    }
  },

  createSectionElement(section) {
    const sectionDiv = document.createElement("div");
    sectionDiv.className = "log-section";
    sectionDiv.dataset.sectionId = section.id;

    if (section.status === "Running" || section.status === "Completed") {
      sectionDiv.classList.add("expanded");
    }

    const statusClass = section.status.toLowerCase();
    // Support both direct fields and nested progress object
    const stepCurrent = section.current_step ?? section.progress?.current ?? 0;
    const stepTotal = section.total_steps ?? section.progress?.total ?? 0;

    sectionDiv.innerHTML = `
            <div class="log-section-header" onclick="ProgressPanel.toggleSection('${section.id}')">
                <span class="section-indicator ${statusClass}"></span>
                <span class="section-name">${this.escapeHtml(section.name)}</span>
                <span class="section-details-link" onclick="event.stopPropagation(); ProgressPanel.viewDetails('${section.id}')">View Details ▸</span>
                <span class="section-step-badge">Step ${stepCurrent}/${stepTotal}</span>
                <span class="section-status-badge ${statusClass}">${section.status}</span>
            </div>
            <div class="log-section-body">
                <div class="log-children" id="log-children-${section.id}">
                </div>
            </div>
        `;

    const childrenContainer = sectionDiv.querySelector(".log-children");

    for (const child of section.children || []) {
      const childEl = this.createChildElement(child, section.id);
      childrenContainer.appendChild(childEl);
    }

    if (
      section.items &&
      section.items.length > 0 &&
      (!section.children || section.children.length === 0)
    ) {
      for (const item of section.items) {
        const itemEl = this.createItemElement(item);
        childrenContainer.appendChild(itemEl);
      }
    }

    return sectionDiv;
  },

  createChildElement(child, parentId) {
    const childDiv = document.createElement("div");
    childDiv.className = "log-child";
    childDiv.dataset.childId = child.id;

    if (child.status === "Running" || child.status === "Completed") {
      childDiv.classList.add("expanded");
    }

    const statusClass = child.status.toLowerCase();
    // Support both direct fields and nested progress object
    const stepCurrent = child.current_step ?? child.progress?.current ?? 0;
    const stepTotal = child.total_steps ?? child.progress?.total ?? 0;
    const duration = child.duration_seconds
      ? this.formatDuration(child.duration_seconds)
      : "";

    childDiv.innerHTML = `
            <div class="log-child-header" onclick="ProgressPanel.toggleChild('${child.id}')">
                <span class="child-indent"></span>
                <span class="child-name">${this.escapeHtml(child.name)}</span>
                <span class="child-details-link" onclick="event.stopPropagation(); ProgressPanel.viewChildDetails('${child.id}')">View Details ▸</span>
                <span class="child-step-badge">Step ${stepCurrent}/${stepTotal}</span>
                <span class="child-status-badge ${statusClass}">${child.status}</span>
            </div>
            <div class="log-child-body">
                <div class="log-items" id="log-items-${child.id}">
                </div>
            </div>
        `;

    const itemsContainer = childDiv.querySelector(".log-items");

    for (const item of child.items || []) {
      const itemEl = this.createItemElement(item);
      itemsContainer.appendChild(itemEl);
    }

    return childDiv;
  },

  createItemElement(item) {
    const itemDiv = document.createElement("div");
    itemDiv.className = "log-item";
    itemDiv.dataset.itemId = item.id;

    const statusClass = item.status.toLowerCase();
    const duration = item.duration_seconds
      ? `Duration: ${this.formatDuration(item.duration_seconds)}`
      : "";
    const checkIcon =
      item.status === "Completed" ? "✓" : item.status === "Running" ? "◎" : "○";

    itemDiv.innerHTML = `
            <span class="item-dot ${statusClass}"></span>
            <span class="item-name">${this.escapeHtml(item.name)}${item.details ? ` - ${this.escapeHtml(item.details)}` : ""}</span>
            <div class="item-info">
                <span class="item-duration">${duration}</span>
                <span class="item-check ${statusClass}">${checkIcon}</span>
            </div>
        `;

    return itemDiv;
  },

  renderTerminal() {
    // Support both formats: terminal_output (direct) and terminal.lines (web JSON)
    const terminalLines =
      this.manifest?.terminal_output || this.manifest?.terminal?.lines || [];

    if (!terminalLines.length) return;

    const container = document.getElementById("terminal-content");
    if (!container) return;

    container.innerHTML = "";

    for (const line of terminalLines.slice(-50)) {
      this.appendTerminalLine(
        container,
        line.content,
        line.type || line.line_type || "info",
      );
    }

    container.scrollTop = container.scrollHeight;
  },

  addTerminalLine(content, lineType) {
    const container = document.getElementById("terminal-content");
    if (!container) return;

    this.appendTerminalLine(container, content, lineType);
    container.scrollTop = container.scrollHeight;

    this.incrementProcessedCount();
  },

  appendTerminalLine(container, content, lineType) {
    const lineDiv = document.createElement("div");
    lineDiv.className = `terminal-line ${lineType || "info"}`;
    lineDiv.textContent = content;
    container.appendChild(lineDiv);
  },

  incrementProcessedCount() {
    const processedEl = document.getElementById("terminal-processed");
    if (processedEl) {
      const current = parseInt(processedEl.textContent, 10) || 0;
      processedEl.textContent = current + 1;
    }
  },

  updateStats(stats) {
    const processedEl = document.getElementById("terminal-processed");
    if (processedEl && stats.data_points_processed !== undefined) {
      processedEl.textContent = stats.data_points_processed;
    }

    const speedEl = document.getElementById("terminal-speed");
    if (speedEl && stats.sources_per_min !== undefined) {
      speedEl.textContent = `~${stats.sources_per_min.toFixed(1)} sources/min`;
    }

    const etaEl = document.getElementById("terminal-eta");
    if (etaEl && stats.estimated_remaining_seconds !== undefined) {
      etaEl.textContent = this.formatDuration(
        stats.estimated_remaining_seconds,
      );
    }
  },

  updateSection(sectionId, status, progress) {
    const sectionEl = document.querySelector(
      `[data-section-id="${sectionId}"]`,
    );
    if (!sectionEl) return;

    const indicator = sectionEl.querySelector(".section-indicator");
    const statusBadge = sectionEl.querySelector(".section-status-badge");
    const stepBadge = sectionEl.querySelector(".section-step-badge");

    if (indicator) {
      indicator.className = `section-indicator ${status.toLowerCase()}`;
    }

    if (statusBadge) {
      statusBadge.className = `section-status-badge ${status.toLowerCase()}`;
      statusBadge.textContent = status;
    }

    if (stepBadge && progress) {
      stepBadge.textContent = `Step ${progress.current}/${progress.total}`;
    }

    if (status === "Running" || status === "Completed") {
      sectionEl.classList.add("expanded");
    }
  },

  updateItem(sectionId, itemId, status, duration) {
    const itemEl = document.querySelector(`[data-item-id="${itemId}"]`);
    if (!itemEl) return;

    const dot = itemEl.querySelector(".item-dot");
    const check = itemEl.querySelector(".item-check");
    const durationEl = itemEl.querySelector(".item-duration");

    const statusClass = status.toLowerCase();

    if (dot) {
      dot.className = `item-dot ${statusClass}`;
    }

    if (check) {
      check.className = `item-check ${statusClass}`;
      check.textContent =
        status === "Completed" ? "✓" : status === "Running" ? "◎" : "○";
    }

    if (durationEl && duration) {
      durationEl.textContent = `Duration: ${this.formatDuration(duration)}`;
    }
  },

  toggleSection(sectionId) {
    const sectionEl = document.querySelector(
      `[data-section-id="${sectionId}"]`,
    );
    if (sectionEl) {
      sectionEl.classList.toggle("expanded");
    }
  },

  toggleChild(childId) {
    const childEl = document.querySelector(`[data-child-id="${childId}"]`);
    if (childEl) {
      childEl.classList.toggle("expanded");
    }
  },

  viewDetails(sectionId) {
    console.log("View details for section:", sectionId);
  },

  viewChildDetails(childId) {
    console.log("View details for child:", childId);
  },

  escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  },

  loadManifest(taskId) {
    fetch(`/api/autotask/${taskId}/manifest`)
      .then((response) => response.json())
      .then((data) => {
        if (data.success && data.manifest) {
          this.manifest = data.manifest;
          this.render();
        }
      })
      .catch((error) => {
        console.error("Failed to load manifest:", error);
      });
  },

  destroy() {
    this.stopRuntimeCounter();
    // Unregister from singleton instead of closing our own connection
    if (
      this._boundHandler &&
      typeof unregisterTaskProgressHandler === "function"
    ) {
      unregisterTaskProgressHandler(this._boundHandler);
      this._boundHandler = null;
      console.log("[ProgressPanel] Unregistered from singleton WebSocket");
    }
    // Don't close the singleton connection - other components may be using it
    this.wsConnection = null;
  },
};

function toggleLogSection(header) {
  const section = header.closest(".log-section");
  if (section) {
    section.classList.toggle("expanded");
  }
}

function toggleLogChild(header) {
  const child = header.closest(".log-child");
  if (child) {
    child.classList.toggle("expanded");
  }
}

function viewSectionDetails(sectionId) {
  ProgressPanel.viewDetails(sectionId);
}

function viewChildDetails(childId) {
  ProgressPanel.viewChildDetails(childId);
}

window.ProgressPanel = ProgressPanel;
