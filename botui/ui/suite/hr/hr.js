(function() {
'use strict';
const API_BASE = '/api/hr';
let currentTab = 'employees';
let employeesData = [];
let jobsData = [];
let attendanceData = [];
let performanceData = {};

document.addEventListener('DOMContentLoaded', () => {
initTabs();
initEvents();
loadTab(currentTab);
});

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
}

async function loadTab(tab) {
try {
switch (tab) {
case 'employees': await loadEmployees(); break;
case 'recruitment': await loadRecruitment(); break;
case 'attendance': await loadAttendance(); break;
case 'performance': await loadPerformance(); break;
}
} catch (e) {
console.error('Failed to load ' + tab, e);
}
}

async function fetchApi(url, options) {
const res = await fetch(API_BASE + url, options);
if (!res.ok) throw new Error('API error: ' + res.status);
return res.json();
}

async function loadEmployees() {
const data = await fetchApi('/employees');
employeesData = data.employees || [];
renderEmployees(employeesData);
}

function renderEmployees(employees) {
const grid = document.getElementById('employeeGrid');
grid.innerHTML = employees.map((e, i) => `
<div class="employee-card">
<div class="avatar avatar-${(i % 5) + 1}">${getInitials(e.first_name, e.last_name)}</div>
<div class="emp-info">
<div class="emp-name">${esc(e.first_name)} ${esc(e.last_name)}</div>
<div class="emp-role">${esc(e.role)}</div>
<div class="emp-dept">${esc(e.department)}</div>
<span class="badge ${empStatusBadge(e.status)}">${esc(e.status)}</span>
</div>
<div class="emp-actions">
<button class="btn-sm btn-edit" onclick="editEmployee('${esc(e.id)}')">Edit</button>
<button class="btn-sm btn-terminate" onclick="terminateEmployee('${esc(e.id)}')">Terminate</button>
</div>
</div>
`).join('');
}

function getInitials(first, last) {
return ((first || '')[0] || '') + ((last || '')[0] || '');
}

function empStatusBadge(s) {
const map = { active: 'badge-success', on_leave: 'badge-warning', terminated: 'badge-danger' };
return map[(s || '').toLowerCase()] || 'badge-neutral';
}

function filterEmployees() {
const q = document.getElementById('empSearch').value.toLowerCase();
const dept = document.getElementById('empDeptFilter').value;
const status = document.getElementById('empStatusFilter').value;
let filtered = employeesData;
if (q) filtered = filtered.filter(e =>
(e.first_name + ' ' + e.last_name).toLowerCase().includes(q) || e.role.toLowerCase().includes(q)
);
if (dept) filtered = filtered.filter(e => e.department === dept);
if (status) filtered = filtered.filter(e => e.status === status);
renderEmployees(filtered);
}

function editEmployee(id) {
const emp = employeesData.find(e => e.id === id);
if (!emp) return;
document.getElementById('empModalTitle').textContent = 'Edit Employee';
document.getElementById('empFirstName').value = emp.first_name;
document.getElementById('empLastName').value = emp.last_name;
document.getElementById('empEmail').value = emp.email;
document.getElementById('empRole').value = emp.role;
document.getElementById('empDept').value = emp.department;
document.getElementById('empHireDate').value = emp.hire_date;
document.getElementById('empForm').dataset.editId = id;
openModal('empModal');
}

async function handleEmpSubmit(e) {
e.preventDefault();
const editId = e.target.dataset.editId;
const payload = {
first_name: document.getElementById('empFirstName').value,
last_name: document.getElementById('empLastName').value,
email: document.getElementById('empEmail').value,
role: document.getElementById('empRole').value,
department: document.getElementById('empDept').value,
hire_date: document.getElementById('empHireDate').value
};
try {
if (editId) {
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
try {
await fetchApi('/employees/' + id + '/terminate', { method: 'POST' });
loadEmployees();
} catch (e) {
console.error('Terminate failed', e);
}
}

async function loadRecruitment() {
const data = await fetchApi('/recruitment');
jobsData = data.jobs || [];
renderJobs(jobsData);
renderPipeline(data.pipeline || {});
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
console.log('Post new job');
}

function viewJob(id) {
console.log('View job:', id);
}

async function loadAttendance() {
const data = await fetchApi('/attendance');
attendanceData = data.records || [];
renderAttendanceTable(attendanceData);
renderDeptSummary(data.department_summary || []);
renderAttendanceStats(data.stats || {});
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
