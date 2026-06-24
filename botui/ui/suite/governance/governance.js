(function () {
  "use strict";

  if (typeof window.governanceInitialized !== "undefined") return;
  window.governanceInitialized = true;

  window.killSession = function () {
    var sessionInput = document.getElementById("session-id-input");
    var reasonInput = document.getElementById("kill-reason-input");
    var resultDiv = document.getElementById("kill-result");
    var sessionId = sessionInput ? sessionInput.value.trim() : "";
    if (!sessionId) {
      if (resultDiv) {
        resultDiv.innerHTML =
          '<span class="kill-error">Please enter a session ID</span>';
      }
      return;
    }
    var payload = { session_id: sessionId };
    if (reasonInput && reasonInput.value.trim()) {
      payload.reason = reasonInput.value.trim();
    }
    if (resultDiv) {
      resultDiv.innerHTML = '<span class="kill-pending">Terminating...</span>';
    }
    fetch("/api/governance/sessions/kill", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(function (data) {
        if (resultDiv) {
          resultDiv.innerHTML =
            '<span class="kill-success">' + data.message + "</span>";
        }
        if (sessionInput) sessionInput.value = "";
        if (reasonInput) reasonInput.value = "";
      })
      .catch(function (err) {
        if (resultDiv) {
          resultDiv.innerHTML =
            '<span class="kill-error">Error: ' + err.message + "</span>";
        }
      });
  };

  document.addEventListener("DOMContentLoaded", function () {
    var dashboardEl = document.querySelector(".governance-metrics-grid");
    if (dashboardEl) {
      htmx.process(dashboardEl);
    }
  });
})();
