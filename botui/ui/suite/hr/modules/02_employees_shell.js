/* HR app — module 2/6: shell, employees and tab loading (split from hr/hr.js for #1731) */

(function (app) {
'use strict';
const { fetchApi, nameParts, esc, openModal, closeModal, showFeedback } = app;

let currentTab = 'employees';
let employeesData = [];

// Linked lazily — provided by later modules, resolved on first call.
const loadPayroll = (...args) => app.loadPayroll(...args);
const loadBenefits = (...args) => app.loadBenefits(...args);
const loadTraining = (...args) => app.loadTraining(...args);
const loadReports = (...args) => app.loadReports(...args);
const loadRecruitment = (...args) => app.loadRecruitment(...args);
const postJob = (...args) => app.postJob(...args);
const exportEmployees = (...args) => app.exportEmployees(...args);
const runPayroll = (...args) => app.runPayroll(...args);
const startReview = (...args) => app.startReview(...args);
const addCourse = (...args) => app.addCourse(...args);
const viewJob = (...args) => app.viewJob(...args);
const loadAttendance = (...args) => app.loadAttendance(...args);
const loadPerformance = (...args) => app.loadPerformance(...args);

function bootHr() {
initTabs();
initEvents();
setupDelegation();
loadStats();
loadTab(currentTab);
}

// #1205/#1211 — row actions used inline `onclick="editEmployee(...)"` but
// those functions live inside this IIFE, so the handlers were never
// reachable (dead buttons), and one-shot DOMContentLoaded bindings are lost
// when the app is re-injected via HTMX. Delegate one listener on the
// document, guarded by a marker so repeated injections do not stack it.
function setupDelegation() {
if (document.body.dataset.hrDelegated === '1') return;
document.body.dataset.hrDelegated = '1';
document.addEventListener('click', (ev) => {
const el = ev.target && ev.target.closest ? ev.target : null;
if (!el) return;
const edit = el.closest('.btn-edit');
if (edit && edit.dataset.id) { editEmployee(edit.dataset.id); return; }
const term = el.closest('.btn-terminate');
if (term && term.dataset.id) { terminateEmployee(term.dataset.id); return; }
const view = el.closest('.btn-view');
if (view && view.dataset.id) { viewJob(view.dataset.id); return; }
const exp = el.closest('.btn-export');
if (exp) { showFeedback('Payslip export not available', 'error'); return; }
});
}

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
<button class="btn-sm btn-edit" data-id="${esc(e.id)}">Edit</button>
<button class="btn-sm btn-terminate" data-id="${esc(e.id)}">Terminate</button>
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

Object.assign(app, {
  bootHr,
  setupDelegation,
  initTabs,
  initEvents,
  loadStats,
  loadTab,
  loadEmployees,
  renderEmployees,
  getInitials,
  empStatusBadge,
  filterEmployees,
  editEmployee,
  handleEmpSubmit,
  terminateEmployee,
});
})(window.__gbHr);
