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
 *   step: 1,  // or 2, 3, 4
 *   data: {
 *     llm_provider: "openai"|"nvidia"|"anthropic"|"local",
 *     user_profile: "developer"|"business"|"support",
 *     bot_name: "string",
 *     bot_purpose: "string",
 *     training_files: ["filename1.pdf", ...]
 *   }
 * }
 * Response: { success: true, setup_complete: true|false }
 */

var SetupWizardState = {
  currentStep: 1,
  totalSteps: 4,
  selections: {
    llm_provider: null,
    user_profile: null,
    bot_name: "",
    bot_purpose: "",
    bot_template: ""
  },
  trainingFiles: []
};

function checkSetupStatus() {
  if (localStorage.getItem("gb_setup_wizard_done") === "true") {
    return;
  }
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
    case 3:
      var botNameEl = document.getElementById("setupBotName");
      var botPurposeEl = document.getElementById("setupBotPurpose");
      var botTemplateEl = document.getElementById("wizardBotTemplate");
      if (!botNameEl || !botNameEl.value.trim()) {
        notify("Please enter a bot name", "warning");
        return null;
      }
      SetupWizardState.selections.bot_name = botNameEl.value.trim();
      SetupWizardState.selections.bot_purpose = botPurposeEl ? botPurposeEl.value.trim() : "";
      SetupWizardState.selections.bot_template = botTemplateEl ? botTemplateEl.value : "";
      return {
        bot_name: SetupWizardState.selections.bot_name,
        bot_purpose: SetupWizardState.selections.bot_purpose,
        bot_template: SetupWizardState.selections.bot_template
      };
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

function handleSetupFileDrop(event) {
  event.preventDefault();
  var zone = document.getElementById("setupDropZone");
  if (zone) zone.classList.remove("dragover");
  if (event.dataTransfer.files) {
    addFilesToList(event.dataTransfer.files);
  }
}

function handleSetupFileSelect(event) {
  if (event.target.files) {
    addFilesToList(event.target.files);
  }
}

function addFilesToList(fileList) {
  for (var i = 0; i < fileList.length; i++) {
    var file = fileList[i];
    SetupWizardState.trainingFiles.push(file);
    renderFileItem(file);
  }
}

function renderFileItem(file) {
  var list = document.getElementById("setupFileList");
  if (!list) return;
  var div = document.createElement("div");
  div.className = "setup-file-item";
  div.dataset.fileName = file.name;
  var size = (file.size / 1024).toFixed(1) + " KB";
  if (file.size > 1024 * 1024) size = (file.size / (1024 * 1024)).toFixed(1) + " MB";
  div.innerHTML =
    '<span class="file-name">' + escapeHtml(file.name) + '</span>' +
    '<span class="file-size">' + size + '</span>' +
    '<span class="file-status pending">Pending</span>';
  list.appendChild(div);
}

function escapeHtml(text) {
  var d = document.createElement("div");
  d.textContent = text;
  return d.innerHTML;
}

function uploadTrainingFiles(botName) {
  return new Promise(function(resolve, reject) {
    var files = SetupWizardState.trainingFiles;
    if (files.length === 0) {
      resolve();
      return;
    }

    var pending = files.length;
    var hasError = false;

    files.forEach(function(file, idx) {
      var formData = new FormData();
      formData.append("file", file);

      fetch("/api/files/upload?path=" + encodeURIComponent(botName + ".gbai/" + botName + ".gbkb/training/"), {
        method: "POST",
        body: formData
      })
        .then(function(r) {
          if (!r.ok) throw new Error("Upload failed");
          return r.json();
        })
        .then(function() {
          updateFileStatus(file.name, "uploaded");
        })
        .catch(function(e) {
          console.warn("[SetupWizard] Upload failed for " + file.name + ":", e);
          updateFileStatus(file.name, "error");
          hasError = true;
        })
        .finally(function() {
          pending--;
          if (pending === 0) {
            if (hasError) reject(new Error("Some files failed to upload"));
            else resolve();
          }
        });
    });
  });
}

function updateFileStatus(fileName, status) {
  var items = document.querySelectorAll(".setup-file-item");
  items.forEach(function(item) {
    if (item.dataset.fileName === fileName) {
      var statusEl = item.querySelector(".file-status");
      if (statusEl) {
        statusEl.className = "file-status " + status;
        statusEl.textContent = status.charAt(0).toUpperCase() + status.slice(1);
      }
    }
  });
}

function setupWizardFinish() {
  var botName = SetupWizardState.selections.bot_name;
  if (!botName) {
    var botNameEl = document.getElementById("setupBotName");
    if (!botNameEl || !botNameEl.value.trim()) {
      notify("Please enter a bot name", "warning");
      return;
    }
    botName = botNameEl.value.trim();
    SetupWizardState.selections.bot_name = botName;
  }

  // Gather step 3 data if not already saved
  if (!SetupWizardState.selections.bot_purpose) {
    var botPurposeEl = document.getElementById("setupBotPurpose");
    SetupWizardState.selections.bot_purpose = botPurposeEl ? botPurposeEl.value.trim() : "";
  }

  var finishBtn = document.getElementById("setupFinishBtn");
  if (finishBtn) {
    finishBtn.disabled = true;
    finishBtn.textContent = "Saving...";
  }

  // Upload training files first, then save final config
  uploadTrainingFiles(botName)
    .catch(function() {
      // Continue even if some uploads fail
    })
    .finally(function() {
      var finalData = {
        step: 4,
        data: {
          llm_provider: SetupWizardState.selections.llm_provider,
          user_profile: SetupWizardState.selections.user_profile,
          bot_name: botName,
          bot_purpose: SetupWizardState.selections.bot_purpose,
          bot_template: SetupWizardState.selections.bot_template,
          training_files: SetupWizardState.trainingFiles.map(function(f) { return f.name; })
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
            localStorage.setItem("gb_setup_wizard_done", "true");
            hideSetupWizard();
            notify("Setup complete! Welcome to General Bots.", "success");
          } else {
            notify("Setup failed: " + (result.error || "Unknown error"), "error");
          }
        })
        .catch(function(e) {
          console.warn("[SetupWizard] Failed to complete setup:", e);
          hideSetupWizard();
        })
        .finally(function() {
          if (finishBtn) {
            finishBtn.disabled = false;
            finishBtn.textContent = "Finish";
          }
        });
    });
}

function setupWizardSkip() {
  localStorage.setItem("gb_setup_wizard_done", "true");
  hideSetupWizard();
  notify("You can complete setup later from Settings.", "info");
}

// Expose globals for inline onclick handlers
window.selectSetupOption = selectSetupOption;
window.setupWizardNext = setupWizardNext;
window.setupWizardBack = setupWizardBack;
window.setupWizardFinish = setupWizardFinish;
window.setupWizardSkip = setupWizardSkip;
window.handleSetupFileDrop = handleSetupFileDrop;
window.handleSetupFileSelect = handleSetupFileSelect;

// Auto-check setup status on load
if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    (function(){ var __cb = checkSetupStatus; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    checkSetupStatus();
  }
}
