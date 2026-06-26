const API_BASE = '/api/cloud';

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  document.getElementById('user-email').textContent = localStorage.getItem('management_email') || '';
  await loadPlans(token);
});

let allPlans = {};
let currentPeriod = 'monthly';

async function loadPlans(token) {
  try {
    const res = await fetch(`${API_BASE}/plans`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { showError('Failed to load plans'); return; }
    const data = await res.json();
    allPlans = data.plans || {};
    renderPlans(currentPeriod);
    renderFeatureMatrix();
  } catch (err) {
    showError('Error: ' + err.message);
  }
}

function renderPlans(period) {
  const grid = document.getElementById('plans-grid');
  const entries = Object.entries(allPlans);

  grid.innerHTML = entries.map(([id, plan]) => {
    const isFree = plan.price.type === 'free';
    const isCustom = plan.price.type === 'custom';
    const monthlyAmount = isFree ? 0 : (plan.price.amount || 0) / 100;
    const yearlyAmount = monthlyAmount * 12 * 0.8;
    const displayAmount = period === 'yearly' ? yearlyAmount : monthlyAmount;
    const periodLabel = period === 'yearly' ? '/yr' : '/mo';

    return `
      <div class="saas-plan-card">
        <h3>${escapeHtml(plan.name)}</h3>
        <div class="plan-subtitle">${escapeHtml(plan.description || '')}</div>
        <div class="price">
          ${isFree ? 'Free' : isCustom ? 'Custom' : '$' + displayAmount.toFixed(2) + '<span class="price-period">' + periodLabel + '</span>'}
        </div>
        ${plan.trial_days ? `<div class="plan-trial">${plan.trial_days}-day free trial</div>` : ''}
        <ul class="plan-features">
          ${(plan.features || []).map(f => '<li>' + featureLabel(f) + '</li>').join('')}
        </ul>
        <div class="plan-limits">
          <div class="limit-row"><span>Messages/day</span><span>${formatLimit(plan.limits.messages_per_day)}</span></div>
          <div class="limit-row"><span>Storage</span><span>${formatStorage(plan.limits.storage_mb)}</span></div>
          <div class="limit-row"><span>Bots</span><span>${formatLimit(plan.limits.bots)}</span></div>
          <div class="limit-row"><span>Users</span><span>${formatLimit(plan.limits.users)}</span></div>
        </div>
        <button class="btn-primary btn-block" onclick="selectPlan('${id}')">
          ${isFree ? 'Get Started' : isCustom ? 'Contact Us' : 'Choose Plan'}
        </button>
      </div>
    `;
  }).join('');
}

function renderFeatureMatrix() {
  const thead = document.getElementById('features-header');
  const tbody = document.getElementById('features-body');
  const entries = Object.entries(allPlans);

  thead.innerHTML = '<th>Feature</th>' + entries.map(([id, p]) => '<th>' + escapeHtml(p.name) + '</th>').join('');

  const allFeatures = [...new Set(entries.flatMap(([_, p]) => p.features || []))];
  const allLimits = ['messages_per_day', 'storage_mb', 'bots', 'users', 'api_calls_per_day', 'kb_documents', 'apps'];

  tbody.innerHTML = '';

  allFeatures.forEach(feat => {
    tbody.innerHTML += '<tr><td class="feature-name">' + featureLabel(feat) + '</td>' +
      entries.map(([_, p]) => '<td class="' + ((p.features || []).includes(feat) ? 'check' : 'cross') + '">' +
        ((p.features || []).includes(feat) ? '✓' : '—') + '</td>').join('') + '</tr>';
  });

  tbody.innerHTML += '<tr class="section-divider"><td colspan="' + (entries.length + 1) + '">Limits</td></tr>';

  allLimits.forEach(limit => {
    tbody.innerHTML += '<tr><td class="feature-name">' + limitLabel(limit) + '</td>' +
      entries.map(([_, p]) => {
        const val = p.limits ? p.limits[limit] : null;
        return '<td>' + (val === null ? '∞' : formatLimit(val)) + '</td>';
      }).join('') + '</tr>';
  });
}

function toggleBillingPeriod() {
  currentPeriod = document.getElementById('period-toggle').checked ? 'yearly' : 'monthly';
  renderPlans(currentPeriod);
}

function selectPlan(planId) {
  const plan = allPlans[planId];
  if (!plan) return;
  if (plan.price.type === 'custom') {
    window.location.href = 'mailto:sales@pragmatismo.com.br?subject=' + encodeURIComponent('Enterprise plan inquiry');
    return;
  }
  const isFree = plan.price.type === 'free';
  if (isFree) {
    window.location.href = '/cloud/signup?plan=free';
    return;
  }
  const payload = encodeURIComponent(JSON.stringify({
    plan: planId,
    period: currentPeriod,
    storage: 5,
    ai: [],
    total: currentPeriod === 'yearly' ? ((plan.price.amount || 0) / 100 * 12 * 0.8) : ((plan.price.amount || 0) / 100),
    currency: 'usd',
  }));
  window.location.href = '/cloud/checkout?payload=' + payload;
}

function featureLabel(f) {
  const labels = {
    basic_chat: 'Basic Chat',
    file_upload: 'File Upload',
    email_support: 'Email Support',
    priority_support: 'Priority Support',
    custom_branding: 'Custom Branding',
    api_access: 'API Access',
    analytics: 'Analytics',
    sso_saml: 'SSO / SAML',
    sla_guarantee: 'SLA Guarantee',
    dedicated_support: 'Dedicated Support',
    on_premise: 'On-Premise Option',
    audit_logs: 'Audit Logs',
  };
  return labels[f] || f.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

function limitLabel(l) {
  const labels = {
    messages_per_day: 'Messages / Day',
    storage_mb: 'Storage',
    bots: 'Bots',
    users: 'Users',
    api_calls_per_day: 'API Calls / Day',
    kb_documents: 'KB Documents',
    apps: 'Apps',
  };
  return labels[l] || l.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

function formatLimit(val) {
  if (val === null || val === undefined) return '∞';
  if (val >= 1000000) return (val / 1000000).toFixed(0) + 'M';
  if (val >= 1000) return (val / 1000).toFixed(0) + 'k';
  return val.toString();
}

function formatStorage(mb) {
  if (mb === null || mb === undefined) return '∞';
  if (mb >= 1024 * 1024) return (mb / 1024 / 1024).toFixed(0) + 'TB';
  if (mb >= 1024) return (mb / 1024).toFixed(0) + 'GB';
  return mb + 'MB';
}

function showError(msg) {
  document.getElementById('plans-grid').innerHTML = '<div class="saas-loading" style="color:red">' + escapeHtml(msg) + '</div>';
}

function requireAuth() {
  const token = localStorage.getItem('management_token');
  if (!token) window.location.href = '/cloud/login';
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/cloud';
}

function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
