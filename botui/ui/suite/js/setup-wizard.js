/*
 * Setup Wizard - Issue #465
 *
 * Backend API Contract:
 *
 * GET /api/setup/status
 * Response: { setup_complete: true|false }
 *
 * POST /api/setup/configure
 * Request: {
 *   step: 1,  // or 2, 3
 *   data: {
 *     llm_provider: "openai"|"nvidia"|"anthropic"|"local",
 *     user_profile: "developer"|"business"|"support",
 *     bot_name: "string",
 *     bot_purpose: "string"
 *   }
 * }
 * Response: { success: true, setup_complete: true|false }
 */

var SetupWizardState = {
  currentStep: 1,
  totalSteps: 3,
  selections: {
    llm_provider: null,
    user_profile: null,
    bot_name: "",
    bot_purpose: ""
  }
};

function checkSetupStatus() {
  fetch("/api/setup/status")
    .then(function(response) { return response.json(); })
    .then(function(data) {
      if (!data.setup_complete) {
        showSetupWizard();
      }
    })
    .catch(function(e) {
      console.warn("[SetupWizard] Could not check setup status:", e);
    });
}

function showSetupWizard() {
  var overlay = document.getElementById("setupWizardOverlay");
  if (overlay) {
    overlay.style.display = "flex";
  }
}

function hideSetupWizard() {
  var overlay = document.getElementById("setupWizardOverlay");
  if (overlay) {
    overlay.style.display = "none";
  }
}

function updateProgressBar() {
  var fill = document.getElementById("setupProgressFill");
  var steps = document.querySelectorAll(".setup-step");
  if (fill) {
    var pct = (SetupWizardState.currentStep / SetupWizardState.totalSteps) * 100;
    fill.style.width = pct + "%";
  }
  steps.forEach(function(s) {
    var stepNum = parseInt(s.getAttribute("data-step"), 10);
    if (stepNum <= SetupWizardState.currentStep) {
      s.classList.add("active");
    } else {
      s.classList.remove("active");
    }
  });
}

function showStep(step) {
  for (var i = 1; i <= SetupWizardState.totalSteps; i++) {
    var el = document.getElementById("setupStep" + i);
    if (el) {
      el.classList.toggle("active", i === step);
    }
  }
  var backBtn = document.getElementById("setupBackBtn");
  var nextBtn = document.getElementById("setupNextBtn");
  var finishBtn = document.getElementById("setupFinishBtn");
  if (backBtn) backBtn.style.display = step === 1 ? "none" : "inline-flex";
  if (nextBtn) nextBtn.style.display = step === SetupWizardState.totalSteps ? "none" : "inline-flex";
  if (finishBtn) finishBtn.style.display = step === SetupWizardState.totalSteps ? "inline-flex" : "none";
  updateProgressBar();
}

function selectSetupOption(element, category) {
  var parent = element.parentElement;
  var options = parent.querySelectorAll(".setup-option");
  options.forEach(function(o) { o.classList.remove("selected"); });
  element.classList.add("selected");
  SetupWizardState.selections[category] = element.getAttribute("data-value");
}

function setupWizardNext() {
  if (SetupWizardState.currentStep < SetupWizardState.totalSteps) {
    var stepData = getStepData(SetupWizardState.currentStep);
    if (!stepData) return;
    saveStepData(SetupWizardState.currentStep, stepData);
    SetupWizardState.currentStep++;
    showStep(SetupWizardState.currentStep);
  }
}

function setupWizardBack() {
  if (SetupWizardState.currentStep > 1) {
    SetupWizardState.currentStep--;
    showStep(SetupWizardState.currentStep);
  }
}

function getStepData(step) {
  switch (step) {
    case 1:
      if (!SetupWizardState.selections.llm_provider) {
        notify("Please select an LLM provider", "warning");
        return null;
      }
      return { llm_provider: SetupWizardState.selections.llm_provider };
    case 2:
      if (!SetupWizardState.selections.user_profile) {
        notify("Please select a user profile", "warning");
        return null;
      }
      return { user_profile: SetupWizardState.selections.user_profile };
    default:
      return null;
  }
}

function saveStepData(step, data) {
  fetch("/api/setup/configure", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ step: step, data: data })
  }).catch(function(e) {
    console.warn("[SetupWizard] Failed to save step " + step + ":", e);
  });
}

function setupWizardFinish() {
  var botName = document.getElementById("setupBotName");
  var botPurpose = document.getElementById("setupBotPurpose");
  if (!botName || !botName.value.trim()) {
    notify("Please enter a bot name", "warning");
    return;
  }
  var finalData = {
    step: 3,
    data: {
      llm_provider: SetupWizardState.selections.llm_provider,
      user_profile: SetupWizardState.selections.user_profile,
      bot_name: botName.value.trim(),
      bot_purpose: botPurpose ? botPurpose.value.trim() : ""
    }
  };
  fetch("/api/setup/configure", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(finalData)
  })
    .then(function(r) { return r.json(); })
    .then(function(result) {
      if (result.success) {
        hideSetupWizard();
        notify("Setup complete! Welcome to General Bots.", "success");
      } else {
        notify("Setup failed: " + (result.error || "Unknown error"), "error");
      }
    })
    .catch(function(e) {
      console.warn("[SetupWizard] Failed to complete setup:", e);
      hideSetupWizard();
    });
}

function setupWizardSkip() {
  hideSetupWizard();
  notify("You can complete setup later from Settings.", "info");
}

// Auto-check setup status on load
if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', checkSetupStatus);
  } else {
    checkSetupStatus();
  }
}
