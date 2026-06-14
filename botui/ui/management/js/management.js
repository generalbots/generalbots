const API_BASE = '/api/management';

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  document.getElementById('user-email').textContent = localStorage.getItem('management_email') || '';
  await loadOrgs(token);
  await loadPlans(token);
});

async function loadOrgs(token) {
  try {
    const res = await fetch(`${API_BASE}/organizations`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { document.getElementById('orgs-list').innerHTML = '<p>Failed to load organizations</p>'; return; }
    const orgs = await res.json();
    renderOrgs(orgs);
  } catch (err) {
    document.getElementById('orgs-list').innerHTML = `<p>Error: ${err.message}</p>`;
  }
}

function renderOrgs(orgs) {
  const container = document.getElementById('orgs-list');
  if (!orgs || orgs.length === 0) {
    container.innerHTML = `
      <div class="saas-loading">
        <p>No organizations yet.</p>
        <button onclick="showNewOrgModal()" class="btn-primary">Create Your First Organization</button>
      </div>`;
    return;
  }
  container.innerHTML = orgs.map(o => `
    <div class="saas-org-card">
      <div class="saas-org-info">
        <h3>${escapeHtml(o.name)}</h3>
        <p>${o.plan} · <span class="saas-org-badge ${o.status}">${o.status}</span></p>
        ${o.vps_address ? `<p>VPS: ${o.vps_address}</p>` : ''}
      </div>
      <div class="saas-org-actions">
        <a href="${o.vps_address ? 'https://' + o.vps_address : '#'}" class="btn-secondary" ${o.vps_address ? 'target="_blank"' : 'disabled'}>Open</a>
        <button onclick="openBilling('${o.id}')" class="btn-text">Billing</button>
      </div>
    </div>
  `).join('');
}

async function loadPlans(token) {
  try {
    const res = await fetch(`${API_BASE}/plans`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) return;
    const data = await res.json();
    renderPlans(data.plans);
  } catch (err) {
    console.error('Failed to load plans:', err);
  }
}

function renderPlans(plans) {
  const grid = document.getElementById('plans-grid');
  if (!plans) return;
  grid.innerHTML = Object.entries(plans).map(([id, plan]) => `
    <div class="saas-plan-card">
      <h3>${escapeHtml(plan.name)}</h3>
      <div class="price">${plan.price.type === 'free' ? 'Free' : '$' + (plan.price.amount / 100).toFixed(2) + '/' + plan.price.period}</div>
      <ul>
        ${(plan.features || []).map(f => `<li>${escapeHtml(f)}</li>`).join('')}
      </ul>
    </div>
  `).join('');

  const select = document.getElementById('org-plan');
  if (select) {
    select.innerHTML = Object.entries(plans).map(([id, plan]) =>
      `<option value="${id}">${escapeHtml(plan.name)}</option>`
    ).join('');
  }
}

async function createOrg(e) {
  e.preventDefault();
  const token = requireAuth();
  const name = document.getElementById('org-name').value;
  const plan = document.getElementById('org-plan').value;
  const period = document.getElementById('org-period').value;
  const storage = parseFloat(document.getElementById('org-storage').value) || 0;

  try {
    const res = await fetch(`${API_BASE}/organizations`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ name, plan, period, storage_gb: storage, ai_addons: [] }),
    });
    if (!res.ok) { const err = await res.json(); alert(err.detail || 'Failed'); return; }
    const data = await res.json();
    if (data.checkout_url) {
      window.location.href = data.checkout_url;
    } else {
      hideNewOrgModal();
      await loadOrgs(token);
    }
  } catch (err) {
    alert('Error: ' + err.message);
  }
}

async function openBilling(orgId) {
  const token = requireAuth();
  try {
    const res = await fetch(`${API_BASE}/organizations/${orgId}/billing`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { alert('Failed to open billing portal'); return; }
    const data = await res.json();
    if (data.url) window.open(data.url, '_blank');
  } catch (err) {
    alert('Error: ' + err.message);
  }
}

function showNewOrgModal() {
  document.getElementById('new-org-modal').style.display = 'flex';
}

function hideNewOrgModal() {
  document.getElementById('new-org-modal').style.display = 'none';
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/management';
}

function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
