function showKeyboardShortcuts() {
  window.showNotification(
    "Alt+1-9,0 for apps, Alt+A Admin, Alt+M Monitoring, Alt+S Settings, Alt+, quick settings",
    "info",
    8000,
  );
}

function announceToScreenReader(message) {
  const liveRegion = document.getElementById("aria-live");
  if (liveRegion) {
    liveRegion.textContent = message;
    setTimeout(() => {
      liveRegion.textContent = "";
    }, 1000);
  }
}

document.body.addEventListener("htmx:beforeRequest", function (e) {
  const target = e.detail.target;
  if (target && target.id === "main-content") {
    target.setAttribute("aria-busy", "true");
    announceToScreenReader("Loading content...");
  }
});

document.body.addEventListener("htmx:afterSwap", function (e) {
  const target = e.detail.target;
  if (target && target.id === "main-content") {
    target.setAttribute("aria-busy", "false");
    target.focus();
    announceToScreenReader("Content loaded");
  }
});

document.body.addEventListener("htmx:responseError", function (e) {
  const target = e.detail.target;
  if (target) {
    target.setAttribute("aria-busy", "false");
  }
  announceToScreenReader("Error loading content. Please try again.");
});

document.addEventListener("keydown", function (e) {
  const appsGrid = document.querySelector(".apps-grid");
  if (!appsGrid || !appsGrid.closest(".show")) return;

  const items = Array.from(appsGrid.querySelectorAll(".app-item"));
  const currentIndex = items.findIndex(
    (item) => item === document.activeElement,
  );

  if (currentIndex === -1) return;

  let newIndex = currentIndex;
  const columns = 3;

  switch (e.key) {
    case "ArrowRight":
      newIndex = Math.min(currentIndex + 1, items.length - 1);
      break;
    case "ArrowLeft":
      newIndex = Math.max(currentIndex - 1, 0);
      break;
    case "ArrowDown":
      newIndex = Math.min(currentIndex + columns, items.length - 1);
      break;
    case "ArrowUp":
      newIndex = Math.max(currentIndex - columns, 0);
      break;
    case "Home":
      newIndex = 0;
      break;
    case "End":
      newIndex = items.length - 1;
      break;
    default:
      return;
  }

  if (newIndex !== currentIndex) {
    e.preventDefault();
    items[newIndex].focus();
  }
});

window.showNotification = function (message, type = "info", duration = 5000) {
  const container = document.getElementById("notifications");
  if (!container) return;

  const notification = document.createElement("div");
  notification.className = `notification ${type}`;
  notification.innerHTML = `
        <div class="notification-content">
            <div class="notification-message">${message}</div>
        </div>
        <button class="notification-close" onclick="this.parentElement.remove()" style="background:none;border:none;color:var(--text-secondary);cursor:pointer;font-size:1.25rem;">×</button>
    `;
  container.appendChild(notification);
  if (duration > 0) {
    setTimeout(() => notification.remove(), duration);
  }
};

const htmxRetryConfig = {
  maxRetries: 3,
  retryDelay: 1000,
  retryCount: new Map(),
};

function getRetryKey(elt) {
  return (
    elt.getAttribute("hx-get") ||
    elt.getAttribute("hx-post") ||
    elt.getAttribute("hx-put") ||
    elt.getAttribute("hx-delete") ||
    elt.id ||
    Math.random().toString(36)
  );
}

function showErrorState(target, errorMessage, retryCallback) {
  const errorHtml = `
        <div class="error-state">
            <svg class="error-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <div class="error-state-title">Something went wrong</div>
            <div class="error-state-message">${errorMessage}</div>
            <div class="error-state-actions">
                <button class="btn-retry" onclick="window.retryLastRequest(this)">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M23 4v6h-6"></path>
                        <path d="M1 20v-6h6"></path>
                        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"></path>
                        <path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"></path>
                    </svg>
                    Try Again
                </button>
            </div>
        </div>
    `;
  target.innerHTML = errorHtml;
  target.dataset.retryCallback = retryCallback;
}

window.retryLastRequest = function (btn) {
  const target = btn.closest(".error-state").parentElement;
  const retryCallback = target.dataset.retryCallback;

  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Retrying...';

  if (retryCallback && window[retryCallback]) {
    window[retryCallback]();
  } else {
    const triggers = target.querySelectorAll("[hx-get], [hx-post]");
    if (triggers.length > 0) {
      htmx.trigger(triggers[0], "htmx:trigger");
    } else {
      const activeApp = document.querySelector(".app-item.active");
      if (activeApp) {
        activeApp.click();
      }
    }
  }
};

document.body.addEventListener("htmx:responseError", function (e) {
  const target = e.detail.target;
  const xhr = e.detail.xhr;
  const retryKey = getRetryKey(e.detail.elt);

  let currentRetries = htmxRetryConfig.retryCount.get(retryKey) || 0;

  if (
    (xhr.status === 0 || xhr.status >= 500) &&
    currentRetries < htmxRetryConfig.maxRetries
  ) {
    htmxRetryConfig.retryCount.set(retryKey, currentRetries + 1);
    const delay = htmxRetryConfig.retryDelay * Math.pow(2, currentRetries);

    window.showNotification(
      `Request failed. Retrying in ${delay / 1000}s... (${currentRetries + 1}/${htmxRetryConfig.maxRetries})`,
      "warning",
      delay,
    );

    setTimeout(() => {
      htmx.trigger(e.detail.elt, "htmx:trigger");
    }, delay);
  } else {
    htmxRetryConfig.retryCount.delete(retryKey);

    let errorMessage = "We couldn't load the content.";
    if (xhr.status === 401) {
      errorMessage = "Your session has expired. Please log in again.";
    } else if (xhr.status === 403) {
      errorMessage = "You don't have permission to access this resource.";
    } else if (xhr.status === 404) {
      errorMessage = "The requested content was not found.";
    } else if (xhr.status >= 500) {
      errorMessage =
        "The server is experiencing issues. Please try again later.";
    } else if (xhr.status === 0) {
      errorMessage =
        "Unable to connect. Please check your internet connection.";
    }

    if (target && target.id === "main-content") {
      showErrorState(target, errorMessage);
    } else {
      window.showNotification(errorMessage, "error", 8000);
    }
  }
});

document.body.addEventListener("htmx:afterRequest", function (e) {
  if (e.detail.successful) {
    const retryKey = getRetryKey(e.detail.elt);
    htmxRetryConfig.retryCount.delete(retryKey);
  }
});

document.body.addEventListener("htmx:timeout", function (e) {
  window.showNotification(
    "Request timed out. Please try again.",
    "warning",
    5000,
  );
});

document.body.addEventListener("htmx:sendError", function (e) {
  window.showNotification(
    "Network error. Please check your connection.",
    "error",
    5000,
  );
});
