document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  const emailEl = document.getElementById('user-email');
  if (emailEl) emailEl.textContent = localStorage.getItem('management_email') || '';
  await loadServices(token);
});

const PLAN_INCLUDED = {
  free: {
    label: 'Free Plan',
    price: '$0/mo',
    items: ['1 bot', '20 MB storage', '10 messages/day', 'Unlimited apps'],
    cta: { text: 'See what Shared includes', href: '/plans' }
  },
  shared: {
    label: 'Shared Plan',
    price: '$3.99/mo',
    items: ['5 bots / workspaces', '50 GB storage', 'Phone numbers', 'Domains', 'LLM model access'],
    cta: { text: 'Upgrade to Private Cloud', href: '/plans' }
  },
  'private-cloud': {
    label: 'Private Cloud',
    price: 'custom',
    items: ['Dedicated VPS included', 'Unlimited workspaces', 'GPU computing', 'Own branding', 'Full LLM catalog'],
    cta: { text: 'View store', href: '/store' }
  }
};

let currentPlan = 'free';

async function loadServices(token) {
  // Resolve the current plan for the included-services summary
  try {
    const orgRes = await cloudFetch(`${API_BASE}/organizations`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (orgRes.ok) {
      const orgData = await orgRes.json();
      const orgs = orgData.organizations || [];
      if (orgs.length) currentPlan = orgs[0].plan || 'free';
    }
  } catch (_) {}
  renderPlanSummary();

  try {
    const res = await cloudFetch(`${API_BASE}/services`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) {
      // Anonymous: show the empty state (CTA banner invites sign-in)
      renderServices([]);
      return;
    }
    const data = await res.json();
    renderServices(data.services || []);
  } catch (err) {
    showError('Error: ' + err.message);
  }
}

function renderPlanSummary() {
  const container = document.getElementById('services-list');
  if (!container) return;
  const plan = PLAN_INCLUDED[currentPlan] || PLAN_INCLUDED.free;
  const planCard = document.createElement('div');
  planCard.className = 'service-plan-card';
  planCard.innerHTML = `
    <div class="service-plan-head">
      <div>
        <div class="service-plan-name">${escapeHtml(plan.label)}</div>
        <div class="service-plan-price">${escapeHtml(plan.price)}</div>
      </div>
      <div class="service-plan-items">
        ${plan.items.map(i => `<span class="service-plan-item">✓ ${escapeHtml(i)}</span>`).join('')}
      </div>
      <a href="${plan.cta.href}" class="btn-secondary btn-sm service-plan-cta">${escapeHtml(plan.cta.text)}</a>
    </div>
  `;
  container.insertBefore(planCard, container.firstChild);
}

function renderServices(services) {
  window.__allServices = services.filter(s => !s.is_base_system);
  window.__activeFilter = 'all';
  renderFilteredServices();
}

function filterServices(btn, filter) {
  window.__activeFilter = filter;
  document.querySelectorAll('.cat-pill').forEach(p => p.classList.toggle('active', p === btn));
  renderFilteredServices();
}

function renderFilteredServices() {
  const container = document.getElementById('services-list');
  if (!container) return;
  const planCard = container.querySelector('.service-plan-card');
  container.innerHTML = '';
  if (planCard) container.appendChild(planCard);

  const filter = window.__activeFilter || 'all';
  const all = window.__allServices || [];
  const keyword = { compute: ['vps', 'gpu', 'virtual machine'], storage: ['storage', 'gb'], number: ['number', 'phone'], domain: ['domain', 'com'] };
  const list = filter === 'all' ? all : all.filter(s => {
    const hay = ((s.name || '') + ' ' + (s.description || '')).toLowerCase();
    return (keyword[filter] || []).some(k => hay.includes(k));
  });

  if (list.length === 0) {
    const addon = document.createElement('div');
    addon.className = 'saas-loading';
    addon.textContent = filter === 'all'
      ? 'No extra services yet. Add VPS, GPU, storage, phone numbers or domains from the Store.'
      : 'Nothing in this category yet. Visit the Store to add it.';
    container.appendChild(addon);
    return;
  }
  list.forEach(s => {
    const card = document.createElement('div');
    card.className = 'service-card';
    card.innerHTML = `
      <div class="service-header">
        <h3>${escapeHtml(s.name)}</h3>
        <span class="service-badge ${s.status}">${s.status}</span>
      </div>
      <div class="service-body">
        <p>${escapeHtml(s.description || '')}</p>
        <div class="service-meta">
          <span>Created: ${new Date(s.created_at).toLocaleDateString()}</span>
          ${s.expires_at ? `<span>Expires: ${new Date(s.expires_at).toLocaleDateString()}</span>` : ''}
        </div>
      </div>
      <div class="service-actions">
        ${s.dashboard_url ? `<a href="${s.dashboard_url}" class="btn-secondary" target="_blank">Open</a>` : ''}
        ${s.status === 'active' ? `<button class="btn-text" onclick="cancelService('${s.id}')">Cancel</button>` : ''}
      </div>
    `;
    container.appendChild(card);
  });
}

async function cancelService(id) {
  if (!confirm('Are you sure you want to cancel this service?')) return;
  const token = requireAuth();
  try {
    const res = await cloudFetch(`${API_BASE}/services/${id}/cancel`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { alert('Cancel failed'); return; }
    await loadServices(token);
  } catch (err) {
    alert('Error: ' + err.message);
  }
}

function requireAuth() {
  const token = localStorage.getItem('management_token');
  if (!token) {
    // Anonymous browsing allowed — the CTA banner invites sign-in
    return null;
  }
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/';
}

function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function showError(msg) {
  document.getElementById('services-list').innerHTML = '<div class="saas-loading" style="color:red">' + escapeHtml(msg) + '</div>';
}
