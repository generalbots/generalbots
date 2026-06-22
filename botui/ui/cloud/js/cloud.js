const API_BASE = '/api/cloud';

const SVG = {
  robot: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="4" y="7" width="16" height="13" rx="3"/><circle cx="9" cy="11" r="1.5" fill="currentColor"/><circle cx="15" cy="11" r="1.5" fill="currentColor"/><path d="M9 16c1.5 1.5 4.5 1.5 6 0"/><path d="M8 7V4M16 7V4"/></svg>',
  dashboard: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="8" height="8" rx="2"/><rect x="13" y="3" width="8" height="5" rx="2"/><rect x="3" y="13" width="8" height="8" rx="2"/><rect x="13" y="10" width="8" height="11" rx="2"/></svg>',
  vps: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="5" y="3" width="14" height="18" rx="2"/><path d="M9 7h6M9 11h6M9 15h6"/><path d="M9 19h6"/></svg>',
  gpu: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="4" y="6" width="16" height="12" rx="2"/><circle cx="10" cy="12" r="2"/><circle cx="17" cy="11" r="1"/><path d="M10 16v2M14 16v2"/></svg>',
  storage: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><ellipse cx="12" cy="6" rx="8" ry="3"/><path d="M4 6v6c0 1.66 3.58 3 8 3s8-1.34 8-3V6"/><path d="M4 12v6c0 1.66 3.58 3 8 3s8-1.34 8-3v-6"/></svg>',
  phone: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 16.92v3a2 2 0 01-2.18 2 19.8 19.8 0 01-8.63-3.07 19.5 19.5 0 01-6-6 19.8 19.8 0 01-3.07-8.67A2 2 0 014.11 2h3a2 2 0 012 1.72c.127.96.362 1.903.7 2.81a2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45c.907.338 1.85.573 2.81.7A2 2 0 0122 16.92z"/></svg>',
  appstore: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>',
  services: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 7l-8-4-8 4v10l8 4 8-4V7z"/><path d="M12 11v6"/><path d="M8 9v2"/><path d="M16 9v2"/></svg>',
  invoices: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6z"/><path d="M14 2v6h6"/><path d="M16 13H8"/><path d="M16 17H8"/><path d="M10 9H8"/></svg>',
  cards: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1" y="6" width="22" height="12" rx="2"/><path d="M1 10h22"/><circle cx="7" cy="14" r="1"/><circle cx="11" cy="14" r="1"/></svg>',
  profile: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.58-7 8-7s8 3 8 7"/></svg>',
  orgs: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87"/><path d="M16 3.13a4 4 0 010 7.75"/></svg>',
  offers: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>',
  llm: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>',
  settings: '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.32 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>',
  logout: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/><path d="M16 17l5-5-5-5"/><path d="M21 12H9"/></svg>',
};


// ── Sidebar: loaded from /cloud/partials/sidebar.html (single source of truth) ──
async function loadSidebar() {
  const shell = document.querySelector('.mgmt-shell');
  if (!shell) return;
  if (shell.querySelector('.mgmt-sidebar')) return; // already present

  try {
    const res = await fetch('/cloud/partials/sidebar.html');
    const html = res.ok ? await res.text() : '';
    if (html) {
      const temp = document.createElement('div');
      temp.innerHTML = html;
      const sidebar = temp.querySelector('.mgmt-sidebar') || temp.firstElementChild;
      if (sidebar) shell.insertBefore(sidebar, shell.firstChild);
    }
  } catch (_) { /* sidebar unavailable, page still works */ }
}

function initNavActive(sidebar) {
  const path = window.location.pathname;
  const params = new URLSearchParams(window.location.search);
  sidebar.querySelectorAll('.mgmt-nav-link').forEach(a => {
    const href = a.getAttribute('href') || '';
    const cat = href.includes('cat=') ? new URLSearchParams(href.split('?')[1] || '').get('cat') : null;
    if (cat) {
      if (params.get('cat') === cat) a.classList.add('active');
    } else if (href && href !== '#' && href !== 'javascript:void(0)') {
      const hrefPath = href.split('?')[0];
      if (path === hrefPath || (hrefPath !== '/cloud' && path.startsWith(hrefPath))) {
        a.classList.add('active');
      }
    }
  });
}

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();

  await loadSidebar();

  const sidebar = document.querySelector('.mgmt-sidebar');
  if (sidebar) initNavActive(sidebar);

  // Fill user email + avatar
  const email = localStorage.getItem('management_email') || '';
  const emailEl = document.getElementById('sidebar-email');
  if (emailEl) emailEl.textContent = email;

  const avatarEl = document.getElementById('sidebar-avatar');
  if (avatarEl && email) {
    avatarEl.textContent = email[0].toUpperCase();
    avatarEl.title = email;
  }

  const userEmail = document.getElementById('user-email');
  if (userEmail) userEmail.textContent = email;

  try { loadDashboardOrgs(token).catch(() => {}); } catch (_) {}
  try { loadDashboardPlans(token).catch(() => {}); } catch (_) {}
});


async function loadDashboardOrgs(token) {
  const container = document.getElementById('orgs-list');
  if (!container) return;
  try {
    const res = await fetch(`${API_BASE}/organizations`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { container.innerHTML = '<p>Failed to load organizations</p>'; return; }
    const orgs = await res.json();
    renderDashboardOrgs(orgs);
  } catch (err) {
    container.innerHTML = `<p>Error: ${err.message}</p>`;
  }
}

function renderDashboardOrgs(orgs) {
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

async function loadDashboardPlans(token) {
  const grid = document.getElementById('plans-grid');
  if (!grid) return;
  try {
    const res = await fetch(`${API_BASE}/plans`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) return;
    const data = await res.json();
    renderDashboardPlans(data.plans);
  } catch (err) {
    console.error('Failed to load plans:', err);
  }
}

function renderDashboardPlans(plans) {
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

async function createOrgFromDashboard(e) {
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
    else { hideNewOrgModal(); await loadDashboardOrgs(token); }
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
function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  sessionStorage.setItem('gb-signed-out', 'true');
  window.location.href = '/cloud';
}
function escapeHtml(str) { if (!str) return ''; const d = document.createElement('div'); d.textContent = str; return d.innerHTML; }
function getToken() { return localStorage.getItem('management_token'); }
function requireAuth() {
  let t = getToken();
  if (!t) {
    if (location.hostname === 'localhost' || location.hostname === '127.0.0.1') {
      if (sessionStorage.getItem('gb-signed-out') === 'true') {
        window.location.href = '/cloud/login';
        return null;
      }
      localStorage.setItem('management_token', 'dev-token');
      localStorage.setItem('management_email', 'admin@localhost');
      devAutoLogin();
      t = 'dev-token';
    } else {
      window.location.href = '/cloud/login';
    }
  }
  return t;
}

async function devAutoLogin() {
  if (location.hostname !== 'localhost' && location.hostname !== '127.0.0.1') return;
  try {
    const res = await fetch('/api/cloud/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: 'admin@localhost', password: 'dev' })
    });
    if (!res.ok) return;
    const data = await res.json();
    if (data.token) {
      localStorage.setItem('management_token', data.token);
      localStorage.setItem('management_email', data.email || 'admin@localhost');
    }
  } catch (_) { /* silent */ }
}

// ── Calculator: scroll to calc-panel if on store page, else navigate to store ──
function openCalculator() {
  if (window.location.pathname === '/cloud/store') {
    const panel = document.getElementById('calc-panel');
    if (panel) { panel.scrollIntoView({ behavior: 'smooth', block: 'start' }); return; }
  }
  window.location.href = '/cloud/store';
}
function calcUpdate() { /* defined in 02_calc.js */ }
function submitCalculator() { /* defined in 02_calc.js */ }

// ── Toast notification system ──
function showToast(message, type) {
  type = type || 'info';
  let container = document.getElementById('cloud-toast');
  if (!container) {
    container = document.createElement('div');
    container.id = 'cloud-toast';
    document.body.appendChild(container);
  }
  const item = document.createElement('div');
  item.className = 'cloud-toast-item ' + type;
  const icons = {
    success: '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#00d4aa" stroke-width="2"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>',
    error:   '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#ff6b6b" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>',
    info:    '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#6c63ff" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>'
  };
  item.innerHTML = (icons[type] || icons.info) + '<span>' + escapeHtml(message) + '</span>';
  container.appendChild(item);
  setTimeout(() => {
    item.style.opacity = '0';
    item.style.transform = 'translateX(20px)';
    item.style.transition = 'all .3s ease';
    setTimeout(() => item.remove(), 320);
  }, 3500);
}
