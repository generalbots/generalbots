/* Dashboards app — split from dashboards/dashboards.js for #1731 */

  // DATA SOURCES
  // =============================================================================

  async function loadDataSources() {
    try {
      const response = await fetch("/api/dashboards/data-sources");
      if (response.ok) {
        const data = await response.json();
        state.dataSources = data.data_sources || [];
        renderDataSourcesList();
      }
    } catch (e) {
      console.error("Failed to load data sources:", e);
    }
  }

  function renderDataSourcesList() {
    const container = document.getElementById("data-sources-list");
    if (!container) return;

    if (state.dataSources.length === 0) {
      container.innerHTML = `
        <p class="empty-message">No data sources configured</p>
      `;
      return;
    }

    container.innerHTML = state.dataSources
      .map(
        (source) => `
      <div class="data-source-item" data-id="${source.id}">
        <span class="source-icon">${getSourceIcon(source.source_type)}</span>
        <div class="source-info">
          <span class="source-name">${dashEscapeHtml(source.name)}</span>
          <span class="source-type">${source.source_type}</span>
        </div>
        <button class="btn-icon btn-sm" onclick="removeDataSource('${source.id}')" title="Remove">✕</button>
      </div>
    `,
      )
      .join("");
  }

  function getSourceIcon(sourceType) {
    const icons = {
      postgresql: "🐘",
      mysql: "🐬",
      api: "🔌",
      csv: "📄",
      json: "📋",
      elasticsearch: "🔍",
      mongodb: "🍃",
    };
    return icons[sourceType] || "📊";
  }

  function showAddDataSourceModal() {
    const modal = document.getElementById("addDataSourceModal");
    if (modal) {
      modal.style.display = "flex";
      const form = modal.querySelector("form");
      if (form) form.reset();
    }
  }

  function closeAddDataSourceModal() {
    const modal = document.getElementById("addDataSourceModal");
    if (modal) {
      modal.style.display = "none";
    }
  }

  async function testDataSourceConnection() {
    const form = document.getElementById("addDataSourceForm");
    if (!form) return;

    const formData = new FormData(form);
    showNotification("Testing connection...", "info");

    try {
      const response = await fetch("/api/dashboards/data-sources/test", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          source_type: formData.get("source_type"),
          connection_string: formData.get("connection_string"),
        }),
      });

      if (response.ok) {
        showNotification("Connection successful!", "success");
      } else {
        showNotification("Connection failed", "error");
      }
    } catch (e) {
      showNotification("Connection test failed", "error");
    }
  }

  async function addDataSource(formData) {
    const data = {
      name: formData.get("name"),
      source_type: formData.get("source_type"),
      connection_string: formData.get("connection_string"),
    };

    try {
      const response = await fetch("/api/dashboards/data-sources", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      if (response.ok) {
        showNotification("Data source added", "success");
        closeAddDataSourceModal();
        loadDataSources();
      } else {
        showNotification("Failed to add data source", "error");
      }
    } catch (e) {
      console.error("Failed to add data source:", e);
      showNotification("Failed to add data source", "error");
    }
  }

  async function removeDataSource(sourceId) {
    if (!confirm("Remove this data source?")) return;

    try {
      const response = await fetch(`/api/dashboards/data-sources/${sourceId}`, {
        method: "DELETE",
      });

      if (response.ok) {
        showNotification("Data source removed", "success");
        loadDataSources();
      }
    } catch (e) {
      console.error("Failed to remove data source:", e);
    }
  }
