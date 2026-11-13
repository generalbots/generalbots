    function updateConnectionFields() {
        const type = document.getElementById("sourceType").value;
        const container = document.getElementById("connectionFields");

        const dbFields = `
        <div class="form-row">
            <div class="form-group">
                <label for="connHost" data-i18n="dashboards-host">Host</label>
                <input type="text" id="connHost" name="connection.host" placeholder="localhost"/>
            </div>
            <div class="form-group">
                <label for="connPort" data-i18n="dashboards-port">Port</label>
                <input type="number" id="connPort" name="connection.port" placeholder="5432"/>
            </div>
        </div>
        <div class="form-group">
            <label for="connDatabase" data-i18n="dashboards-database">Database</label>
            <input type="text" id="connDatabase" name="connection.database" required/>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label for="connUsername" data-i18n="dashboards-username">Username</label>
                <input type="text" id="connUsername" name="connection.username"/>
            </div>
            <div class="form-group">
                <label for="connPassword" data-i18n="dashboards-password">Password</label>
                <input type="password" id="connPassword" name="connection.password"/>
            </div>
        </div>
        <div class="form-group checkbox-group">
            <label>
                <input type="checkbox" id="connSsl" name="connection.ssl"/>
                <span data-i18n="dashboards-use-ssl">Use SSL</span>
            </label>
        </div>
    `;

        const apiFields = `
        <div class="form-group">
            <label for="connUrl" data-i18n="dashboards-api-url">API URL</label>
            <input type="url" id="connUrl" name="connection.url" required placeholder="https://api.example.com"/>
        </div>
        <div class="form-group">
            <label for="connApiKey" data-i18n="dashboards-api-key">API Key</label>
            <input type="password" id="connApiKey" name="connection.api_key"/>
        </div>
    `;

        const fileFields = `
        <div class="form-group">
            <label for="connUrl" data-i18n="dashboards-file-url">File URL or Path</label>
            <input type="text" id="connUrl" name="connection.url" required placeholder="/path/to/file.csv or https://..."/>
        </div>
    `;

        if (["postgresql", "mysql", "sqlserver", "mongodb"].includes(type)) {
            container.innerHTML = dbFields;
        } else if (["rest_api", "graphql_api"].includes(type)) {
            container.innerHTML = apiFields;
        } else if (["csv", "excel", "google_sheets"].includes(type)) {
            container.innerHTML = fileFields;
        } else if (["bigquery", "snowflake", "redshift"].includes(type)) {
            container.innerHTML = `
            <div class="form-group">
                <label for="connString" data-i18n="dashboards-connection-string">Connection String</label>
                <textarea id="connString" name="connection.connection_string" rows="3" required></textarea>
            </div>
        `;
        } else {
            container.innerHTML = "";
        }
    }

    function testDataSourceConnection() {
        const form = document.getElementById("addDataSourceForm");
        const formData = new FormData(form);
        const data = Object.fromEntries(formData.entries());

        htmx.ajax("POST", "/api/dashboards/sources/test", {
            target: "#connectionFields",
            swap: "beforeend",
            values: data,
        });
    }

    function escapeHtml(text) {
        if (!text) return "";
        return String(text)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    }

    function showNotification(message, type) {
        var isError = type === "error";
        var notification = document.createElement("div");
        notification.className = "notification " + (type || "info");
        notification.setAttribute("role", isError ? "alert" : "status");
        notification.setAttribute("aria-live", isError ? "assertive" : "polite");
        notification.setAttribute("aria-atomic", "true");
        notification.textContent = message;
        document.body.appendChild(notification);
        setTimeout(function () { notification.remove(); }, 3000);
    }

    document.querySelectorAll(".template-option").forEach((btn) => {
        btn.addEventListener("click", function () {
            document
                .querySelectorAll(".template-option")
                .forEach((b) => b.classList.remove("active"));
            this.classList.add("active");
            document.getElementById("selectedTemplate").value =
                this.dataset.template;
        });
    });

    document.addEventListener("keydown", function (e) {
        if (e.key === "Escape") {
            closeCreateDashboardModal();
            closeAddDataSourceModal();
            closeAddWidgetModal();
            closeEditWidgetModal();
            if (currentDashboardId && !editMode) {
                closeDashboardViewer();
            }
        }
    });
