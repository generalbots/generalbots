(function () {
  "use strict";

  if (window.__GB_HTMX_ERRORS_INSTALLED__) return;
  window.__GB_HTMX_ERRORS_INSTALLED__ = true;

  var htmxRetryConfig = {
    maxRetries: 3,
    retryDelay: 1000,
    retryCount: new Map(),
  };
  var lastNotifyAt = 0;
  var lastNotifyKey = "";

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function announceToScreenReader(message) {
    var liveRegion = document.getElementById("aria-live");
    if (!liveRegion) return;
    liveRegion.textContent = String(message || "");
    setTimeout(function () {
      if (liveRegion.textContent === message) {
        liveRegion.textContent = "";
      }
    }, 1500);
  }

  function notify(message, type, duration) {
    var msg = String(message || "");
    var kind = type || "info";
    var ms = typeof duration === "number" ? duration : 6000;
    var dedupeKey = kind + "|" + msg;
    var now = Date.now();
    if (dedupeKey === lastNotifyKey && now - lastNotifyAt < 1500) return;
    lastNotifyKey = dedupeKey;
    lastNotifyAt = now;

    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show("Request issue", msg, kind);
      return;
    }
    if (window.GBAlerts && typeof window.GBAlerts.add === "function") {
      window.GBAlerts.add({ type: kind, title: "Request issue", message: msg });
      return;
    }
    if (typeof window.showNotification === "function" && window.showNotification !== notifyFallback) {
      try {
        window.showNotification(msg, kind, ms);
        return;
      } catch (err) {}
    }
    notifyFallback(msg, kind, ms);
  }

  function notifyFallback(message, type, duration) {
    var container = document.getElementById("notifications");
    if (!container) {
      container = document.createElement("div");
      container.id = "notifications";
      container.setAttribute("aria-live", "polite");
      container.style.position = "fixed";
      container.style.right = "16px";
      container.style.bottom = "16px";
      container.style.zIndex = "10000";
      document.body.appendChild(container);
    }
    var notification = document.createElement("div");
    notification.className = "notification " + String(type || "info");
    notification.innerHTML =
      '<div class="notification-content"><div class="notification-message">' +
      escapeHtml(message) +
      '</div></div><button class="notification-close" type="button" ' +
      'style="background:none;border:none;cursor:pointer;font-size:1.25rem;margin-left:8px;">×</button>';
    var closeBtn = notification.querySelector(".notification-close");
    if (closeBtn) closeBtn.addEventListener("click", function () { notification.remove(); });
    container.appendChild(notification);
    if (duration > 0) {
      setTimeout(function () { notification.remove(); }, duration);
    }
  }

  // Preserve a simple fallback API for legacy callers without raw HTML injection.
  window.showNotification = notifyFallback;

  function getRequestPath(detail) {
    try {
      if (detail && detail.pathInfo && detail.pathInfo.requestPath) {
        return detail.pathInfo.requestPath;
      }
      if (detail && detail.requestConfig && detail.requestConfig.path) {
        return detail.requestConfig.path;
      }
    } catch (err) {}
    return "";
  }

  function getVerb(detail) {
    try {
      if (detail && detail.requestConfig && detail.requestConfig.verb) {
        return String(detail.requestConfig.verb).toLowerCase();
      }
      var elt = detail && detail.elt;
      if (elt) {
        if (elt.getAttribute("hx-post")) return "post";
        if (elt.getAttribute("hx-put")) return "put";
        if (elt.getAttribute("hx-delete")) return "delete";
        if (elt.getAttribute("hx-patch")) return "patch";
        if (elt.getAttribute("hx-get")) return "get";
      }
    } catch (err) {}
    return "get";
  }

  function isIdempotent(verb) {
    return verb === "get" || verb === "head" || verb === "options" || verb === "trace";
  }

  function getRetryKey(detail) {
    var elt = detail && detail.elt;
    if (!elt) return "unknown";
    var attr =
      elt.getAttribute("hx-get") ||
      elt.getAttribute("hx-post") ||
      elt.getAttribute("hx-put") ||
      elt.getAttribute("hx-delete") ||
      elt.getAttribute("hx-patch");
    return (getVerb(detail) + ":" + (attr || elt.id || "elt"));
  }

  function clearBusy(detail) {
    var target = detail && detail.target;
    var elt = detail && detail.elt;
    if (target) {
      try {
        target.setAttribute("aria-busy", "false");
        target.classList.remove("htmx-request");
      } catch (err) {}
    }
    if (elt && elt.classList) {
      try { elt.classList.remove("htmx-request"); } catch (err) {}
    }
  }

  function safeMessageForStatus(status, kind) {
    if (kind === "timeout") return "Request timed out. Please try again.";
    if (kind === "network" || status === 0) {
      return "Unable to connect. Please check your internet connection.";
    }
    if (status === 401) return "Your session has expired. Please sign in again.";
    if (status === 403) return "You do not have permission to access this resource.";
    if (status === 404) return "The requested content was not found.";
    if (status === 408) return "Request timed out. Please try again.";
    if (status === 429) return "Too many requests. Please try again in a moment.";
    if (status >= 500) return "The server is experiencing issues. Please try again later.";
    if (status >= 400) return "We could not complete that request. Please try again.";
    return "Something went wrong. Please try again.";
  }

  function showErrorState(target, message, canRetry) {
    if (!target) return;
    var retryHtml = canRetry
      ? '<div class="error-state-actions"><button class="btn-retry" type="button" data-gb-htmx-retry="1">' +
        "Try again</button></div>"
      : "";
    target.innerHTML =
      '<div class="error-state" role="alert">' +
      '<div class="error-state-title">Something went wrong</div>' +
      '<div class="error-state-message">' + escapeHtml(message) + "</div>" +
      retryHtml +
      "</div>";
  }

  function triggerRetry(detail) {
    var elt = detail && detail.elt;
    if (!elt || typeof htmx === "undefined") return false;
    try {
      htmx.trigger(elt, "htmx:trigger");
      return true;
    } catch (err) {
      return false;
    }
  }

  function handleAuthFlow(detail, status) {
    if (status !== 401 && status !== 403) return false;
    var path = getRequestPath(detail);
    if (window.GBSecurity && typeof window.GBSecurity.handleUnauthorized === "function") {
      try { window.GBSecurity.handleUnauthorized(path); } catch (err) {}
    } else if (status === 401) {
      window.dispatchEvent(new CustomEvent("gb:auth:unauthorized", { detail: { url: path } }));
    }
    return true;
  }

  function handleFailure(detail, options) {
    var opts = options || {};
    var xhr = detail && detail.xhr;
    var status = xhr && typeof xhr.status === "number" ? xhr.status : 0;
    var target = detail && detail.target;
    var verb = getVerb(detail);
    var idempotent = isIdempotent(verb);
    var retryKey = getRetryKey(detail);
    var message = safeMessageForStatus(status, opts.kind);
    var isAuth = status === 401 || status === 403;

    clearBusy(detail);

    if (isAuth) {
      handleAuthFlow(detail, status);
      announceToScreenReader("Session authorization required.");
      return;
    }

    var canAutoRetry =
      idempotent && (opts.kind === "timeout" || opts.kind === "network" || status === 0 || status >= 500);
    var currentRetries = htmxRetryConfig.retryCount.get(retryKey) || 0;
    if (canAutoRetry && currentRetries < htmxRetryConfig.maxRetries) {
      htmxRetryConfig.retryCount.set(retryKey, currentRetries + 1);
      var delay = htmxRetryConfig.retryDelay * Math.pow(2, currentRetries);
      notify(
        "Request failed. Retrying in " + (delay / 1000) + "s... (" +
          (currentRetries + 1) + "/" + htmxRetryConfig.maxRetries + ")",
        "warning",
        delay
      );
      announceToScreenReader("Request failed. Retrying.");
      setTimeout(function () { triggerRetry(detail); }, delay);
      return;
    }

    htmxRetryConfig.retryCount.delete(retryKey);
    announceToScreenReader(message);

    if (target && target.id === "main-content") {
      showErrorState(target, message, idempotent);
      if (idempotent) {
        target.__gbHtmxRetryDetail = { elt: detail.elt, target: detail.target, requestConfig: detail.requestConfig };
      }
    } else {
      notify(message, opts.kind === "timeout" ? "warning" : "error", 8000);
    }
  }

  document.body.addEventListener("htmx:beforeRequest", function (e) {
    var target = e.detail && e.detail.target;
    if (target && target.id === "main-content") {
      try {
        target.setAttribute("aria-busy", "true");
      } catch (err) {}
      announceToScreenReader("Loading content...");
    }
  });

  document.body.addEventListener("htmx:afterSwap", function (e) {
    var target = e.detail && e.detail.target;
    if (target && target.id === "main-content") {
      try {
        target.setAttribute("aria-busy", "false");
        target.focus();
      } catch (err) {}
      announceToScreenReader("Content loaded");
    }
  });

  document.body.addEventListener("htmx:responseError", function (e) {
    var detail = e.detail || {};
    e.__gbHtmxErrorHandled = true;
    handleFailure(detail, { kind: "response" });
  });

  document.body.addEventListener("htmx:timeout", function (e) {
    var detail = e.detail || {};
    e.__gbHtmxErrorHandled = true;
    handleFailure(detail, { kind: "timeout" });
  });

  document.body.addEventListener("htmx:sendError", function (e) {
    var detail = e.detail || {};
    e.__gbHtmxErrorHandled = true;
    handleFailure(detail, { kind: "network" });
  });

  document.body.addEventListener("htmx:afterRequest", function (e) {
    var detail = e.detail || {};
    if (detail.successful) {
      htmxRetryConfig.retryCount.delete(getRetryKey(detail));
      clearBusy(detail);
    }
  });

  document.body.addEventListener("click", function (e) {
    var btn = e.target && e.target.closest ? e.target.closest("[data-gb-htmx-retry='1']") : null;
    if (!btn) return;
    var container = btn.closest(".error-state");
    var target = container && container.parentElement ? container.parentElement : null;
    if (!target) return;
    btn.disabled = true;
    btn.textContent = "Retrying...";
    var detail = target.__gbHtmxRetryDetail;
    if (detail && detail.elt && typeof htmx !== "undefined") {
      try {
        htmx.trigger(detail.elt, "htmx:trigger");
        return;
      } catch (err) {}
    }
    var triggers = target.querySelectorAll("[hx-get], [hx-head]");
    if (triggers.length > 0 && typeof htmx !== "undefined") {
      try { htmx.trigger(triggers[0], "htmx:trigger"); return; } catch (err) {}
    }
    var activeApp = document.querySelector(".app-item.active");
    if (activeApp) activeApp.click();
  });
})();
