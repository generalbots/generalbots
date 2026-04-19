/* Logs page JavaScript */

// Logs State - guard against duplicate declarations on HTMX reload
if (typeof window.logsModuleInitialized === "undefined") {
  window.logsModuleInitialized = true;
  var isStreaming = true;
  var autoScroll = true;
  var logCounts = { debug: 0, info: 0, warn: 0, error: 0, fatal: 0 };
  var searchDebounceTimer = null;
  var currentFilters = {
    level: "all",
    service: "all",
    search: "",
  };
  var logsWs = null;
}

function initLogsWebSocket() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  logsWs = new WebSocket(`${protocol}//${window.location.host}/ws/logs`);

  logsWs.onopen = function () {
    updateLogsConnectionStatus("connected", "Connected");
  };

  logsWs.onclose = function () {
    updateLogsConnectionStatus("disconnected", "Disconnected");
    // Reconnect after 3 seconds
    setTimeout(initLogsWebSocket, 3000);
  };

  logsWs.onerror = function () {
    updateLogsConnectionStatus("disconnected", "Error");
  };

  logsWs.onmessage = function (event) {
    if (!isStreaming) return;

    try {
      const logData = JSON.parse(event.data);
      appendLog(logData);
    } catch (e) {
      console.error("Failed to parse log message:", e);
    }
  };
}

function updateLogsConnectionStatus(status, text) {
  const statusEl = document.getElementById("connection-status");
  if (statusEl) {
    statusEl.className = `connection-status ${status}`;
    statusEl.querySelector(".status-text").textContent = text;
  }
}

function appendLog(log) {
  const stream = document.getElementById("log-stream");
  if (!stream) return;

  const placeholder = stream.querySelector(".log-placeholder");
  if (placeholder) {
    placeholder.remove();
  }

  const entry = document.createElement("div");
  entry.className = "log-entry";
  entry.dataset.level = log.level || "info";
  entry.dataset.service = log.service || "unknown";
  entry.dataset.id = log.id || Date.now();

  entry.innerHTML = `
        <span class="log-timestamp">${formatLogTimestamp(log.timestamp)}</span>
        <span class="log-level">${(log.level || "INFO").toUpperCase()}</span>
        <span class="log-service">${log.service || "unknown"}</span>
        <span class="log-message">${escapeLogHtml(log.message || "")}</span>
        <button class="log-expand" onclick="expandLog(this)" title="View details">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
        </button>
    `;

  // Store full log data for detail view
  entry._logData = log;

  // Apply current filters
  if (!matchesLogFilters(entry)) {
    entry.classList.add("hidden");
  }

  stream.appendChild(entry);

  // Update counts
  const level = log.level || "info";
  if (logCounts[level] !== undefined) {
    logCounts[level]++;
    const countEl = document.getElementById(`${level}-count`);
    if (countEl) countEl.textContent = logCounts[level];
  }
  const totalEl = document.getElementById("total-count");
  if (totalEl) {
    totalEl.textContent = Object.values(logCounts).reduce((a, b) => a + b, 0);
  }

  // Auto-scroll to bottom
  if (autoScroll) {
    stream.scrollTop = stream.scrollHeight;
  }

  // Limit log entries to prevent memory issues
  const maxEntries = 1000;
  while (stream.children.length > maxEntries) {
    const removed = stream.firstChild;
    if (removed._logData) {
      const removedLevel = removed._logData.level || "info";
      if (logCounts[removedLevel] > 0) {
        logCounts[removedLevel]--;
      }
    }
    stream.removeChild(removed);
  }
}

function formatLogTimestamp(timestamp) {
  if (!timestamp) return "--";
  const date = new Date(timestamp);
  return date.toISOString().replace("T", " ").slice(0, 23);
}

function escapeLogHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function matchesLogFilters(entry) {
  // Level filter
  if (
    currentFilters.level !== "all" &&
    entry.dataset.level !== currentFilters.level
  ) {
    return false;
  }

  // Service filter
  if (
    currentFilters.service !== "all" &&
    entry.dataset.service !== currentFilters.service
  ) {
    return false;
  }

  // Search filter
  if (currentFilters.search) {
    const text = entry.textContent.toLowerCase();
    if (!text.includes(currentFilters.search.toLowerCase())) {
      return false;
    }
  }

  return true;
}

function applyLogFilters() {
  currentFilters.level =
    document.getElementById("log-level-filter")?.value || "all";
  currentFilters.service =
    document.getElementById("service-filter")?.value || "all";

  const entries = document.querySelectorAll(".log-entry");
  entries.forEach((entry) => {
    if (matchesLogFilters(entry)) {
      entry.classList.remove("hidden");
    } else {
      entry.classList.add("hidden");
    }
  });
}

function debounceLogSearch(value) {
  clearTimeout(searchDebounceTimer);
  searchDebounceTimer = setTimeout(() => {
    currentFilters.search = value;
    applyLogFilters();
  }, 300);
}

function toggleStream() {
  isStreaming = !isStreaming;
  const btn = document.getElementById("stream-toggle");
  if (!btn) return;

  if (isStreaming) {
    btn.classList.remove("paused");
    btn.innerHTML = `
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="6" y="4" width="4" height="16"></rect>
                <rect x="14" y="4" width="4" height="16"></rect>
            </svg>
            <span>Pause</span>
        `;
  } else {
    btn.classList.add("paused");
    btn.innerHTML = `
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
            <span>Resume</span>
        `;
  }
}

function clearLogs() {
  if (confirm("Are you sure you want to clear all logs?")) {
    const stream = document.getElementById("log-stream");
    if (!stream) return;

    stream.innerHTML = `
            <div class="log-placeholder">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                    <polyline points="14 2 14 8 20 8"></polyline>
                    <line x1="16" y1="13" x2="8" y2="13"></line>
                    <line x1="16" y1="17" x2="8" y2="17"></line>
                    <polyline points="10 9 9 9 8 9"></polyline>
                </svg>
                <p>Logs cleared</p>
                <span class="placeholder-hint">New logs will appear here</span>
            </div>
        `;

    // Reset counts
    logCounts = { debug: 0, info: 0, warn: 0, error: 0, fatal: 0 };
    Object.keys(logCounts).forEach((level) => {
      const el = document.getElementById(`${level}-count`);
      if (el) el.textContent = "0";
    });
    const totalEl = document.getElementById("total-count");
    if (totalEl) totalEl.textContent = "0";
  }
}

function downloadLogs() {
  const entries = document.querySelectorAll(".log-entry");
  let logs = [];

  entries.forEach((entry) => {
    if (entry._logData) {
      logs.push(entry._logData);
    }
  });

  const blob = new Blob([JSON.stringify(logs, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `logs-${new Date().toISOString().slice(0, 19).replace(/[T:]/g, "-")}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function scrollToTop() {
  const stream = document.getElementById("log-stream");
  if (stream) {
    stream.scrollTop = 0;
    autoScroll = false;
    updateLogScrollButtons();
  }
}

function scrollToBottom() {
  const stream = document.getElementById("log-stream");
  if (stream) {
    stream.scrollTop = stream.scrollHeight;
    autoScroll = true;
    updateLogScrollButtons();
  }
}

function updateLogScrollButtons() {
  const topBtn = document.getElementById("scroll-top-btn");
  const bottomBtn = document.getElementById("scroll-bottom-btn");
  if (topBtn) topBtn.classList.toggle("active", !autoScroll);
  if (bottomBtn) bottomBtn.classList.toggle("active", autoScroll);
}

function expandLog(btn) {
  const entry = btn.closest(".log-entry");
  const logData = entry._logData || {
    timestamp: entry.querySelector(".log-timestamp").textContent,
    level: entry.dataset.level,
    service: entry.dataset.service,
    message: entry.querySelector(".log-message").textContent,
  };

  const panel = document.getElementById("log-detail-panel");
  const content = document.getElementById("log-detail-content");
  if (!panel || !content) return;

  content.innerHTML = `
        <div class="detail-section">
            <div class="detail-label">Timestamp</div>
            <div class="detail-value">${logData.timestamp || "--"}</div>
        </div>
        <div class="detail-section">
            <div class="detail-label">Level</div>
            <div class="detail-value">${(logData.level || "info").toUpperCase()}</div>
        </div>
        <div class="detail-section">
            <div class="detail-label">Service</div>
            <div class="detail-value">${logData.service || "unknown"}</div>
        </div>
        <div class="detail-section">
            <div class="detail-label">Message</div>
            <div class="detail-value">${escapeLogHtml(logData.message || "")}</div>
        </div>
        ${
          logData.stack
            ? `
        <div class="detail-section">
            <div class="detail-label">Stack Trace</div>
            <div class="detail-value">${escapeLogHtml(logData.stack)}</div>
        </div>
        `
            : ""
        }
        ${
          logData.context
            ? `
        <div class="detail-section">
            <div class="detail-label">Context</div>
            <div class="detail-value">${escapeLogHtml(JSON.stringify(logData.context, null, 2))}</div>
        </div>
        `
            : ""
        }
    `;

  panel.classList.add("open");
}

function closeLogDetail() {
  const panel = document.getElementById("log-detail-panel");
  if (panel) panel.classList.remove("open");
}

// Initialize on page load
document.addEventListener("DOMContentLoaded", function () {
  // Initialize WebSocket connection if on logs page
  if (document.getElementById("log-stream")) {
    initLogsWebSocket();
  }
});
