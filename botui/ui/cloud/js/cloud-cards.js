const API_BASE = '/api/cloud';

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  document.getElementById('user-email').textContent = localStorage.getItem('management_email') || '';
  await loadCards(token);
});

async function loadCards(token) {
  try {
    const res = await fetch(`${API_BASE}/payment-cards`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { showError('Failed to load cards'); return; }
    const data = await res.json();
    renderCards(data.cards || []);
  } catch (err) {
    showError('Error: ' + err.message);
  }
}

function renderCards(cards) {
  const container = document.getElementById('cards-list');
  if (cards.length === 0) {
    container.innerHTML = '<div class="saas-loading">No payment cards saved. <a href="#" onclick="openBillingPortal()">Add one in Stripe</a>.</div>';
    return;
  }
  container.innerHTML = cards.map(card => `
    <div class="card-item">
      <div class="card-info">
        <div class="card-brand">${card.brand === 'visa' ? '💳' : card.brand === 'mastercard' ? '💳' : '💳'}</div>
        <div class="card-details">
          <h4>${escapeHtml(card.brand || 'Card')} ···· ${escapeHtml(card.last4 || '')}</h4>
          <p>Expires ${card.exp_month}/${card.exp_year}${card.is_default ? ' · <span class="card-default">Default</span>' : ''}</p>
        </div>
      </div>
    </div>
  `).join('');
}

async function openBillingPortal() {
  const token = requireAuth();
  try {
    const res = await fetch(`${API_BASE}/billing-portal`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { alert('Failed to open billing portal'); return; }
    const data = await res.json();
    if (data.url) window.open(data.url, '_blank');
  } catch (err) {
    alert('Error: ' + err.message);
  }
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

function showError(msg) {
  document.getElementById('cards-list').innerHTML = '<div class="saas-loading" style="color:red">' + escapeHtml(msg) + '</div>';
}
