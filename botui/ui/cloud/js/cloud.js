const API_BASE = '/api/cloud';

const SIDEBAR_HTML = `<nav class="mgmt-sidebar" id="mgmt-sidebar">
  <div class="mgmt-logo">
    <a href="/cloud" class="mgmt-logo-mark">
      <div class="mgmt-logo-icon">\u{1f916}</div>
      <div>
        <div class="mgmt-logo-text">General Bots</div>
        <span class="mgmt-logo-sub">Cloud</span>
      </div>
    </a>
  </div>
  <nav class="mgmt-nav">
    <div class="mgmt-nav-section">Overview</div>
    <a href="/cloud/dashboard" class="mgmt-nav-link" data-page="dashboard"><span class="mgmt-nav-icon">\u{1f4ca}</span> Dashboard</a>
    <div class="mgmt-nav-section">Add-ons</div>
    <a href="/cloud/store" class="mgmt-nav-link" data-page="store"><span class="mgmt-nav-icon">\u{1f3ea}</span> Store</a>
    <div class="mgmt-nav-section">Account</div>
    <a href="/cloud/services" class="mgmt-nav-link" data-page="services"><span class="mgmt-nav-icon">\u{1f4e6}</span> My Services</a>
    <a href="/cloud/invoices" class="mgmt-nav-link" data-page="invoices"><span class="mgmt-nav-icon">\u{1f4c4}</span> Invoices</a>
    <a href="/cloud/payment-cards" class="mgmt-nav-link" data-page="cards"><span class="mgmt-nav-icon">\u{1f4b3}</span> Payment Cards</a>
    <a href="/cloud/profile" class="mgmt-nav-link" data-page="profile"><span class="mgmt-nav-icon">\u{1f464}</span> My Profile</a>
  </nav>
  <div class="mgmt-sidebar-footer">
    <div class="mgmt-user-chip">
      <div class="mgmt-avatar" id="sidebar-avatar">?</div>
      <span class="mgmt-user-email" id="sidebar-email">\u2026</span>
      <button class="mgmt-logout" onclick="doLogout()" title="Sign out">\u238b</button>
    </div>
  </div>
</nav>`;

document.addEventListener('DOMContentLoaded', () => {
  const token = requireAuth();

  // Remover sidebars antigas e injetar a nova
  const shell = document.querySelector('.mgmt-shell');
  if (shell) {
    document.querySelectorAll('.mgmt-sidebar').forEach(el => el.remove());
    const temp = document.createElement('div');
    temp.innerHTML = SIDEBAR_HTML;
    const newSidebar = temp.firstElementChild;
    shell.insertBefore(newSidebar, shell.firstChild);

    // Marcar link ativo
    const path = window.location.pathname;
    newSidebar.querySelectorAll('.mgmt-nav-link').forEach(a => {
      const href = a.getAttribute('href');
      if (href && path.startsWith(href.split('?')[0])) {
        a.classList.add('active');
      }
    });
    // Preencher email
    const emailEl = document.getElementById('sidebar-email');
    if (emailEl) emailEl.textContent = localStorage.getItem('management_email') || '';
  }

  const userEmail = document.getElementById('user-email');
  if (userEmail) userEmail.textContent = localStorage.getItem('management_email') || '';
  try { loadOrgs(token).catch(() => {}); } catch (_) {}
  try { loadPlans(token).catch(() => {}); } catch (_) {}
});

async function loadOrgs(token) {
  const container = document.getElementById('orgs-list');
  if (!container) return;
  try {
    const res = await fetch(`${API_BASE}/organizations`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { container.innerHTML = '<p>Failed to load organizations</p>'; return; }
    const orgs = await res.json();
    renderOrgs(orgs);
  } catch (err) {
    container.innerHTML = `<p>Error: ${err.message}</p>`;
  }
}

function renderOrgs(orgs) {
  const container = document.getElementById('orgs-list');
  if (!container || !orgs || orgs.length === 0) {
    if (container) container.innerHTML = '<div class="saas-loading"><p>No organizations yet.</p><button onclick="showNewOrgModal()" class="btn-primary">Create Your First Organization</button></div>';
    return;
  }
  container.innerHTML = orgs.map(o => `
    <div class="saas-org-card">
      <div class="saas-org-info">
        <h3>${escapeHtml(o.name)}</h3>
        <p>${o.plan} <span class="saas-org-badge ${o.status}">${o.status}</span></p>
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
  const grid = document.getElementById('plans-grid');
  if (!grid) return;
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
  if (!grid || !plans) return;
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
    if (data.checkout_url) window.location.href = data.checkout_url;
    else { hideNewOrgModal(); await loadOrgs(token); }
  } catch (err) { alert('Error: ' + err.message); }
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
  } catch (err) { alert('Error: ' + err.message); }
}

function showNewOrgModal() { const m = document.getElementById('new-org-modal'); if (m) m.style.display = 'flex'; }
function hideNewOrgModal() { const m = document.getElementById('new-org-modal'); if (m) m.style.display = 'none'; }
function doLogout() { localStorage.removeItem('management_token'); localStorage.removeItem('management_email'); window.location.href = '/cloud'; }
function escapeHtml(str) { if (!str) return ''; const d = document.createElement('div'); d.textContent = str; return d.innerHTML; }
function getToken() { return localStorage.getItem('management_token'); }
function requireAuth() { const t = getToken(); if (!t) window.location.href = '/cloud/login'; return t; }
