"use strict";
/* Tasks collab wiring — mounts a "Comments" button on the task detail panel
 * and opens the shared threaded-comments panel (GBCollabComments) anchored to
 * the selected task. resource_type = "task", resource_id = task id. */
(function (window) {
  function mount() {
    var panel = document.getElementById("task-detail-panel");
    if (!panel || panel.__gbcMounted) return;
    panel.__gbcMounted = true;

    var btn = document.createElement("button");
    btn.type = "button";
    btn.className = "task-comments-btn";
    btn.textContent = "\uD83D\uDCAC Comments";
    btn.style.cssText =
      "display:block;width:calc(100% - 24px);margin:12px;background:#1e293b;color:#f8fafc;" +
      "border:1px solid #334155;border-radius:6px;padding:8px 12px;font-size:13px;cursor:pointer;text-align:left;";
    btn.addEventListener("click", function () {
      var taskId = window.TasksState && window.TasksState.selectedTaskId;
      if (!taskId) {
        window.alert("Select a task first to comment on it.");
        return;
      }
      window.GBCollabComments.open({
        resourceType: "task",
        resourceId: String(taskId),
        title: "Task #" + taskId + " comments",
        notify: function (msg, type) {
          if (typeof window.showToast === "function") window.showToast(msg, type);
        },
      });
    });
    panel.insertBefore(btn, panel.firstChild);
  }

  // The detail panel may be present at init, or arrive later via HTMX.
  function tryMount() {
    if (document.getElementById("task-detail-panel")) mount();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tryMount);
  } else {
    tryMount();
  }
  // Retry a few times in case the tasks app is injected later.
  var attempts = 0;
  var retry = setInterval(function () {
    attempts++;
    if (document.getElementById("task-detail-panel")) { mount(); clearInterval(retry); }
    else if (attempts > 40) { clearInterval(retry); }
  }, 250);
})(window);
