(function() {
    var checks = [];
    var issues = [];
    var auditLog = [];
    var risks = [];
    var training = [];
    var currentTab = 'dashboard';

    function showFeedback(msg, type) {
        var el = document.getElementById('compliance-feedback');
        if (!el) {
            el = document.createElement('div');
            el.id = 'compliance-feedback';
            el.style.cssText = 'position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity 0.3s';
            document.body.appendChild(el);
        }
        el.textContent = msg;
        el.style.background = type === 'error' ? '#ef4444' : '#22c55e';
        el.style.color = '#fff';
        el.style.opacity = '1';
        setTimeout(function() { el.style.opacity = '0'; }, 3000);
    }

    function switchTab(tab) {
        currentTab = tab;
        document.querySelectorAll('.compliance-tab').forEach(function(t) { t.classList.remove('active'); });
        document.querySelectorAll('.compliance-panel').forEach(function(p) { p.classList.remove('active'); });
        var btn = document.querySelector('.compliance-tab[data-tab="' + tab + '"]');
        if (btn) btn.classList.add('active');
        var panel = document.getElementById('panel-' + tab);
        if (panel) panel.classList.add('active');
    }

    function updateStats() {
        var total = checks.length;
        var passed = checks.filter(function(c) { return c.status === 'passed'; }).length;
        var failed = checks.filter(function(c) { return c.status === 'failed'; }).length;
        var warnings = checks.filter(function(c) { return c.status === 'warning'; }).length;
        var score = total > 0 ? Math.round((passed / total) * 100) : 0;
        var statTotal = document.getElementById('stat-total');
        var statPassed = document.getElementById('stat-passed');
        var statFailed = document.getElementById('stat-failed');
        var statWarnings = document.getElementById('stat-warnings');
        var statScore = document.getElementById('stat-score');
        if (statTotal) statTotal.textContent = total;
        if (statPassed) statPassed.textContent = passed;
        if (statFailed) statFailed.textContent = failed;
        if (statWarnings) statWarnings.textContent = warnings;
        if (statScore) statScore.textContent = score + '%';
        renderDashboard();
    }

    function renderDashboard() {
        var frameworks = document.getElementById('frameworks-container');
        if (frameworks) {
            var fwList = [
                { name: 'ISO 27001', key: 'iso27001' },
                { name: 'SOC 2', key: 'soc2' },
                { name: 'GDPR', key: 'gdpr' },
                { name: 'LGPD', key: 'lgpd' }
            ];
            frameworks.innerHTML = fwList.map(function(fw) {
                var fwChecks = checks.filter(function(c) { return c.framework === fw.key; });
                var fwPassed = fwChecks.filter(function(c) { return c.status === 'passed'; }).length;
                var fwTotal = fwChecks.length;
                var pct = fwTotal > 0 ? Math.round((fwPassed / fwTotal) * 100) : 0;
                var colorClass = pct >= 80 ? 'green' : pct >= 50 ? 'yellow' : 'red';
                return '<div class="compliance-framework">'
                    + '<div class="compliance-framework-name">' + fw.name + '</div>'
                    + '<div class="compliance-framework-score">' + pct + '% compliant (' + fwPassed + '/' + fwTotal + ')</div>'
                    + '<div class="compliance-framework-bar"><div class="compliance-framework-bar-fill ' + colorClass + '" style="width:' + pct + '%"></div></div>'
                    + '</div>';
            }).join('');
        }

        var activity = document.getElementById('activity-container');
        if (activity) {
            var recent = auditLog.slice(-5).reverse();
            if (recent.length === 0) {
                activity.innerHTML = '<div class="compliance-empty">No recent activity</div>';
            } else {
                activity.innerHTML = recent.map(function(a) {
                    return '<div class="compliance-activity-item">'
                        + '<span class="compliance-badge ' + (a.type || 'info') + '">' + (a.type || 'info') + '</span>'
                        + '<span>' + (a.description || '') + '</span>'
                        + '<span class="compliance-activity-time">' + (a.timestamp || '') + '</span>'
                        + '</div>';
                }).join('');
            }
        }

        var scoreDisplay = document.getElementById('score-display');
        if (scoreDisplay) {
            var total = checks.length;
            var passed = checks.filter(function(c) { return c.status === 'passed'; }).length;
            var score = total > 0 ? Math.round((passed / total) * 100) : 0;
            var colorClass = score >= 80 ? 'green' : score >= 50 ? 'yellow' : 'red';
            scoreDisplay.innerHTML = '<div class="compliance-score-circle ' + colorClass + '">' + score + '%</div>'
                + '<div class="compliance-score-label">Overall Compliance Score</div>';
        }

        var chart = document.getElementById('score-chart');
        if (chart) {
            var passed = checks.filter(function(c) { return c.status === 'passed'; }).length;
            var failed = checks.filter(function(c) { return c.status === 'failed'; }).length;
            var warnings = checks.filter(function(c) { return c.status === 'warning'; }).length;
            var maxVal = Math.max(passed, failed, warnings, 1);
            chart.innerHTML = '<div class="compliance-chart-col">'
                + '<div class="compliance-chart-value">' + passed + '</div>'
                + '<div class="compliance-chart-bar-fill green" style="height:' + (passed / maxVal * 100) + '%"></div>'
                + '<div class="compliance-chart-label">Passed</div>'
                + '</div>'
                + '<div class="compliance-chart-col">'
                + '<div class="compliance-chart-value">' + failed + '</div>'
                + '<div class="compliance-chart-bar-fill red" style="height:' + (failed / maxVal * 100) + '%"></div>'
                + '<div class="compliance-chart-label">Failed</div>'
                + '</div>'
                + '<div class="compliance-chart-col">'
                + '<div class="compliance-chart-value">' + warnings + '</div>'
                + '<div class="compliance-chart-bar-fill yellow" style="height:' + (warnings / maxVal * 100) + '%"></div>'
                + '<div class="compliance-chart-label">Warnings</div>'
                + '</div>';
        }
    }

    function renderChecks() {
        var tbody = document.getElementById('checks-tbody');
        if (!tbody) return;
        if (checks.length === 0) {
            tbody.innerHTML = '<tr><td colspan="5" class="compliance-empty">No checks found</td></tr>';
            return;
        }
        tbody.innerHTML = checks.map(function(c) {
            return '<tr>'
                + '<td>' + (c.id || '') + '</td>'
                + '<td>' + (c.name || '') + '</td>'
                + '<td><span class="compliance-badge ' + (c.framework || '') + '">' + (c.framework || '') + '</span></td>'
                + '<td><span class="compliance-badge ' + (c.status || '') + '">' + (c.status || '') + '</span></td>'
                + '<td><button class="compliance-action-btn primary" onclick="window._compliance.runCheck(\'' + (c.id || '') + '\')">Run</button></td>'
                + '</tr>';
        }).join('');
    }

    function renderIssues() {
        var tbody = document.getElementById('issues-tbody');
        if (!tbody) return;
        if (issues.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6" class="compliance-empty">No issues found</td></tr>';
            return;
        }
        tbody.innerHTML = issues.map(function(iss) {
            return '<tr>'
                + '<td>' + (iss.id || '') + '</td>'
                + '<td>' + (iss.title || '') + '</td>'
                + '<td><span class="compliance-badge ' + (iss.severity || '') + '">' + (iss.severity || '') + '</span></td>'
                + '<td><span class="compliance-badge ' + (iss.status || '') + '">' + (iss.status || '') + '</span></td>'
                + '<td>' + (iss.assignee || '') + '</td>'
                + '<td>' + (iss.created_at || '') + '</td>'
                + '</tr>';
        }).join('');
    }

    function renderAuditLog() {
        var tbody = document.getElementById('audit-tbody');
        if (!tbody) return;
        if (auditLog.length === 0) {
            tbody.innerHTML = '<tr><td colspan="4" class="compliance-empty">No audit entries</td></tr>';
            return;
        }
        tbody.innerHTML = auditLog.slice().reverse().map(function(a) {
            return '<tr>'
                + '<td>' + (a.timestamp || '') + '</td>'
                + '<td><span class="compliance-badge ' + (a.type || 'info') + '">' + (a.type || 'info') + '</span></td>'
                + '<td>' + (a.description || '') + '</td>'
                + '<td>' + (a.user || '') + '</td>'
                + '</tr>';
        }).join('');
    }

    function renderRisks() {
        var container = document.getElementById('risks-cards');
        if (!container) return;
        if (risks.length === 0) {
            container.innerHTML = '<div class="compliance-empty">No risks recorded</div>';
            return;
        }
        container.innerHTML = risks.map(function(r) {
            var score = (r.likelihood || 1) * (r.impact || 1);
            var riskClass = 'risk-' + score;
            return '<div class="compliance-risk-card">'
                + '<h4>' + (r.title || '') + '</h4>'
                + '<p>' + (r.description || '') + '</p>'
                + '<div class="compliance-risk-card-meta">'
                + '<span class="compliance-badge ' + riskClass + '">Score: ' + score + '</span>'
                + '<span class="compliance-badge ' + (r.status || 'open') + '">' + (r.status || 'open') + '</span>'
                + '</div>'
                + '</div>';
        }).join('');
    }

    function renderTraining() {
        var container = document.getElementById('training-list');
        if (!container) return;
        if (training.length === 0) {
            container.innerHTML = '<div class="compliance-empty">No training records</div>';
            return;
        }
        container.innerHTML = training.map(function(t) {
            return '<div class="compliance-training-card">'
                + '<div class="compliance-training-info">'
                + '<h4>' + (t.title || '') + '</h4>'
                + '<p>' + (t.assignee || '') + ' - Due: ' + (t.due_date || '') + '</p>'
                + '</div>'
                + '<span class="compliance-badge ' + (t.status || 'not-started') + '">' + (t.status || 'not-started') + '</span>'
                + '</div>';
        }).join('');
    }

    async function apiCall(url, options) {
        try {
            var resp = await fetch(url, options);
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            var text = await resp.text();
            return text ? JSON.parse(text) : null;
        } catch (e) {
            showFeedback('API error: ' + e.message, 'error');
            return null;
        }
    }

    async function loadChecks() {
        var data = await apiCall('/api/compliance/checks');
        if (data) { checks = Array.isArray(data) ? data : []; updateStats(); renderChecks(); }
    }

    async function loadIssues() {
        var data = await apiCall('/api/compliance/issues');
        if (data) { issues = Array.isArray(data) ? data : []; renderIssues(); }
    }

    async function loadAuditLog() {
        var data = await apiCall('/api/compliance/audit-log');
        if (data) { auditLog = Array.isArray(data) ? data : []; renderAuditLog(); renderDashboard(); }
    }

    async function loadRisks() {
        var data = await apiCall('/api/compliance/risks');
        if (data) { risks = Array.isArray(data) ? data : []; renderRisks(); }
    }

    async function loadTraining() {
        var data = await apiCall('/api/compliance/training');
        if (data) { training = Array.isArray(data) ? data : []; renderTraining(); }
    }

    async function runCheck(checkId) {
        var data = await apiCall('/api/compliance/checks/' + checkId + '/run', { method: 'POST' });
        if (data) {
            showFeedback('Check executed successfully', 'success');
            loadChecks();
            loadAuditLog();
        }
    }

    async function createIssue() {
        var title = document.getElementById('issue-title').value;
        var severity = document.getElementById('issue-severity').value;
        var assignee = document.getElementById('issue-assignee').value;
        var description = document.getElementById('issue-description').value;
        if (!title) { showFeedback('Title is required', 'error'); return; }
        var data = await apiCall('/api/compliance/issues', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ title: title, severity: severity, assignee: assignee, description: description, status: 'open' })
        });
        if (data) {
            hideModal('issue-modal');
            showFeedback('Issue created', 'success');
            loadIssues();
        }
    }

    async function updateIssue(issueId, status) {
        var data = await apiCall('/api/compliance/issues/' + issueId, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ status: status })
        });
        if (data) { showFeedback('Issue updated', 'success'); loadIssues(); }
    }

    async function createTraining() {
        var title = document.getElementById('training-title').value;
        var assignee = document.getElementById('training-assignee').value;
        var dueDate = document.getElementById('training-due').value;
        if (!title) { showFeedback('Title is required', 'error'); return; }
        var data = await apiCall('/api/compliance/training', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ title: title, assignee: assignee, due_date: dueDate, status: 'not-started' })
        });
        if (data) {
            hideModal('training-modal');
            showFeedback('Training record created', 'success');
            loadTraining();
        }
    }

    function exportReport() {
        var report = {
            generated_at: new Date().toISOString(),
            checks: checks,
            issues: issues,
            audit_log: auditLog,
            risks: risks,
            training: training
        };
        var blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
        var a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = 'compliance-report-' + new Date().toISOString().slice(0, 10) + '.json';
        a.click();
        URL.revokeObjectURL(a.href);
        showFeedback('Report exported', 'success');
    }

    function showModal(id) {
        var el = document.getElementById(id);
        if (el) el.classList.add('visible');
    }

    function hideModal(id) {
        var el = document.getElementById(id);
        if (el) el.classList.remove('visible');
    }

    function loadAll() {
        loadChecks();
        loadIssues();
        loadAuditLog();
        loadRisks();
        loadTraining();
    }

    document.querySelectorAll('.compliance-tab').forEach(function(tab) {
        tab.addEventListener('click', function() { switchTab(this.dataset.tab); });
    });

    window._compliance = {
        runCheck: runCheck,
        createIssue: createIssue,
        updateIssue: updateIssue,
        createTraining: createTraining,
        exportReport: exportReport,
        showModal: showModal,
        hideModal: hideModal,
        loadAll: loadAll
    };

    loadAll();
})();
