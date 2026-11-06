/* HR app — module 4/6: recruitment and payroll/training actions (split from hr/hr.js for #1731) */

(function (app) {
'use strict';
const { fetchApi, esc, showFeedback, loadStats, loadTraining, loadPayroll } = app;

// Linked lazily — loadPerformance is defined by a module loaded after this one.
const loadPerformance = (...args) => app.loadPerformance(...args);

let jobsData = [];

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
<button class="btn-outline btn-view" data-id="${esc(j.id)}">View</button>
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

Object.assign(app, {
  loadRecruitment,
  renderJobs,
  renderPipeline,
  postJob,
  viewJob,
  exportEmployees,
  runPayroll,
  startReview,
  addCourse,
});
})(window.__gbHr);
