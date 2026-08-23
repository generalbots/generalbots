// TASK FILTERING
// =============================================================================

function filterTasks(filter, button) {
  AutoTaskState.currentFilter = filter;

  // Update active tab
  document.querySelectorAll(".filter-tab").forEach((tab) => {
    tab.classList.remove("active");
  });
  button.classList.add("active");

  // Trigger HTMX request
  htmx.ajax("GET", `/api/autotask/list?filter=${filter}`, {
    target: "#task-list",
    swap: "innerHTML",
  });
}

// Sentient status filter
function filterByStatus(status, button) {
  AutoTaskState.currentFilter = status;

  // Update active filter
  document.querySelectorAll(".status-filter").forEach((filter) => {
    filter.classList.remove("active");
  });
  button.classList.add("active");

  // Trigger HTMX request for intent list
  htmx.ajax("GET", `/api/autotask/list?status=${status}`, {
    target: "#intent-list",
    swap: "innerHTML",
  });
}

function refreshTasks() {
  const filter = AutoTaskState.currentFilter;
  htmx.ajax("GET", `/api/autotask/list?filter=${filter}`, {
    target: "#task-list",
    swap: "innerHTML",
  });
  updateStats();
}

function refreshIntents() {
  const status = AutoTaskState.currentFilter;
  htmx.ajax("GET", `/api/autotask/list?status=${status}`, {
    target: "#intent-list",
    swap: "innerHTML",
  });
  updateStats();
}

// =============================================================================
// MODAL FUNCTIONS - SENTIENT
// =============================================================================

function showNewIntentModal() {
  const modal = document.getElementById("new-intent-modal");
  if (modal) {
    modal.style.display = "flex";
    document.body.classList.add("modal-open");
    setTimeout(() => {
      document.getElementById("intent-input")?.focus();
    }, 100);
  }
}

function closeNewIntentModal() {
  const modal = document.getElementById("new-intent-modal");
  if (modal) {
    modal.style.display = "none";
    document.body.classList.remove("modal-open");
  }
}

function openDecisionModal(intentId) {
  const modal = document.getElementById("decision-modal");
  if (!modal) return;

  modal.style.display = "flex";
  document.body.classList.add("modal-open");

  const content = document.getElementById("decision-content");
  if (content) {
    content.innerHTML = `
      <div class="loading-state">
        <div class="spinner"></div>
        <span>Loading decision options...</span>
      </div>
    `;
  }

  // Fetch decision details
  fetch(`/api/autotask/${intentId}/decisions`)
    .then((response) => response.json())
    .then((decisions) => {
      renderDecisionContent(intentId, decisions);
    })
    .catch((error) => {
      console.error("Failed to load decisions:", error);
      if (content) {
        content.innerHTML = `
          <div class="detail-placeholder">
            <span class="placeholder-icon">⚠️</span>
            <p>Failed to load decision options</p>
          </div>
        `;
      }
    });
}

function renderDecisionContent(intentId, decisions) {
  const content = document.getElementById("decision-content");
  if (!content || !decisions || decisions.length === 0) {
    if (content) {
      content.innerHTML = `
        <div class="detail-placeholder">
          <span class="placeholder-icon">✓</span>
          <p>No pending decisions</p>
        </div>
      `;
    }
    return;
  }

  const decision = decisions[0];
  content.innerHTML = `
    <div class="decision-detail">
      <h3>${escapeHtml(decision.title)}</h3>
      <p class="decision-desc">${escapeHtml(decision.description)}</p>

      <div class="decision-options-list">
        ${decision.options
          .map(
            (opt, idx) => `
          <label class="decision-option-item ${opt.recommended ? "recommended" : ""}">
            <input type="radio" name="decision_option" value="${opt.id}" ${idx === 0 ? "checked" : ""}>
            <div class="option-content">
              <span class="option-label">${escapeHtml(opt.label)}</span>
              ${opt.recommended ? '<span class="recommended-tag">Recommended</span>' : ""}
              <p class="option-desc">${escapeHtml(opt.description || "")}</p>
            </div>
          </label>
        `,
          )
          .join("")}
      </div>

      <div class="form-actions">
        <button class="btn-secondary" onclick="closeDecisionModal()">Cancel</button>
        <button class="btn-primary" onclick="submitDecisionFromModal('${intentId}', '${decision.id}')">
          Submit Decision
        </button>
      </div>
    </div>
  `;
}

function submitDecisionFromModal(intentId, decisionId) {
  const selectedOption = document.querySelector(
    'input[name="decision_option"]:checked',
  )?.value;

  if (!selectedOption) {
    showToast("Please select an option", "warning");
    return;
  }

  fetch(`/api/autotask/${intentId}/decide`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      decision_id: decisionId,
      option_id: selectedOption,
    }),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Decision submitted", "success");
        closeDecisionModal();
        refreshIntents();
        if (AutoTaskState.selectedIntentId === intentId) {
          loadIntentDetail(intentId);
        }
      } else {
        showToast(`Failed: ${result.error || "Unknown error"}`, "error");
      }
    })
    .catch((error) => {
      console.error("Failed to submit decision:", error);
      showToast("Failed to submit decision", "error");
    });
}

function closeDecisionModal() {
  const modal = document.getElementById("decision-modal");
  if (modal) {
    modal.style.display = "none";
    document.body.classList.remove("modal-open");
  }
}

function viewDecisionContext(intentId) {
  openDecisionModal(intentId);
}

function closeAllModals() {
  closeNewIntentModal();
  closeDecisionModal();
  document.body.classList.remove("modal-open");
}

// =============================================================================
// COMPILATION HANDLING
// =============================================================================

function onCompilationComplete(event) {
  const result = event.detail.target.querySelector(".compiled-plan");
  if (result) {
    // Scroll to result
    result.scrollIntoView({ behavior: "smooth", block: "start" });

    // Store compiled plan
    const planId = result.dataset?.planId;
    if (planId) {
      AutoTaskState.compiledPlan = planId;
    }

    // Syntax highlight the code
    highlightBasicCode();
  }
}

function highlightBasicCode() {
  const codeBlocks = document.querySelectorAll(".code-preview code");
  codeBlocks.forEach((block) => {
    // Basic syntax highlighting for BASIC keywords
    let html = block.innerHTML;

    // Keywords
    const keywords = [
      "PLAN_START",
      "PLAN_END",
      "STEP",
      "SET",
      "GET",
      "IF",
      "THEN",
      "ELSE",
      "END IF",
      "FOR EACH",
      "NEXT",
      "WHILE",
      "WEND",
      "TALK",
      "HEAR",
      "LLM",
      "CREATE_TASK",
      "RUN_PYTHON",
      "RUN_JAVASCRIPT",
      "RUN_BASH",
      "USE_MCP",
      "POST",
      "GET",
      "PUT",
      "PATCH",
      "DELETE HTTP",
      "REQUIRE_APPROVAL",
      "SIMULATE_IMPACT",
      "AUDIT_LOG",
      "SEND_MAIL",
      "SAVE",
      "UPDATE",
      "INSERT",
      "DELETE",
      "FIND",
    ];

    keywords.forEach((keyword) => {
      const regex = new RegExp(`\\b${keyword}\\b`, "g");
      html = html.replace(regex, `<span class="keyword">${keyword}</span>`);
    });

    // Comments
    html = html.replace(/(\'[^\n]*)/g, '<span class="comment">$1</span>');

    // Strings
    html = html.replace(/("[^"]*")/g, '<span class="string">$1</span>');

    // Numbers
    html = html.replace(/\b(\d+)\b/g, '<span class="number">$1</span>');

    block.innerHTML = html;
  });
}

function copyGeneratedCode() {
  const code = document.querySelector(".code-preview code")?.textContent;
  if (code) {
    navigator.clipboard
      .writeText(code)
      .then(() => {
        showToast("Code copied to clipboard", "success");
      })
      .catch(() => {
        showToast("Failed to copy code", "error");
      });
  }
}

function discardPlan() {
  if (confirm("Are you sure you want to discard this plan?")) {
    document.getElementById("compilation-result").innerHTML = "";
    AutoTaskState.compiledPlan = null;
    document.getElementById("intent-input").value = "";
    document.getElementById("intent-input").focus();
  }
}

function editPlan() {
  if (!AutoTaskState.compiledPlan) {
    showToast("No plan to edit", "warning");
    return;
  }

  const modal = document.createElement("div");
  modal.className = "modal-overlay";
  modal.id = "plan-editor-modal";
  modal.innerHTML = `
    <div class="modal-content large">
      <div class="modal-header">
        <h3>Edit Plan</h3>
        <button class="close-btn" onclick="closePlanEditor()">&times;</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label for="plan-name">Plan Name</label>
          <input type="text" id="plan-name" value="${AutoTaskState.compiledPlan.name || "Untitled Plan"}" />
        </div>
        <div class="form-group">
          <label for="plan-description">Description</label>
          <textarea id="plan-description" rows="3">${AutoTaskState.compiledPlan.description || ""}</textarea>
        </div>
        <div class="form-group">
          <label for="plan-steps">Steps (JSON)</label>
          <textarea id="plan-steps" rows="10" class="code-editor">${JSON.stringify(AutoTaskState.compiledPlan.steps || [], null, 2)}</textarea>
        </div>
        <div class="form-group">
          <label for="plan-priority">Priority</label>
          <select id="plan-priority">
            <option value="low" ${AutoTaskState.compiledPlan.priority === "low" ? "selected" : ""}>Low</option>
            <option value="medium" ${AutoTaskState.compiledPlan.priority === "medium" ? "selected" : ""}>Medium</option>
            <option value="high" ${AutoTaskState.compiledPlan.priority === "high" ? "selected" : ""}>High</option>
            <option value="urgent" ${AutoTaskState.compiledPlan.priority === "urgent" ? "selected" : ""}>Urgent</option>
          </select>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick="closePlanEditor()">Cancel</button>
        <button class="btn btn-primary" onclick="savePlanEdits()">Save Changes</button>
      </div>
    </div>
  `;
  document.body.appendChild(modal);
}

function closePlanEditor() {
  const modal = document.getElementById("plan-editor-modal");
  if (modal) {
    modal.remove();
  }
}

function savePlanEdits() {
  const name = document.getElementById("plan-name").value;
  const description = document.getElementById("plan-description").value;
  const stepsJson = document.getElementById("plan-steps").value;
  const priority = document.getElementById("plan-priority").value;

  let steps;
  try {
    steps = JSON.parse(stepsJson);
  } catch (e) {
    showToast("Invalid JSON in steps", "error");
    return;
  }

  AutoTaskState.compiledPlan = {
    ...AutoTaskState.compiledPlan,
    name: name,
    description: description,
    steps: steps,
    priority: priority,
  };

  closePlanEditor();
  showToast("Plan updated successfully", "success");

  const resultDiv = document.getElementById("compilation-result");
  if (resultDiv && AutoTaskState.compiledPlan) {
    renderCompiledPlan(AutoTaskState.compiledPlan);
  }
}

function renderCompiledPlan(plan) {
  const resultDiv = document.getElementById("compilation-result");
  if (!resultDiv) return;

  const stepsHtml = (plan.steps || [])
    .map(
      (step, i) => `
      <div class="plan-step">
        <span class="step-number">${i + 1}</span>
        <span class="step-action">${step.action || step.type || "Action"}</span>
        <span class="step-target">${step.target || step.description || ""}</span>
      </div>
    `,
    )
    .join("");

  resultDiv.innerHTML = `
    <div class="compiled-plan">
      <div class="plan-header">
        <h4>${plan.name || "Compiled Plan"}</h4>
        <span class="plan-priority priority-${plan.priority || "medium"}">${plan.priority || "medium"}</span>
      </div>
      ${plan.description ? `<p class="plan-description">${plan.description}</p>` : ""}
      <div class="plan-steps">${stepsHtml}</div>
      <div class="plan-actions">
        <button class="btn btn-secondary" onclick="editPlan()">Edit</button>
        <button class="btn btn-secondary" onclick="discardPlan()">Discard</button>
        <button class="btn btn-primary" onclick="executePlan('${plan.id || ""}')">Execute</button>
      </div>
    </div>
  `;
}

// =============================================================================
// PLAN EXECUTION
// =============================================================================
