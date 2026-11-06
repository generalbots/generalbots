/* HR app — module 3/6: payroll, benefits, training and report tables (split from hr/hr.js for #1731) */

(function (app) {
'use strict';
const { fetchApi, esc } = app;

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
      <td><button class="btn-sm btn-export">Export</button></td>
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

Object.assign(app, { loadPayroll, loadBenefits, loadTraining, loadReports });
})(window.__gbHr);
