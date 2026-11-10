/* Admin modals — onboarding and accounts list */
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
  var showModal = A.showModal;
  var showNotification = A.showNotification;

// ONBOARDING FUNCTIONS
// =============================================================================

let currentStep = 1;

function nextStep(step) {
  const currentPanel = document.querySelector(
    `.onboarding-panel[data-step="${step}"]`,
  );
  const nextPanel = document.querySelector(
    `.onboarding-panel[data-step="${step + 1}"]`,
  );

  if (currentPanel) currentPanel.classList.remove("active");
  if (nextPanel) nextPanel.classList.add("active");

  currentStep = step + 1;
  updateStepIndicators();
}

function prevStep(step) {
  const currentPanel = document.querySelector(
    `.onboarding-panel[data-step="${step}"]`,
  );
  const prevPanel = document.querySelector(
    `.onboarding-panel[data-step="${step - 1}"]`,
  );

  if (currentPanel) currentPanel.classList.remove("active");
  if (prevPanel) prevPanel.classList.add("active");

  currentStep = step - 1;
  updateStepIndicators();
}

function updateStepIndicators() {
  document.querySelectorAll(".step-indicator").forEach((indicator, index) => {
    const stepNum = index + 1;
    indicator.classList.remove("active", "completed");
    if (stepNum < currentStep) {
      indicator.classList.add("completed");
    } else if (stepNum === currentStep) {
      indicator.classList.add("active");
    }
  });
}

function skipPayment() {
  showNotification(t("admin-notify-skipping-payment-setup-free-plan-selected", "Skipping payment setup - Free plan selected"), "info");
  nextStep(4);
}

function removeLogo() {
  const logoPreview = document.getElementById("logo-preview");
  const logoInput = document.getElementById("logo-input");

  if (logoPreview) {
    logoPreview.innerHTML =
      '<span class="placeholder-text">No logo uploaded</span>';
  }
  if (logoInput) {
    logoInput.value = "";
  }
  showNotification(t("admin-notify-logo-removed", "Logo removed"), "success");
}

function previewLogo(input) {
  if (input.files && input.files[0]) {
    const reader = new FileReader();
    reader.onload = function (e) {
      const preview = document.getElementById("logo-preview");
      if (preview) {
        preview.innerHTML = `<img src="${e.target.result}" alt="Logo preview" style="max-width: 128px; max-height: 128px;">`;
      }
    };
    reader.readAsDataURL(input.files[0]);
  }
}

// =============================================================================
// ACCOUNTS LIST FUNCTIONS (sources/accounts-list.html)
// =============================================================================

function editAccount(accountId) {
  showNotification(`Loading account ${accountId}...`, "info");

  fetch(`/api/sources/accounts/${accountId}`)
    .then((response) => response.json())
    .then((data) => {
      showModal("edit-account-modal");
      // Populate form fields
      const form = document.getElementById("edit-account-form");
      if (form) {
        form
          .querySelector('[name="account-name"]')
          ?.setAttribute("value", data.name || "");
        form
          .querySelector('[name="account-email"]')
          ?.setAttribute("value", data.email || "");
      }
    })
    .catch((err) =>
      showNotification(t("admin-notify-failed-to-load-account-reason", "Failed to load account: {reason}", { reason: err.message }), "error"),
    );
}

function syncAccount(accountId) {
  showNotification(`Syncing account ${accountId}...`, "info");

  fetch(`/api/sources/accounts/${accountId}/sync`, { method: "POST" })
    .then((response) => response.json())
    .then((data) => {
      if (data.success) {
        showNotification(t("admin-notify-account-sync-started", "Account sync started"), "success");
      } else {
        showNotification(t("admin-notify-sync-failed-reason", "Sync failed: {reason}", { reason: (data.error || "Unknown error") }), "error",
        );
      }
    })
    .catch((err) => showNotification(t("admin-notify-sync-error-reason", "Sync error: {reason}", { reason: err.message }), "error"));
}

// =============================================================================
// =============================================================================

  A.nextStep = nextStep;
  A.prevStep = prevStep;
  A.updateStepIndicators = updateStepIndicators;
  A.skipPayment = skipPayment;
  A.removeLogo = removeLogo;
  A.previewLogo = previewLogo;
  A.editAccount = editAccount;
  A.syncAccount = syncAccount;
})(window);
