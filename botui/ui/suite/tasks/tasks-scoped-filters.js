(function () {
  "use strict";

  function root() {
    return document.getElementById("window-tasks") || document;
  }

  function byId(id) {
    return document.getElementById(id);
  }

  function escapeText(text) {
    if (typeof escapeHtml === "function") return escapeHtml(String(text == null ? "" : text));
    return String(text == null ? "" : text)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function updateFilterCounts() {
    fetch("/api/tasks/stats/json")
      .then(function (response) {
        if (!response.ok) throw new Error("Failed to fetch stats");
        return response.json();
      })
      .then(function (stats) {
        var mappings = [
          ["count-all", stats.total],
          ["count-complete", stats.completed !== undefined ? stats.completed : stats.complete],
          ["count-active", stats.active],
          ["count-awaiting", stats.awaiting],
          ["count-paused", stats.paused],
          ["count-blocked", stats.blocked],
          ["count-overdue", stats.overdue],
          ["count-due-soon", stats.due_soon],
          ["time-saved-value", stats.time_saved],
        ];
        mappings.forEach(function (pair) {
          if (pair[1] === undefined || pair[1] === null) return;
          var el = byId(pair[0]);
          if (el) el.textContent = pair[1];
        });
      })
      .catch(function (e) {
        console.warn("Failed to load task stats:", e);
      });
  }

  function showNewIntentModal() {
    var modal = byId("new-intent-modal");
    if (modal) modal.style.display = "flex";
  }

  function closeNewIntentModal() {
    var modal = byId("new-intent-modal");
    if (modal) modal.style.display = "none";
  }

  function showDecisionModal(decision) {
    var questionEl = byId("decision-question");
    if (decision && questionEl) {
      var title = decision.title || "Decision Required";
      var description = decision.description || "";
      questionEl.innerHTML =
        "<h4>" + escapeText(title) + "</h4>" +
        "<p>" + escapeText(description) + "</p>";
    }
    var modal = byId("decision-modal");
    if (modal) modal.style.display = "flex";
  }

  function closeDecisionModal() {
    var modal = byId("decision-modal");
    if (modal) modal.style.display = "none";
  }

  function submitNewIntent() {
    var form = byId("new-intent-form");
    if (!form) return;
    var intentInput = form.querySelector('[name="intent"]');
    if (!intentInput) return;
    var intent = intentInput.value;
    if (!intent || !intent.trim()) return;
    var quickInput = byId("quick-intent-input");
    if (quickInput) quickInput.value = intent;
    var quickBtn = byId("quick-intent-btn");
    if (quickBtn && typeof htmx !== "undefined" && htmx.trigger) {
      htmx.trigger(quickBtn, "click");
    }
    closeNewIntentModal();
  }

  function skipDecision() {
    closeDecisionModal();
  }

  function initFilterPills() {
    var tasksRoot = byId("window-tasks") || root();
    if (!tasksRoot || !tasksRoot.querySelectorAll) return;
    tasksRoot.querySelectorAll(".filter-pill").forEach(function (pill) {
      if (pill.dataset && pill.dataset.gbFilterBound === "1") return;
      if (pill.dataset) pill.dataset.gbFilterBound = "1";
      pill.addEventListener("click", function () {
        tasksRoot.querySelectorAll(".filter-pill").forEach(function (p) {
          p.classList.remove("active");
        });
        pill.classList.add("active");
      });
    });
  }

  function initSplitter() {
    var splitter = byId("tasks-splitter");
    var tasksRoot = byId("window-tasks") || root();
    var main = tasksRoot ? tasksRoot.querySelector(".tasks-main") : null;
    var leftPanel = tasksRoot ? tasksRoot.querySelector(".tasks-list-panel") : null;
    if (!splitter || !main || !leftPanel || splitter.dataset.gbSplitterBound === "1") return;
    splitter.dataset.gbSplitterBound = "1";

    var dragging = false;
    var startX = 0;
    var startWidth = 0;

    splitter.addEventListener("mousedown", function (e) {
      dragging = true;
      startX = e.clientX;
      startWidth = leftPanel.offsetWidth;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      e.preventDefault();
    });

    document.addEventListener("mousemove", function (e) {
      if (!dragging) return;
      var diff = e.clientX - startX;
      var nextWidth = Math.max(200, Math.min(600, startWidth + diff));
      leftPanel.style.flex = "0 0 " + nextWidth + "px";
      leftPanel.style.width = nextWidth + "px";
    });

    document.addEventListener("mouseup", function () {
      if (!dragging) return;
      dragging = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    });
  }

  function onHtmxAfterRequest(evt) {
    if (!evt || !evt.detail) return;
    var elt = evt.detail.elt;
    if (elt && elt.id === "pending-fill-form" && evt.detail.successful) {
      if (typeof htmx !== "undefined" && htmx.trigger) htmx.trigger(document.body, "taskCreated");
      var pendingValue = byId("pending-fill-value");
      if (pendingValue) pendingValue.value = "";
      if (typeof addAgentLog === "function") addAgentLog("success", "[OK] Pending info filled successfully");
      return;
    }

    var pathInfo = evt.detail.pathInfo || {};
    var requestPath = pathInfo.requestPath || "";
    if (requestPath !== "/api/ui/autotask/create" && requestPath !== "/api/autotask/create") return;

    var intentResult = byId("intent-result");
    if (!intentResult) return;
    var xhr = evt.detail.xhr;
    if (!xhr) return;

    if (xhr.status === 202) {
      try {
        var response = JSON.parse(xhr.responseText || "{}");
        if (response.success && response.task_id) {
          intentResult.innerHTML = '<span class="intent-success">✓ Task created - running...</span>';
          intentResult.style.display = "block";
          var quickInput = byId("quick-intent-input");
          if (quickInput) quickInput.value = "";
          if (typeof selectTask === "function") selectTask(response.task_id);
          setTimeout(function () { intentResult.style.display = "none"; }, 2000);
          return;
        }
        var msg = response.message || "Failed to create task";
        intentResult.innerHTML = '<span class="intent-error">✗ ' + escapeText(msg) + "</span>";
        intentResult.style.display = "block";
      } catch (e) {
        intentResult.innerHTML = '<span class="intent-error">✗ Failed to parse response</span>';
        intentResult.style.display = "block";
      }
      return;
    }

    if (xhr.status >= 400) {
      var errorMsg = "Error: " + xhr.status;
      try {
        var errorResponse = JSON.parse(xhr.responseText || "{}");
        errorMsg = errorResponse.error || errorResponse.message || errorMsg;
      } catch (e) {}
      intentResult.innerHTML = '<span class="intent-error">✗ ' + escapeText(errorMsg) + "</span>";
      intentResult.style.display = "block";
    }
  }

  function onHtmxAfterSwap(evt) {
    if (!evt || !evt.detail || !evt.detail.target) return;
    if (evt.detail.target.id !== "task-list") return;
    updateFilterCounts();
    var taskList = byId("task-list");
    var emptyState = byId("empty-state");
    if (taskList && emptyState) {
      emptyState.style.display = taskList.querySelector(".task-card") ? "none" : "flex";
    }
  }

  window.showNewIntentModal = showNewIntentModal;
  window.closeNewIntentModal = closeNewIntentModal;
  window.showDecisionModal = showDecisionModal;
  window.closeDecisionModal = closeDecisionModal;
  window.submitNewIntent = submitNewIntent;
  window.skipDecision = skipDecision;
  window.updateFilterCounts = updateFilterCounts;
  window.loadTaskStats = updateFilterCounts;

  document.addEventListener("htmx:afterRequest", onHtmxAfterRequest);
  document.addEventListener("htmx:afterSwap", onHtmxAfterSwap);
  document.body.addEventListener("taskCreated", updateFilterCounts);

  function init() {
    initFilterPills();
    initSplitter();
    updateFilterCounts();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
