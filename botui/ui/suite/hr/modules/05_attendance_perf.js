/* HR app — module 5/6: attendance and performance views (split from hr/hr.js for #1731) */

(function (app) {
'use strict';
const { fetchApi, esc } = app;

let attendanceData = [];
let performanceData = {};

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

Object.assign(app, {
  loadAttendance,
  renderAttendanceTable,
  attStatusBadge,
  renderDeptSummary,
  renderAttendanceStats,
  loadPerformance,
  renderReviewCycles,
  cycleStatusBadge,
  renderGoals,
});
})(window.__gbHr);
