const API_BASE = '/api/management';

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  document.getElementById('user-email').textContent = localStorage.getItem('management_email') || '';
  await loadInvoices(token);
});

async function loadInvoices(token) {
  try {
    const res = await fetch(`${API_BASE}/invoices`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { showError('Failed to load invoices'); return; }
    const data = await res.json();
    renderInvoices(data.invoices || []);
  } catch (err) {
    showError('Error: ' + err.message);
  }
}

function renderInvoices(invoices) {
  const container = document.getElementById('invoices-list');
  if (invoices.length === 0) {
    container.innerHTML = '<div class="saas-loading">No invoices yet.</div>';
    return;
  }
  container.innerHTML = invoices.map(inv => `
    <div class="invoice-card">
      <div class="invoice-info">
        <h3>${escapeHtml(inv.number || 'Invoice')}</h3>
        <p>${new Date(inv.created_at).toLocaleDateString()} · <span class="invoice-status ${inv.status}">${inv.status}</span></p>
      </div>
      <div style="text-align:right">
        <div class="invoice-amount">$${((inv.amount || 0) / 100).toFixed(2)}</div>
        ${inv.status === 'open' || inv.status === 'overdue' ? `<a href="${inv.pay_url || '#'}" class="btn-primary" style="margin-top:0.25rem;font-size:0.75rem;padding:0.25rem 0.75rem">Pay Now</a>` : ''}
      </div>
    </div>
  `).join('');
}

function requireAuth() {
  const token = localStorage.getItem('management_token');
  if (!token) window.location.href = '/management/login';
  return token;
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

function showError(msg) {
  document.getElementById('invoices-list').innerHTML = '<div class="saas-loading" style="color:red">' + escapeHtml(msg) + '</div>';
}
