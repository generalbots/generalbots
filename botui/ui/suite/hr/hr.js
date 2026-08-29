(function() {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("hr");
const API_BASE = '/api/hr';
let currentTab = 'employees';
let employeesData = [];
let jobsData = [];
let attendanceData = [];
let performanceData = {};

(function(){ var __cb = () => {
initTabs();
initEvents();
loadStats();
loadTab(currentTab);
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

function initTabs() {
document.querySelectorAll('.tab').forEach(tab => {
tab.addEventListener('click', () => {
document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
tab.classList.add('active');
const tabName = tab.dataset.tab;
document.getElementById('panel-' + tabName).classList.add('active');
currentTab = tabName;
loadTab(tabName);
});
});
}

function initEvents() {
document.getElementById('refreshBtn').addEventListener('click', () => loadTab(currentTab));
document.getElementById('addEmployeeBtn').addEventListener('click', () => {
document.getElementById('empModalTitle').textContent = 'Add Employee';
document.getElementById('empForm').reset();
openModal('empModal');
});
document.getElementById('closeEmpModal').addEventListener('click', () => closeModal('empModal'));
document.getElementById('cancelEmpBtn').addEventListener('click', () => closeModal('empModal'));
document.getElementById('empForm').addEventListener('submit', handleEmpSubmit);
document.getElementById('empSearch').addEventListener('input', filterEmployees);
document.getElementById('empDeptFilter').addEventListener('change', filterEmployees);
document.getElementById('empStatusFilter').addEventListener('change', filterEmployees);
document.getElementById('postJobBtn').addEventListener('click', postJob);
document.getElementById('exportEmployeesBtn').addEventListener('click', exportEmployees);
document.getElementById('runPayrollBtn').addEventListener('click', runPayroll);
document.getElementById('startReviewBtn').addEventListener('click', startReview);
document.getElementById('addCourseBtn').addEventListener('click', addCourse);
}

async function loadStats() {
  try {
    const [emp, rec, perf] = await Promise.allSettled([
      fetchApi('/employees'), fetchApi('/recruitment'), fetchApi('/performance')
    ]);
    const emps = emp.status === 'fulfilled' ? (emp.value.items || []) : [];
    const recs = rec.status === 'fulfilled' ? (rec.value.items || []) : [];
    const cycles = perf.status === 'fulfilled' ? (perf.value.review_cycles || []) : [];
    const openReqs = recs.filter(r => (r.status || '') === 'open').length;
    const onLeave = emps.filter(e => (e.status || '') === 'on_leave').length;
    const pendingReviews = cycles.filter(c => (c.status || '') === 'active').reduce((s, c) => s + ((c.total || 0) - (c.completed || 0)), 0);
    const set = (id, v) => { const el = document.getElementById(id); if (el) el.textContent = v; };
    set('hr-stat-headcount', emps.length);
    set('hr-stat-leave', onLeave);
    set('hr-stat-reqs', openReqs);
    set('hr-stat-reviews', pendingReviews);
    set('hr-stat-turnover', emps.length ? ((emps.filter(e => (e.status || '') === 'terminated').length / emps.length) * 100).toFixed(0) + '%' : '0%');
    set('hrOpenPositions', openReqs);
    set('hrUpdated', new Date().toLocaleString());
  } catch (e) { /* non-fatal */ }
}

async function loadTab(tab) {
try {
switch (tab) {
case 'employees': await loadEmployees(); break;
case 'recruitment': await loadRecruitment(); break;
case 'attendance': await loadAttendance(); break;
case 'performance': await loadPerformance(); break;
case 'payroll': await loadPayroll(); break;
case 'benefits': await loadBenefits(); break;
case 'training': await loadTraining(); break;
case 'reports': await loadReports(); break;
}
} catch (e) {
console.error('Failed to load ' + tab, e);
}
}

async function fetchApi(url, options) {
const res = await fetch(API_BASE + url, options);
if (res.status === 401) {
let tok = null;
try {
tok = (window.getGBAccessToken ? window.getGBAccessToken() : null) ||
localStorage.getItem('gb-access-token') || localStorage.getItem('management_token');
} catch (e) { tok = null; }
if (!tok) {
const loginUrl = window.GB_LOGIN_URL || '/login';
// Full URL — a relative redirect would resolve against the login host
// after auth and bounce back to login (login loop).
window.location.href = loginUrl + '?redirect=' + encodeURIComponent(window.location.href);
throw new Error('Not authenticated');
}
}
if (!res.ok) throw new Error('API error: ' + res.status);
return res.json();
}

function nameParts(name) {
const parts = (name || '').trim().split(/\s+/);
return { first: parts[0] || '', last: parts.slice(1).join(' ') };
}

async function loadEmployees() {
const data = await fetchApi('/employees');
employeesData = data.items || [];
renderEmployees(employeesData);
}

function renderEmployees(employees) {
const grid = document.getElementById('employeeGrid');
grid.innerHTML = employees.map((e, i) => {
const np = nameParts(e.name);
return `
<div class="employee-card">
<div class="avatar avatar-${(i % 5) + 1}">${getInitials(np.first, np.last)}</div>
<div class="emp-info">
<div class="emp-name">${esc(e.name)}</div>
<div class="emp-role">${esc(e.role)}</div>
<div class="emp-dept">${esc(e.department)}</div>
<span class="badge ${empStatusBadge(e.status)}">${esc(e.status)}</span>
</div>
<div class="emp-actions">
<button class="btn-sm btn-edit" onclick="editEmployee('${esc(e.id)}')">Edit</button>
<button class="btn-sm btn-terminate" onclick="terminateEmployee('${esc(e.id)}')">Terminate</button>
</div>
</div>
`;
}).join('');
}

function getInitials(first, last) {
return ((first || '')[0] || '') + ((last || '')[0] || '');
}

function empStatusBadge(s) {
const map = { active: 'badge-success', on_leave: 'badge-warning', terminated: 'badge-danger', interview: 'badge-info' };
return map[(s || '').toLowerCase()] || 'badge-neutral';
}

function filterEmployees() {
const q = document.getElementById('empSearch').value.toLowerCase();
const dept = document.getElementById('empDeptFilter').value;
const status = document.getElementById('empStatusFilter').value;
let filtered = employeesData;
if (q) filtered = filtered.filter(e => (e.name || '').toLowerCase().includes(q) || (e.role || '').toLowerCase().includes(q));
if (dept) filtered = filtered.filter(e => e.department === dept);
if (status) filtered = filtered.filter(e => e.status === status);
renderEmployees(filtered);
}

function editEmployee(id) {
const emp = employeesData.find(e => e.id === id);
if (!emp) return;
const np = nameParts(emp.name);
document.getElementById('empModalTitle').textContent = 'Edit Employee';
document.getElementById('empFirstName').value = np.first;
document.getElementById('empLastName').value = np.last;
document.getElementById('empEmail').value = emp.email || '';
document.getElementById('empRole').value = emp.role || '';
document.getElementById('empDept').value = emp.department || '';
document.getElementById('empHireDate').value = (emp.hired_at || '').slice(0, 10);
document.getElementById('empForm').dataset.editId = id;
openModal('empModal');
}

async function handleEmpSubmit(e) {
e.preventDefault();
const editId = e.target.dataset.editId;
const first = document.getElementById('empFirstName').value;
const last = document.getElementById('empLastName').value;
const payload = {
name: (first + ' ' + last).trim(),
email: document.getElementById('empEmail').value,
role: document.getElementById('empRole').value,
department: document.getElementById('empDept').value
};
try {
if (editId) {
payload.status = 'active';
await fetchApi('/employees/' + editId, {
method: 'PUT',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify(payload)
});
delete e.target.dataset.editId;
} else {
await fetchApi('/employees', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify(payload)
});
}
closeModal('empModal');
e.target.reset();
loadEmployees();
} catch (err) {
console.error('Save failed', err);
}
}

async function terminateEmployee(id) {
if (!confirm('Are you sure you want to terminate this employee?')) return;
const emp = employeesData.find(e => e.id === id);
if (!emp) return;
try {
await fetchApi('/employees/' + id, {
method: 'PUT',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify({
name: emp.name,
email: emp.email || '',
role: emp.role || '',
department: emp.department || '',
status: 'terminated'
})
});
loadEmployees();
} catch (e) {
console.error('Terminate failed', e);
}
}

async function loadRecruitment() {
const data = await fetchApi('/recruitment');
jobsData = (data.items || []).map(j => ({
id: j.id,
title: j.position,
department: j.department,
location: '—',
posted_date: (j.opened_at || '').slice(0, 10),
candidate_count: j.candidates || 0,
status: j.status || ''
}));
renderJobs(jobsData);
renderPipeline({});
}

function renderJobs(jobs) {
const list = document.getElementById('jobList');
list.innerHTML = jobs.map(j => `
<div class="job-card">
<div class="job-info">
<div class="job-title">${esc(j.title)}</div>
<div class="job-meta">
<span>${esc(j.department)}</span>
<span>${esc(j.location)}</span>
<span>Posted: ${esc(j.posted_date)}</span>
</div>
</div>
<div class="job-candidates">${j.candidate_count || 0} candidates</div>
<button class="btn-outline" onclick="viewJob('${esc(j.id)}')">View</button>
</div>
`).join('');
}

function renderPipeline(p) {
const grid = document.getElementById('pipelineGrid');
grid.innerHTML = `
<div class="pipeline-stage">
<div class="pipeline-label">Applied</div>
<div class="pipeline-count">${p.applied || 0}</div>
</div>
<div class="pipeline-stage">
<div class="pipeline-label">Screening</div>
<div class="pipeline-count">${p.screening || 0}</div>
</div>
<div class="pipeline-stage">
<div class="pipeline-label">Interview</div>
<div class="pipeline-count">${p.interview || 0}</div>
</div>
<div class="pipeline-stage">
<div class="pipeline-label">Hired</div>
<div class="pipeline-count">${p.hired || 0}</div>
</div>
`;
}

async function postJob() {
const position = prompt('Job title / position:');
if (!position) return;
const department = document.getElementById('empDeptFilter') ? document.getElementById('empDeptFilter').value : '';
try {
  const data = await fetchApi('/recruitment', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ position, department })
  });
  await loadRecruitment();
  loadStats();
  showFeedback('Job posted', 'success');
} catch (e) {
  console.error('Post job failed', e);
  showFeedback('Failed to post job', 'error');
}
}

function viewJob(id) {
const job = jobsData.find(j => j.id === id);
if (!job) return;
showFeedback('Job: ' + (job.title || '') + ' — ' + (job.candidate_count || 0) + ' candidates', 'success');
}

function showFeedback(msg, type) {
  let el = document.getElementById('hr-feedback');
  if (!el) { el = document.createElement('div'); el.id = 'hr-feedback'; el.style.cssText = 'position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;color:#fff;transition:opacity .3s'; document.body.appendChild(el); }
  el.textContent = msg;
  el.style.background = type === 'error' ? '#ef4444' : '#22c55e';
  el.style.opacity = '1';
  setTimeout(() => { el.style.opacity = '0'; }, 3000);
}

async function exportEmployees() {
  try {
    const data = await fetchApi('/employees');
    const items = (data.items || []).map(e => [e.name || '', e.email || '', e.role || '', e.department || '', e.status || '']);
    const header = ['name', 'email', 'role', 'department', 'status'];
    const csv = [header, ...items].map(r => r.map(v => '"' + String(v).replace(/"/g, '""') + '"').join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'employees.csv'; a.click();
    URL.revokeObjectURL(url);
  } catch (e) { console.error('Export failed', e); }
}

async function runPayroll() {
  try {
    await fetchApi('/payroll/run', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
    await loadPayroll();
    showFeedback('Payroll run completed', 'success');
  } catch (e) { console.error('Run payroll failed', e); showFeedback('Payroll run failed', 'error'); }
}

async function startReview() {
  const name = prompt('Review cycle name:', 'Q' + (new Date().getMonth() + 1) + ' ' + new Date().getFullYear() + ' review');
  if (!name) return;
  try {
    await fetchApi('/review-cycles', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name }) });
    await loadPerformance();
    loadStats();
    showFeedback('Review cycle started', 'success');
  } catch (e) { console.error('Start review failed', e); showFeedback('Failed to start review', 'error'); }
}

async function addCourse() {
  const course = prompt('Course name:');
  if (!course) return;
  const provider = prompt('Provider:', '') || '';
  const duration = prompt('Duration (e.g. 4h):', '') || '';
  try {
    await fetchApi('/training', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ course, provider, duration, assigned: 0, completed: 0, status: 'open' }) });
    await loadTraining();
    showFeedback('Course added', 'success');
  } catch (e) { console.error('Add course failed', e); showFeedback('Failed to add course', 'error'); }
}

async function loadPayroll() {
  const data = await fetchApi('/payroll');
  const items = data.items || [];
  const fmt = n => (parseFloat(n) || 0).toLocaleString(undefined, { style: 'currency', currency: 'BRL' });
  document.getElementById('payrollBody').innerHTML = items.length ? items.map(r => `
    <tr>
      <td>${esc(r.period_label)}</td>
      <td>${r.employee_count || 0}</td>
      <td>${fmt(r.gross)}</td>
      <td>${fmt(r.net)}</td>
      <td>${fmt(r.taxes)}</td>
      <td><span class="badge badge-success">${esc(r.status)}</span></td>
      <td><button class="btn-sm" onclick="showFeedback('Payslip export not available','error')">Export</button></td>
    </tr>`).join('') : '<tr><td colspan="7" class="o365-empty">No payroll runs yet</td></tr>';
}

async function loadBenefits() {
  const data = await fetchApi('/benefits');
  const items = data.items || [];
  const fmt = n => (parseFloat(n) || 0).toLocaleString(undefined, { style: 'currency', currency: 'BRL' });
  document.getElementById('benefitsBody').innerHTML = items.length ? items.map(b => `
    <tr>
      <td>${esc(b.plan)}</td>
      <td>${esc(b.provider)}</td>
      <td>${esc(b.type)}</td>
      <td>${b.enrolled || 0}</td>
      <td>${fmt(b.monthly_cost)}</td>
      <td><span class="badge badge-${b.status === 'active' ? 'success' : 'neutral'}">${esc(b.status)}</span></td>
    </tr>`).join('') : '<tr><td colspan="6" class="o365-empty">No benefits enrolled</td></tr>';
}

async function loadTraining() {
  const data = await fetchApi('/training');
  const items = data.items || [];
  document.getElementById('trainingBody').innerHTML = items.length ? items.map(c => `
    <tr>
      <td>${esc(c.course)}</td>
      <td>${esc(c.provider)}</td>
      <td>${esc(c.duration)}</td>
      <td>${c.assigned || 0}</td>
      <td>${c.completed || 0}</td>
      <td><span class="badge badge-${c.status === 'open' ? 'info' : 'success'}">${esc(c.status)}</span></td>
    </tr>`).join('') : '<tr><td colspan="6" class="o365-empty">No training courses</td></tr>';
}

async function loadReports() {
  const data = await fetchApi('/reports');
  const fmtPct = n => (parseFloat(n) || 0).toFixed(1) + '%';
  const dept = (data.departments || []).map(d => `${esc(d.department)}: ${d.count}`).join('<br>') || 'No data';
  document.getElementById('hrReportDept').innerHTML = dept;
  document.getElementById('hrReportAttrition').textContent = data.attrition_rate ? fmtPct(data.attrition_rate) : '0.0%';
  document.getElementById('hrReportTimeToHire').textContent = data.avg_time_to_hire_days ? data.avg_time_to_hire_days + ' days' : '0.0 days';
  document.getElementById('hrReportComp').innerHTML = `Active: ${data.active || 0}<br>Terminated: ${data.terminated || 0}<br>Total: ${data.total || 0}`;
}

async function loadAttendance() {
const data = await fetchApi('/attendance');
attendanceData = (data.items || []).map(r => ({
employee: r.employee_id || '—',
department: '—',
clock_in: (r.clock_in || '').slice(0, 16).replace('T', ' ') || '--',
clock_out: (r.clock_out || '').slice(0, 16).replace('T', ' ') || '--',
hours: r.hours_worked ? Number(r.hours_worked) : 0,
status: r.clock_in ? 'present' : 'absent'
}));
renderAttendanceTable(attendanceData);
renderDeptSummary([]);
renderAttendanceStats({});
}

function renderAttendanceTable(records) {
const tbody = document.querySelector('#attendanceTable tbody');
tbody.innerHTML = records.map(r => `
<tr>
<td>${esc(r.employee)}</td>
<td>${esc(r.department)}</td>
<td>${esc(r.clock_in || '--')}</td>
<td>${esc(r.clock_out || '--')}</td>
<td>${r.hours ? r.hours.toFixed(1) + 'h' : '--'}</td>
<td><span class="badge ${attStatusBadge(r.status)}">${esc(r.status)}</span></td>
</tr>
`).join('');
}

function attStatusBadge(s) {
const map = { present: 'badge-success', late: 'badge-warning', absent: 'badge-danger', partial: 'badge-info' };
return map[(s || '').toLowerCase()] || 'badge-neutral';
}

function renderDeptSummary(depts) {
const grid = document.getElementById('deptSummaryGrid');
grid.innerHTML = depts.map(d => `
<div class="dept-card">
<div class="dept-name">${esc(d.department)}</div>
<div class="dept-meta">
<span>Present: ${d.present}</span>
<span>Absent: ${d.absent}</span>
<span>Rate: ${d.attendance_rate}%</span>
</div>
</div>
`).join('');
}

function renderAttendanceStats(stats) {
const container = document.getElementById('attendanceStats');
container.innerHTML = `
<div class="stat-item">
<div class="stat-label">Present Today</div>
<div class="stat-value">${stats.present || 0}</div>
</div>
<div class="stat-item">
<div class="stat-label">Absent Today</div>
<div class="stat-value">${stats.absent || 0}</div>
</div>
<div class="stat-item">
<div class="stat-label">On Leave</div>
<div class="stat-value">${stats.on_leave || 0}</div>
</div>
<div class="stat-item">
<div class="stat-label">Attendance Rate</div>
<div class="stat-value">${stats.attendance_rate || 0}%</div>
</div>
`;
}

async function loadPerformance() {
const data = await fetchApi('/performance');
performanceData = data;
renderReviewCycles(data.review_cycles || []);
renderGoals(data.goals || []);
}

function renderReviewCycles(cycles) {
const container = document.getElementById('reviewCycles');
container.innerHTML = cycles.map(c => `
<div class="review-card">
<div class="review-info">
<div class="review-title">${esc(c.name)}</div>
<div class="review-meta">${esc(c.start_date)} to ${esc(c.end_date)} | ${c.completed || 0}/${c.total || 0} completed</div>
</div>
<span class="badge ${cycleStatusBadge(c.status)}">${esc(c.status)}</span>
</div>
`).join('');
}

function cycleStatusBadge(s) {
const map = { active: 'badge-info', upcoming: 'badge-warning', completed: 'badge-success', draft: 'badge-neutral' };
return map[(s || '').toLowerCase()] || 'badge-neutral';
}

function renderGoals(goals) {
const grid = document.getElementById('goalsGrid');
grid.innerHTML = goals.map(g => {
const pct = g.completion || 0;
const cls = pct >= 75 ? 'high' : pct >= 40 ? 'medium' : 'low';
return `
<div class="goal-card">
<div class="goal-header">
<span class="goal-title">${esc(g.title)}</span>
<span class="goal-percent">${pct}%</span>
</div>
<div class="progress-bar">
<div class="progress-fill ${cls}" style="width: ${pct}%"></div>
</div>
<div class="goal-meta">${esc(g.employee)} | ${esc(g.due_date)}</div>
</div>
`;
}).join('');
}

function openModal(id) {
document.getElementById(id).classList.add('open');
}

function closeModal(id) {
document.getElementById(id).classList.remove('open');
}

function esc(str) {
if (str === null || str === undefined) return '';
const div = document.createElement('div');
div.textContent = String(str);
return div.innerHTML;
}
})();