// Tab switching
    function switchSecurityTab(tabName) {
        document.querySelectorAll(".security-tab").forEach((tab) => {
            tab.classList.remove("active");
        });
        document
            .querySelector(`.security-tab[data-tab="${tabName}"]`)
            .classList.add("active");

        document
            .querySelectorAll(".security-tab-content")
            .forEach((content) => {
                content.classList.remove("active");
            });
        document.getElementById(`tab-${tabName}`).classList.add("active");
    }

    // Compliance data loading — real data from the compliance engine
    async function loadComplianceChecks() {
        const body = document.getElementById("compliance-results-body");
        if (!body) return;
        body.innerHTML =
            '<tr id="compliance-loading-row"><td colspan="5" class="security-empty-state">' +
            '<div class="security-empty-icon">🔍</div>' +
            '<div class="security-empty-title">Scanning compliance posture</div>' +
            '<div class="security-empty-desc">Loading checks from the compliance engine…</div>' +
            "</td></tr>";

        try {
            const res = await fetch("/api/compliance/checks", {
                headers: { Accept: "application/json" },
            });
            if (!res.ok) {
                throw new Error(`Compliance API returned ${res.status}`);
            }
            const checks = await res.json();
            renderComplianceChecks(checks);
        } catch (err) {
            console.error("Failed to load compliance checks:", err);
            body.innerHTML =
                '<tr><td colspan="5" class="security-empty-state">' +
                '<div class="security-empty-icon">⚠️</div>' +
                '<div class="security-empty-title">Unable to load compliance checks</div>' +
                '<div class="security-empty-desc">' +
                String(err.message || err) +
                "</div></td></tr>";
        }
    }

    function statusToSeverity(status) {
        switch (String(status || "").toLowerCase()) {
            case "non_compliant":
            case "fail":
                return "critical";
            case "partial_compliance":
                return "high";
            case "in_progress":
            case "pending":
                return "medium";
            case "compliant":
            case "pass":
                return "low";
            default:
                return "info";
        }
    }

    function escapeHtml(value) {
        return String(value ?? "")
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    }

    function renderComplianceChecks(checks) {
        const body = document.getElementById("compliance-results-body");
        if (!body) return;

        if (!Array.isArray(checks) || checks.length === 0) {
            body.innerHTML =
                '<tr><td colspan="5" class="security-empty-state">' +
                '<div class="security-empty-icon">✅</div>' +
                '<div class="security-empty-title">No checks recorded yet</div>' +
                '<div class="security-empty-desc">Run a compliance scan to evaluate your security posture.</div>' +
                "</td></tr>";
            updateComplianceStats([]);
            return;
        }

        const rows = checks.map((check) => {
            const severity = statusToSeverity(check.status);
            const framework = escapeHtml(check.framework || "general");
            const statusLabel = escapeHtml(
                String(check.status || "unknown").replace(/_/g, " "),
            );
            const controlName = escapeHtml(
                check.control_name || check.control_id || framework,
            );
            const score = Number.isFinite(Number(check.score))
                ? Number(check.score)
                : null;
            const checkedAt = check.checked_at
                ? new Date(check.checked_at).toLocaleString()
                : "—";
            const description = check.notes
                ? escapeHtml(check.notes)
                : `Compliance check for framework "${framework}".`;
            const remediation =
                check.issues && check.issues.length > 0
                    ? check.issues
                          .map((i) => escapeHtml(i.remediation || i.title || ""))
                          .join("; ")
                    : null;

            return `<tr data-severity="${severity}" data-type="${framework}">
                <td>
                    <span class="security-severity ${severity}">
                        <span class="security-severity-dot"></span>
                        ${severity}
                    </span>
                </td>
                <td>
                    <div class="security-issue-type">
                        <div class="security-issue-icon security">
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                                <line x1="12" y1="9" x2="12" y2="13"></line>
                                <line x1="12" y1="17" x2="12.01" y2="17"></line>
                            </svg>
                        </div>
                        <div>
                            <div class="security-issue-title">${controlName}</div>
                            <div class="security-issue-category">${framework}</div>
                        </div>
                    </div>
                </td>
                <td>
                    <div class="security-file-path">${framework}</div>
                    <div class="security-file-line">Checked ${checkedAt}</div>
                </td>
                <td>
                    <div class="security-description">
                        ${description}
                        ${
                            score !== null
                                ? `<div class="security-code-snippet">Score: ${score.toFixed(1)}</div>`
                                : ""
                        }
                        ${
                            remediation
                                ? `<div class="security-code-snippet">Remediation: ${remediation}</div>`
                                : ""
                        }
                    </div>
                </td>
                <td>
                    <span class="security-status-badge ${severity}">${statusLabel}</span>
                </td>
            </tr>`;
        });

        body.innerHTML = rows.join("");
        updateComplianceStats(checks);
        filterComplianceResults();
    }

    // Compliance filtering
    function filterComplianceResults() {
        const severity = document.getElementById("filter-severity").value;
        const type = document.getElementById("filter-type").value;
        const search = document
            .getElementById("filter-search")
            .value.toLowerCase();
        const rows = document.querySelectorAll("#compliance-results-body tr");
        let visible = 0;

        rows.forEach((row) => {
            const rowSeverity = row.dataset.severity;
            const rowType = row.dataset.type;
            const text = row.textContent.toLowerCase();

            const matchSeverity =
                severity === "all" || rowSeverity === severity;
            const matchType = type === "all" || rowType === type;
            const matchSearch = !search || text.includes(search);

            if (matchSeverity && matchType && matchSearch) {
                row.style.display = "";
                visible++;
            } else {
                row.style.display = "none";
            }
        });

        document.getElementById("results-count").textContent =
            `${visible} issues found`;
    }

    function updateComplianceStats(checks) {
        const rows = document.querySelectorAll("#compliance-results-body tr");
        let stats = { critical: 0, high: 0, medium: 0, low: 0, info: 0 };

        rows.forEach((row) => {
            const severity = row.dataset.severity;
            if (stats.hasOwnProperty(severity)) {
                stats[severity]++;
            }
        });

        document.getElementById("stat-critical").textContent = stats.critical;
        document.getElementById("stat-high").textContent = stats.high;
        document.getElementById("stat-medium").textContent = stats.medium;
        document.getElementById("stat-low").textContent = stats.low;
        document.getElementById("stat-info").textContent = stats.info;

        const total =
            stats.critical + stats.high + stats.medium + stats.low + stats.info;
        const countEl = document.getElementById("results-count");
        if (countEl) {
            countEl.textContent = `${total} issues found`;
        }
    }

    // Run compliance scan via the real backend endpoint
    async function runComplianceScan() {
        const btn = document.getElementById("scan-btn");
        const original = btn ? btn.innerHTML : "";
        if (btn) {
            btn.disabled = true;
            btn.innerHTML =
                '<span class="scan-spinner"></span><span>Scanning…</span>';
        }
        try {
            const res = await fetch("/api/compliance/scan", { method: "POST" });
            if (!res.ok) {
                throw new Error(`Scan failed with status ${res.status}`);
            }
            const result = await res.json();
            console.log("Compliance scan result:", result);
            await loadComplianceChecks();
        } catch (err) {
            console.error("Compliance scan failed:", err);
            const body = document.getElementById("compliance-results-body");
            if (body) {
                body.innerHTML =
                    '<tr><td colspan="5" class="security-empty-state">' +
                    '<div class="security-empty-icon">⚠️</div>' +
                    '<div class="security-empty-title">Scan failed</div>' +
                    '<div class="security-empty-desc">' +
                    String(err.message || err) +
                    "</div></td></tr>";
            }
        } finally {
            if (btn) {
                btn.disabled = false;
                btn.innerHTML = original;
            }
        }
    }

    // Export the compliance report (CSV download)
    function exportComplianceReport() {
        window.location.href = "/api/compliance/export";
    }

    // Initialize on page load
    document.addEventListener("DOMContentLoaded", function () {
        loadComplianceChecks();
    });
