Object.assign(ProgressPanel, {
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
    const sectionEl = document.getElementById("window-tasks").querySelector(
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
    const itemEl = document.getElementById("window-tasks").querySelector(`[data-item-id="${itemId}"]`);
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
    const sectionEl = document.getElementById("window-tasks").querySelector(
      `[data-section-id="${sectionId}"]`,
    );
    if (sectionEl) {
      sectionEl.classList.toggle("expanded");
    }
  },

  toggleChild(childId) {
    const childEl = document.getElementById("window-tasks").querySelector(`[data-child-id="${childId}"]`);
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
});
