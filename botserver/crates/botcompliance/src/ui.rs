use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::DbPool;

pub async fn handle_compliance_dashboard_page(State(_pool): State<Arc<DbPool>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Compliance Dashboard</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 1400px; margin: 0 auto; padding: 24px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.header h1 { font-size: 28px; color: #1a1a1a; }
.btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-primary:hover { background: #0052a3; }
.stats-row { display: grid; grid-template-columns: repeat(5, 1fr); gap: 16px; margin-bottom: 24px; }
.stat-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.stat-value { font-size: 32px; font-weight: 600; }
.stat-value.green { color: #2e7d32; }
.stat-value.yellow { color: #f9a825; }
.stat-value.red { color: #c62828; }
.stat-label { font-size: 13px; color: #666; margin-top: 4px; }
.stat-change { font-size: 12px; margin-top: 8px; }
.stat-change.positive { color: #2e7d32; }
.stat-change.negative { color: #c62828; }
.tabs { display: flex; gap: 4px; margin-bottom: 24px; border-bottom: 1px solid #e0e0e0; }
.tab { padding: 12px 24px; background: none; border: none; cursor: pointer; font-size: 14px; color: #666; border-bottom: 2px solid transparent; }
.tab.active { color: #0066cc; border-bottom-color: #0066cc; }
.content-grid { display: grid; grid-template-columns: 2fr 1fr; gap: 24px; }
.section { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); margin-bottom: 24px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.section-title { font-size: 18px; font-weight: 600; }
.framework-card { display: flex; align-items: center; justify-content: space-between; padding: 16px; border: 1px solid #e0e0e0; border-radius: 8px; margin-bottom: 12px; cursor: pointer; }
.framework-card:hover { background: #f8f9fa; }
.framework-info { display: flex; align-items: center; gap: 16px; }
.framework-icon { width: 48px; height: 48px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 600; }
.framework-gdpr { background: #e3f2fd; color: #1565c0; }
.framework-soc2 { background: #f3e5f5; color: #7b1fa2; }
.framework-iso { background: #e8f5e9; color: #2e7d32; }
.framework-hipaa { background: #fff3e0; color: #ef6c00; }
.framework-pci { background: #fce4ec; color: #c2185b; }
.framework-name { font-weight: 600; font-size: 16px; }
.framework-meta { font-size: 13px; color: #666; }
.score-badge { padding: 8px 16px; border-radius: 20px; font-weight: 600; font-size: 14px; }
.score-high { background: #e8f5e9; color: #2e7d32; }
.score-medium { background: #fff3e0; color: #ef6c00; }
.score-low { background: #ffebee; color: #c62828; }
.issue-item { display: flex; align-items: flex-start; gap: 12px; padding: 12px 0; border-bottom: 1px solid #f0f0f0; }
.issue-item:last-child { border-bottom: none; }
.issue-severity { width: 8px; height: 8px; border-radius: 50%; margin-top: 6px; flex-shrink: 0; }
.severity-critical { background: #c62828; }
.severity-high { background: #ef6c00; }
.severity-medium { background: #f9a825; }
.severity-low { background: #66bb6a; }
.issue-content { flex: 1; }
.issue-title { font-weight: 500; font-size: 14px; margin-bottom: 4px; }
.issue-meta { font-size: 12px; color: #666; }
.empty-state { text-align: center; padding: 40px; color: #666; }
</style>
</head>
<body>
<div class="container">
<div class="header">
<h1>Compliance Dashboard</h1>
<button class="btn btn-primary" onclick="runAudit()">Run Compliance Check</button>
</div>
<div class="stats-row">
<div class="stat-card">
<div class="stat-value green" id="overallScore">--</div>
<div class="stat-label">Overall Score</div>
<div class="stat-change positive">+2.5% from last month</div>
</div>
<div class="stat-card">
<div class="stat-value" id="totalControls">0</div>
<div class="stat-label">Controls Checked</div>
</div>
<div class="stat-card">
<div class="stat-value green" id="compliantControls">0</div>
<div class="stat-label">Compliant</div>
</div>
<div class="stat-card">
<div class="stat-value yellow" id="partialControls">0</div>
<div class="stat-label">Partial</div>
</div>
<div class="stat-card">
<div class="stat-value red" id="openIssues">0</div>
<div class="stat-label">Open Issues</div>
</div>
</div>
<div class="tabs">
<button class="tab active" data-view="overview">Overview</button>
<button class="tab" data-view="frameworks">Frameworks</button>
<button class="tab" data-view="issues">Issues</button>
<button class="tab" data-view="audit-log">Audit Log</button>
<button class="tab" data-view="training">Training</button>
</div>
<div class="content-grid">
<div>
<div class="section">
<div class="section-header">
<h2 class="section-title">Compliance Frameworks</h2>
<button class="btn" style="padding: 6px 12px; font-size: 12px;" onclick="addFramework()">+ Add Framework</button>
</div>
<div id="frameworksList">
<div class="framework-card" onclick="openFramework('gdpr')">
<div class="framework-info">
<div class="framework-icon framework-gdpr">GDPR</div>
<div>
<div class="framework-name">GDPR</div>
<div class="framework-meta">General Data Protection Regulation • 12 controls</div>
</div>
</div>
<span class="score-badge score-high">95%</span>
</div>
<div class="framework-card" onclick="openFramework('soc2')">
<div class="framework-info">
<div class="framework-icon framework-soc2">SOC2</div>
<div>
<div class="framework-name">SOC 2 Type II</div>
<div class="framework-meta">Service Organization Control • 24 controls</div>
</div>
</div>
<span class="score-badge score-high">92%</span>
</div>
<div class="framework-card" onclick="openFramework('iso27001')">
<div class="framework-info">
<div class="framework-icon framework-iso">ISO</div>
<div>
<div class="framework-name">ISO 27001</div>
<div class="framework-meta">Information Security Management • 18 controls</div>
</div>
</div>
<span class="score-badge score-medium">78%</span>
</div>
</div>
</div>
<div class="section">
<div class="section-header">
<h2 class="section-title">Recent Audit Activity</h2>
<a href="/suite/compliance/audit-log" style="color: #0066cc; font-size: 14px; text-decoration: none;">View All →</a>
</div>
<div id="auditActivity">
<div class="empty-state">No recent audit activity</div>
</div>
</div>
</div>
<div>
<div class="section">
<div class="section-header">
<h2 class="section-title">Open Issues</h2>
<a href="/suite/compliance/issues" style="color: #0066cc; font-size: 14px; text-decoration: none;">View All →</a>
</div>
<div id="issuesList">
<div class="issue-item">
<div class="issue-severity severity-critical"></div>
<div class="issue-content">
<div class="issue-title">Data retention policy needs update</div>
<div class="issue-meta">GDPR • Due in 5 days</div>
</div>
</div>
<div class="issue-item">
<div class="issue-severity severity-high"></div>
<div class="issue-content">
<div class="issue-title">Access review overdue for 3 users</div>
<div class="issue-meta">SOC 2 • Due in 2 days</div>
</div>
</div>
<div class="issue-item">
<div class="issue-severity severity-medium"></div>
<div class="issue-content">
<div class="issue-title">Security training incomplete</div>
<div class="issue-meta">ISO 27001 • Due in 14 days</div>
</div>
</div>
</div>
</div>
<div class="section">
<div class="section-header">
<h2 class="section-title">Upcoming Reviews</h2>
</div>
<div id="upcomingReviews">
<div class="issue-item">
<div class="issue-content">
<div class="issue-title">Quarterly Access Review</div>
<div class="issue-meta">Jan 31, 2025</div>
</div>
</div>
<div class="issue-item">
<div class="issue-content">
<div class="issue-title">Annual Security Assessment</div>
<div class="issue-meta">Feb 15, 2025</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>
<div id="framework-modal" class="modal-overlay">
<div class="modal">
<div class="modal-header">
<h3>New Compliance Framework</h3>
<button class="modal-close" onclick="closeFrameworkModal()">&times;</button>
</div>
<div class="modal-body">
<div class="form-group">
<label>Name (LGPD, GDPR, SOC 2, ISO 27001, PCI-DSS, custom)</label>
<input type="text" id="fw-name" placeholder="e.g. LGPD">
</div>
<div class="form-group">
<label>Version</label>
<input type="text" id="fw-version" placeholder="1.0.0">
</div>
<div class="form-group">
<label>Description</label>
<textarea id="fw-description" rows="3" placeholder="Describe the framework scope..."></textarea>
</div>
</div>
<div class="modal-footer">
<button class="btn" onclick="closeFrameworkModal()">Cancel</button>
<button class="btn btn-primary" onclick="submitFramework()">Create Framework</button>
</div>
</div>
</div>
<style>
.modal-overlay { display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 1000; align-items: center; justify-content: center; }
.modal-overlay.visible { display: flex; }
.modal { background: white; border-radius: 12px; width: 480px; max-width: 90vw; box-shadow: 0 8px 30px rgba(0,0,0,0.2); }
.modal-header { display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid #e0e0e0; }
.modal-header h3 { font-size: 16px; }
.modal-close { background: none; border: none; font-size: 22px; cursor: pointer; color: #666; }
.modal-body { padding: 20px; }
.modal-footer { display: flex; justify-content: flex-end; gap: 8px; padding: 16px 20px; border-top: 1px solid #e0e0e0; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 13px; color: #555; margin-bottom: 6px; }
.form-group input, .form-group textarea { width: 100%; padding: 10px 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; box-sizing: border-box; }
</style>
<script>
document.querySelectorAll('.tab').forEach(tab => {
tab.addEventListener('click', () => {
document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
tab.classList.add('active');
loadView(tab.dataset.view);
});
});

async function loadDashboard() {
try {
const response = await fetch('/api/compliance/report');
const report = await response.json();
if (report) {
document.getElementById('overallScore').textContent = Math.round(report.overall_score || 0) + '%';
document.getElementById('totalControls').textContent = report.total_controls_checked || 0;
document.getElementById('compliantControls').textContent = report.compliant_controls || 0;
document.getElementById('openIssues').textContent = report.total_issues || 0;
}
} catch (e) {
console.error('Failed to load dashboard:', e);
}
}

function closeFrameworkModal() {
document.getElementById('framework-modal').classList.remove('visible');
}

function loadView(view) {
switch(view) {
case 'issues':
window.location = '/suite/compliance/issues';
break;
case 'audit-log':
window.location = '/suite/compliance/audit-log';
break;
case 'training':
window.location = '/suite/compliance/training';
break;
}
}

function runAudit() {
if (confirm('Run a full compliance check? This may take a few minutes.')) {
fetch('/api/compliance/checks', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify({ framework: 'gdpr' })
}).then(() => {
alert('Compliance check started');
loadDashboard();
});
}
}

function openFramework(name) {
window.location = `/suite/compliance/framework/${encodeURIComponent(name)}`;
}

function addFramework() {
document.getElementById('framework-modal').classList.add('visible');
}

async function submitFramework() {
const name = document.getElementById('fw-name').value.trim();
const version = document.getElementById('fw-version').value.trim() || '1.0.0';
const description = document.getElementById('fw-description').value.trim();
if (!name) { alert('Framework name is required'); return; }
try {
const response = await fetch('/api/compliance/frameworks', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify({ name, version, description, framework_key: name.toLowerCase().replace(/\s+/g, '_') })
});
if (!response.ok) { const err = await response.json().catch(() => ({})); throw new Error(err.error || 'Failed to create framework'); }
const fw = await response.json();
document.getElementById('framework-modal').classList.remove('visible');
loadFrameworks();
openFramework(fw.name);
} catch (e) { alert(e.message); }
}

async function loadFrameworks() {
try {
const response = await fetch('/api/compliance/frameworks');
const frameworks = await response.json();
const list = document.getElementById('frameworksList');
if (!frameworks || frameworks.length === 0) {
list.innerHTML = '<div class="empty-state">No frameworks configured. Click \"+ Add Framework\" to create one.</div>';
return;
}
list.innerHTML = frameworks.map(fw => `
<div class="framework-card" onclick="openFramework('${fw.name}')">
<div class="framework-info">
<div class="framework-icon framework-gdpr">${(fw.name || '?').substring(0, 4).toUpperCase()}</div>
<div>
<div class="framework-name">${fw.name}</div>
<div class="framework-meta">v${fw.version || '1.0.0'} • ${fw.controls_count || 0} controls</div>
</div>
</div>
<span class="score-badge ${fw.status === 'archived' ? 'score-low' : 'score-medium'}">${fw.status || 'active'}</span>
</div>
`).join('');
} catch (e) { console.error('Failed to load frameworks:', e); }
}

loadFrameworks();
loadDashboard();
</script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_compliance_issues_page(State(_pool): State<Arc<DbPool>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Compliance Issues</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 1200px; margin: 0 auto; padding: 24px; }
.back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.header h1 { font-size: 24px; }
.btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.filters { display: flex; gap: 12px; margin-bottom: 24px; }
.search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; }
.filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
.issues-table { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); overflow: hidden; }
.table-header { display: grid; grid-template-columns: 40px 1fr 120px 120px 120px 100px; padding: 16px 20px; background: #f9f9f9; font-weight: 600; font-size: 13px; color: #666; border-bottom: 1px solid #e0e0e0; }
.table-row { display: grid; grid-template-columns: 40px 1fr 120px 120px 120px 100px; padding: 16px 20px; border-bottom: 1px solid #f0f0f0; align-items: center; cursor: pointer; }
.table-row:hover { background: #f8f9fa; }
.severity-dot { width: 10px; height: 10px; border-radius: 50%; }
.severity-critical { background: #c62828; }
.severity-high { background: #ef6c00; }
.severity-medium { background: #f9a825; }
.severity-low { background: #66bb6a; }
.issue-title { font-weight: 500; }
.issue-framework { font-size: 12px; color: #666; margin-top: 4px; }
.status-badge { padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 500; }
.status-open { background: #ffebee; color: #c62828; }
.status-in-progress { background: #fff3e0; color: #ef6c00; }
.status-resolved { background: #e8f5e9; color: #2e7d32; }
.empty-state { text-align: center; padding: 60px; color: #666; }
</style>
</head>
<body>
<div class="container">
<a href="/suite/compliance" class="back-link">← Back to Compliance</a>
<div class="header">
<h1>Compliance Issues</h1>
<button class="btn btn-primary" onclick="createIssue()">+ Report Issue</button>
</div>
<div class="filters">
<input type="text" class="search-box" placeholder="Search issues..." id="searchInput">
<select class="filter-select" id="severityFilter">
<option value="">All Severities</option>
<option value="critical">Critical</option>
<option value="high">High</option>
<option value="medium">Medium</option>
<option value="low">Low</option>
</select>
<select class="filter-select" id="statusFilter">
<option value="">All Status</option>
<option value="open">Open</option>
<option value="in_progress">In Progress</option>
<option value="resolved">Resolved</option>
</select>
<select class="filter-select" id="frameworkFilter">
<option value="">All Frameworks</option>
<option value="gdpr">GDPR</option>
<option value="soc2">SOC 2</option>
<option value="iso27001">ISO 27001</option>
<option value="hipaa">HIPAA</option>
</select>
</div>
<div class="issues-table">
<div class="table-header">
<span></span>
<span>Issue</span>
<span>Framework</span>
<span>Status</span>
<span>Due Date</span>
<span>Assignee</span>
</div>
<div id="issuesList">
<div class="empty-state">Loading issues...</div>
</div>
</div>
</div>
<script>
async function loadIssues() {
try {
const response = await fetch('/api/compliance/issues');
const issues = await response.json();
renderIssues(issues);
} catch (e) {
console.error('Failed to load issues:', e);
document.getElementById('issuesList').innerHTML = '<div class="empty-state">Failed to load issues</div>';
}
}

function renderIssues(issues) {
const list = document.getElementById('issuesList');
if (!issues || issues.length === 0) {
list.innerHTML = '<div class="empty-state">No compliance issues found</div>';
return;
}

list.innerHTML = issues.map(i => `
<div class="table-row" onclick="openIssue('${i.id}')">
<div class="severity-dot severity-${i.severity || 'medium'}"></div>
<div>
<div class="issue-title">${i.title}</div>
<div class="issue-framework">${i.description ? i.description.substring(0, 60) + '...' : ''}</div>
</div>
<span>${i.framework || '-'}</span>
<span class="status-badge status-${(i.status || 'open').replace(' ', '-')}">${i.status || 'Open'}</span>
<span>${i.due_date ? new Date(i.due_date).toLocaleDateString() : '-'}</span>
<span>${i.assigned_to || 'Unassigned'}</span>
</div>
`).join('');
}

function openIssue(id) {
window.location = `/suite/compliance/issues/${id}`;
}

function createIssue() {
window.location = '/suite/compliance/issues/new';
}

loadIssues();
</script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_compliance_issue_detail_page(
    State(_pool): State<Arc<DbPool>>,
    Path(issue_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Compliance Issue</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
.container {{ max-width: 900px; margin: 0 auto; padding: 24px; }}
.back-link {{ color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }}
.issue-card {{ background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); margin-bottom: 24px; }}
.issue-header {{ display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 20px; }}
.issue-title {{ font-size: 24px; font-weight: 600; margin-bottom: 12px; }}
.issue-meta {{ display: flex; gap: 16px; flex-wrap: wrap; }}
.meta-item {{ font-size: 13px; color: #666; }}
.severity-badge {{ padding: 4px 12px; border-radius: 12px; font-size: 12px; font-weight: 500; }}
.severity-critical {{ background: #ffebee; color: #c62828; }}
.severity-high {{ background: #fff3e0; color: #ef6c00; }}
.severity-medium {{ background: #fff8e1; color: #f9a825; }}
.severity-low {{ background: #e8f5e9; color: #2e7d32; }}
.issue-description {{ line-height: 1.7; color: #444; margin-bottom: 20px; }}
.section {{ margin-top: 24px; padding-top: 24px; border-top: 1px solid #e0e0e0; }}
.section-title {{ font-size: 16px; font-weight: 600; margin-bottom: 12px; }}
.btn {{ padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }}
.btn-primary {{ background: #0066cc; color: white; }}
.btn-success {{ background: #2e7d32; color: white; }}
.btn-outline {{ background: white; border: 1px solid #ddd; color: #333; }}
.actions {{ display: flex; gap: 12px; }}
.remediation-box {{ background: #f9f9f9; border-radius: 8px; padding: 16px; line-height: 1.6; }}
</style>
</head>
<body>
<div class="container">
<a href="/suite/compliance/issues" class="back-link">← Back to Issues</a>
<div class="issue-card">
<div class="issue-header">
<div>
<h1 class="issue-title" id="issueTitle">Loading...</h1>
<div class="issue-meta">
<span class="severity-badge severity-medium" id="issueSeverity">Medium</span>
<span class="meta-item" id="issueFramework">Framework: -</span>
<span class="meta-item" id="issueStatus">Status: Open</span>
<span class="meta-item" id="issueDue">Due: -</span>
</div>
</div>
<div class="actions">
<button class="btn btn-success" onclick="resolveIssue()">Mark Resolved</button>
<button class="btn btn-outline" onclick="editIssue()">Edit</button>
</div>
</div>
<div class="issue-description" id="issueDescription">
Loading issue details...
</div>
<div class="section">
<h3 class="section-title">Remediation Steps</h3>
<div class="remediation-box" id="issueRemediation">
No remediation steps provided.
</div>
</div>
<div class="section">
<h3 class="section-title">Assignment</h3>
<p id="issueAssignee">Unassigned</p>
</div>
</div>
</div>
<script>
const issueId = '{issue_id}';

async function loadIssue() {{
try {{
const response = await fetch(`/api/compliance/issues`);
const issues = await response.json();
const issue = issues.find(i => i.id === issueId);
if (issue) {{
document.getElementById('issueTitle').textContent = issue.title;
document.getElementById('issueDescription').textContent = issue.description || 'No description provided.';
document.getElementById('issueRemediation').textContent = issue.remediation || 'No remediation steps provided.';

const severityEl = document.getElementById('issueSeverity');
severityEl.textContent = (issue.severity || 'medium').charAt(0).toUpperCase() + (issue.severity || 'medium').slice(1);
severityEl.className = `severity-badge severity-${{issue.severity || 'medium'}}`;

document.getElementById('issueFramework').textContent = `Framework: ${{issue.framework || '-'}}`;
document.getElementById('issueStatus').textContent = `Status: ${{issue.status || 'Open'}}`;
document.getElementById('issueDue').textContent = issue.due_date ? `Due: ${{new Date(issue.due_date).toLocaleDateString()}}` : 'Due: -';
document.getElementById('issueAssignee').textContent = issue.assigned_to || 'Unassigned';
}}
}} catch (e) {{
console.error('Failed to load issue:', e);
}}
}}

async function resolveIssue() {{
if (!confirm('Mark this issue as resolved?')) return;
try {{
await fetch(`/api/compliance/issues/${{issueId}}`, {{
method: 'PUT',
headers: {{ 'Content-Type': 'application/json' }},
body: JSON.stringify({{ status: 'resolved' }})
}});
window.location = '/suite/compliance/issues';
}} catch (e) {{
alert('Failed to update issue');
}}
}}

function editIssue() {{
window.location = `/suite/compliance/issues/${{issueId}}/edit`;
}}

loadIssue();
</script>
</body>
</html>"#);
    Html(html)
}

/// `GET /suite/compliance/framework/:name`
///
/// Framework configuration page: controls catalog editor, evidence attachment
/// and coverage display. Driven by the `/api/compliance/frameworks/*` API.
pub async fn handle_compliance_framework_page(
    State(_pool): State<Arc<DbPool>>,
    Path(framework_name): Path<String>,
) -> Html<String> {
    let fw_name = framework_name.replace('+', " ");
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Framework Configuration</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 1200px; margin: 0 auto; padding: 24px; }
.back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 12px; }
.header h1 { font-size: 24px; }
.meta { color: #666; font-size: 14px; }
.btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-outline { background: white; border: 1px solid #ddd; color: #333; }
.btn-danger { background: #fff; border: 1px solid #fca5a5; color: #dc2626; }
.stats-row { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 24px; }
.stat-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.stat-value { font-size: 28px; font-weight: 600; }
.stat-value.green { color: #2e7d32; }
.stat-label { font-size: 13px; color: #666; margin-top: 4px; }
.section { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); margin-bottom: 24px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.section-title { font-size: 18px; font-weight: 600; }
.controls-table { width: 100%; border-collapse: collapse; }
.controls-table th, .controls-table td { text-align: left; padding: 12px; border-bottom: 1px solid #f0f0f0; font-size: 14px; }
.controls-table th { color: #666; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; background: #fafafa; }
.badge { padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 500; }
.badge-active { background: #e8f5e9; color: #2e7d32; }
.badge-draft { background: #fff3e0; color: #ef6c00; }
.badge-approved { background: #e8f5e9; color: #2e7d32; }
.badge-mandatory { background: #f3e5f5; color: #7b1fa2; }
.badge-optional { background: #e3f2fd; color: #1565c0; }
.evidence-item { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border: 1px solid #eee; border-radius: 8px; margin-top: 6px; font-size: 13px; }
.evidence-meta { color: #666; font-size: 12px; }
.empty-state { text-align: center; padding: 40px; color: #666; }
.modal-overlay { display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 1000; align-items: center; justify-content: center; }
.modal-overlay.visible { display: flex; }
.modal { background: white; border-radius: 12px; width: 520px; max-width: 92vw; box-shadow: 0 8px 30px rgba(0,0,0,0.2); }
.modal-header { display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid #e0e0e0; }
.modal-close { background: none; border: none; font-size: 22px; cursor: pointer; color: #666; }
.modal-body { padding: 20px; }
.modal-footer { display: flex; justify-content: flex-end; gap: 8px; padding: 16px 20px; border-top: 1px solid #e0e0e0; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 13px; color: #555; margin-bottom: 6px; }
.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 10px 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; box-sizing: border-box; }
</style>
</head>
<body>
<div class="container">
<a href="/suite/compliance" class="back-link">← Back to Compliance</a>
<div class="header">
<div>
<h1 id="fwTitle">Loading framework...</h1>
<div class="meta" id="fwMeta"></div>
</div>
<div style="display:flex;gap:8px">
<button class="btn btn-outline" onclick="exportCsv()">Export CSV</button>
<button class="btn" onclick="archiveFramework()">Archive</button>
</div>
</div>
<div class="stats-row">
<div class="stat-card"><div class="stat-value" id="statTotal">0</div><div class="stat-label">Total Controls</div></div>
<div class="stat-card"><div class="stat-value green" id="statCovered">0</div><div class="stat-label">Controls With Evidence</div></div>
<div class="stat-card"><div class="stat-value green" id="statCoverage">0%</div><div class="stat-label">Coverage</div></div>
</div>
<div class="section">
<div class="section-header">
<h2 class="section-title">Controls</h2>
<button class="btn btn-primary" style="padding:8px 14px;font-size:13px" onclick="showControlModal()">+ Add Control</button>
</div>
<div id="controlsList">
<div class="empty-state">Loading controls...</div>
</div>
</div>
</div>

<div class="modal-overlay" id="control-modal">
<div class="modal">
<div class="modal-header"><h3>Add Control</h3><button class="modal-close" onclick="hideModal('control-modal')">&times;</button></div>
<div class="modal-body">
<div class="form-group"><label>Control ID</label><input type="text" id="ctl-id" placeholder="e.g. CC6.1"></div>
<div class="form-group"><label>Title</label><input type="text" id="ctl-title" placeholder="Control title"></div>
<div class="form-group"><label>Category</label><input type="text" id="ctl-category" placeholder="e.g. Security"></div>
<div class="form-group"><label>Description</label><textarea id="ctl-desc" rows="2"></textarea></div>
<div class="form-group"><label>Mandatory</label><select id="ctl-mandatory"><option value="true">Mandatory</option><option value="false">Optional</option></select></div>
</div>
<div class="modal-footer"><button class="btn" onclick="hideModal('control-modal')">Cancel</button><button class="btn btn-primary" onclick="createControl()">Add Control</button></div>
</div>
</div>

<div class="modal-overlay" id="evidence-modal">
<div class="modal">
<div class="modal-header"><h3>Attach Evidence</h3><button class="modal-close" onclick="hideModal('evidence-modal')">&times;</button></div>
<div class="modal-body">
<div class="form-group"><label>File path (drive artifact)</label><input type="text" id="ev-path" placeholder="docs/policies/access-control.pdf"></div>
<div class="form-group"><label>Description</label><input type="text" id="ev-desc" placeholder="What this artifact proves"></div>
<div class="form-group"><label>Type</label><select id="ev-type"><option value="artifact">Artifact</option><option value="policy">Policy</option><option value="procedure">Procedure</option><option value="report">Report</option><option value="screenshot">Screenshot</option><option value="log">Log</option><option value="certificate">Certificate</option></select></div>
</div>
<div class="modal-footer"><button class="btn" onclick="hideModal('evidence-modal')">Cancel</button><button class="btn btn-primary" onclick="attachEvidence()">Attach</button></div>
</div>
</div>

<script>
const frameworkName = '__FW_NAME__';
let framework = null;

async function loadFramework() {
try {
const res = await fetch('/api/compliance/frameworks');
if (!res.ok) throw new Error('HTTP ' + res.status);
const frameworks = await res.json();
framework = (frameworks || []).find(f => f.name === frameworkName || f.framework_key === frameworkName.toLowerCase().replace(/\s+/g, '_'));
if (!framework) throw new Error('Framework not found');
document.getElementById('fwTitle').textContent = framework.name;
document.getElementById('fwMeta').textContent = 'v' + (framework.version || '1.0.0') + ' • ' + (framework.framework_key || '') + ' • ' + (framework.status || 'active');
await loadFrameworkDetail(framework.id);
} catch (e) {
document.getElementById('fwTitle').textContent = 'Framework not found';
document.getElementById('controlsList').innerHTML = '<div class="empty-state">' + e.message + '</div>';
}
}

async function loadFrameworkDetail(id) {
const res = await fetch('/api/compliance/frameworks/' + id);
if (!res.ok) throw new Error('HTTP ' + res.status);
framework = await res.json();
document.getElementById('statTotal').textContent = framework.total_controls || 0;
document.getElementById('statCovered').textContent = framework.controls_with_evidence || 0;
document.getElementById('statCoverage').textContent = (framework.coverage_percent || 0) + '%';
renderControls(framework.controls || []);
}

function renderControls(controls) {
const list = document.getElementById('controlsList');
if (!controls.length) { list.innerHTML = '<div class="empty-state">No controls yet. Add the first control for this framework.</div>'; return; }
list.innerHTML = '<table class="controls-table"><thead><tr><th>ID</th><th>Title</th><th>Category</th><th>Type</th><th>Evidence</th><th>Actions</th></tr></thead><tbody>'
+ controls.map(c => `
<tr>
<td><strong>${c.control_id}</strong></td>
<td>${c.title}${c.description ? '<br><small class="meta">' + c.description + '</small>' : ''}</td>
<td>${c.category || '-'}</td>
<td><span class="badge ${c.is_mandatory ? 'badge-mandatory' : 'badge-optional'}">${c.is_mandatory ? 'Mandatory' : 'Optional'}</span></td>
<td>
<span class="badge ${c.has_evidence ? 'badge-approved' : 'badge-draft'}">${c.has_evidence ? 'Covered' : 'No evidence'}</span>
${(c.evidence || []).map(e => `<div class="evidence-item"><span>📎 ${e.file_path}</span><span class="evidence-meta">${e.status}${e.approved_by ? ' • approved' : ''} <button class="btn btn-outline" style="padding:2px 8px;font-size:11px" onclick="approveEvidence('${e.id}')">Approve</button></span></div>`).join('')}
</td>
<td><button class="btn btn-outline" style="padding:4px 10px;font-size:12px" onclick="showEvidenceModal('${c.id}')">+ Evidence</button></td>
</tr>`).join('') + '</tbody></table>';
}

function showControlModal() { document.getElementById('control-modal').classList.add('visible'); }
function hideModal(id) { document.getElementById(id).classList.remove('visible'); }
let evidenceControlId = null;
function showEvidenceModal(controlId) { evidenceControlId = controlId; document.getElementById('evidence-modal').classList.add('visible'); }

async function createControl() {
const control_id = document.getElementById('ctl-id').value.trim();
const title = document.getElementById('ctl-title').value.trim();
if (!control_id || !title) { alert('Control ID and title are required'); return; }
const body = {
framework_id: framework.id,
control_id,
title,
category: document.getElementById('ctl-category').value.trim() || null,
description: document.getElementById('ctl-desc').value.trim() || null,
is_mandatory: document.getElementById('ctl-mandatory').value === 'true'
};
const res = await fetch('/api/compliance/controls', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
if (!res.ok) { const err = await res.json().catch(() => ({})); alert(err.error || 'Failed to create control'); return; }
hideModal('control-modal');
document.getElementById('ctl-id').value = ''; document.getElementById('ctl-title').value = ''; document.getElementById('ctl-desc').value = '';
await loadFrameworkDetail(framework.id);
}

async function attachEvidence() {
const file_path = document.getElementById('ev-path').value.trim();
if (!file_path || !evidenceControlId) { alert('File path is required'); return; }
const body = {
control_id: evidenceControlId,
file_path,
description: document.getElementById('ev-desc').value.trim() || null,
evidence_type: document.getElementById('ev-type').value
};
const res = await fetch('/api/compliance/evidence/attach', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
if (!res.ok) { const err = await res.json().catch(() => ({})); alert(err.error || 'Failed to attach evidence'); return; }
hideModal('evidence-modal');
document.getElementById('ev-path').value = ''; document.getElementById('ev-desc').value = '';
await loadFrameworkDetail(framework.id);
}

async function approveEvidence(evidenceId) {
const res = await fetch('/api/compliance/evidence/' + evidenceId + '/approve', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({}) });
if (!res.ok) { alert('Failed to approve evidence'); return; }
await loadFrameworkDetail(framework.id);
}

async function archiveFramework() {
if (!confirm('Archive this framework? Controls are preserved but it stops appearing as active.')) return;
const res = await fetch('/api/compliance/frameworks/' + framework.id + '/archive', { method: 'POST' });
if (!res.ok) { alert('Failed to archive framework'); return; }
window.location = '/suite/compliance';
}

function exportCsv() {
window.location = '/api/compliance/frameworks/' + framework.id + '/export.csv';
}

loadFramework();
</script>
</body>
</html>"#;
    let html = html.replace("__FW_NAME__", &fw_name);
    Html(html)
}

pub fn configure_compliance_ui_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/suite/compliance", get(handle_compliance_dashboard_page))
        .route("/suite/compliance/issues", get(handle_compliance_issues_page))
        .route("/suite/compliance/issues/:id", get(handle_compliance_issue_detail_page))
        .route("/suite/compliance/framework/:name", get(handle_compliance_framework_page))
}
