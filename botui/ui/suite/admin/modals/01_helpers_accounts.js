/* Admin modals — modal helpers and accounts */
(function (window) {
  "use strict";
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
    showNotification("Please fill in SMTP host and port", "error");
    return;
  }

  showNotification("Testing SMTP connection...", "info");

  fetch("/api/settings/smtp/test", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ host, port: parseInt(port), username }),
  })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification("SMTP connection successful!", "success");
      } else {
        showNotification(
          "SMTP connection failed: " + (data.error || "Unknown error"),
          "error",
        );
      }
    })
    .catch((err) => {
      showNotification("Connection test failed: " + err.message, "error");
    });
}

function connectAccount(provider) {
  showNotification(`Connecting to ${provider}...`, "info");
  // OAuth flow would redirect to provider
  window.location.href = `/api/auth/oauth/${provider}?redirect=/admin/accounts`;
}

function disconnectAccount(provider) {
  if (!confirm(`Disconnect ${provider} account?`)) return;

  fetch(`/api/settings/accounts/${provider}/disconnect`, { method: "POST" })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(`${provider} disconnected`, "success");
        location.reload();
      } else {
        showNotification("Failed to disconnect: " + data.error, "error");
      }
    })
    .catch((err) => showNotification("Error: " + err.message, "error"));
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
