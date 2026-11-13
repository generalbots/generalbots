    let currentDashboardId = null;
    let editMode = false;

    function showCreateDashboardModal() {
        document.getElementById("createDashboardModal").style.display = "flex";
    }

    function closeCreateDashboardModal() {
        document.getElementById("createDashboardModal").style.display = "none";
        document.getElementById("createDashboardForm").reset();
    }

    function showAddDataSourceModal() {
        document.getElementById("addDataSourceModal").style.display = "flex";
    }

    function closeAddDataSourceModal() {
        document.getElementById("addDataSourceModal").style.display = "none";
        document.getElementById("addDataSourceForm").reset();
        document.getElementById("connectionFields").innerHTML = "";
    }

    function showAddWidgetModal() {
        document.getElementById("addWidgetModal").style.display = "flex";
    }

    function closeAddWidgetModal() {
        document.getElementById("addWidgetModal").style.display = "none";
    }

    async function selectWidgetType(type) {
        closeAddWidgetModal();
        if (!currentDashboardId) return;
        try {
            const resp = await fetch(`/api/dashboards/${currentDashboardId}/widgets`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    widget_type: type,
                    title: `New ${type.replace("_", " ")}`,
                    position: { x: 0, y: 0, width: 4, height: 2 },
                    config: {},
                    data_query: null,
                    style: null,
                }),
            });
            if (resp.ok) {
                showNotification("Widget added", "success");
                loadDashboard(currentDashboardId);
            } else {
                const body = await resp.json().catch(() => null);
                showNotification((body && body.error) || "Failed to add widget", "error");
            }
        } catch (e) {
            console.error("Failed to add widget:", e);
            showNotification("Failed to add widget", "error");
        }
    }

    function openDashboard(id, name) {
        currentDashboardId = id;
        document.getElementById("viewer-title").textContent = name;
        document.getElementById("dashboards-grid").style.display = "none";
        document.getElementById("dashboard-viewer").style.display = "block";
        loadDashboard(id);
    }

    async function loadDashboard(id) {
        try {
            const resp = await fetch(`/api/dashboards/${id}`);
            if (!resp.ok) throw new Error("HTTP " + resp.status);
            const dashboard = await resp.json();
            if (!dashboard) {
                document.getElementById("viewer-content").innerHTML =
                    '<p class="empty-message">Dashboard not found</p>';
                return;
            }
            renderWidgetGrid(dashboard.widgets || []);
        } catch (e) {
            console.error("Failed to load dashboard:", e);
            document.getElementById("viewer-content").innerHTML =
                '<p class="empty-message">Failed to load dashboard</p>';
        }
    }

    function renderWidgetGrid(widgets) {
        const container = document.getElementById("viewer-content");
        if (!container) return;
        if (!widgets.length) {
            container.innerHTML =
                '<div class="widget-grid empty-widgets"><p class="empty-message">No widgets yet — add one below</p></div>';
            return;
        }
        container.innerHTML =
            '<div class="widget-grid">' +
            widgets
                .map(
                    (w) =>
                        '<div class="widget" data-id="' +
                        w.id +
                        '" style="grid-column: span ' +
                        (w.position && w.position.width ? w.position.width : 4) +
                        '; grid-row: span ' +
                        (w.position && w.position.height ? w.position.height : 2) +
                        ';">' +
                        '<div class="widget-header">' +
                        '<h5>' +
                        escapeHtml(w.title || "Widget") +
                        "</h5>" +
                        '<div class="widget-actions">' +
                        '<button class="btn-icon btn-sm" title="Edit" onclick="editWidget(\'' +
                        w.id +
                        '\')">⚙️</button>' +
                        '<button class="btn-icon btn-sm" title="Remove" onclick="removeWidget(\'' +
                        w.id +
                        '\')">✕</button>' +
                        "</div>" +
                        "</div>" +
                        '<div class="widget-content">' +
                        renderWidgetContent(w) +
                        "</div>" +
                        "</div>",
                )
                .join("");
        if (window.htmx) htmx.process(container);
    }

    function renderWidgetContent(widget) {
        const icons = {
            line_chart: "📈",
            bar_chart: "📊",
            pie_chart: "🥧",
            area_chart: "📉",
            scatter_plot: "⚬",
            kpi: "🎯",
            table: "📋",
            gauge: "⏲️",
            map: "🗺️",
            text: "📝",
            image: "🖼️",
            iframe: "🔗",
        };
        return (
            '<div class="widget-placeholder">' +
            '<span class="widget-icon">' +
            (icons[widget.widget_type] || "📊") +
            "</span>" +
            '<span class="widget-type">' +
            escapeHtml(widget.widget_type || "") +
            "</span>" +
            "</div>"
        );
    }

    function closeDashboardViewer() {
        currentDashboardId = null;
        document.getElementById("dashboard-viewer").style.display = "none";
        document.getElementById("dashboards-grid").style.display = "grid";
    }

    function refreshDashboard() {
        if (currentDashboardId) {
            loadDashboard(currentDashboardId);
        }
    }

    async function populateEditDataSource() {
        const select = document.getElementById("editWidgetDataSource");
        if (!select) return;
        select.innerHTML = '<option value="">All Data Sources</option>';
        try {
            const resp = await fetch("/api/dashboards/sources");
            if (!resp.ok) return;
            const sources = await resp.json();
            (sources || []).forEach((s) => {
                const opt = document.createElement("option");
                opt.value = s.id;
                opt.textContent = s.name || s.id;
                select.appendChild(opt);
            });
        } catch (e) {
            console.error("Failed to load data sources:", e);
        }
    }

    async function editWidget(widgetId) {
        if (!currentDashboardId) return;
        try {
            const resp = await fetch(`/api/dashboards/${currentDashboardId}`);
            if (!resp.ok) throw new Error("HTTP " + resp.status);
            const dashboard = await resp.json();
            const widget = (dashboard && dashboard.widgets || []).find((w) => w.id === widgetId);
            if (!widget) {
                showNotification("Widget not found", "error");
                return;
            }
            await populateEditDataSource();
            document.getElementById("editWidgetId").value = widget.id;
            document.getElementById("editWidgetTitle").value = widget.title || "";
            document.getElementById("editWidgetType").value = widget.widget_type || "line_chart";
            const pos = widget.position || {};
            document.getElementById("editWidgetWidth").value = pos.width || 4;
            document.getElementById("editWidgetHeight").value = pos.height || 2;
            const cfg = widget.config || {};
            const chart = cfg.chart_config || {};
            const series = (chart.series || []);
            document.getElementById("editWidgetXAxis").value = chart.x_axis || (series[0] ? series[0].field : "");
            document.getElementById("editWidgetYAxis").value = chart.y_axis || (series[1] ? series[1].field : "");
            document.getElementById("editWidgetDataSource").value =
                (widget.data_query && widget.data_query.source_id) || "";
            const modal = document.getElementById("editWidgetModal");
            if (modal) modal.style.display = "flex";
        } catch (e) {
            console.error("Failed to load widget:", e);
            showNotification("Failed to load widget", "error");
        }
    }

    function closeEditWidgetModal() {
        const modal = document.getElementById("editWidgetModal");
        if (modal) modal.style.display = "none";
    }

    async function saveWidgetEdit() {
        const widgetId = document.getElementById("editWidgetId").value;
        if (!widgetId || !currentDashboardId) return;
        const sourceId = document.getElementById("editWidgetDataSource").value || null;
        const xAxis = document.getElementById("editWidgetXAxis").value.trim();
        const yAxis = document.getElementById("editWidgetYAxis").value.trim();
        const payload = {
            title: document.getElementById("editWidgetTitle").value.trim() || "Widget",
            position: {
                x: 0,
                y: 0,
                width: parseInt(document.getElementById("editWidgetWidth").value, 10) || 4,
                height: parseInt(document.getElementById("editWidgetHeight").value, 10) || 2,
            },
            config: {
                chart_config: {
                    x_axis: xAxis || null,
                    y_axis: yAxis || null,
                    series: xAxis ? [{ name: yAxis || xAxis, field: yAxis || xAxis }] : [],
                    legend_position: "bottom",
                    show_labels: true,
                    animations: true,
                },
                text_content: null,
                image_url: null,
                iframe_url: null,
            },
            data_query: sourceId
                ? { source_id: sourceId, query_type: "table", filters: [] }
                : null,
            style: null,
        };
        try {
            const resp = await fetch(`/api/dashboards/${currentDashboardId}/widgets/${widgetId}`, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            if (resp.ok) {
                closeEditWidgetModal();
                showNotification("Widget updated", "success");
                loadDashboard(currentDashboardId);
            } else {
                const body = await resp.json().catch(() => null);
                showNotification((body && body.error) || "Failed to update widget", "error");
            }
        } catch (e) {
            console.error("Failed to update widget:", e);
            showNotification("Failed to update widget", "error");
        }
    }

    function gbDashViewerConfirm(message, title) {
        if (window.GBModal && typeof window.GBModal.confirm === "function") {
            return new Promise((resolve) => {
                try {
                    window.GBModal.confirm(message, title || "Confirm", () => resolve(true), () => resolve(false));
                } catch (e) {
                    console.error("GBModal.confirm failed", e);
                    resolve(false);
                }
            });
        }
        return Promise.resolve(false);
    }

    async function removeWidget(widgetId) {
        if (!currentDashboardId || !(await gbDashViewerConfirm("Remove this widget?", "Confirm"))) return;
        try {
            const resp = await fetch(`/api/dashboards/${currentDashboardId}/widgets/${widgetId}`, {
                method: "DELETE",
            });
            if (resp.ok) {
                showNotification("Widget removed", "success");
                loadDashboard(currentDashboardId);
            } else {
                showNotification("Failed to remove widget", "error");
            }
        } catch (e) {
            console.error("Failed to remove widget:", e);
            showNotification("Failed to remove widget", "error");
        }
    }

    function updateDashboardData() {
        const range = document.getElementById("viewerDateRange").value;
        if (currentDashboardId) {
            htmx.ajax(
                "GET",
                `/api/dashboards/${currentDashboardId}?range=${range}`,
                {
                    target: "#viewer-content",
                    swap: "innerHTML",
                },
            );
        }
    }

    function editDashboard() {
        editMode = !editMode;
        const viewer = document.getElementById("viewer-content");
        if (editMode) {
            viewer.classList.add("edit-mode");
            showAddWidgetModal();
        } else {
            viewer.classList.remove("edit-mode");
        }
    }

    function shareDashboard() {
        if (currentDashboardId) {
            const url = `${window.location.origin}/dashboards/${currentDashboardId}`;
            navigator.clipboard.writeText(url).then(() => {
                showNotification(
                    "Dashboard link copied to clipboard!",
                    "success",
                );
            });
        }
    }

    function exportDashboard() {
        if (currentDashboardId) {
            window.open(
                `/api/dashboards/${currentDashboardId}/export?format=pdf`,
                "_blank",
            );
        }
    }
