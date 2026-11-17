
function showToast(message, type = "info", duration = 4000, onClick = null) {
  let container = document.getElementById("toast-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "toast-container";
    container.style.cssText = `
            position: fixed;
            bottom: 24px;
            right: 24px;
            z-index: 10000;
            display: flex;
            flex-direction: column;
            gap: 8px;
        `;
    document.body.appendChild(container);
  }

  const toast = document.createElement("div");
  const bgColors = {
    success: "rgba(34, 197, 94, 0.95)",
    error: "rgba(239, 68, 68, 0.95)",
    warning: "rgba(245, 158, 11, 0.95)",
    info: "rgba(59, 130, 246, 0.95)",
  };

  const icons = {
    success: "✓",
    error: "✕",
    warning: "⚠",
    info: "ℹ",
  };

  const isAlert = type === "error" || type === "warning";
  toast.setAttribute("role", isAlert ? "alert" : "status");
  toast.setAttribute("aria-live", isAlert ? "assertive" : "polite");
  toast.setAttribute("aria-atomic", "true");

  toast.style.cssText = `
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 16px;
        background: ${bgColors[type] || bgColors.info};
        border-radius: 10px;
        color: white;
        font-size: 14px;
        font-weight: 500;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        animation: slideIn 0.3s ease;
    `;

  toast.innerHTML = `
        <span style="font-size: 16px;" aria-hidden="true">${icons[type] || icons.info}</span>
        <span>${message}</span>
    `;

  if (onClick) {
    toast.style.cursor = "pointer";
    toast.addEventListener("click", onClick);
  }

  container.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = "fadeOut 0.3s ease forwards";
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

function gbConfirm(message, title) {
  const t = title || "Confirm";
  return new Promise((resolve) => {
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
      try {
        window.GBModal.confirm(String(message), t, () => resolve(true), () => resolve(false));
        return;
      } catch (e) {
        /* fall through */
      }
    }
    const overlay = document.createElement("div");
    overlay.style.cssText = "position:fixed;inset:0;z-index:10050;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.45)";
    overlay.innerHTML = '<div style="background:#1e293b;color:#f8fafc;border:1px solid #334155;border-radius:10px;padding:16px;min-width:280px;max-width:420px;font-size:14px;box-shadow:0 8px 24px rgba(0,0,0,.4)">' +
      '<div style="font-weight:600;margin-bottom:8px">' + String(t).replace(/</g, "&lt;") + '</div>' +
      '<div style="margin-bottom:14px;white-space:pre-wrap">' + String(message).replace(/</g, "&lt;") + '</div>' +
      '<div style="display:flex;gap:8px;justify-content:flex-end">' +
      '<button data-gb-cancel style="padding:6px 12px;border-radius:6px;border:1px solid #475569;background:#334155;color:#f8fafc;cursor:pointer">Cancel</button>' +
      '<button data-gb-ok style="padding:6px 12px;border-radius:6px;border:0;background:#3b82f6;color:#fff;cursor:pointer">OK</button>' +
      '</div></div>';
    const done = (v) => { overlay.remove(); resolve(v); };
    overlay.querySelector("[data-gb-ok]").addEventListener("click", () => done(true));
    overlay.querySelector("[data-gb-cancel]").addEventListener("click", () => done(false));
    overlay.addEventListener("click", (e) => { if (e.target === overlay) done(false); });
    document.body.appendChild(overlay);
  });
}

function gbAlert(message, title) {
  const t = title || "Notice";
  if (window.GBModal && typeof window.GBModal.alert === "function") {
    try { window.GBModal.alert(String(message), t); return; } catch (e) { /* fallback */ }
  }
  if (window.GBToasts && typeof window.GBToasts.show === "function") {
    window.GBToasts.show(t, String(message), "info");
    return;
  }
  if (typeof showToast === "function") showToast(String(message), "info");
}

function gbPrompt(message, defaultValue, title) {
  const t = title || "Input";
  const d = defaultValue == null ? "" : String(defaultValue);
  return new Promise((resolve) => {
    if (window.GBModal && typeof window.GBModal.prompt === "function") {
      try {
        window.GBModal.prompt(String(message), d, t, (v) => resolve(v == null ? null : String(v)));
        return;
      } catch (e) { /* fallback */ }
    }
    const overlay = document.createElement("div");
    overlay.style.cssText = "position:fixed;inset:0;z-index:10050;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.45)";
    overlay.innerHTML = '<div style="background:#1e293b;color:#f8fafc;border:1px solid #334155;border-radius:10px;padding:16px;min-width:300px;max-width:420px;font-size:14px;box-shadow:0 8px 24px rgba(0,0,0,.4)">' +
      '<div style="font-weight:600;margin-bottom:8px">' + String(t).replace(/</g, "&lt;") + '</div>' +
      '<div style="margin-bottom:8px;white-space:pre-wrap">' + String(message).replace(/</g, "&lt;") + '</div>' +
      '<input data-gb-input style="width:100%;box-sizing:border-box;margin-bottom:14px;padding:8px;border-radius:6px;border:1px solid #475569;background:#0f172a;color:#f8fafc" />' +
      '<div style="display:flex;gap:8px;justify-content:flex-end">' +
      '<button data-gb-cancel style="padding:6px 12px;border-radius:6px;border:1px solid #475569;background:#334155;color:#f8fafc;cursor:pointer">Cancel</button>' +
      '<button data-gb-ok style="padding:6px 12px;border-radius:6px;border:0;background:#3b82f6;color:#fff;cursor:pointer">OK</button>' +
      '</div></div>';
    const input = overlay.querySelector("[data-gb-input]");
    input.value = d;
    const done = (v) => { overlay.remove(); resolve(v); };
    overlay.querySelector("[data-gb-ok]").addEventListener("click", () => done(String(input.value)));
    overlay.querySelector("[data-gb-cancel]").addEventListener("click", () => done(null));
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") done(String(input.value)); if (e.key === "Escape") done(null); });
    overlay.addEventListener("click", (e) => { if (e.target === overlay) done(null); });
    document.body.appendChild(overlay);
    setTimeout(() => input.focus(), 0);
  });
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

function tStatus(key, fallback) {
  return window.gbT ? window.gbT(key, fallback) : fallback;
}

function formatStatus(status) {
  const statusMap = {
    complete: tStatus("tasks-status-complete", "Complete"),
    running: tStatus("tasks-status-running", "Running"),
    awaiting: tStatus("tasks-status-awaiting", "Awaiting"),
    paused: tStatus("tasks-status-paused", "Paused"),
    blocked: tStatus("tasks-status-blocked", "Blocked"),
  };
  return statusMap[status] || status;
}

function formatTime(seconds) {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    return `${mins}m`;
  }
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
}

// =============================================================================
// GLOBAL STYLES FOR TOAST ANIMATIONS
// =============================================================================

if (typeof taskStyleElement === "undefined") {
  var taskStyleElement = document.createElement("style");
  taskStyleElement.textContent = `
    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateX(20px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    @keyframes fadeOut {
        from {
            opacity: 1;
            transform: translateX(0);
        }
        to {
            opacity: 0;
            transform: translateX(20px);
        }
    }

    @keyframes slideInRight {
        from {
            opacity: 0;
            transform: translateX(100px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    @keyframes slideOutRight {
        from {
            opacity: 1;
            transform: translateX(0);
        }
        to {
            opacity: 0;
            transform: translateX(100px);
        }
    }
`;
  document.head.appendChild(taskStyleElement);
}

// =============================================================================
// GOALS, PENDING INFO, SCHEDULERS, MONITORS
// =============================================================================

function getTasksRoot() {
  return document.querySelector(".tasks-app") || document.body || document;
}

function ensureTaskDetailVisible() {
  const empty = document.getElementById("detail-empty");
  const content = document.getElementById("task-detail-content");
  if (empty) empty.style.display = "none";
  if (content) content.style.display = "block";
  return content;
}

function escapeTasksText(value) {
  if (typeof escapeHtml === "function") return escapeHtml(String(value == null ? "" : value));
  return String(value == null ? "" : value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderTaskDetailFallback(title, rows) {
  const content = ensureTaskDetailVisible();
  if (!content) return;
  const body = (rows || [])
    .filter(function (row) { return row && row[1] !== undefined && row[1] !== null && row[1] !== ""; })
    .map(function (row) {
      return '<div class="detail-row"><strong>' + escapeTasksText(row[0]) + ':</strong> <span>' + escapeTasksText(row[1]) + '</span></div>';
    })
    .join("");
  content.innerHTML =
    '<div class="task-detail-fallback">' +
    '<h3 class="detail-title">' + escapeTasksText(title || "Details") + '</h3>' +
    (body || '<p class="detail-empty-note">No additional details available.</p>') +
    '</div>';
}

// Select a goal and show its details
