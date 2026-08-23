
function simulatePlan(planId) {
  showSimulationModal();

  fetch(`/api/autotask/simulate/${planId}`, {
    method: "POST",
  })
    .then((response) => response.json())
    .then((result) => {
      renderSimulationResult(result);
    })
    .catch((error) => {
      document.getElementById("simulation-content").innerHTML = `
            <div class="error-message">
                <span class="error-icon">❌</span>
                <p>Failed to simulate plan: ${error.message}</p>
            </div>
        `;
    });
}

function executePlan(planId) {
  const executionMode =
    document.querySelector('[name="execution_mode"]')?.value ||
    "semi-automatic";
  const priority =
    document.querySelector('[name="priority"]')?.value || "medium";

  if (!confirm("Are you sure you want to execute this plan?")) {
    return;
  }

  fetch("/api/autotask/execute", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      plan_id: planId,
      execution_mode: executionMode,
      priority: priority,
    }),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task execution started!", "success");
        document.getElementById("compilation-result").innerHTML = "";
        document.getElementById("intent-input").value = "";
        refreshTasks();
      } else {
        showToast(`Failed to start execution: ${result.error}`, "error");
      }
    })
    .catch((error) => {
      showToast(`Failed to execute plan: ${error.message}`, "error");
    });
}

// =============================================================================
// TASK ACTIONS
// =============================================================================

function viewTaskDetails(taskId) {
  window.location.href = `/suite/tasks/detail/${taskId}`;
}

function simulateTask(taskId) {
  showSimulationModal();

  fetch(`/api/autotask/${taskId}/simulate`, {
    method: "POST",
  })
    .then((response) => response.json())
    .then((result) => {
      result.task_id = taskId;
      renderSimulationResult(result);
    })
    .catch((error) => {
      document.getElementById("simulation-content").innerHTML = `
            <div class="error-message">
                <span class="error-icon">❌</span>
                <p>Failed to simulate task: ${error.message}</p>
            </div>
        `;
    });
}

function pauseTask(taskId) {
  fetch(`/api/autotask/${taskId}/pause`, {
    method: "POST",
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task paused", "success");
        refreshTasks();
      } else {
        showToast(`Failed to pause task: ${result.error}`, "error");
      }
    });
}

function resumeTask(taskId) {
  fetch(`/api/autotask/${taskId}/resume`, {
    method: "POST",
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task resumed", "success");
        refreshTasks();
      } else {
        showToast(`Failed to resume task: ${result.error}`, "error");
      }
    });
}

function cancelTask(taskId) {
  if (
    !confirm(
      "Are you sure you want to cancel this task? This may not be reversible.",
    )
  ) {
    return;
  }

  fetch(`/api/autotask/${taskId}/cancel`, {
    method: "POST",
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Task cancelled", "success");
        refreshTasks();
      } else {
        showToast(`Failed to cancel task: ${result.error}`, "error");
      }
    });
}

function updateTaskInList(task) {
  const taskElement = document.querySelector(`[data-task-id="${task.id}"]`);
  if (taskElement) {
    // Update status badge
    const statusBadge = taskElement.querySelector(".task-status-badge");
    if (statusBadge) {
      statusBadge.className = `task-status-badge status-${task.status}`;
      statusBadge.textContent = task.status.replace(/-/g, " ");
    }

    // Update progress
    const progressFill = taskElement.querySelector(".progress-fill");
    const progressText = taskElement.querySelector(".progress-text");
    if (progressFill && progressText) {
      progressFill.style.width = `${task.progress}%`;
      progressText.textContent = `${task.current_step}/${task.total_steps} steps (${Math.round(task.progress)}%)`;
    }

    // Update data attribute
    taskElement.dataset.status = task.status;
  }
}

function updateStepProgress(taskId, step, progress) {
  const taskElement = document.querySelector(`[data-task-id="${taskId}"]`);
  if (taskElement) {
    const currentStep = taskElement.querySelector(".current-step");
    if (currentStep) {
      currentStep.querySelector(".step-name").textContent =
        `Step ${step.order}: ${step.name}`;
      currentStep.querySelector(".step-status").textContent =
        `${Math.round(progress)}%`;
    }
  }
}

// =============================================================================
// DECISIONS
// =============================================================================

function viewDecisions(taskId) {
  showDecisionModal();

  fetch(`/api/autotask/${taskId}/decisions`)
    .then((response) => response.json())
    .then((decisions) => {
      renderDecisions(taskId, decisions);
    })
    .catch((error) => {
      document.getElementById("decision-content").innerHTML = `
                <div class="error-message">
                    <span class="error-icon">❌</span>
                    <p>Failed to load decisions: ${error.message}</p>
                </div>
            `;
    });
}

function renderDecisions(taskId, decisions) {
  const container = document.getElementById("decision-content");

  if (!decisions || decisions.length === 0) {
    container.innerHTML = '<p class="no-decisions">No pending decisions.</p>';
    return;
  }

  let html = '<div class="decisions-list">';

  decisions.forEach((decision) => {
    html += `
            <div class="decision-item" data-decision-id="${decision.id}">
                <h4>${decision.title}</h4>
                <p class="decision-description">${decision.description}</p>

                <div class="decision-options">
                    ${decision.options
                      .map(
                        (opt) => `
                        <div class="decision-option ${opt.recommended ? "recommended" : ""}" data-option-id="${opt.id}">
                            <div class="option-header">
                                <input type="radio" name="decision_${decision.id}" value="${opt.id}" id="opt_${opt.id}" ${opt.recommended ? "checked" : ""}>
                                <label for="opt_${opt.id}">
                                    <span class="option-label">${opt.label}</span>
                                    ${opt.recommended ? '<span class="recommended-badge">Recommended</span>' : ""}
                                </label>
                            </div>
                            <p class="option-description">${opt.description}</p>
                            <div class="option-impact">
                                <span class="impact-cost">💰 ${opt.estimated_impact.cost_change >= 0 ? "+" : ""}$${opt.estimated_impact.cost_change}</span>
                                <span class="impact-time">⏱️ ${opt.estimated_impact.time_change_minutes >= 0 ? "+" : ""}${opt.estimated_impact.time_change_minutes}m</span>
                                <span class="impact-risk risk-${opt.risk_level.toLowerCase()}">⚠️ ${opt.risk_level}</span>
                            </div>
                        </div>
                    `,
                      )
                      .join("")}
                </div>

                <div class="decision-actions">
                    <button class="btn-secondary" onclick="skipDecision('${taskId}', '${decision.id}')">Skip</button>
                    <button class="btn-primary" onclick="submitDecision('${taskId}', '${decision.id}')">Submit Decision</button>
                </div>
            </div>
        `;
  });

  html += "</div>";
  container.innerHTML = html;
}

function submitDecision(taskId, decisionId) {
  const selectedOption = document.querySelector(
    `input[name="decision_${decisionId}"]:checked`,
  )?.value;

  if (!selectedOption) {
    showToast("Please select an option", "warning");
    return;
  }

  fetch(`/api/autotask/${taskId}/decide`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
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
        refreshTasks();
      } else {
        showToast(`Failed to submit decision: ${result.error}`, "error");
      }
    });
}

function skipDecision(taskId, decisionId) {
  if (
    !confirm(
      "Are you sure you want to skip this decision? The default option will be used.",
    )
  ) {
    return;
  }

  fetch(`/api/autotask/${taskId}/decide`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      decision_id: decisionId,
      skip: true,
    }),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        showToast("Decision skipped", "info");
        closeDecisionModal();
        refreshTasks();
      } else {
        showToast(`Failed to skip decision: ${result.error}`, "error");
      }
    });
}

function showDecisionNotification(decision) {
  showToast(`Decision required: ${decision.title}`, "warning", 10000);
  updateStats();
}

// =============================================================================
// APPROVALS
// =============================================================================

function viewApprovals(taskId) {
  showApprovalModal();

  fetch(`/api/autotask/${taskId}/approvals`)
    .then((response) => response.json())
    .then((approvals) => {
      renderApprovals(taskId, approvals);
    })
    .catch((error) => {
      document.getElementById("approval-content").innerHTML = `
                <div class="error-message">
                    <span class="error-icon">❌</span>
                    <p>Failed to load approvals: ${error.message}</p>
                </div>
            `;
    });
}

function renderApprovals(taskId, approvals) {
  const container = document.getElementById("approval-content");

  if (!approvals || approvals.length === 0) {
    container.innerHTML = '<p class="no-approvals">No pending approvals.</p>';
    return;
  }

  let html = '<div class="approvals-list">';

  approvals.forEach((approval) => {
    html += `
            <div class="approval-item" data-approval-id="${approval.id}">
                <div class="approval-header">
                    <span class="approval-type type-${approval.approval_type.toLowerCase().replace(/_/g, "-")}">${approval.approval_type.replace(/_/g, " ")}</span>
                    <span class="approval-risk risk-${approval.risk_level.toLowerCase()}">${approval.risk_level} Risk</span>
                </div>

                <h4>${approval.title}</h4>
                <p class="approval-description">${approval.description}</p>

                <div class="approval-impact">
                    <h5>Impact Summary</h5>
                    <p>${approval.impact_summary}</p>
                </div>

                ${
                  approval.simulation_result
                    ? `
                    <div class="simulation-preview">
                        <h5>Simulation Result</h5>
                        <div class="simulation-summary">
                            <span class="sim-risk risk-${approval.simulation_result.risk_level.toLowerCase()}">Risk: ${approval.simulation_result.risk_level}</span>
                            <span class="sim-confidence">Confidence: ${Math.round(approval.simulation_result.confidence * 100)}%</span>
                        </div>
                    </div>
                `
                    : ""
                }

                <div class="approval-meta">
                    <span>Step: ${approval.step_name || "N/A"}</span>
                    <span>Expires: ${formatRelativeTime(approval.expires_at)}</span>
                    <span>Default: ${approval.default_action}</span>
                </div>

                <div class="approval-actions">
                    <button class="btn-reject" onclick="rejectApproval('${taskId}', '${approval.id}')">
                        <span>❌</span> Reject
                    </button>
                    <button class="btn-defer" onclick="deferApproval('${taskId}', '${approval.id}')">
                        <span>⏸️</span> Defer
                    </button>
                    <button class="btn-approve" onclick="approveApproval('${taskId}', '${approval.id}')">
                        <span>✅</span> Approve
                    </button>
                </div>
            </div>
        `;
  });

  html += "</div>";
  container.innerHTML = html;
}

function approveApproval(taskId, approvalId) {
  submitApprovalDecision(taskId, approvalId, "approve");
}

function rejectApproval(taskId, approvalId) {
  if (!confirm("Are you sure you want to reject this action?")) {
    return;
  }
  submitApprovalDecision(taskId, approvalId, "reject");
}

function deferApproval(taskId, approvalId) {
  submitApprovalDecision(taskId, approvalId, "defer");
}

function submitApprovalDecision(taskId, approvalId, action) {
  fetch(`/api/autotask/${taskId}/approve`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      approval_id: approvalId,
      action: action,
    }),
  })
    .then((response) => response.json())
    .then((result) => {
      if (result.success) {
        const messages = {
          approve: "Approval granted",
          reject: "Approval rejected",
          defer: "Approval deferred",
        };
        showToast(messages[action], "success");
        closeApprovalModal();
        refreshTasks();
      } else {
        showToast(`Failed to ${action}: ${result.error}`, "error");
      }
    });
}
