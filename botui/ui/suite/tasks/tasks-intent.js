
function submitNewIntent() {
  var form = document.getElementById("new-intent-form");
  if (!form) return;

  var intentInput = form.querySelector('[name="intent"]');
  if (!intentInput) return;

  var intent = intentInput.value;
  if (intent && intent.trim()) {
    var quickInput = document.getElementById("quick-intent-input");
    if (quickInput) {
      quickInput.value = intent;
    }
    var quickBtn = document.getElementById("quick-intent-btn");
    if (quickBtn && typeof htmx !== "undefined") {
      htmx.trigger(quickBtn, "click");
    }
    closeNewIntentModal();
  }
}

function skipDecision() {
  closeDecisionModal();
}

// =============================================================================
// TASK STATS LOADING
// =============================================================================

function loadTaskStats() {
  fetch("/api/tasks/stats/json")
    .then(function (response) {
      if (!response.ok) {
        throw new Error("Failed to fetch stats");
      }
      return response.json();
    })
    .then(function (stats) {
      var mappings = [
        { key: "complete", id: "count-complete" },
        { key: "completed", id: "count-complete" },
        { key: "active", id: "count-active" },
        { key: "awaiting", id: "count-awaiting" },
        { key: "paused", id: "count-paused" },
        { key: "blocked", id: "count-blocked" },
        { key: "time_saved", id: "time-saved-value" },
        { key: "total", id: "count-all" },
      ];

      mappings.forEach(function (mapping) {
        if (stats[mapping.key] !== undefined) {
          var el = document.getElementById(mapping.id);
          if (el) {
            el.textContent = stats[mapping.key];
          }
        }
      });
    })
    .catch(function (e) {
      console.warn("Failed to load stats:", e);
    });
}

// =============================================================================
// SPLITTER DRAG FUNCTIONALITY
// =============================================================================

(function initSplitter() {
  var splitter = document.getElementById("tasks-splitter");
  var main = document.querySelector(".tasks-main");
  var leftPanel = document.querySelector(".tasks-list-panel");

  if (!splitter || !main || !leftPanel) return;

  var isDragging = false;
  var startX = 0;
  var startWidth = 0;

  splitter.addEventListener("mousedown", function (e) {
    isDragging = true;
    startX = e.clientX;
    startWidth = leftPanel.offsetWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
  });

  document.addEventListener("mousemove", function (e) {
    if (!isDragging) return;

    var diff = e.clientX - startX;
    var newWidth = Math.max(200, Math.min(600, startWidth + diff));
    leftPanel.style.flex = "0 0 " + newWidth + "px";
    leftPanel.style.width = newWidth + "px";
  });

  document.addEventListener("mouseup", function () {
    if (isDragging) {
      isDragging = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
  });
})();

// =============================================================================
// HTMX TASK CREATION HANDLER
// =============================================================================

document.body.addEventListener("htmx:afterRequest", function (evt) {
  if (!evt.detail.pathInfo) return;
  if (evt.detail.pathInfo.requestPath !== "/api/autotask/create") return;

  var xhr = evt.detail.xhr;
  var intentResult = document.getElementById("intent-result");
  if (!intentResult) return;

  if (xhr && xhr.status === 202) {
    try {
      var response = JSON.parse(xhr.responseText);
      if (response.success && response.task_id) {
        console.log("[TASK] Created task:", response.task_id);

        intentResult.innerHTML =
          '<span class="intent-success">✓ Task created - running...</span>';
        intentResult.style.display = "block";

        var quickInput = document.getElementById("quick-intent-input");
        if (quickInput) {
          quickInput.value = "";
        }

        selectTask(response.task_id);

        setTimeout(function () {
          intentResult.style.display = "none";
        }, 2000);
      } else {
        var msg = response.message || "Failed to create task";
        intentResult.innerHTML =
          '<span class="intent-error">✗ ' + escapeHtml(msg) + "</span>";
        intentResult.style.display = "block";
      }
    } catch (e) {
      console.warn("Failed to parse create response:", e);
      intentResult.innerHTML =
        '<span class="intent-error">✗ Failed to parse response</span>';
      intentResult.style.display = "block";
    }
  } else if (xhr && xhr.status >= 400) {
    try {
      var errorResponse = JSON.parse(xhr.responseText);
      var errorMsg =
        errorResponse.error || errorResponse.message || "Error creating task";
      intentResult.innerHTML =
        '<span class="intent-error">✗ ' + escapeHtml(errorMsg) + "</span>";
    } catch (e) {
      intentResult.innerHTML =
        '<span class="intent-error">✗ Error: ' + xhr.status + "</span>";
    }
    intentResult.style.display = "block";
  }
});

// =============================================================================
// FILTER PILL CLICK HANDLER
// =============================================================================

document.querySelectorAll(".filter-pill").forEach(function (pill) {
  pill.addEventListener("click", function () {
    document.querySelectorAll(".filter-pill").forEach(function (p) {
      p.classList.remove("active");
    });
    this.classList.add("active");
  });
});

// =============================================================================
// HTMX TASK LIST REFRESH HANDLER
// =============================================================================

document.body.addEventListener("htmx:afterSwap", function (e) {
  if (e.detail.target && e.detail.target.id === "task-list") {
    loadTaskStats();
    var taskList = document.getElementById("task-list");
    var emptyState = document.getElementById("empty-state");
    if (taskList && emptyState) {
      var hasTasks = taskList.querySelector(".task-card");
      emptyState.style.display = hasTasks ? "none" : "flex";
    }
  }
});
