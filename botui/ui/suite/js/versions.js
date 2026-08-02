/*
 * System Versions Panel - Issue #468
 *
 * Backend API Contract:
 *
 * GET /api/system/versions
 * Response: {
 *   botserver: "0.1.0",
 *   botui: "0.1.0",
 *   rust: "1.75.0",
 *   postgresql: "16.1",
 *   valkey: "7.2.4",
 *   minio: "2024-01-20T00:00:00Z",
 *   qdrant: "1.7.0",
 *   vault: "1.15.0",
 *   zitadel: "2.45.0"
 * }
 *
 * POST /api/system/check-updates
 * Request: { component: "botserver" }
 * Response: { component: "botserver", current: "0.1.0", latest: "0.1.1", update_available: true }
 */

var VersionComponents = [
  { id: "botserver", name: "BotServer", icon: "\u2699\uFE0F" },
  { id: "botui", name: "BotUI", icon: "\u{1F4BB}" },
  { id: "rust", name: "Rust", icon: "\u{1F7E2}" },
  { id: "postgresql", name: "PostgreSQL", icon: "\u{1F4BE}" },
  { id: "valkey", name: "Valkey / Redis", icon: "\u{1F5C4}\uFE0F" },
  { id: "minio", name: "MinIO", icon: "\u{1F4E6}" },
  { id: "qdrant", name: "Qdrant", icon: "\u{1F9EA}" },
  { id: "vault", name: "Vault", icon: "\u{1F512}" },
  { id: "zitadel", name: "Zitadel", icon: "\u{1F464}" }
];

function loadVersions() {
  var container = document.getElementById("versionsTableContainer");
  if (!container) return;

  fetch("/api/system/versions")
    .then(function(response) {
      if (!response.ok) throw new Error("HTTP " + response.status);
      return response.json();
    })
    .then(function(versions) {
      renderVersionsTable(container, versions);
    })
    .catch(function(e) {
      console.warn("[Versions] Could not load versions:", e);
      container.innerHTML = renderVersionsFallback();
    });
}

function renderVersionsTable(container, versions) {
  var html = '<table class="versions-table">' +
    '<thead><tr><th>Component</th><th>Version</th><th>Status</th><th>Action</th></tr></thead><tbody>';

  VersionComponents.forEach(function(comp) {
    var ver = versions[comp.id] || "N/A";
    var statusClass = ver !== "N/A" ? "up-to-date" : "unknown";
    var statusText = ver !== "N/A" ? "Up to date" : "Unknown";

    html += '<tr>' +
      '<td><div class="version-component">' +
        '<div class="version-component-icon">' + comp.icon + '</div>' +
        '<span class="version-component-name">' + comp.name + '</span>' +
      '</div></td>' +
      '<td><span class="version-number">' + escapeHtml(ver) + '</span></td>' +
      '<td><span class="version-status ' + statusClass + '">' + statusText + '</span></td>' +
      '<td><button class="version-check-btn" onclick="checkUpdate(\'' + comp.id + '\', this)" ' +
        (ver === "N/A" ? 'disabled' : '') + '>Check Update</button></td>' +
      '</tr>';
  });

  html += '</tbody></table>';
  container.innerHTML = html;
}

function renderVersionsFallback() {
  var html = '<div style="padding: 24px; text-align: center; color: var(--text-secondary, #888);">';
  html += '<p style="font-size: 24px; margin: 0 0 12px;">&#9888;&#65039;</p>';
  html += '<p>Could not load version information.</p>';
  html += '<p style="font-size: 13px;">Make sure the backend server is running.</p>';
  html += '<button class="version-check-btn" onclick="loadVersions()" style="margin-top: 12px;">Retry</button>';
  html += '</div>';
  return html;
}

function checkUpdate(componentId, btn) {
  btn.disabled = true;
  btn.textContent = "Checking...";

  fetch("/api/system/check-updates", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ component: componentId })
  })
    .then(function(r) { return r.json(); })
    .then(function(result) {
      var row = btn.closest("tr");
      if (!row) return;
      var statusCell = row.querySelector(".version-status");
      if (statusCell) {
        if (result.update_available) {
          statusCell.className = "version-status update-available";
          statusCell.textContent = "Update: " + escapeHtml(result.latest);
        } else {
          statusCell.className = "version-status up-to-date";
          statusCell.textContent = "Up to date";
        }
      }
      btn.textContent = "Checked";
      setTimeout(function() {
        btn.disabled = false;
        btn.textContent = "Check Update";
      }, 3000);
    })
    .catch(function(e) {
      console.warn("[Versions] Check update failed:", e);
      btn.textContent = "Error";
      setTimeout(function() {
        btn.disabled = false;
        btn.textContent = "Check Update";
      }, 3000);
    });
}

// Load versions when the panel is shown
if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    (function(){ var __cb = loadVersions; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    loadVersions();
  }
}
