/* Admin modals — modal helpers and accounts */
(function (window) {
  "use strict";
  // #2093 — translated admin message with an authored English fallback.
  function t(key, fallback, params) {
    var value = window.gbT ? window.gbT(key, fallback, params) : fallback;
    if (value !== fallback || !params) return value;
    return Object.keys(params).reduce(function (text, name) {
      return text.replace(new RegExp("\\{\\s*" + name + "\\s*\\}", "g"), params[name]);
    }, fallback);
  }
  var A = (window.__GBAdminModals = window.__GBAdminModals || {});


// =============================================================================
// MODAL HELPERS
// =============================================================================

function showModal(modalId) {
  const modal = document.getElementById(modalId);
  if (modal) {
    if (modal.showModal) {
      modal.showModal();
    } else {
      modal.classList.add("open");
      modal.style.display = "flex";
    }
  }
}

function hideModal(modalId) {
  const modal = document.getElementById(modalId);
  if (modal) {
    if (modal.close) {
      modal.close();
    } else {
      modal.classList.remove("open");
      modal.style.display = "none";
    }
  }
}

function showNotification(message, type) {
  if (typeof window.showNotification === "function") {
    window.showNotification(message, type);
  } else if (typeof window.GBAlerts !== "undefined") {
    if (type === "success") window.GBAlerts.success("Admin", message);
    else if (type === "error") window.GBAlerts.error("Admin", message);
    else if (type === "warning") window.GBAlerts.warning("Admin", message);
    else window.GBAlerts.info("Admin", message);
  } else {
    console.log(`[${type}] ${message}`);
  }
}

// =============================================================================
// ACCOUNTS.HTML FUNCTIONS
// =============================================================================

function showSmtpModal() {
  showModal("smtp-modal");
}

function closeSmtpModal() {
  hideModal("smtp-modal");
}

function testSmtpConnection() {
  const host = document.getElementById("smtp-host")?.value;
  const port = document.getElementById("smtp-port")?.value;
  const username = document.getElementById("smtp-username")?.value;

  if (!host || !port) {
    showNotification(t("admin-notify-please-fill-in-smtp-host-and-port", "Please fill in SMTP host and port"), "error");
    return;
  }

  showNotification(t("admin-notify-testing-smtp-connection", "Testing SMTP connection..."), "info");

  fetch("/api/settings/smtp/test", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ host, port: parseInt(port), username }),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-smtp-connection-successful", "SMTP connection successful!"), "success");
      } else {
        showNotification(t("admin-notify-smtp-connection-failed-reason", "SMTP connection failed: {reason}", { reason: (data.error || "Unknown error") }), "error",
        );
      }
    })
    .catch((err) => {
      showNotification(t("admin-notify-connection-test-failed-reason", "Connection test failed: {reason}", { reason: err.message }), "error");
    });
}

function connectAccount(provider) {
  showNotification(`Connecting to ${provider}...`, "info");
  // OAuth flow would redirect to provider
  window.location.href = `/api/auth/oauth/${provider}?redirect=/admin/accounts`;
}

function disconnectAccount(provider) {
  var msg = `Disconnect ${provider} account?`;
  var doDisconnect = function () {
    fetch(`/api/settings/accounts/${provider}/disconnect`, { method: "POST" })
      .then((response) => response.json())
      .then((data) => {
        if (data.success) {
          showNotification(`${provider} disconnected`, "success");
          location.reload();
        } else {
          showNotification(t("admin-notify-failed-to-disconnect-reason", "Failed to disconnect: {reason}", { reason: data.error }), "error");
        }
      })
      .catch((err) => showNotification(t("admin-notify-error-reason", "Error: {reason}", { reason: err.message }), "error"));
  };

  if (window.GBModal && typeof window.GBModal.confirm === "function") {
    window.GBModal.confirm(msg, "Confirm", doDisconnect);
    return;
  }
  showNotification("Confirmation dialog unavailable", "error");
}

// =============================================================================

  A.showModal = showModal;
  A.hideModal = hideModal;
  A.showNotification = showNotification;
  A.showSmtpModal = showSmtpModal;
  A.closeSmtpModal = closeSmtpModal;
  A.testSmtpConnection = testSmtpConnection;
  A.connectAccount = connectAccount;
  A.disconnectAccount = disconnectAccount;
})(window);
